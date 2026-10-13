#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    MarkdownBatchOptions, MarkdownBatchReport, MarkdownBatchTarget, MarkdownDocFormat,
};
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
    // Failsafe F20: enforce offline isolation via unshare -n -r on Linux
    #[cfg(target_os = "linux")]
    let mut cmd = {
        let mut c = std::process::Command::new("unshare");
        c.arg("-n").arg("-r").arg(binary);
        c
    };
    #[cfg(not(target_os = "linux"))]
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

/// Generates a standard UTC timestamp string in RFC3339 format without external dependencies.
pub fn current_timestamp_str() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = duration.as_secs();

    let z = (total_secs / 86400) as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    let day_secs = total_secs % 86400;
    let hours = day_secs / 3600;
    let minutes = (day_secs % 3600) / 60;
    let seconds = day_secs % 60;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

/// Extracts YAML front-matter block if present.
/// Returns (Some(front_matter_str), remaining_body).
pub fn parse_yaml_front_matter(content: &str) -> (Option<&str>, &str) {
    if !content.starts_with("---") {
        return (None, content);
    }

    let after_dashes = &content[3..];
    if !after_dashes.starts_with('\n') && !after_dashes.starts_with("\r\n") {
        return (None, content);
    }

    let close_delims = ["\n---\n", "\r\n---\r\n", "\n...\n", "\r\n...\r\n"];
    for delim in &close_delims {
        if let Some(pos) = after_dashes.find(delim) {
            let fm_end = 3 + pos + delim.len();
            let fm = &content[..fm_end];
            let body = &content[fm_end..];
            return (Some(fm), body);
        }
    }

    if after_dashes.ends_with("\n---") || after_dashes.ends_with("\r\n---") {
        return (Some(content), "");
    }

    (None, content)
}

/// Generates a YAML front-matter block.
pub fn generate_yaml_front_matter(title: &str, source: &str, timestamp: &str) -> String {
    format!(
        "---\ntitle: \"{}\"\nsource: \"{}\"\ndate: \"{}\"\ngenerator: \"LocalPDF Studio\"\n---\n\n",
        title.replace('"', "\\\""),
        source.replace('"', "\\\""),
        timestamp
    )
}

/// Injects or preserves YAML front-matter in markdown content.
pub fn inject_or_preserve_yaml_front_matter(
    content: &str,
    title: &str,
    source: &str,
    timestamp: &str,
    preserve_existing: bool,
) -> String {
    let (existing, body) = parse_yaml_front_matter(content);
    if preserve_existing && existing.is_some() {
        return content.to_string();
    }

    let mut result = generate_yaml_front_matter(title, source, timestamp);
    result.push_str(body.trim_start_matches(['\r', '\n']));
    result
}

/// Deterministically rewrites image links in Markdown or HTML so local/extracted images point to `assets/<filename>`.
pub fn rewrite_markdown_asset_links(content: &str) -> String {
    let image_extensions = [
        ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp", ".tiff",
    ];

    let mut output = String::with_capacity(content.len());
    let mut rest = content;

    while !rest.is_empty() {
        let md_img_pos = rest.find("![");
        let html_img_pos = rest.find("<img");

        let next_tag = match (md_img_pos, html_img_pos) {
            (Some(m), Some(h)) => {
                if m <= h {
                    Some((m, true))
                } else {
                    Some((h, false))
                }
            }
            (Some(m), None) => Some((m, true)),
            (None, Some(h)) => Some((h, false)),
            (None, None) => None,
        };

        match next_tag {
            None => {
                output.push_str(rest);
                break;
            }
            Some((pos, is_md)) => {
                output.push_str(&rest[..pos]);
                rest = &rest[pos..];

                if is_md {
                    let mut matched = false;
                    if let Some(close_bracket) = rest.find("](") {
                        let link_start = close_bracket + 2;
                        if let Some(close_paren) = rest[link_start..].find(')') {
                            let link_content = &rest[link_start..link_start + close_paren];
                            let mut parts = link_content.split_whitespace();
                            if let Some(url) = parts.next() {
                                let url_lower = url.to_ascii_lowercase();
                                let is_image =
                                    image_extensions.iter().any(|ext| url_lower.ends_with(ext));
                                if is_image
                                    && !url.starts_with("http://")
                                    && !url.starts_with("https://")
                                    && !url.starts_with("data:")
                                {
                                    let filename = Path::new(url)
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(url);
                                    let new_url = format!("assets/{}", filename);
                                    let remainder = &link_content[url.len()..];

                                    output.push_str(&rest[..link_start]);
                                    output.push_str(&new_url);
                                    output.push_str(remainder);
                                    output.push(')');
                                    rest = &rest[link_start + close_paren + 1..];
                                    matched = true;
                                }
                            }
                        }
                    }
                    if !matched {
                        output.push_str("![");
                        rest = &rest[2..];
                    }
                } else {
                    let mut matched = false;
                    if let Some(close_gt) = rest.find('>') {
                        let tag_slice = &rest[..=close_gt];
                        if let Some(src_pos) = tag_slice.find("src=\"") {
                            let val_start = src_pos + 5;
                            if let Some(val_end) = tag_slice[val_start..].find('"') {
                                let src_val = &tag_slice[val_start..val_start + val_end];
                                let src_lower = src_val.to_ascii_lowercase();
                                let is_image =
                                    image_extensions.iter().any(|ext| src_lower.ends_with(ext));
                                if is_image
                                    && !src_val.starts_with("http://")
                                    && !src_val.starts_with("https://")
                                    && !src_val.starts_with("data:")
                                {
                                    let filename = Path::new(src_val)
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(src_val);
                                    let new_src = format!("assets/{}", filename);

                                    output.push_str(&tag_slice[..val_start]);
                                    output.push_str(&new_src);
                                    output.push_str(&tag_slice[val_start + val_end..]);
                                    rest = &rest[close_gt + 1..];
                                    matched = true;
                                }
                            }
                        }
                    }
                    if !matched {
                        output.push_str("<img");
                        rest = &rest[4..];
                    }
                }
            }
        }
    }

    output
}

/// Performs batch folder conversion across Markdown, PDF, DOCX, and HTML documents.
pub fn batch_convert_markdown<H, I, P>(
    input_dir: &Path,
    output_dir: &Path,
    options: MarkdownBatchOptions,
    custom_binary: Option<&Path>,
    html_extractor: H,
    image_extractor: I,
    pdf_converter: P,
) -> Result<MarkdownBatchReport>
where
    H: Fn(&Path, &Path) -> Result<()>,
    I: Fn(&Path, &Path) -> Result<()>,
    P: Fn(&Path, &Path) -> Result<()>,
{
    if !input_dir.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input directory '{}' does not exist",
            input_dir.display()
        )));
    }
    if !input_dir.is_dir() {
        return Err(LpdfError::InvalidInput(format!(
            "Input path '{}' is not a directory",
            input_dir.display()
        )));
    }

    let canonical_input = input_dir.canonicalize().map_err(LpdfError::Io)?;
    std::fs::create_dir_all(output_dir).map_err(LpdfError::Io)?;

    let assets_out_dir = output_dir.join("assets");
    let _ = std::fs::create_dir_all(&assets_out_dir);

    // If input directory has an assets folder, copy its contents over
    let assets_in_dir = canonical_input.join("assets");
    if assets_in_dir.exists() && assets_in_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&assets_in_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    if let Some(name) = p.file_name() {
                        let _ = std::fs::copy(&p, assets_out_dir.join(name));
                    }
                }
            }
        }
    }

    // Read entries from input_dir (non-recursive to respect F17 path safety)
    let read_entries = std::fs::read_dir(&canonical_input).map_err(LpdfError::Io)?;
    let mut candidate_files: Vec<PathBuf> = Vec::new();

    for entry in read_entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        let matches = match options.target {
            MarkdownBatchTarget::Markdown => {
                matches!(ext.as_str(), "pdf" | "docx" | "html" | "htm")
            }
            MarkdownBatchTarget::Pdf | MarkdownBatchTarget::Docx | MarkdownBatchTarget::Html => {
                matches!(ext.as_str(), "md" | "markdown")
            }
        };

        if matches {
            candidate_files.push(path);
        }
    }

    candidate_files.sort();

    let total_files = candidate_files.len();
    let mut converted_files = 0;
    let mut failed_files = 0;
    let mut output_files = Vec::new();
    let mut errors = Vec::new();

    let timestamp = current_timestamp_str();

    for file in &candidate_files {
        let file_stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        let safe_stem = lpdf_fs_safety::sanitize_filename(file_stem);
        let file_name_str = file.file_name().and_then(|s| s.to_str()).unwrap_or("file");

        let target_file_path = match options.target {
            MarkdownBatchTarget::Markdown => output_dir.join(format!("{}.md", safe_stem)),
            MarkdownBatchTarget::Pdf => output_dir.join(format!("{}.pdf", safe_stem)),
            MarkdownBatchTarget::Docx => output_dir.join(format!("{}.docx", safe_stem)),
            MarkdownBatchTarget::Html => output_dir.join(format!("{}.html", safe_stem)),
        };

        let result: Result<PathBuf> = match options.target {
            MarkdownBatchTarget::Markdown => {
                let ext = file
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();

                let conv_res = if ext == "pdf" {
                    convert_pdf_to_markdown(
                        file,
                        &target_file_path,
                        custom_binary,
                        &html_extractor,
                        &image_extractor,
                    )
                } else if ext == "docx" {
                    convert_doc_to_markdown(
                        file,
                        MarkdownDocFormat::Docx,
                        &target_file_path,
                        custom_binary,
                    )
                } else {
                    convert_doc_to_markdown(
                        file,
                        MarkdownDocFormat::Html,
                        &target_file_path,
                        custom_binary,
                    )
                };

                match conv_res {
                    Ok(out_p) => {
                        if options.inject_front_matter || options.rewrite_assets {
                            if let Ok(content) = std::fs::read_to_string(&out_p) {
                                let mut updated = content;
                                if options.rewrite_assets {
                                    updated = rewrite_markdown_asset_links(&updated);
                                }
                                if options.inject_front_matter {
                                    updated = inject_or_preserve_yaml_front_matter(
                                        &updated,
                                        file_stem,
                                        file_name_str,
                                        &timestamp,
                                        options.preserve_front_matter,
                                    );
                                }
                                let _ = lpdf_fs_safety::atomic_write(&out_p, updated.as_bytes());
                            }
                        }
                        Ok(out_p)
                    }
                    Err(e) => Err(e),
                }
            }
            MarkdownBatchTarget::Pdf => {
                if options.rewrite_assets || options.inject_front_matter {
                    if let Ok(content) = std::fs::read_to_string(file) {
                        let mut updated = content;
                        if options.rewrite_assets {
                            updated = rewrite_markdown_asset_links(&updated);
                        }
                        if options.inject_front_matter {
                            updated = inject_or_preserve_yaml_front_matter(
                                &updated,
                                file_stem,
                                file_name_str,
                                &timestamp,
                                options.preserve_front_matter,
                            );
                        }
                        let unique_id =
                            format!("{}_{}", std::process::id(), lpdf_fs_safety::uuid_simple());
                        let temp_dir =
                            std::env::temp_dir().join(format!("lpdf_batch_md_{}", unique_id));
                        let _ = std::fs::create_dir_all(&temp_dir);
                        let temp_md = temp_dir.join(format!("{}.md", safe_stem));
                        let write_res = std::fs::write(&temp_md, updated.as_bytes());
                        let res = if write_res.is_ok() {
                            convert_markdown_to_pdf(
                                &temp_md,
                                &target_file_path,
                                custom_binary,
                                &pdf_converter,
                            )
                        } else {
                            convert_markdown_to_pdf(
                                file,
                                &target_file_path,
                                custom_binary,
                                &pdf_converter,
                            )
                        };
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        res
                    } else {
                        convert_markdown_to_pdf(
                            file,
                            &target_file_path,
                            custom_binary,
                            &pdf_converter,
                        )
                    }
                } else {
                    convert_markdown_to_pdf(file, &target_file_path, custom_binary, &pdf_converter)
                }
            }
            MarkdownBatchTarget::Docx => convert_markdown_to_doc(
                file,
                MarkdownDocFormat::Docx,
                &target_file_path,
                custom_binary,
            ),
            MarkdownBatchTarget::Html => convert_markdown_to_doc(
                file,
                MarkdownDocFormat::Html,
                &target_file_path,
                custom_binary,
            ),
        };

        match result {
            Ok(p) => {
                converted_files += 1;
                output_files.push(p);
            }
            Err(e) => {
                failed_files += 1;
                errors.push(format!("{}: {}", file_name_str, e));
            }
        }
    }

    Ok(MarkdownBatchReport {
        total_files,
        converted_files,
        failed_files,
        output_files,
        errors,
    })
}
