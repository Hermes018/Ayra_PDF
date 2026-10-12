use lpdf_core_types::LpdfError;
use lpdf_engine_fallback::FallbackAdapter;
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::PdfEngine;
use lpdf_engine_switcher::fuzz::*;
use lpdf_engine_switcher::hostile::*;
use lpdf_engine_switcher::EngineSwitcher;
use std::fs;
use std::path::PathBuf;

/// Deterministic, fast pseudo-random number generator (Xorshift64Star).
struct SeededPrng {
    state: u64,
}

impl SeededPrng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_usize(&mut self, max: usize) -> usize {
        if max == 0 {
            0
        } else {
            (self.next_u64() as usize) % max
        }
    }

    fn next_u8(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }

    fn fill_bytes(&mut self, buf: &mut [u8]) {
        for b in buf.iter_mut() {
            *b = self.next_u8();
        }
    }
}

fn get_corpus_pdf(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus")
        .join(name)
}

#[test]
fn test_fuzz_pdfcraft_adapter_seeded_mutations() {
    let base_pdf = get_corpus_pdf("standard-text.pdf");
    let base_bytes = fs::read(&base_pdf)
        .unwrap_or_else(|_| b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec());
    let mut rng = SeededPrng::new(0xABCD1234F00DCAFE);

    // 150 iterations of mutated byte buffers
    for i in 0..150 {
        let mut mutated = base_bytes.clone();
        match i % 5 {
            0 => {
                // Bit flips
                let flips = rng.next_usize(20) + 1;
                for _ in 0..flips {
                    let idx = rng.next_usize(mutated.len());
                    let bit = rng.next_usize(8);
                    mutated[idx] ^= 1 << bit;
                }
            }
            1 => {
                // Truncation
                let trunc_len = rng.next_usize(mutated.len());
                mutated.truncate(trunc_len);
            }
            2 => {
                // Header corruption
                let bad_headers: [&[u8]; 5] = [
                    b"%NOTPDF-1.4\n",
                    b"\x00\x00\x00\x00\x00\x00\x00\x00",
                    b"%PDF-999999.9999\n",
                    b"",
                    b"\xFF\xFE\xFD\xFC",
                ];
                let chosen = bad_headers[i % bad_headers.len()];
                if mutated.len() >= chosen.len() {
                    mutated[..chosen.len()].copy_from_slice(chosen);
                } else {
                    mutated = chosen.to_vec();
                }
            }
            3 => {
                // Xref and EOF corruption
                let corruptions: [&[u8]; 4] = [
                    b"\nxref\ncorrupted_table_data_random\n%%EOF",
                    b"\nstartxref\n-999999999999\n%%EOF",
                    b"\ntrailer\n<< /Size 9999999999 /Root 999 0 R >>\nstartxref\n0\n%%EOF",
                    b"\n%%EOF_CORRUPT_TRAILING_BYTES",
                ];
                mutated.extend_from_slice(corruptions[i % corruptions.len()]);
            }
            _ => {
                // Completely random byte buffer
                let len = rng.next_usize(2048) + 1;
                mutated = vec![0u8; len];
                rng.fill_bytes(&mut mutated);
            }
        }

        // Must never panic
        fuzz_pdfcraft_bytes(&mut mutated);
    }
}

#[test]
fn test_fuzz_fallback_adapter_seeded_mutations() {
    let base_pdf = get_corpus_pdf("standard-text.pdf");
    let base_bytes = fs::read(&base_pdf)
        .unwrap_or_else(|_| b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF".to_vec());
    let mut rng = SeededPrng::new(0xFEEDFACECAFED00D);

    // 150 iterations of mutated byte buffers
    for i in 0..150 {
        let mut mutated = base_bytes.clone();
        match i % 5 {
            0 => {
                // Multiple bit flips
                let flips = rng.next_usize(15) + 1;
                for _ in 0..flips {
                    let idx = rng.next_usize(mutated.len());
                    mutated[idx] ^= rng.next_u8();
                }
            }
            1 => {
                // Extreme truncation
                let len = rng.next_usize(mutated.len().min(100));
                mutated.truncate(len);
            }
            2 => {
                // Deep nesting brackets
                let mut nested = b"%PDF-1.4\n1 0 obj\n".to_vec();
                for _ in 0..100 {
                    nested.extend_from_slice(b"<< /Nested ");
                }
                nested.extend_from_slice(b">> endobj\ntrailer << /Root 1 0 R >>\n%%EOF");
                mutated = nested;
            }
            3 => {
                // Truncated object stream
                mutated.extend_from_slice(b"\n10 0 obj\n<< /Type /ObjStm /Length 50 >>\nstream\n");
            }
            _ => {
                // Random bytes
                let len = rng.next_usize(1024) + 1;
                mutated = vec![0u8; len];
                rng.fill_bytes(&mut mutated);
            }
        }

        fuzz_fallback_bytes(&mut mutated);
    }
}

#[test]
fn test_fuzz_engine_switcher_degradation_paths() {
    let base_pdf = get_corpus_pdf("standard-text.pdf");
    let base_bytes = fs::read(&base_pdf).unwrap_or_else(|_| b"%PDF-1.4\n".to_vec());
    let mut rng = SeededPrng::new(0xCAFE1234BEEF5678);

    // 150 iterations verifying primary-to-fallback degradation
    for i in 0..150 {
        let mut mutated = base_bytes.clone();
        match i % 4 {
            0 => {
                // Subtle xref shift to trigger fallback lopdf repair
                if let Some(pos) = mutated.windows(4).position(|w| w == b"xref") {
                    mutated[pos] = b'X';
                }
            }
            1 => {
                // Truncate end of file
                let cut = mutated.len().saturating_sub(rng.next_usize(500));
                mutated.truncate(cut);
            }
            2 => {
                // Cyclic reference attempt
                let cyclic =
                    b"\n1 0 obj << /Next 2 0 R >> endobj\n2 0 obj << /Next 1 0 R >> endobj\n";
                mutated.extend_from_slice(cyclic);
            }
            _ => {
                // Random junk
                let len = rng.next_usize(512);
                mutated = vec![0u8; len];
                rng.fill_bytes(&mut mutated);
            }
        }

        fuzz_engine_switcher_bytes(&mut mutated);
    }
}

#[test]
fn test_fuzz_fs_and_ai_rpc_adversarial_inputs() {
    let mut rng = SeededPrng::new(0x1337B007CAFEBABE);

    // 150 iterations of adversarial path strings and AI worker payloads
    for i in 0..150 {
        let adversarial_data = match i % 6 {
            0 => {
                // Path traversal attempts
                b"../../../../../../etc/passwd\x00.pdf".to_vec()
            }
            1 => {
                // Windows reserved device names and invalid characters
                let reserved = [
                    "CON.pdf",
                    "PRN",
                    "AUX",
                    "NUL",
                    "COM1",
                    "LPT1",
                    "file:with*invalid?chars\"<bar>|pipe.pdf",
                    "   spaced_and____underscored___   ",
                    "\x00\x01\x02\x03\x1f_control_chars.pdf",
                ];
                reserved[i % reserved.len()].as_bytes().to_vec()
            }
            2 => {
                // Malformed JSON RPC frames
                let json_samples = [
                    b"{ \"action\": \"Ping\" }".as_slice(),
                    b"{ \"action\": \"ValidateModel\", \"path\": \"/tmp/nonexistent\", \"expected_sha256\": \"bad\" }".as_slice(),
                    b"{ \"action\": \"InferChunk\", \"prompt\": \"hello\", \"max_tokens\": 999999999, \"temperature\": 999.0 }".as_slice(),
                    b"{ \"action\": \"UnknownAction\", \"data\": 123 }".as_slice(),
                    b"{ invalid json syntax ...".as_slice(),
                    b"".as_slice(),
                ];
                json_samples[i % json_samples.len()].to_vec()
            }
            3 => {
                // Long strings (deep nesting / buffer stress)
                let repeated = "nested_dir/".repeat(50);
                repeated.into_bytes()
            }
            4 => {
                // Model header test data (corrupted magic bytes)
                let headers: [&[u8]; 4] = [
                    b"GGUF\x00\x00\x00\x00short".as_slice(),
                    b"NOTGGUF_MAGIC_BYTES_12345678".as_slice(),
                    b"ONNX_TRUNCATED".as_slice(),
                    b"\x00\x00".as_slice(),
                ];
                headers[i % headers.len()].to_vec()
            }
            _ => {
                // Arbitrary random bytes
                let len = rng.next_usize(256) + 1;
                let mut buf = vec![0u8; len];
                rng.fill_bytes(&mut buf);
                buf
            }
        };

        fuzz_fs_and_ai_rpc_bytes(&adversarial_data);
    }
}

#[test]
fn test_fuzz_typed_lpdf_errors_on_malformed_inputs() {
    let temp_dir = std::env::temp_dir().join(format!(
        "lpdf_typed_err_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let malformed_pdf = temp_dir.join("malformed.pdf");
    fs::write(&malformed_pdf, b"NOT_A_VALID_PDF_AT_ALL").unwrap();

    // 1. PdfcraftAdapter must return typed LpdfError::Engine or Io
    let mut pdfcraft = PdfcraftAdapter::new();
    let res1 = pdfcraft.load(&malformed_pdf);
    assert!(
        res1.is_err(),
        "PdfcraftAdapter must return error on malformed PDF"
    );
    match res1.unwrap_err() {
        LpdfError::Engine(_) | LpdfError::Io(_) => {}
        other => panic!("Unexpected error variant from PdfcraftAdapter: {:?}", other),
    }

    // 2. FallbackAdapter must return typed LpdfError::Engine or Io
    let mut fallback = FallbackAdapter::new();
    let res2 = fallback.load(&malformed_pdf);
    assert!(
        res2.is_err(),
        "FallbackAdapter must return error on malformed PDF"
    );
    match res2.unwrap_err() {
        LpdfError::Engine(_) | LpdfError::Io(_) => {}
        other => panic!("Unexpected error variant from FallbackAdapter: {:?}", other),
    }

    // 3. EngineSwitcher must fail cleanly with typed LpdfError
    let mut switcher = EngineSwitcher::new();
    let res3 = switcher.load(&malformed_pdf);
    assert!(
        res3.is_err(),
        "EngineSwitcher must return error on malformed PDF"
    );
    match res3.unwrap_err() {
        LpdfError::Engine(_) | LpdfError::Io(_) => {}
        other => panic!("Unexpected error variant from EngineSwitcher: {:?}", other),
    }

    // 4. sanitize_filename guarantees safe path tokens
    assert_eq!(lpdf_fs_safety::sanitize_filename(""), "export");
    assert_eq!(lpdf_fs_safety::sanitize_filename("   "), "export");
    assert_eq!(
        lpdf_fs_safety::sanitize_filename("../../../etc/passwd"),
        "etc_passwd"
    );
    assert_eq!(lpdf_fs_safety::sanitize_filename("CON.pdf"), "CON_pdf");

    let _ = fs::remove_dir_all(&temp_dir);
}

fn run_hostile_vector_on_all_adapters(bytes: &[u8], name: &str) {
    let temp_dir = std::env::temp_dir().join(format!(
        "lpdf_hostile_test_{}_{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();
    let pdf_path = temp_dir.join(format!("{}.pdf", name));
    fs::write(&pdf_path, bytes).unwrap();

    // 1. PdfcraftAdapter
    let mut pdfcraft = PdfcraftAdapter::new();
    exercise_all_operations_on_hostile_file(&mut pdfcraft, &pdf_path, &temp_dir)
        .expect("PdfcraftAdapter hostile run failed");

    // 2. FallbackAdapter
    let mut fallback = FallbackAdapter::new();
    exercise_all_operations_on_hostile_file(&mut fallback, &pdf_path, &temp_dir)
        .expect("FallbackAdapter hostile run failed");

    // 3. EngineSwitcher
    let mut switcher = EngineSwitcher::new();
    exercise_all_operations_on_hostile_file(&mut switcher, &pdf_path, &temp_dir)
        .expect("EngineSwitcher hostile run failed");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_hostile_flate_bomb_handling() {
    let bytes = generate_flate_bomb_pdf();
    run_hostile_vector_on_all_adapters(&bytes, "flate_bomb");
}

#[test]
fn test_hostile_deep_page_tree_handling() {
    let bytes = generate_deep_page_tree_pdf(500);
    run_hostile_vector_on_all_adapters(&bytes, "deep_page_tree");
}

#[test]
fn test_hostile_cyclic_page_tree_handling() {
    let bytes = generate_cyclic_page_tree_pdf();
    run_hostile_vector_on_all_adapters(&bytes, "cyclic_page_tree");
}

#[test]
fn test_hostile_broken_xref_handling() {
    let bytes = generate_broken_xref_pdf();
    run_hostile_vector_on_all_adapters(&bytes, "broken_xref");
}

#[test]
fn test_hostile_js_launch_handling() {
    let bytes = generate_js_launch_pdf();
    run_hostile_vector_on_all_adapters(&bytes, "js_launch");
}

#[test]
fn test_hostile_pathological_geometry_handling() {
    let bytes = generate_pathological_geometry_pdf();
    run_hostile_vector_on_all_adapters(&bytes, "geometry");
}

#[test]
fn test_hostile_paths_and_device_names() {
    let temp_parent = std::env::temp_dir().join(format!(
        "lpdf_hostile_paths_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_parent).unwrap();

    for &path_str in hostile_path_candidates() {
        // 1. sanitize_filename guarantees no traversal or invalid tokens
        let sanitized = lpdf_fs_safety::sanitize_filename(path_str);
        assert!(
            !sanitized.is_empty(),
            "Sanitized filename must never be empty"
        );
        assert!(!sanitized.contains('/'), "Must not contain forward slash");
        assert!(!sanitized.contains('\\'), "Must not contain backslash");
        assert!(!sanitized.contains('\0'), "Must not contain null bytes");
        assert!(!sanitized.contains(".."), "Must not contain traversal dots");

        // 2. Safe output dir creation under temp_parent
        let safe_dir = lpdf_fs_safety::create_safe_output_dir(&temp_parent, path_str)
            .expect("create_safe_output_dir must not fail");
        assert!(safe_dir.exists(), "Safe dir must be created successfully");
        assert!(
            safe_dir.starts_with(&temp_parent),
            "Safe dir must stay inside parent dir"
        );

        // 3. atomic_write with safe filenames
        let target_file = safe_dir.join(format!("{}.pdf", sanitized));
        lpdf_fs_safety::atomic_write(&target_file, b"%PDF-1.4\n%%EOF")
            .expect("atomic_write must succeed");
        assert!(target_file.exists());
    }

    let _ = fs::remove_dir_all(&temp_parent);
}

#[test]
fn test_corpus_hostile_files_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!(
        "lpdf_corpus_hostile_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir).unwrap();

    let hostile_bomb = get_corpus_pdf("hostile-bombs.pdf");
    if hostile_bomb.exists() {
        let mut switcher = EngineSwitcher::new();
        exercise_all_operations_on_hostile_file(&mut switcher, &hostile_bomb, &temp_dir)
            .expect("Corpus hostile bomb exercise failed");
    }

    let hostile_xref = get_corpus_pdf("hostile-broken-xref.pdf");
    if hostile_xref.exists() {
        let mut switcher = EngineSwitcher::new();
        exercise_all_operations_on_hostile_file(&mut switcher, &hostile_xref, &temp_dir)
            .expect("Corpus hostile xref exercise failed");
    }

    let _ = fs::remove_dir_all(&temp_dir);
}
