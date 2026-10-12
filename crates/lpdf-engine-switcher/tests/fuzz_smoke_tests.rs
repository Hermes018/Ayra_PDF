use lpdf_core_types::{HardwareTier, LpdfError};
use lpdf_engine_fallback::FallbackAdapter;
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::PdfEngine;
use lpdf_engine_switcher::fuzz::*;
use lpdf_engine_switcher::hostile::*;
use lpdf_engine_switcher::perf::*;
use lpdf_engine_switcher::sbom::*;
use lpdf_engine_switcher::soak::*;
use lpdf_engine_switcher::EngineSwitcher;
use std::fs;
use std::path::{Path, PathBuf};

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

#[test]
fn test_tier_perf_suite_and_memory_sampling() {
    let sample = sample_memory();
    assert!(sample.rss_kb > 0, "RSS must be non-zero");
    assert!(sample.hwm_kb > 0, "HWM must be non-zero");

    let sample_pdf = get_corpus_pdf("standard-text.pdf");
    assert!(sample_pdf.exists(), "Sample corpus PDF must exist");

    let report = run_tier_perf_suite(HardwareTier::Low, &sample_pdf)
        .expect("Tier perf suite must complete successfully");

    assert_eq!(report.tier, HardwareTier::Low);
    assert!(report.total_duration_ms > 0);
    assert!(report.peak_rss_mb > 0.0);

    let required_ops = [
        "info",
        "rotate_document",
        "compress",
        "pdf_to_text",
        "summarize_extractive",
        "redact",
        "synthetic_multi_page",
    ];

    for op in &required_ops {
        assert!(
            report.operations.contains_key(*op),
            "Report must contain metrics for operation '{}'",
            op
        );
        let m = &report.operations[*op];
        assert!(m.peak_rss_mb > 0.0);
    }
}

#[test]
fn test_perf_regression_detector_pass_and_fail() {
    let mut baseline = default_tier_budgets()
        .into_iter()
        .find(|b| b.tier == HardwareTier::Low)
        .expect("Low tier budget must exist");

    // 1. Passing verification
    let mut report = PerfProfileReport {
        tier: HardwareTier::Low,
        total_duration_ms: 100,
        start_rss_mb: 50.0,
        peak_rss_mb: 200.0,
        operations: std::collections::HashMap::new(),
    };
    report.operations.insert(
        "info".to_string(),
        OperationMetrics {
            latency_ms: 50,
            peak_rss_mb: 100.0,
        },
    );
    report.operations.insert(
        "rotate_document".to_string(),
        OperationMetrics {
            latency_ms: 100,
            peak_rss_mb: 120.0,
        },
    );

    // Baseline: info 100ms, rotate 200ms, max_rss 500.0MB
    baseline.latency_budgets_ms.insert("info".to_string(), 100);
    baseline
        .latency_budgets_ms
        .insert("rotate_document".to_string(), 200);
    baseline.max_rss_mb = 500.0;

    let res_ok = verify_against_baseline(&report, &baseline, 10.0);
    assert!(res_ok.is_ok(), "Under-budget run must pass");

    // 2. Failing verification (> 10.0% latency regression)
    // Baseline: 100ms. Allowed with 10% = 110ms. Measured = 125ms (25% regression).
    report.operations.insert(
        "info".to_string(),
        OperationMetrics {
            latency_ms: 125,
            peak_rss_mb: 100.0,
        },
    );
    let res_fail_lat = verify_against_baseline(&report, &baseline, 10.0);
    assert!(res_fail_lat.is_err(), "Regressed latency must fail");
    let violations = res_fail_lat.unwrap_err();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].metric_name, "info_latency_ms");
    assert!((violations[0].regression_pct - 25.0).abs() < 1e-3);
    assert_eq!(violations[0].baseline_value, 100.0);
    assert_eq!(violations[0].measured_value, 125.0);

    // 3. Failing verification (> 10.0% memory regression)
    // Baseline: 500MB. Allowed with 10% = 550MB. Measured = 600MB (20% regression).
    report.operations.insert(
        "info".to_string(),
        OperationMetrics {
            latency_ms: 50,
            peak_rss_mb: 100.0,
        },
    );
    report.peak_rss_mb = 600.0;
    let res_fail_mem = verify_against_baseline(&report, &baseline, 10.0);
    assert!(res_fail_mem.is_err(), "Regressed memory must fail");
    let violations_mem = res_fail_mem.unwrap_err();
    assert_eq!(violations_mem.len(), 1);
    assert_eq!(violations_mem[0].metric_name, "peak_rss_mb");
    assert!((violations_mem[0].regression_pct - 20.0).abs() < 1e-3);
}

#[test]
fn test_load_baseline_budgets_file() {
    let budgets_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../protocol/perf_budgets.json");
    assert!(
        budgets_path.exists(),
        "protocol/perf_budgets.json must exist"
    );

    let budgets = load_baseline_budgets(&budgets_path)
        .expect("load_baseline_budgets must parse protocol/perf_budgets.json");

    assert!(budgets.contains_key(&HardwareTier::Low));
    assert!(budgets.contains_key(&HardwareTier::Medium));
    assert!(budgets.contains_key(&HardwareTier::High));

    let low = &budgets[&HardwareTier::Low];
    assert_eq!(low.concurrency, 1);
    assert_eq!(low.render_threads, 2);
    assert!(low.max_rss_mb > 0.0);
    assert!(low.latency_budgets_ms.contains_key("info"));
    assert!(low.latency_budgets_ms.contains_key("synthetic_multi_page"));
}

#[test]
fn test_cyclonedx_sbom_generation() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let cargo_lock = workspace_root.join("Cargo.lock");
    let package_json = workspace_root.join("app-tauri/package.json");
    let sbom_out = workspace_root.join("logs/sbom.cdx.json");

    assert!(cargo_lock.exists(), "Cargo.lock must exist");
    assert!(package_json.exists(), "app-tauri/package.json must exist");

    let bom = generate_cyclonedx_sbom(&cargo_lock, &package_json)
        .expect("CycloneDX SBOM generation must succeed");

    assert_eq!(bom.bom_format, "CycloneDX");
    assert_eq!(bom.spec_version, "1.5");
    assert!(
        bom.components.len() > 100,
        "SBOM must contain comprehensive list of dependencies"
    );

    // Verify presence of both Cargo crates and npm packages
    let has_cargo = bom
        .components
        .iter()
        .any(|c| c.name == "serde" || c.name == "lpdf-core-types");
    let has_npm = bom
        .components
        .iter()
        .any(|c| c.name == "svelte" || c.name == "typescript");
    assert!(has_cargo, "SBOM must include Cargo crate components");
    assert!(has_npm, "SBOM must include npm package components");

    // Write to logs/sbom.cdx.json
    write_sbom_to_file(&bom, &sbom_out).expect("Writing SBOM to logs/sbom.cdx.json must succeed");
    assert!(sbom_out.exists(), "logs/sbom.cdx.json must exist on disk");

    let written_content = fs::read_to_string(&sbom_out).expect("Read written SBOM");
    assert!(!written_content.is_empty(), "SBOM must not be empty");
    let reparsed: CycloneDxBom =
        serde_json::from_str(&written_content).expect("Re-parsed SBOM must be valid JSON");
    assert_eq!(reparsed.bom_format, "CycloneDX");
}

#[test]
fn test_supply_chain_forbid_unsafe_revet() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let crates_dir = workspace_root.join("crates");
    let app_tauri_dir = workspace_root.join("app-tauri");
    let cargo_lock = workspace_root.join("Cargo.lock");
    let deny_toml = workspace_root.join("deny.toml");

    let verified_entrypoints = verify_forbid_unsafe_in_workspace(&crates_dir, &app_tauri_dir)
        .expect("All workspace crates must enforce #![forbid(unsafe_code)]");
    assert!(
        verified_entrypoints.len() >= 15,
        "Expected at least 15 entrypoints with #![forbid(unsafe_code)], verified {}",
        verified_entrypoints.len()
    );

    verify_no_copyleft_dependencies(&cargo_lock, &deny_toml)
        .expect("No copyleft GPL/AGPL dependencies may be linked");
}

#[test]
fn test_threat_model_matrix_coverage() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let threat_model_path = workspace_root.join("protocol/THREAT_MODEL.md");

    verify_threat_model_coverage(&threat_model_path)
        .expect("Threat model must cover all required threats T1 through T6");
}

#[test]
fn test_a11y_protocol_audit_and_svelte_landmarks() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let audit_doc = workspace_root.join("protocol/A11Y_AUDIT.md");
    assert!(audit_doc.exists(), "protocol/A11Y_AUDIT.md must exist");

    let doc_content = fs::read_to_string(&audit_doc).expect("Read A11Y_AUDIT.md");
    assert!(!doc_content.is_empty(), "A11Y_AUDIT.md must not be empty");
    assert!(doc_content.contains("WCAG 2.1 Level AA"));
    assert!(doc_content.contains("Landmark"));
    assert!(doc_content.contains("aria-current"));
    assert!(doc_content.contains("role=\"tablist\""));
    assert!(doc_content.contains("role=\"progressbar\""));
    assert!(doc_content.contains("role=\"alert\""));

    let svelte_src = workspace_root.join("app-tauri/src");
    assert!(svelte_src.exists(), "app-tauri/src must exist");

    fn collect_svelte_files(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_svelte_files(&path, files);
                } else if path.extension().and_then(|s| s.to_str()) == Some("svelte") {
                    files.push(path);
                }
            }
        }
    }

    let mut svelte_files = Vec::new();
    collect_svelte_files(&svelte_src, &mut svelte_files);

    assert!(
        svelte_files.len() >= 10,
        "Expected at least 10 Svelte components, found {}",
        svelte_files.len()
    );

    // Verify key landmark files
    let mut found_skip_link = false;
    let mut found_primary_nav = false;
    let mut found_status_landmark = false;
    let mut found_tablist = false;

    for file_path in &svelte_files {
        let content = fs::read_to_string(file_path).expect("Read svelte file");
        if content.contains("class=\"skip-link\"") && content.contains("id=\"main-content\"") {
            found_skip_link = true;
        }
        if content.contains("aria-label=\"Primary Navigation\"") && content.contains("aria-current")
        {
            found_primary_nav = true;
        }
        if content.contains("role=\"status\"") && content.contains("aria-label=\"System status\"") {
            found_status_landmark = true;
        }
        if content.contains("role=\"tablist\"") && content.contains("role=\"tab\"") {
            found_tablist = true;
        }
    }

    assert!(
        found_skip_link,
        "Must find skip-link and main-content landmarks in Svelte layout"
    );
    assert!(
        found_primary_nav,
        "Must find Primary Navigation landmark and aria-current in Sidebar"
    );
    assert!(
        found_status_landmark,
        "Must find status bar landmark in StatusBar"
    );
    assert!(found_tablist, "Must find WAI-ARIA tablist in ConvertView");
}

#[test]
fn test_soak_suite_zero_memory_fd_and_temp_leaks() {
    let sample_pdf_path = get_corpus_pdf("standard-text.pdf");
    let sample_bytes = fs::read(&sample_pdf_path).expect("Read sample PDF");
    let duration_secs = std::env::var("LPDF_SOAK_DURATION_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let config = SoakConfig {
        target_duration_secs: duration_secs,
        min_iterations: 150,
        max_rss_drift_mb: 35.0,
        max_fd_delta: 5,
    };

    let report = run_soak_suite(&config, &sample_bytes).expect("Run soak suite");
    assert!(
        report.passed,
        "Soak suite failed leak limits: {:?}",
        report.violations(&config)
    );
    assert!(
        report.iterations_completed >= 150,
        "Expected at least 150 iterations, got {}",
        report.iterations_completed
    );
    assert!(
        report.operations_executed >= 1500,
        "Expected at least 1500 operations, got {}",
        report.operations_executed
    );
    assert_eq!(
        report.orphaned_temp_files, 0,
        "Orphaned temporary files must be zero"
    );
    assert!(
        report
            .final_open_fds
            .saturating_sub(report.initial_open_fds)
            <= config.max_fd_delta,
        "Open FD delta must not exceed {}",
        config.max_fd_delta
    );
}

#[test]
fn test_soak_leak_detector_catches_violations() {
    let config = SoakConfig {
        target_duration_secs: 0,
        min_iterations: 10,
        max_rss_drift_mb: 10.0,
        max_fd_delta: 2,
    };

    // 1. Simulated excessive RSS drift
    let drift_report = SoakReport {
        iterations_completed: 10,
        operations_executed: 100,
        elapsed_ms: 1000,
        initial_rss_mb: 50.0,
        final_rss_mb: 75.0,
        peak_rss_mb: 80.0,
        rss_drift_mb: 25.0,
        initial_open_fds: 10,
        final_open_fds: 11,
        orphaned_temp_files: 0,
        passed: false,
    };
    assert!(!drift_report.evaluate_leak_limits(&config));
    let violations = drift_report.violations(&config);
    assert!(violations.iter().any(|v| v.contains("RSS drift")));

    // 2. Simulated FD leak
    let fd_leak_report = SoakReport {
        iterations_completed: 10,
        operations_executed: 100,
        elapsed_ms: 1000,
        initial_rss_mb: 50.0,
        final_rss_mb: 52.0,
        peak_rss_mb: 55.0,
        rss_drift_mb: 2.0,
        initial_open_fds: 10,
        final_open_fds: 18,
        orphaned_temp_files: 0,
        passed: false,
    };
    assert!(!fd_leak_report.evaluate_leak_limits(&config));
    let violations = fd_leak_report.violations(&config);
    assert!(violations.iter().any(|v| v.contains("Open FD delta")));

    // 3. Simulated orphaned temp file leak
    let temp_leak_report = SoakReport {
        iterations_completed: 10,
        operations_executed: 100,
        elapsed_ms: 1000,
        initial_rss_mb: 50.0,
        final_rss_mb: 52.0,
        peak_rss_mb: 55.0,
        rss_drift_mb: 2.0,
        initial_open_fds: 10,
        final_open_fds: 10,
        orphaned_temp_files: 3,
        passed: false,
    };
    assert!(!temp_leak_report.evaluate_leak_limits(&config));
    let violations = temp_leak_report.violations(&config);
    assert!(violations
        .iter()
        .any(|v| v.contains("orphaned temporary files")));
}
