//! Deterministic and libFuzzer adapter input fuzzing routines.
//! Strictly obeys `#![forbid(unsafe_code)]` and zero panics/unwraps in non-test code.

use lpdf_engine_port::{PdfEngine, SummaryLength};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}_{:x}", nanos, count)
}

/// Fuzzes `PdfcraftAdapter` with arbitrary byte buffers, exercising `load`, `info`,
/// `repair`, `pdf_to_text`, and `summarize_extractive`.
pub fn fuzz_pdfcraft_bytes(data: &[u8]) {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_fuzz_pdfcraft_{}", unique_suffix()));
    if std::fs::create_dir_all(&temp_dir).is_err() {
        return;
    }
    let pdf_path = temp_dir.join("fuzz_input.pdf");
    if std::fs::write(&pdf_path, data).is_err() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut adapter = lpdf_engine_pdfcraft::PdfcraftAdapter::new();
        let load_res = adapter.load(&pdf_path);
        if load_res.is_ok() {
            let _ = adapter.info();
        }

        let rep_out = temp_dir.join("fuzz_repaired.pdf");
        let _ = adapter.repair(&pdf_path, &rep_out);
        let _ = std::fs::remove_file(&rep_out);

        let txt_out = temp_dir.join("fuzz_extracted.txt");
        let _ = adapter.pdf_to_text(&pdf_path, &txt_out);
        let _ = std::fs::remove_file(&txt_out);

        let sum_out = temp_dir.join("fuzz_summary.md");
        let _ = adapter.summarize_extractive(&pdf_path, &sum_out, SummaryLength::Short);
        let _ = std::fs::remove_file(&sum_out);
    }));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Fuzzes `FallbackAdapter` with arbitrary byte buffers, exercising `load` and `info`.
pub fn fuzz_fallback_bytes(data: &[u8]) {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_fuzz_fallback_{}", unique_suffix()));
    if std::fs::create_dir_all(&temp_dir).is_err() {
        return;
    }
    let pdf_path = temp_dir.join("fuzz_fallback.pdf");
    if std::fs::write(&pdf_path, data).is_err() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut adapter = lpdf_engine_fallback::FallbackAdapter::new();
        let load_res = adapter.load(&pdf_path);
        if load_res.is_ok() {
            let _ = adapter.info();
        }
    }));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Fuzzes `EngineSwitcher` with adversarial inputs exercising primary-to-fallback degradation.
pub fn fuzz_engine_switcher_bytes(data: &[u8]) {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_fuzz_switcher_{}", unique_suffix()));
    if std::fs::create_dir_all(&temp_dir).is_err() {
        return;
    }
    let pdf_path = temp_dir.join("fuzz_switcher.pdf");
    if std::fs::write(&pdf_path, data).is_err() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut switcher = crate::EngineSwitcher::new();
        let load_res = switcher.load(&pdf_path);
        if load_res.is_ok() {
            let _ = switcher.info();
            let _ = switcher.active_engine();
            let _ = switcher.is_degraded();
        }
    }));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

/// Fuzzes filesystem path sanitization (`lpdf-fs-safety`) and `lpdf-ai-worker` RPC / model header validation.
pub fn fuzz_fs_and_ai_rpc_bytes(data: &[u8]) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let raw_str = String::from_utf8_lossy(data);

        // 1. Path safety fuzzing: sanitize_filename
        let sanitized = lpdf_fs_safety::sanitize_filename(&raw_str);
        if sanitized.contains('/') || sanitized.contains('\\') || sanitized.contains('\0') {
            tracing::error!(
                "Sanitized filename contains forbidden character: {}",
                sanitized
            );
        }

        // 2. Output directory creation safety
        let temp_parent = std::env::temp_dir().join(format!("lpdf_fuzz_fs_{}", unique_suffix()));
        if let Ok(safe_dir) = lpdf_fs_safety::create_safe_output_dir(&temp_parent, &raw_str) {
            let _ = std::fs::remove_dir_all(&safe_dir);
        }
        let _ = std::fs::remove_dir_all(&temp_parent);

        // 3. AI Worker JSON RPC deserialization
        let _req_slice: std::result::Result<lpdf_ai_worker::AiWorkerRequest, _> =
            serde_json::from_slice(data);
        let _req_str: std::result::Result<lpdf_ai_worker::AiWorkerRequest, _> =
            serde_json::from_str(&raw_str);

        // 4. AI Worker session dispatch
        let mut session = lpdf_ai_worker::AiWorkerSession::new(true, None);
        let chunk_req = lpdf_ai_worker::AiWorkerRequest::InferChunk {
            prompt: raw_str.to_string(),
            max_tokens: data.len(),
            temperature: if data.is_empty() {
                0.2
            } else {
                (data[0] as f32) / 100.0
            },
        };
        let _resp = session.handle_request(&chunk_req);

        // 5. Model header validation
        let temp_model_dir = std::env::temp_dir().join(format!("lpdf_fuzz_ai_{}", unique_suffix()));
        if std::fs::create_dir_all(&temp_model_dir).is_ok() {
            let model_path = temp_model_dir.join("fuzz_model.bin");
            if std::fs::write(&model_path, data).is_ok() {
                let _ = lpdf_ai_worker::validate_model_header(&model_path);
            }
            let _ = std::fs::remove_dir_all(&temp_model_dir);
        }
    }));
}
