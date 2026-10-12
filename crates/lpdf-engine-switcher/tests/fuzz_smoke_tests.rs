use lpdf_core_types::LpdfError;
use lpdf_engine_fallback::FallbackAdapter;
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::PdfEngine;
use lpdf_engine_switcher::fuzz::*;
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
