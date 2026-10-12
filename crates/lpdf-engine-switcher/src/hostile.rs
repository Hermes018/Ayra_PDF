//! Hostile PDF generators and parser hardening test vectors.
//! Strictly obeys `#![forbid(unsafe_code)]` and zero unwraps/panics in non-test code.

#![forbid(unsafe_code)]

/// Decompression / Flate Bomb: A PDF containing a declared /Length mismatch
/// and nested /FlateDecode streams designed to trigger unbounded allocation.
pub fn generate_flate_bomb_pdf() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    out.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n");
    // Astronomical declared length to verify memory bounds enforcement
    out.extend_from_slice(b"4 0 obj\n<< /Length 1073741824 /Filter /FlateDecode >>\nstream\n");
    out.extend_from_slice(&[
        0x78, 0x9c, 0xec, 0xc1, 0x81, 0x00, 0x00, 0x00, 0x00, 0x80, 0xa0, 0xfd, 0xa3, 0x01, 0x00,
        0x00, 0x00,
    ]);
    out.extend_from_slice(b"\nendstream\nendobj\n");
    out.extend_from_slice(b"xref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000201 00000 n \n");
    out.extend_from_slice(b"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n380\n%%EOF\n");
    out
}

/// Deep Page Tree: A deeply nested /Pages hierarchy (e.g. 500 levels).
pub fn generate_deep_page_tree_pdf(depth: usize) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    let safe_depth = depth.clamp(10, 1000);
    for i in 2..safe_depth + 2 {
        if i == safe_depth + 1 {
            // Leaf Page
            let line = format!(
                "{} 0 obj\n<< /Type /Page /Parent {} 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
                i,
                i - 1
            );
            out.extend_from_slice(line.as_bytes());
        } else {
            // Intermediate Pages node
            let line = format!(
                "{} 0 obj\n<< /Type /Pages /Parent {} 0 R /Kids [{} 0 R] /Count 1 >>\nendobj\n",
                i,
                if i == 2 { 1 } else { i - 1 },
                i + 1
            );
            out.extend_from_slice(line.as_bytes());
        }
    }
    let total_objs = safe_depth + 2;
    out.extend_from_slice(b"xref\n0 1\n0000000000 65535 f \n");
    let trailer = format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n10\n%%EOF\n",
        total_objs
    );
    out.extend_from_slice(trailer.as_bytes());
    out
}

/// Cyclic Page Tree: A self-referential /Pages graph (3 0 obj -> /Kids [4 0 R], 4 0 obj -> /Kids [3 0 R]).
pub fn generate_cyclic_page_tree_pdf() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 2 >>\nendobj\n");
    out.extend_from_slice(b"3 0 obj\n<< /Type /Pages /Kids [2 0 R] /Count 2 >>\nendobj\n");
    out.extend_from_slice(b"xref\n0 4\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000116 00000 n \n");
    out.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n174\n%%EOF\n");
    out
}

/// Corrupted / Looping Xref Tables: Negative offsets, out-of-bounds byte offsets, and circular /Prev trailer pointers.
pub fn generate_broken_xref_pdf() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [] /Count 0 >>\nendobj\n");
    out.extend_from_slice(b"xref\n0 3\n");
    out.extend_from_slice(b"0000000000 65535 f \n");
    out.extend_from_slice(b"-000000001 00000 n \n");
    out.extend_from_slice(b"9999999999 00000 n \n");
    out.extend_from_slice(
        b"trailer\n<< /Size 3 /Root 1 0 R /Prev 9999999999 >>\nstartxref\n120\n%%EOF\n",
    );
    out
}

/// Malicious Actions & Embedded Payloads: Executable action dictionaries (/OpenAction, /AA, /JavaScript, /Launch).
pub fn generate_js_launch_pdf() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R /OpenAction 4 0 R /Names << /JavaScript 5 0 R >> /AA << /O 6 0 R >> >>\nendobj\n");
    out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
    out.extend_from_slice(b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /AA << /O 4 0 R >> >>\nendobj\n");
    // OpenAction JavaScript
    out.extend_from_slice(b"4 0 obj\n<< /Type /Action /S /JavaScript /JS (app.alert('malicious_js_executed');) >>\nendobj\n");
    // Names JavaScript dictionary
    out.extend_from_slice(b"5 0 obj\n<< /Names [(Exploit) << /S /JavaScript /JS (util.printd('evil');) >>] >>\nendobj\n");
    // Launch action attempting to spawn external process
    out.extend_from_slice(b"6 0 obj\n<< /Type /Action /S /Launch /F (calc.exe) /Win << /F (powershell.exe) /P (-Command Start-Process evil) >> >>\nendobj\n");
    out.extend_from_slice(b"xref\n0 7\n0000000000 65535 f \n0000000009 00000 n \n0000000130 00000 n \n0000000187 00000 n \n0000000280 00000 n \n0000000380 00000 n \n0000000470 00000 n \n");
    out.extend_from_slice(b"trailer\n<< /Size 7 /Root 1 0 R >>\nstartxref\n620\n%%EOF\n");
    out
}

/// Pathological Page Geometry: Zero-area, inverted, and astronomical floating point bounds.
pub fn generate_pathological_geometry_pdf() -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
    out.extend_from_slice(
        b"2 0 obj\n<< /Type /Pages /Kids [3 0 R 4 0 R 5 0 R] /Count 3 >>\nendobj\n",
    );
    // Page 1: zero-area MediaBox
    out.extend_from_slice(
        b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 0 0] >>\nendobj\n",
    );
    // Page 2: inverted MediaBox (UR < LL)
    out.extend_from_slice(
        b"4 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [612 792 0 0] >>\nendobj\n",
    );
    // Page 3: astronomical dimensions (floating point stress)
    out.extend_from_slice(b"5 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [-100000000000 -100000000000 100000000000 100000000000] /CropBox [999999999 999999999 10 10] >>\nendobj\n");
    out.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000130 00000 n \n0000000210 00000 n \n0000000295 00000 n \n");
    out.extend_from_slice(b"trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n450\n%%EOF\n");
    out
}

/// Hostile path candidates for path traversal, null-byte injection, and device-name attacks.
pub fn hostile_path_candidates() -> &'static [&'static str] {
    &[
        "../../../../../../etc/passwd",
        "..\\..\\..\\Windows\\System32\\cmd.exe",
        "file:///etc/shadow",
        "CON.pdf",
        "PRN.pdf",
        "AUX.pdf",
        "NUL.pdf",
        "COM1.pdf",
        "LPT1.pdf",
        "normal_prefix\0evil_suffix.pdf",
        "   leading_and_trailing_spaces.pdf   ",
        "......trailing_dots....",
        "\x01\x02\x03\x1b\x7f_control_chars.pdf",
        "a/b/c/d/e/../../../../../../root.pdf",
    ]
}

/// Exercises all major adapter operations (load, info, repair, rotate, compress,
/// pdf_to_text, summarize_extractive, redact) against a hostile input file.
/// Asserts zero unwinds, execution timeout bounding (<2s), atomic fail-closed behavior,
/// and that the input file remains untouched.
pub fn exercise_all_operations_on_hostile_file(
    engine: &mut dyn lpdf_engine_port::PdfEngine,
    input_path: &std::path::Path,
    work_dir: &std::path::Path,
) -> std::result::Result<(), String> {
    let initial_bytes = match std::fs::read(input_path) {
        Ok(b) => b,
        Err(e) => return Err(format!("Failed to read input hostile file: {}", e)),
    };
    let initial_len = initial_bytes.len();

    // 1. load
    let start = std::time::Instant::now();
    let load_res =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| engine.load(input_path)));
    if load_res.is_err() {
        return Err("engine.load panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.load exceeded 2s execution timeout".into());
    }

    // 2. info (if load succeeded)
    if let Ok(Ok(_)) = load_res {
        let start = std::time::Instant::now();
        let info_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| engine.info()));
        if info_res.is_err() {
            return Err("engine.info panicked on hostile file".into());
        }
        if start.elapsed() >= std::time::Duration::from_secs(2) {
            return Err("engine.info exceeded 2s execution timeout".into());
        }
    }

    // 3. repair
    let out_repair = work_dir.join("hostile_repaired.pdf");
    let start = std::time::Instant::now();
    let rep_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.repair(input_path, &out_repair)
    }));
    if rep_res.is_err() {
        return Err("engine.repair panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.repair exceeded 2s execution timeout".into());
    }
    let _ = std::fs::remove_file(&out_repair);

    // 4. rotate_document
    let out_rotate = work_dir.join("hostile_rotated.pdf");
    let start = std::time::Instant::now();
    let rot_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.rotate_document(input_path, 90, &out_rotate)
    }));
    if rot_res.is_err() {
        return Err("engine.rotate_document panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.rotate_document exceeded 2s execution timeout".into());
    }
    let _ = std::fs::remove_file(&out_rotate);

    // 5. compress
    let out_compress = work_dir.join("hostile_compressed.pdf");
    let start = std::time::Instant::now();
    let comp_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.compress(
            input_path,
            lpdf_engine_port::OptimizeProfile::Medium,
            &out_compress,
        )
    }));
    if comp_res.is_err() {
        return Err("engine.compress panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.compress exceeded 2s execution timeout".into());
    }
    let _ = std::fs::remove_file(&out_compress);

    // 6. pdf_to_text
    let out_text = work_dir.join("hostile_text.txt");
    let start = std::time::Instant::now();
    let txt_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.pdf_to_text(input_path, &out_text)
    }));
    if txt_res.is_err() {
        return Err("engine.pdf_to_text panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.pdf_to_text exceeded 2s execution timeout".into());
    }
    let _ = std::fs::remove_file(&out_text);

    // 7. summarize_extractive
    let out_sum = work_dir.join("hostile_summary.md");
    let start = std::time::Instant::now();
    let sum_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.summarize_extractive(input_path, &out_sum, lpdf_engine_port::SummaryLength::Short)
    }));
    if sum_res.is_err() {
        return Err("engine.summarize_extractive panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.summarize_extractive exceeded 2s execution timeout".into());
    }
    let _ = std::fs::remove_file(&out_sum);

    // 8. redact
    let out_redact = work_dir.join("hostile_redacted.pdf");
    let start = std::time::Instant::now();
    let red_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        engine.redact(input_path, vec![], &out_redact)
    }));
    if red_res.is_err() {
        return Err("engine.redact panicked on hostile file".into());
    }
    if start.elapsed() >= std::time::Duration::from_secs(2) {
        return Err("engine.redact exceeded 2s execution timeout".into());
    }
    let _ = std::fs::remove_file(&out_redact);

    // Check that original input file is completely untouched
    let after_bytes = match std::fs::read(input_path) {
        Ok(b) => b,
        Err(e) => {
            return Err(format!(
                "Failed to re-read input file after operations: {}",
                e
            ))
        }
    };
    if after_bytes.len() != initial_len || after_bytes != initial_bytes {
        return Err("Original hostile file was modified by engine operations!".into());
    }

    Ok(())
}
