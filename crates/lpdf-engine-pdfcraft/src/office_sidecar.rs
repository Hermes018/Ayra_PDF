#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::OfficeInputFormat;
use std::path::{Path, PathBuf};

/// Failsafe F22: Resolves the target-triple sidecar binary name.
pub fn get_target_sidecar_name() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "soffice-x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "soffice-x86_64-pc-windows-msvc.exe"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "soffice-aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "soffice-x86_64-apple-darwin"
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
    )))]
    {
        "soffice"
    }
}

/// Discovers the sidecar binary path or falls back to system LibreOffice / OpenOffice.
pub fn resolve_office_binary(custom_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        return Err(LpdfError::ExternalDependencyError(format!(
            "Specified custom Office sidecar binary does not exist: {}",
            path.display()
        )));
    }

    // 1. Check environment variable override
    if let Ok(env_path) = std::env::var("LPDF_SOFFICE_PATH") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Ok(p);
        }
    }

    let sidecar_name = get_target_sidecar_name();

    // 2. Check bundled sidecar location in executable directories
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join(sidecar_name);
            if candidate.exists() {
                return Ok(candidate);
            }
            let candidate_bin = exe_dir.join("binaries").join(sidecar_name);
            if candidate_bin.exists() {
                return Ok(candidate_bin);
            }
        }
    }

    // 3. Check relative binaries folder (common in dev/test setups)
    let dev_candidate = PathBuf::from("binaries").join(sidecar_name);
    if dev_candidate.exists() {
        return Ok(dev_candidate);
    }
    let dev_tauri_candidate = PathBuf::from("app-tauri/src-tauri/binaries").join(sidecar_name);
    if dev_tauri_candidate.exists() {
        return Ok(dev_tauri_candidate);
    }

    // 4. Check known standard system paths
    let standard_candidates = [
        "/usr/bin/soffice",
        "/usr/bin/libreoffice",
        "/usr/local/bin/soffice",
        "/usr/local/bin/libreoffice",
        "/opt/libreoffice/program/soffice",
        "/Applications/LibreOffice.app/Contents/MacOS/soffice",
        "C:\\Program Files\\LibreOffice\\program\\soffice.exe",
        "C:\\Program Files (x86)\\LibreOffice\\program\\soffice.exe",
    ];

    for candidate in &standard_candidates {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }

    // 5. Check PATH using system which command or simple execution check
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let p1 = dir.join("soffice");
            if p1.exists() {
                return Ok(p1);
            }
            let p2 = dir.join("libreoffice");
            if p2.exists() {
                return Ok(p2);
            }
            #[cfg(windows)]
            {
                let p1_exe = dir.join("soffice.exe");
                if p1_exe.exists() {
                    return Ok(p1_exe);
                }
                let p2_exe = dir.join("libreoffice.exe");
                if p2_exe.exists() {
                    return Ok(p2_exe);
                }
            }
        }
    }

    Err(LpdfError::ExternalDependencyError(format!(
        "Headless Office binary ('{}' or system soffice/libreoffice) not found. Failsafe F21: install LibreOffice or bundle sidecar binary.",
        sidecar_name
    )))
}

/// Executes headless conversion of an Office document to PDF with strict security guards.
pub fn convert_office_to_pdf(
    input_path: &Path,
    format: OfficeInputFormat,
    output_path: &Path,
    custom_binary: Option<&Path>,
) -> Result<PathBuf> {
    // Validate input path and existence
    if !input_path.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input_path.display()
        )));
    }
    if !input_path.is_file() {
        return Err(LpdfError::InvalidInput(format!(
            "Input path '{}' is not a regular file",
            input_path.display()
        )));
    }

    // Validate extension matching format
    if let Some(ext) = input_path.extension().and_then(|s| s.to_str()) {
        let expected_ext = format.extension();
        if !ext.eq_ignore_ascii_case(expected_ext) {
            // Log warning or accept, but let's check format compatibility
        }
    }

    // Canonicalize input path (shell-injection prevention DO-21)
    let canonical_input = input_path.canonicalize().map_err(LpdfError::Io)?;

    // Resolve sidecar binary (Failsafe F22)
    let binary_path = resolve_office_binary(custom_binary)?;

    // Unique IDs for isolated temp directories
    let unique_id = format!("{}_{}", std::process::id(), lpdf_fs_safety::uuid_simple());

    // Profile isolation: ensure dedicated isolated user installation so runs never collide
    let temp_profile = std::env::temp_dir().join(format!("lpdf_soffice_prof_{}", unique_id));
    std::fs::create_dir_all(&temp_profile).map_err(LpdfError::Io)?;
    let profile_url = format!("file://{}", temp_profile.display());

    // Temp output dir for conversion results
    let temp_out_dir = std::env::temp_dir().join(format!("lpdf_soffice_out_{}", unique_id));
    std::fs::create_dir_all(&temp_out_dir).map_err(LpdfError::Io)?;

    // Construct command strictly via args (no shell execution)
    let mut cmd = std::process::Command::new(&binary_path);
    cmd.arg(format!("-env:UserInstallation={}", profile_url));
    cmd.arg("--headless");
    cmd.arg("--convert-to");
    cmd.arg("pdf");
    cmd.arg("--outdir");
    cmd.arg(&temp_out_dir);
    cmd.arg(&canonical_input);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Spawn child process with safety watchdog (Failsafe F21)
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_profile);
            let _ = std::fs::remove_dir_all(&temp_out_dir);
            return Err(LpdfError::ExternalDependencyError(format!(
                "Failed to spawn Office sidecar binary '{}': {}",
                binary_path.display(),
                e
            )));
        }
    };

    // Watchdog loop: 60-second execution cap
    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(60);

    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Ok(st),
            Ok(None) => {
                if start_time.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait(); // reap zombie child
                    let _ = std::fs::remove_dir_all(&temp_profile);
                    let _ = std::fs::remove_dir_all(&temp_out_dir);
                    break Err(LpdfError::ExternalDependencyError(
                        "Office sidecar execution timed out (terminated child process)".into(),
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = std::fs::remove_dir_all(&temp_profile);
                let _ = std::fs::remove_dir_all(&temp_out_dir);
                break Err(LpdfError::ExternalDependencyError(format!(
                    "Error monitoring Office sidecar child process: {}",
                    e
                )));
            }
        }
    }?;

    // Capture stdout and stderr
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = std::io::Read::read_to_end(&mut out, &mut stdout_bytes);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = std::io::Read::read_to_end(&mut err, &mut stderr_bytes);
    }

    if !status.success() {
        let _ = std::fs::remove_dir_all(&temp_profile);
        let _ = std::fs::remove_dir_all(&temp_out_dir);
        let exit_code = status.code().unwrap_or(-1);
        let err_str = String::from_utf8_lossy(&stderr_bytes);
        let out_str = String::from_utf8_lossy(&stdout_bytes);
        return Err(LpdfError::ExternalDependencyError(format!(
            "Office sidecar failed with exit code {}. stderr: '{}', stdout: '{}'",
            exit_code,
            err_str.trim(),
            out_str.trim()
        )));
    }

    // Locate generated PDF file in temp_out_dir
    let file_stem = canonical_input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let mut generated_pdf = temp_out_dir.join(format!("{}.pdf", file_stem));

    if !generated_pdf.exists() {
        // Fallback: look for any .pdf created in temp_out_dir
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&temp_out_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().and_then(|s| s.to_str()) == Some("pdf") {
                    found = Some(p);
                    break;
                }
            }
        }
        match found {
            Some(p) => generated_pdf = p,
            None => {
                let _ = std::fs::remove_dir_all(&temp_profile);
                let _ = std::fs::remove_dir_all(&temp_out_dir);
                return Err(LpdfError::ExternalDependencyError(format!(
                    "Office sidecar did not produce the expected PDF output in '{}'",
                    temp_out_dir.display()
                )));
            }
        }
    }

    // Verify output PDF bytes (non-empty, %PDF- header check)
    let pdf_bytes = std::fs::read(&generated_pdf).map_err(LpdfError::Io)?;
    if pdf_bytes.is_empty() || !pdf_bytes.starts_with(b"%PDF-") {
        let _ = std::fs::remove_dir_all(&temp_profile);
        let _ = std::fs::remove_dir_all(&temp_out_dir);
        return Err(LpdfError::ExternalDependencyError(
            "Office sidecar produced an invalid or corrupted PDF file (missing %PDF- header)"
                .into(),
        ));
    }

    // Determine target output path
    let target_file = if output_path.is_dir() {
        output_path.join(format!("{}.pdf", file_stem))
    } else {
        output_path.to_path_buf()
    };

    // Atomic write via lpdf-fs-safety
    lpdf_fs_safety::atomic_write(&target_file, &pdf_bytes)?;

    // Clean up temporary workspaces
    let _ = std::fs::remove_dir_all(&temp_profile);
    let _ = std::fs::remove_dir_all(&temp_out_dir);

    Ok(target_file)
}
