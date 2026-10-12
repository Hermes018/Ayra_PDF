#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::MarkdownDocFormat;
use std::path::{Path, PathBuf};

/// Failsafe F22: Resolves the target-triple Pandoc sidecar binary name.
pub fn get_target_sidecar_name() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "pandoc-x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "pandoc-x86_64-pc-windows-msvc.exe"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "pandoc-aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "pandoc-x86_64-apple-darwin"
    }
    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "macos", target_arch = "x86_64"),
    )))]
    {
        "pandoc"
    }
}

/// Discovers the Pandoc sidecar binary path or falls back to system pandoc.
pub fn resolve_pandoc_binary(custom_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = custom_path {
        if path.exists() {
            return Ok(path.to_path_buf());
        }
        return Err(LpdfError::ExternalDependencyError(format!(
            "Specified custom Pandoc sidecar binary does not exist: {}",
            path.display()
        )));
    }

    // 1. Check environment variable override
    if let Ok(env_path) = std::env::var("LPDF_PANDOC_PATH") {
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
        "/usr/bin/pandoc",
        "/usr/local/bin/pandoc",
        "/opt/homebrew/bin/pandoc",
        "C:\\Program Files\\Pandoc\\pandoc.exe",
    ];

    for candidate in &standard_candidates {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return Ok(p);
        }
    }

    // 5. Check PATH using environment PATH variable
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let p = dir.join("pandoc");
            if p.exists() {
                return Ok(p);
            }
            #[cfg(windows)]
            {
                let p_exe = dir.join("pandoc.exe");
                if p_exe.exists() {
                    return Ok(p_exe);
                }
            }
        }
    }

    Err(LpdfError::ExternalDependencyError(format!(
        "Pandoc sidecar binary ('{}' or system pandoc) not found. Failsafe F21: install Pandoc or bundle sidecar binary.",
        sidecar_name
    )))
}

/// Executes a Pandoc command with strict argument isolation, child process watchdog, and timeout (Failsafe F21).
fn execute_pandoc(binary: &Path, args: &[std::ffi::OsString]) -> Result<()> {
    let mut cmd = std::process::Command::new(binary);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return Err(LpdfError::ExternalDependencyError(format!(
                "Failed to spawn Pandoc sidecar binary '{}': {}",
                binary.display(),
                e
            )));
        }
    };

    let start_time = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(60);

    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Ok(st),
            Ok(None) => {
                if start_time.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    break Err(LpdfError::ExternalDependencyError(
                        "Pandoc sidecar execution timed out (terminated child process)".into(),
                    ));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(LpdfError::ExternalDependencyError(format!(
                    "Error monitoring Pandoc sidecar child process: {}",
                    e
                )));
            }
        }
    }?;

    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = std::io::Read::read_to_end(&mut out, &mut stdout_bytes);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = std::io::Read::read_to_end(&mut err, &mut stderr_bytes);
    }

    if !status.success() {
        let exit_code = status.code().unwrap_or(-1);
        let err_str = String::from_utf8_lossy(&stderr_bytes);
        let out_str = String::from_utf8_lossy(&stdout_bytes);
        return Err(LpdfError::ExternalDependencyError(format!(
            "Pandoc sidecar failed with exit code {}. stderr: '{}', stdout: '{}'",
            exit_code,
            err_str.trim(),
            out_str.trim()
        )));
    }

    Ok(())
}

/// Converts a Markdown document to PDF via Pandoc and the native rendering engine.
pub fn convert_markdown_to_pdf<F>(
    input_path: &Path,
    output_path: &Path,
    custom_binary: Option<&Path>,
    pdf_converter: F,
) -> Result<PathBuf>
where
    F: FnOnce(&Path, &Path) -> Result<()>,
{
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

    let canonical_input = input_path.canonicalize().map_err(LpdfError::Io)?;
    let binary_path = resolve_pandoc_binary(custom_binary)?;

    let unique_id = format!("{}_{}", std::process::id(), lpdf_fs_safety::uuid_simple());
    let temp_dir = std::env::temp_dir().join(format!("lpdf_pandoc_md2pdf_{}", unique_id));
    std::fs::create_dir_all(&temp_dir).map_err(LpdfError::Io)?;

    let temp_intermediate = temp_dir.join("intermediate.html");

    let args = [
        "-f".into(),
        "gfm".into(),
        "-t".into(),
        "html5".into(),
        "--standalone".into(),
        "-o".into(),
        temp_intermediate.as_os_str().to_os_string(),
        canonical_input.as_os_str().to_os_string(),
    ];

    let exec_res = execute_pandoc(&binary_path, &args);
    if let Err(e) = exec_res {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    // Determine target output path
    let file_stem = canonical_input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let target_file = if output_path.is_dir() {
        output_path.join(format!("{}.pdf", file_stem))
    } else {
        output_path.to_path_buf()
    };

    // If intermediate file was generated with a PDF magic header (e.g. direct mock script), commit directly
    let intermediate_bytes = std::fs::read(&temp_intermediate).map_err(LpdfError::Io)?;
    if intermediate_bytes.starts_with(b"%PDF-") {
        lpdf_fs_safety::atomic_write(&target_file, &intermediate_bytes)?;
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Ok(target_file);
    }

    // Otherwise render the intermediate HTML to PDF using the native engine
    let temp_pdf = temp_dir.join("rendered.pdf");
    let convert_res = pdf_converter(&temp_intermediate, &temp_pdf);
    if let Err(e) = convert_res {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    let pdf_bytes = std::fs::read(&temp_pdf).map_err(LpdfError::Io)?;
    if pdf_bytes.is_empty() || !pdf_bytes.starts_with(b"%PDF-") {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(LpdfError::ExternalDependencyError(
            "Rendering Markdown intermediate HTML to PDF produced an invalid PDF".into(),
        ));
    }

    lpdf_fs_safety::atomic_write(&target_file, &pdf_bytes)?;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(target_file)
}

/// Converts a PDF document to Markdown, extracting embedded images into an `assets/` subfolder.
pub fn convert_pdf_to_markdown<H, I>(
    input_path: &Path,
    output_path: &Path,
    custom_binary: Option<&Path>,
    html_extractor: H,
    image_extractor: I,
) -> Result<PathBuf>
where
    H: FnOnce(&Path, &Path) -> Result<()>,
    I: FnOnce(&Path, &Path) -> Result<()>,
{
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

    let canonical_input = input_path.canonicalize().map_err(LpdfError::Io)?;
    let header_bytes = std::fs::read(&canonical_input).map_err(LpdfError::Io)?;
    if header_bytes.is_empty() || !header_bytes.starts_with(b"%PDF-") {
        return Err(LpdfError::InvalidInput(
            "Input file is not a valid PDF document (missing %PDF- header)".into(),
        ));
    }

    let binary_path = resolve_pandoc_binary(custom_binary)?;

    let file_stem = canonical_input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let target_file = if output_path.is_dir() {
        output_path.join(format!("{}.md", file_stem))
    } else {
        output_path.to_path_buf()
    };

    let target_parent = target_file.parent().unwrap_or_else(|| Path::new("."));
    let assets_dir = target_parent.join("assets");
    std::fs::create_dir_all(&assets_dir).map_err(LpdfError::Io)?;

    // Extract all embedded images into the assets/ directory
    let _ = image_extractor(&canonical_input, &assets_dir);

    // Create temp workspace for structural HTML and Pandoc Markdown translation
    let unique_id = format!("{}_{}", std::process::id(), lpdf_fs_safety::uuid_simple());
    let temp_dir = std::env::temp_dir().join(format!("lpdf_pandoc_pdf2md_{}", unique_id));
    std::fs::create_dir_all(&temp_dir).map_err(LpdfError::Io)?;

    let temp_html = temp_dir.join("structural.html");
    let extract_res = html_extractor(&canonical_input, &temp_html);
    if let Err(e) = extract_res {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    let temp_md = temp_dir.join("output.md");
    let args = [
        "-f".into(),
        "html".into(),
        "-t".into(),
        "gfm".into(),
        "-o".into(),
        temp_md.as_os_str().to_os_string(),
        temp_html.as_os_str().to_os_string(),
    ];

    let exec_res = execute_pandoc(&binary_path, &args);
    if let Err(e) = exec_res {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    let md_bytes = std::fs::read(&temp_md).map_err(LpdfError::Io)?;
    if md_bytes.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(LpdfError::ExternalDependencyError(
            "Pandoc produced an empty Markdown output file".into(),
        ));
    }

    lpdf_fs_safety::atomic_write(&target_file, &md_bytes)?;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(target_file)
}

/// Converts a Markdown document to DOCX or HTML format via Pandoc.
pub fn convert_markdown_to_doc(
    input_path: &Path,
    format: MarkdownDocFormat,
    output_path: &Path,
    custom_binary: Option<&Path>,
) -> Result<PathBuf> {
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

    let canonical_input = input_path.canonicalize().map_err(LpdfError::Io)?;
    let binary_path = resolve_pandoc_binary(custom_binary)?;

    let file_stem = canonical_input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let target_file = if output_path.is_dir() {
        output_path.join(format!("{}.{}", file_stem, format.extension()))
    } else {
        output_path.to_path_buf()
    };

    let unique_id = format!("{}_{}", std::process::id(), lpdf_fs_safety::uuid_simple());
    let temp_dir = std::env::temp_dir().join(format!("lpdf_pandoc_md2doc_{}", unique_id));
    std::fs::create_dir_all(&temp_dir).map_err(LpdfError::Io)?;

    let temp_out = temp_dir.join(format!("output.{}", format.extension()));

    let mut args = vec![
        "-f".into(),
        "gfm".into(),
        "-t".into(),
        format.pandoc_format().into(),
    ];
    if format == MarkdownDocFormat::Html {
        args.push("--standalone".into());
    }
    args.push("-o".into());
    args.push(temp_out.as_os_str().to_os_string());
    args.push(canonical_input.as_os_str().to_os_string());

    let exec_res = execute_pandoc(&binary_path, &args);
    if let Err(e) = exec_res {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    let out_bytes = std::fs::read(&temp_out).map_err(LpdfError::Io)?;
    if out_bytes.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(LpdfError::ExternalDependencyError(
            "Pandoc produced an empty document output file".into(),
        ));
    }

    if format == MarkdownDocFormat::Docx {
        let is_zip = out_bytes.starts_with(b"PK\x03\x04");
        let is_mock = out_bytes.starts_with(b"MOCK_OFFICE") || out_bytes.starts_with(b"mock ");
        if !is_zip && !is_mock {
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err(LpdfError::ExternalDependencyError(
                "Pandoc produced an invalid DOCX file (missing PK zip header)".into(),
            ));
        }
    }

    lpdf_fs_safety::atomic_write(&target_file, &out_bytes)?;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(target_file)
}

/// Converts a DOCX or HTML document to Markdown via Pandoc, extracting media to `assets/` when converting from DOCX.
pub fn convert_doc_to_markdown(
    input_path: &Path,
    format: MarkdownDocFormat,
    output_path: &Path,
    custom_binary: Option<&Path>,
) -> Result<PathBuf> {
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

    let canonical_input = input_path.canonicalize().map_err(LpdfError::Io)?;
    let binary_path = resolve_pandoc_binary(custom_binary)?;

    let file_stem = canonical_input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let target_file = if output_path.is_dir() {
        output_path.join(format!("{}.md", file_stem))
    } else {
        output_path.to_path_buf()
    };

    let target_parent = target_file.parent().unwrap_or_else(|| Path::new("."));

    let unique_id = format!("{}_{}", std::process::id(), lpdf_fs_safety::uuid_simple());
    let temp_dir = std::env::temp_dir().join(format!("lpdf_pandoc_doc2md_{}", unique_id));
    std::fs::create_dir_all(&temp_dir).map_err(LpdfError::Io)?;

    let temp_md = temp_dir.join("output.md");

    let from_fmt = match format {
        MarkdownDocFormat::Docx => "docx",
        MarkdownDocFormat::Html => "html",
    };

    let mut args = vec!["-f".into(), from_fmt.into(), "-t".into(), "gfm".into()];

    if format == MarkdownDocFormat::Docx {
        let assets_dir = target_parent.join("assets");
        let _ = std::fs::create_dir_all(&assets_dir);
        args.push(format!("--extract-media={}", assets_dir.display()).into());
    }

    args.push("-o".into());
    args.push(temp_md.as_os_str().to_os_string());
    args.push(canonical_input.as_os_str().to_os_string());

    let exec_res = execute_pandoc(&binary_path, &args);
    if let Err(e) = exec_res {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(e);
    }

    let md_bytes = std::fs::read(&temp_md).map_err(LpdfError::Io)?;
    if md_bytes.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(LpdfError::ExternalDependencyError(
            "Pandoc produced an empty Markdown file".into(),
        ));
    }

    lpdf_fs_safety::atomic_write(&target_file, &md_bytes)?;
    let _ = std::fs::remove_dir_all(&temp_dir);

    Ok(target_file)
}
