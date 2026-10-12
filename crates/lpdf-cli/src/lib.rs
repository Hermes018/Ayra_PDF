#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    AiModelManifest, LlmSummaryOptions, MarkdownBatchOptions, MarkdownBatchReport,
    MarkdownBatchTarget, MarkdownDocFormat, OptimizeProfile, PdfAConformance, PdfEngine,
    Permissions, RedactionRegion, SummaryLength, WatermarkSettings,
};
use lpdf_engine_switcher::EngineSwitcher;
use std::path::{Path, PathBuf};

/// Creates a standard engine instance wired to primary (pdfcraft) and fallback.
pub fn create_default_engine() -> EngineSwitcher {
    EngineSwitcher::new()
}

/// Dispatches CLI arguments to the appropriate tool command.
pub fn run_cli(args: &[String]) -> Result<String> {
    let mut engine = create_default_engine();
    run_cli_with_engine(args, &mut engine)
}

/// Dispatches CLI arguments using an explicit engine implementation (useful for testing).
pub fn run_cli_with_engine<E: PdfEngine>(args: &[String], engine: &mut E) -> Result<String> {
    if args.len() <= 1 {
        return Ok(usage_summary());
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => Ok(usage_summary()),
        "info" => handle_info_command(&args[2..], engine),
        "merge" => handle_merge_command(&args[2..], engine),
        "split" => handle_split_command(&args[2..], engine),
        "rotate" => handle_rotate_command(&args[2..], engine),
        "extract" => handle_extract_command(&args[2..], engine),
        "delete" => handle_delete_command(&args[2..], engine),
        "compress" => handle_compress_command(&args[2..], engine),
        "repair" => handle_repair_command(&args[2..], engine),
        "ocr" => handle_ocr_command(&args[2..], engine),
        "watermark" => handle_watermark_command(&args[2..], engine),
        "flatten-comments" => handle_flatten_comments_command(&args[2..], engine),
        "flatten-forms" => handle_flatten_forms_command(&args[2..], engine),
        "protect" => handle_protect_command(&args[2..], engine),
        "unlock" => handle_unlock_command(&args[2..], engine),
        "redact" => handle_redact_command(&args[2..], engine),
        "pdfa" => handle_pdfa_command(&args[2..], engine),
        "to-text" => handle_to_text_command(&args[2..], engine),
        "to-html" => handle_to_html_command(&args[2..], engine),
        "to-images" => handle_to_images_command(&args[2..], engine),
        "pipeline" => handle_pipeline_command(&args[2..], engine),
        "md" | "markdown" => handle_markdown_command(&args[2..], engine),
        "summarize" => handle_summarize_command(&args[2..], engine),
        other => Err(LpdfError::InvalidInput(format!(
            "Unknown command '{}'. Run 'localpdf --help' for usage.",
            other
        ))),
    }
}

fn usage_summary() -> String {
    r#"LocalPDF Studio CLI (localpdf)

USAGE:
    localpdf <COMMAND> [OPTIONS]

COMMANDS:
    Document Inspection:
        info <input.pdf> [--json]
            Display page count, encryption status, PDF version, and metadata.

    Organize:
        merge <output.pdf> <input1.pdf> <input2.pdf> ...
            Merge multiple PDF files into a single document.
        split <input.pdf> <output_dir> [--at <page_numbers...>]
            Split PDF into individual pages or segments at cut points.
        rotate <input.pdf> <output.pdf> --degrees <90|180|270> [--pages <1,2,...>]
            Rotate entire document or specific pages.
        extract <input.pdf> <output.pdf> --pages <1,2,...>
            Extract selected pages to a new PDF document.
        delete <input.pdf> <output.pdf> --pages <1,2,...>
            Remove selected pages from document.

    Process:
        compress <input.pdf> <output.pdf> [--profile <low|medium|high>]
            Optimize and reduce PDF file size.
        repair <input.pdf> <output.pdf>
            Repair corrupted PDF structure and cross-reference table.
        ocr <input.pdf> <output.pdf>
            Perform OCR text recognition and inject searchable text.

    Edit & Forms:
        watermark <input.pdf> <output.pdf> --text <TEXT> [--opacity <0.0-1.0>] [--rotation <deg>]
            Apply text watermark across pages.
        flatten-comments <input.pdf> <output.pdf>
            Flatten annotations and reviewer comments into page contents.
        flatten-forms <input.pdf> <output.pdf>
            Flatten interactive form fields into static page content.

    Security & Redaction:
        protect <input.pdf> <output.pdf> --password <PASS> [--allow-print] [--allow-copy] [--allow-edit]
            Encrypt document with password and set permissions.
        unlock <input.pdf> <output.pdf> --password <PASS>
            Decrypt and remove password restriction from document.
        redact <input.pdf> <output.pdf> --region <page:x,y,w,h> ...
            Permanently redact rectangular regions from pages.

    Convert & Archival:
        pdfa <input.pdf> <output.pdf> [--conformance <2b|3b|1b>]
            Convert PDF to archival PDF/A standard.
        to-text <input.pdf> <output.txt>
            Extract plain text from document.
        to-html <input.pdf> <output.html>
            Convert PDF pages to HTML.
        to-images <input.pdf> <output_dir>
            Render all pages to image files in output directory.

    Workflows / Pipelines:
        pipeline presets
            List built-in workflow presets and transformation steps.
        pipeline run <preset_id> <input.pdf> <output.pdf>
            Execute multi-step pipeline workflow with atomic staging.

    Markdown Toolset:
        md <SUBCOMMAND>
            Pandoc sidecar Markdown conversion operations.

    AI & Intelligence:
        summarize <input.pdf> <output.md> [--length <short|medium|detailed>]
            Generate deterministic zero-model extractive summary report.

Run 'localpdf <COMMAND> --help' for details on a specific command.
"#
    .to_string()
}

fn parse_page_list(s: &str) -> Result<Vec<usize>> {
    let mut pages = Vec::new();
    for part in s.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let p: usize = trimmed.parse().map_err(|_| {
            LpdfError::InvalidInput(format!("Invalid page number '{}' in '{}'", trimmed, s))
        })?;
        if p == 0 {
            return Err(LpdfError::InvalidInput(
                "Page numbers must be 1-indexed (>= 1)".into(),
            ));
        }
        pages.push(p);
    }
    if pages.is_empty() {
        return Err(LpdfError::InvalidInput("Page list cannot be empty".into()));
    }
    Ok(pages)
}

fn parse_pdf_version(bytes: &[u8]) -> String {
    let limit = bytes.len().min(1024);
    let header = &bytes[..limit];
    if let Some(pos) = header.windows(5).position(|w| w == b"%PDF-") {
        let rest = &header[pos + 5..];
        let mut ver_bytes = Vec::new();
        for &b in rest {
            if b.is_ascii_digit() || b == b'.' {
                ver_bytes.push(b);
            } else {
                break;
            }
        }
        if !ver_bytes.is_empty() {
            if let Ok(s) = std::str::from_utf8(&ver_bytes) {
                return s.to_string();
            }
        }
    }
    "1.7".to_string()
}

fn handle_info_command<E: PdfEngine>(args: &[String], engine: &mut E) -> Result<String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" || args[0] == "help" {
        return Ok("Usage: localpdf info <input.pdf> [--json]".to_string());
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }

    let mut as_json = false;
    for arg in &args[1..] {
        if arg == "--json" {
            as_json = true;
        } else {
            return Err(LpdfError::InvalidInput(format!("Unknown option '{}'", arg)));
        }
    }

    let file_bytes = std::fs::read(input).map_err(LpdfError::Io)?;
    let version = parse_pdf_version(&file_bytes);
    let encrypted = file_bytes.windows(8).any(|w| w == b"/Encrypt");

    engine.load(input)?;
    let info = engine.info()?;

    if as_json {
        let json_obj = serde_json::json!({
            "page_count": info.page_count,
            "version": version,
            "encrypted": encrypted,
            "title": info.title,
            "author": info.author,
        });
        serde_json::to_string_pretty(&json_obj)
            .map_err(|e| LpdfError::Engine(format!("Failed to serialize JSON: {}", e)))
    } else {
        let mut out = format!(
            "File: {}\nPages: {}\nPDF Version: {}\nEncrypted: {}\n",
            input.display(),
            info.page_count,
            version,
            if encrypted { "Yes" } else { "No" }
        );
        if let Some(t) = info.title {
            out.push_str(&format!("Title: {}\n", t));
        }
        if let Some(a) = info.author {
            out.push_str(&format!("Author: {}\n", a));
        }
        Ok(out)
    }
}

fn handle_merge_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf merge <output.pdf> <input1.pdf> <input2.pdf> ...".into(),
        ));
    }
    let output = Path::new(&args[0]);
    let inputs: Vec<&Path> = args[1..].iter().map(Path::new).collect();
    for input in &inputs {
        if !input.exists() {
            return Err(LpdfError::InvalidInput(format!(
                "Input file '{}' does not exist",
                input.display()
            )));
        }
    }
    engine.merge(&inputs, output)?;
    Ok(format!(
        "Merged {} file(s) into '{}'",
        inputs.len(),
        output.display()
    ))
}

fn handle_split_command<E: PdfEngine>(args: &[String], engine: &mut E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf split <input.pdf> <output_dir> [--at <page_numbers...>]".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output_dir = Path::new(&args[1]);
    std::fs::create_dir_all(output_dir).map_err(LpdfError::Io)?;

    let mut at_pages: Option<Vec<usize>> = None;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--at" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --at".into()));
                }
                at_pages = Some(parse_page_list(&args[idx])?);
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    engine.load(input)?;
    let info = engine.info()?;
    let total_pages = info.page_count;
    if total_pages == 0 {
        return Err(LpdfError::InvalidInput("Document has 0 pages".into()));
    }

    let mut count = 0;
    if let Some(mut cut_points) = at_pages {
        cut_points.sort_unstable();
        cut_points.dedup();
        for &cp in &cut_points {
            if cp < 1 || cp > total_pages {
                return Err(LpdfError::InvalidInput(format!(
                    "Cut point {} is outside page range 1..{}",
                    cp, total_pages
                )));
            }
        }

        let mut start_page = 1;
        let mut segment_idx = 1;
        for &cp in &cut_points {
            if cp > start_page {
                let seg_pages: Vec<usize> = (start_page..cp).collect();
                let seg_out = output_dir.join(format!("part_{}.pdf", segment_idx));
                engine.extract_pages(input, &seg_pages, &seg_out)?;
                count += 1;
                segment_idx += 1;
                start_page = cp;
            }
        }
        if start_page <= total_pages {
            let seg_pages: Vec<usize> = (start_page..=total_pages).collect();
            let seg_out = output_dir.join(format!("part_{}.pdf", segment_idx));
            engine.extract_pages(input, &seg_pages, &seg_out)?;
            count += 1;
        }
    } else {
        for p in 1..=total_pages {
            let page_out = output_dir.join(format!("page_{}.pdf", p));
            engine.extract_pages(input, &[p], &page_out)?;
            count += 1;
        }
    }

    Ok(format!(
        "Split '{}' into {} file(s) in '{}'",
        input.display(),
        count,
        output_dir.display()
    ))
}

fn handle_rotate_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf rotate <input.pdf> <output.pdf> --degrees <90|180|270> [--pages <1,2,...>]".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut degrees: Option<i32> = None;
    let mut pages: Option<Vec<usize>> = None;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--degrees" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --degrees".into(),
                    ));
                }
                let deg: i32 = args[idx].parse().map_err(|_| {
                    LpdfError::InvalidInput(format!("Invalid degrees '{}'", args[idx]))
                })?;
                if deg % 90 != 0 {
                    return Err(LpdfError::InvalidInput(
                        "Degrees must be a multiple of 90 (e.g. 90, 180, 270)".into(),
                    ));
                }
                degrees = Some(deg);
            }
            "--pages" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --pages".into()));
                }
                pages = Some(parse_page_list(&args[idx])?);
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let deg = degrees.ok_or_else(|| {
        LpdfError::InvalidInput("Missing required argument --degrees <90|180|270>".into())
    })?;

    if let Some(pgs) = pages {
        engine.rotate_pages(input, &pgs, deg, output)?;
        Ok(format!(
            "Rotated {} page(s) by {} degrees in '{}'",
            pgs.len(),
            deg,
            output.display()
        ))
    } else {
        engine.rotate_document(input, deg, output)?;
        Ok(format!(
            "Rotated document by {} degrees in '{}'",
            deg,
            output.display()
        ))
    }
}

fn handle_extract_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf extract <input.pdf> <output.pdf> --pages <1,2,...>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut pages: Option<Vec<usize>> = None;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--pages" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --pages".into()));
                }
                pages = Some(parse_page_list(&args[idx])?);
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let pgs = pages.ok_or_else(|| {
        LpdfError::InvalidInput("Missing required argument --pages <1,2,...>".into())
    })?;

    engine.extract_pages(input, &pgs, output)?;
    Ok(format!(
        "Extracted {} page(s) into '{}'",
        pgs.len(),
        output.display()
    ))
}

fn handle_delete_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf delete <input.pdf> <output.pdf> --pages <1,2,...>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut pages: Option<Vec<usize>> = None;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--pages" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --pages".into()));
                }
                pages = Some(parse_page_list(&args[idx])?);
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let pgs = pages.ok_or_else(|| {
        LpdfError::InvalidInput("Missing required argument --pages <1,2,...>".into())
    })?;

    engine.delete_pages(input, &pgs, output)?;
    Ok(format!(
        "Deleted {} page(s), saved result to '{}'",
        pgs.len(),
        output.display()
    ))
}

fn handle_compress_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf compress <input.pdf> <output.pdf> [--profile <low|medium|high>]"
                .into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut profile = OptimizeProfile::Medium;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--profile" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --profile".into(),
                    ));
                }
                match args[idx].to_ascii_lowercase().as_str() {
                    "low" => profile = OptimizeProfile::Low,
                    "medium" => profile = OptimizeProfile::Medium,
                    "high" => profile = OptimizeProfile::High,
                    other => {
                        return Err(LpdfError::InvalidInput(format!(
                            "Invalid profile '{}'. Expected low, medium, or high.",
                            other
                        )))
                    }
                }
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    engine.compress(input, profile, output)?;
    Ok(format!(
        "Compressed '{}' with {:?} profile to '{}'",
        input.display(),
        profile,
        output.display()
    ))
}

fn handle_repair_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf repair <input.pdf> <output.pdf>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let issues = engine.repair(input, output)?;
    let msg = if issues.is_empty() {
        format!("Repaired '{}' -> '{}'", input.display(), output.display())
    } else {
        format!(
            "Repaired '{}' -> '{}' (resolved {} issues)",
            input.display(),
            output.display(),
            issues.len()
        )
    };
    Ok(msg)
}

fn handle_ocr_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf ocr <input.pdf> <output.pdf>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    engine.ocr(input, "eng", output)?;
    Ok(format!(
        "Performed OCR on '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_watermark_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf watermark <input.pdf> <output.pdf> --text <TEXT> [--opacity <0.0-1.0>] [--rotation <deg>]".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut text: Option<String> = None;
    let mut opacity: f64 = 0.35;
    let mut rotation: f64 = 45.0;

    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--text" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --text".into()));
                }
                text = Some(args[idx].clone());
            }
            "--opacity" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --opacity".into(),
                    ));
                }
                opacity = args[idx].parse().map_err(|_| {
                    LpdfError::InvalidInput(format!("Invalid opacity '{}'", args[idx]))
                })?;
                if !(0.0..=1.0).contains(&opacity) {
                    return Err(LpdfError::InvalidInput(
                        "Opacity must be between 0.0 and 1.0".into(),
                    ));
                }
            }
            "--rotation" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --rotation".into(),
                    ));
                }
                rotation = args[idx].parse().map_err(|_| {
                    LpdfError::InvalidInput(format!("Invalid rotation '{}'", args[idx]))
                })?;
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let wm_text = text
        .ok_or_else(|| LpdfError::InvalidInput("Missing required argument --text <TEXT>".into()))?;

    let settings = WatermarkSettings {
        text: wm_text,
        opacity,
        rotation,
        color: [0.8, 0.2, 0.2],
    };

    engine.add_watermark(input, &settings, output)?;
    Ok(format!(
        "Added watermark to '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_flatten_comments_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf flatten-comments <input.pdf> <output.pdf>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);
    engine.flatten_comments(input, output)?;
    Ok(format!(
        "Flattened comments in '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_flatten_forms_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf flatten-forms <input.pdf> <output.pdf>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);
    engine.flatten_forms(input, output)?;
    Ok(format!(
        "Flattened forms in '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_protect_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf protect <input.pdf> <output.pdf> --password <PASS> [--allow-print] [--allow-copy] [--allow-edit]".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut password: Option<String> = None;
    let mut permissions = Permissions {
        allow_print: false,
        allow_copy: false,
        allow_edit: false,
    };

    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--password" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --password".into(),
                    ));
                }
                password = Some(args[idx].clone());
            }
            "--allow-print" => permissions.allow_print = true,
            "--allow-copy" => permissions.allow_copy = true,
            "--allow-edit" => permissions.allow_edit = true,
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let pass = password.ok_or_else(|| {
        LpdfError::InvalidInput("Missing required argument --password <PASS>".into())
    })?;

    engine.protect(input, &pass, permissions, output)?;
    Ok(format!(
        "Protected document '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_unlock_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf unlock <input.pdf> <output.pdf> --password <PASS>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut password: Option<String> = None;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--password" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --password".into(),
                    ));
                }
                password = Some(args[idx].clone());
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let pass = password.ok_or_else(|| {
        LpdfError::InvalidInput("Missing required argument --password <PASS>".into())
    })?;

    engine.unlock(input, &pass, output)?;
    Ok(format!(
        "Unlocked document '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn parse_region(val: &str) -> Result<RedactionRegion> {
    let parts: Vec<&str> = val.split(':').collect();
    if parts.len() != 2 {
        return Err(LpdfError::InvalidInput(format!(
            "Invalid region format '{}'. Expected <page:x,y,w,h>",
            val
        )));
    }
    let page: usize = parts[0]
        .trim()
        .parse()
        .map_err(|_| LpdfError::InvalidInput(format!("Invalid page number in region '{}'", val)))?;
    let coords: Vec<&str> = parts[1].split(',').collect();
    if coords.len() != 4 {
        return Err(LpdfError::InvalidInput(format!(
            "Invalid coordinates in region '{}'. Expected x,y,w,h",
            val
        )));
    }
    let x: f64 = coords[0].trim().parse().map_err(|_| {
        LpdfError::InvalidInput(format!("Invalid x coordinate in region '{}'", val))
    })?;
    let y: f64 = coords[1].trim().parse().map_err(|_| {
        LpdfError::InvalidInput(format!("Invalid y coordinate in region '{}'", val))
    })?;
    let width: f64 = coords[2]
        .trim()
        .parse()
        .map_err(|_| LpdfError::InvalidInput(format!("Invalid width in region '{}'", val)))?;
    let height: f64 = coords[3]
        .trim()
        .parse()
        .map_err(|_| LpdfError::InvalidInput(format!("Invalid height in region '{}'", val)))?;

    Ok(RedactionRegion {
        page,
        x,
        y,
        width,
        height,
    })
}

fn handle_redact_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf redact <input.pdf> <output.pdf> --region <page:x,y,w,h> ...".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut regions = Vec::new();
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--region" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --region".into()));
                }
                regions.push(parse_region(&args[idx])?);
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    if regions.is_empty() {
        return Err(LpdfError::InvalidInput(
            "At least one --region <page:x,y,w,h> is required for redact".into(),
        ));
    }

    engine.redact(input, regions.clone(), output)?;
    Ok(format!(
        "Redacted {} region(s) in '{}' -> '{}'",
        regions.len(),
        input.display(),
        output.display()
    ))
}

fn handle_pdfa_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf pdfa <input.pdf> <output.pdf> [--conformance <2b|3b|1b>]".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut conformance = PdfAConformance::PdfA2b;
    let mut idx = 2;
    while idx < args.len() {
        match args[idx].as_str() {
            "--conformance" => {
                idx += 1;
                if idx >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --conformance".into(),
                    ));
                }
                match args[idx].to_ascii_lowercase().as_str() {
                    "2b" | "pdfa2b" => conformance = PdfAConformance::PdfA2b,
                    "3b" | "pdfa3b" => conformance = PdfAConformance::PdfA3b,
                    "1b" | "pdfa1b" => conformance = PdfAConformance::PdfA1b,
                    other => {
                        return Err(LpdfError::InvalidInput(format!(
                            "Invalid conformance '{}'. Expected 2b, 3b, or 1b.",
                            other
                        )))
                    }
                }
            }
            unknown => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown option '{}'",
                    unknown
                )));
            }
        }
        idx += 1;
    }

    let report = engine.pdf_to_pdfa(input, conformance, output)?;
    Ok(format!(
        "Converted to PDF/A ({:?}): compliant={}, output='{}'",
        report.conformance,
        report.is_compliant,
        output.display()
    ))
}

fn handle_to_text_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf to-text <input.pdf> <output.txt>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);
    engine.pdf_to_text(input, output)?;
    Ok(format!(
        "Extracted text from '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_to_html_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf to-html <input.pdf> <output.html>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);
    engine.pdf_to_html(input, output)?;
    Ok(format!(
        "Exported HTML from '{}' -> '{}'",
        input.display(),
        output.display()
    ))
}

fn handle_to_images_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() < 2 || args[0] == "-h" || args[0] == "--help" {
        return Err(LpdfError::InvalidInput(
            "Usage: localpdf to-images <input.pdf> <output_dir>".into(),
        ));
    }
    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output_dir = Path::new(&args[1]);
    engine.pdf_to_images(input, output_dir)?;
    Ok(format!(
        "Rendered pages to images in '{}'",
        output_dir.display()
    ))
}

fn pipeline_help() -> String {
    r#"LocalPDF Studio Pipeline Subcommands (localpdf pipeline)

USAGE:
    localpdf pipeline <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    presets
        List built-in workflow presets and their steps.

    run <preset_id> <input.pdf> <output.pdf>
        Execute multi-step pipeline workflow on a document with atomic staging.
"#
    .to_string()
}

fn handle_pipeline_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" || args[0] == "help" {
        return Ok(pipeline_help());
    }

    match args[0].as_str() {
        "presets" => {
            let presets = lpdf_jobs::pipeline::builtin_pipeline_presets();
            let mut out = String::from("Built-in Workflow Presets:\n\n");
            for p in &presets {
                out.push_str(&format!("Preset: {}\n", p.id));
                out.push_str(&format!("  Name: {}\n", p.name));
                out.push_str(&format!("  Description: {}\n", p.description));
                out.push_str("  Steps:\n");
                for (i, step) in p.steps.iter().enumerate() {
                    out.push_str(&format!("    {}. {:?}\n", i + 1, step));
                }
                out.push('\n');
            }
            Ok(out)
        }
        "run" => {
            if args.len() < 4 {
                return Err(LpdfError::InvalidInput(
                    "Usage: localpdf pipeline run <preset_id> <input.pdf> <output.pdf>".into(),
                ));
            }
            let preset_id = &args[1];
            let input = Path::new(&args[2]);
            if !input.exists() {
                return Err(LpdfError::InvalidInput(format!(
                    "Input file '{}' does not exist",
                    input.display()
                )));
            }
            let output = Path::new(&args[3]);

            let presets = lpdf_jobs::pipeline::builtin_pipeline_presets();
            let preset = presets
                .into_iter()
                .find(|p| p.id.eq_ignore_ascii_case(preset_id))
                .ok_or_else(|| {
                    LpdfError::InvalidInput(format!(
                        "Unknown pipeline preset '{}'. Run 'localpdf pipeline presets' to see available presets.",
                        preset_id
                    ))
                })?;

            execute_pipeline_steps(engine, &preset.steps, input, output)?;
            Ok(format!(
                "Successfully executed workflow '{}' ({} steps) on '{}' -> '{}'",
                preset.name,
                preset.steps.len(),
                input.display(),
                output.display()
            ))
        }
        other => Err(LpdfError::InvalidInput(format!(
            "Unknown pipeline subcommand '{}'. Run 'localpdf pipeline --help' for usage.",
            other
        ))),
    }
}

fn execute_pipeline_steps<E: PdfEngine>(
    engine: &E,
    steps: &[lpdf_jobs::pipeline::PipelineStep],
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    if steps.is_empty() {
        return Err(LpdfError::InvalidInput("Pipeline has no steps".into()));
    }

    let parent_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    let stage_id = uuid::Uuid::new_v4().to_string();
    let stage_dir = parent_dir.join(format!(".cli_pipeline_stage_{}", stage_id));
    std::fs::create_dir_all(&stage_dir).map_err(LpdfError::Io)?;

    // RAII cleaner ensuring stage_dir is removed on exit or error
    struct StageCleaner<'a>(&'a Path);
    impl<'a> Drop for StageCleaner<'a> {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0);
        }
    }
    let _cleaner = StageCleaner(&stage_dir);

    let mut current_input = input_path.to_path_buf();

    for (idx, step) in steps.iter().enumerate() {
        let step_output = stage_dir.join(format!("step_{}.pdf", idx + 1));

        match step {
            lpdf_jobs::pipeline::PipelineStep::Repair => {
                let _ = engine.repair(&current_input, &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::Compress { profile } => {
                engine.compress(&current_input, *profile, &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::Ocr => {
                engine.ocr(&current_input, "eng", &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::Rotate { degrees, pages } => {
                if pages.is_empty() {
                    engine.rotate_document(&current_input, *degrees, &step_output)?;
                } else {
                    let usize_pages: Vec<usize> = pages.iter().map(|&p| p as usize).collect();
                    engine.rotate_pages(&current_input, &usize_pages, *degrees, &step_output)?;
                }
            }
            lpdf_jobs::pipeline::PipelineStep::FlattenComments => {
                engine.flatten_comments(&current_input, &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::FlattenForms => {
                engine.flatten_forms(&current_input, &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::Watermark { settings } => {
                engine.add_watermark(&current_input, settings, &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::Protect {
                password,
                permissions,
            } => {
                engine.protect(&current_input, password, permissions.clone(), &step_output)?;
            }
            lpdf_jobs::pipeline::PipelineStep::PdfToPdfA { conformance } => {
                let _ = engine.pdf_to_pdfa(&current_input, *conformance, &step_output)?;
            }
        }

        let meta = std::fs::metadata(&step_output).map_err(LpdfError::Io)?;
        if meta.len() == 0 {
            return Err(LpdfError::Engine(format!(
                "Pipeline step {} produced an empty file",
                idx + 1
            )));
        }

        current_input = step_output;
    }

    let final_bytes = std::fs::read(&current_input).map_err(LpdfError::Io)?;
    if !final_bytes.starts_with(b"%PDF-") {
        return Err(LpdfError::ParseError(
            "Pipeline final output is not a valid PDF".into(),
        ));
    }

    lpdf_fs_safety::atomic_write(output_path, &final_bytes)?;
    Ok(())
}

fn markdown_help() -> String {
    r#"LocalPDF Studio Markdown Toolset (localpdf md)

USAGE:
    localpdf md <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    to-pdf <input.md> [output.pdf]
        Convert Markdown to PDF

    from-pdf <input.pdf> [output.md]
        Convert PDF to Markdown (extracting images to assets/)

    to-doc <input.md> <docx|html> [output]
        Convert Markdown to DOCX or HTML

    from-doc <input> <docx|html> [output.md]
        Convert DOCX or HTML to Markdown

    batch <input_dir> <output_dir> [OPTIONS]
        Batch convert an entire directory
        Options:
            --target <pdf|docx|html|md>   (default: pdf)
            --no-front-matter             Disable YAML front-matter injection
            --no-preserve-front-matter    Overwrite existing front-matter
            --no-rewrite-assets           Disable asset link rewriting
"#
    .to_string()
}

fn handle_markdown_command<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" || args[0] == "help" {
        return Ok(markdown_help());
    }

    match args[0].as_str() {
        "to-pdf" => {
            if args.len() < 2 {
                return Err(LpdfError::InvalidInput(
                    "Usage: localpdf md to-pdf <input.md> [output.pdf]".into(),
                ));
            }
            let input = Path::new(&args[1]);
            let output_buf;
            let output = if args.len() >= 3 {
                Path::new(&args[2])
            } else {
                let stem = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("document");
                output_buf = input.with_file_name(format!("{}.pdf", stem));
                output_buf.as_path()
            };

            let res = engine.markdown_to_pdf(input, output)?;
            Ok(format!("Converted Markdown to PDF: {}", res.display()))
        }
        "from-pdf" => {
            if args.len() < 2 {
                return Err(LpdfError::InvalidInput(
                    "Usage: localpdf md from-pdf <input.pdf> [output.md]".into(),
                ));
            }
            let input = Path::new(&args[1]);
            let output_buf;
            let output = if args.len() >= 3 {
                Path::new(&args[2])
            } else {
                let stem = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("document");
                output_buf = input.with_file_name(format!("{}.md", stem));
                output_buf.as_path()
            };

            let res = engine.pdf_to_markdown(input, output)?;
            Ok(format!("Converted PDF to Markdown: {}", res.display()))
        }
        "to-doc" => {
            if args.len() < 3 {
                return Err(LpdfError::InvalidInput(
                    "Usage: localpdf md to-doc <input.md> <docx|html> [output]".into(),
                ));
            }
            let input = Path::new(&args[1]);
            let format = match args[2].to_ascii_lowercase().as_str() {
                "docx" => MarkdownDocFormat::Docx,
                "html" => MarkdownDocFormat::Html,
                other => {
                    return Err(LpdfError::InvalidInput(format!(
                        "Invalid format '{}'. Expected 'docx' or 'html'.",
                        other
                    )))
                }
            };
            let output_buf;
            let output = if args.len() >= 4 {
                Path::new(&args[3])
            } else {
                let stem = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("document");
                output_buf = input.with_file_name(format!("{}.{}", stem, format.extension()));
                output_buf.as_path()
            };

            let res = engine.markdown_to_doc(input, format, output)?;
            Ok(format!(
                "Converted Markdown to {}: {}",
                format.extension().to_uppercase(),
                res.display()
            ))
        }
        "from-doc" => {
            if args.len() < 3 {
                return Err(LpdfError::InvalidInput(
                    "Usage: localpdf md from-doc <input> <docx|html> [output.md]".into(),
                ));
            }
            let input = Path::new(&args[1]);
            let format = match args[2].to_ascii_lowercase().as_str() {
                "docx" => MarkdownDocFormat::Docx,
                "html" => MarkdownDocFormat::Html,
                other => {
                    return Err(LpdfError::InvalidInput(format!(
                        "Invalid format '{}'. Expected 'docx' or 'html'.",
                        other
                    )))
                }
            };
            let output_buf;
            let output = if args.len() >= 4 {
                Path::new(&args[3])
            } else {
                let stem = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("document");
                output_buf = input.with_file_name(format!("{}.md", stem));
                output_buf.as_path()
            };

            let res = engine.doc_to_markdown(input, format, output)?;
            Ok(format!(
                "Converted {} to Markdown: {}",
                format.extension().to_uppercase(),
                res.display()
            ))
        }
        "batch" => {
            if args.len() < 3 {
                return Err(LpdfError::InvalidInput(
                    "Usage: localpdf md batch <input_dir> <output_dir> [OPTIONS]".into(),
                ));
            }
            let input_dir = Path::new(&args[1]);
            let output_dir = Path::new(&args[2]);

            let mut options = MarkdownBatchOptions::default();
            let mut idx = 3;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--target" => {
                        idx += 1;
                        if idx >= args.len() {
                            return Err(LpdfError::InvalidInput(
                                "Missing value for --target".into(),
                            ));
                        }
                        match args[idx].to_ascii_lowercase().as_str() {
                            "pdf" => options.target = MarkdownBatchTarget::Pdf,
                            "docx" => options.target = MarkdownBatchTarget::Docx,
                            "html" => options.target = MarkdownBatchTarget::Html,
                            "md" | "markdown" => options.target = MarkdownBatchTarget::Markdown,
                            other => {
                                return Err(LpdfError::InvalidInput(format!(
                                    "Invalid batch target '{}'. Expected pdf, docx, html, or md.",
                                    other
                                )))
                            }
                        }
                    }
                    "--no-front-matter" => {
                        options.inject_front_matter = false;
                    }
                    "--no-preserve-front-matter" => {
                        options.preserve_front_matter = false;
                    }
                    "--no-rewrite-assets" => {
                        options.rewrite_assets = false;
                    }
                    unknown => {
                        return Err(LpdfError::InvalidInput(format!(
                            "Unknown batch option '{}'",
                            unknown
                        )));
                    }
                }
                idx += 1;
            }

            let report = engine.batch_markdown_convert(input_dir, output_dir, options)?;
            Ok(format_batch_report(&report))
        }
        other => Err(LpdfError::InvalidInput(format!(
            "Unknown markdown subcommand '{}'. Run 'localpdf md --help' for usage.",
            other
        ))),
    }
}

fn format_batch_report(report: &MarkdownBatchReport) -> String {
    let mut msg = format!(
        "Batch Markdown Conversion Complete:\n  Total files: {}\n  Converted: {}\n  Failed: {}\n",
        report.total_files, report.converted_files, report.failed_files
    );
    if !report.output_files.is_empty() {
        msg.push_str("Output files:\n");
        for f in &report.output_files {
            msg.push_str(&format!("  - {}\n", f.display()));
        }
    }
    if !report.errors.is_empty() {
        msg.push_str("Errors:\n");
        for err in &report.errors {
            msg.push_str(&format!("  ! {}\n", err));
        }
    }
    msg
}

fn handle_summarize_command<E: PdfEngine>(args: &[String], engine: &mut E) -> Result<String> {
    if args.is_empty() || args[0] == "-h" || args[0] == "--help" || args[0] == "help" {
        return Ok(
            "Usage: localpdf summarize <input.pdf> <output.md> [--length <short|medium|detailed>] [--llm --model <path> [--sha256 <hex>] [--license <spdx>] [--acknowledge-license] [--chunk-words <n>]]"
                .to_string(),
        );
    }

    if args.len() < 2 {
        return Err(LpdfError::InvalidInput(
            "Both input PDF path and output Markdown path are required. Usage: localpdf summarize <input.pdf> <output.md> [...]".into(),
        ));
    }

    let input = Path::new(&args[0]);
    if !input.exists() {
        return Err(LpdfError::InvalidInput(format!(
            "Input file '{}' does not exist",
            input.display()
        )));
    }
    let output = Path::new(&args[1]);

    let mut length = SummaryLength::Medium;
    let mut is_llm = false;
    let mut model_path: Option<PathBuf> = None;
    let mut expected_sha256: Option<String> = None;
    let mut license: Option<String> = None;
    let mut acknowledge_license = false;
    let mut chunk_words: Option<usize> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--length" => {
                if i + 1 >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --length".into()));
                }
                match args[i + 1].to_lowercase().as_str() {
                    "short" => length = SummaryLength::Short,
                    "medium" => length = SummaryLength::Medium,
                    "detailed" => length = SummaryLength::Detailed,
                    other => {
                        return Err(LpdfError::InvalidInput(format!(
                            "Invalid summary length '{}'. Supported values: short, medium, detailed",
                            other
                        )));
                    }
                }
                i += 2;
            }
            "--llm" => {
                is_llm = true;
                i += 1;
            }
            "--model" => {
                if i + 1 >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --model".into()));
                }
                model_path = Some(PathBuf::from(&args[i + 1]));
                i += 2;
            }
            "--sha256" => {
                if i + 1 >= args.len() {
                    return Err(LpdfError::InvalidInput("Missing value for --sha256".into()));
                }
                expected_sha256 = Some(args[i + 1].clone());
                i += 2;
            }
            "--license" => {
                if i + 1 >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --license".into(),
                    ));
                }
                license = Some(args[i + 1].clone());
                i += 2;
            }
            "--acknowledge-license" => {
                acknowledge_license = true;
                i += 1;
            }
            "--chunk-words" => {
                if i + 1 >= args.len() {
                    return Err(LpdfError::InvalidInput(
                        "Missing value for --chunk-words".into(),
                    ));
                }
                let val = args[i + 1].parse::<usize>().map_err(|e| {
                    LpdfError::InvalidInput(format!("Invalid integer for --chunk-words: {}", e))
                })?;
                chunk_words = Some(val);
                i += 2;
            }
            other => {
                return Err(LpdfError::InvalidInput(format!(
                    "Unknown argument '{}'",
                    other
                )));
            }
        }
    }

    if is_llm {
        let m_path = model_path.ok_or_else(|| {
            LpdfError::InvalidInput(
                "Local LLM summarization (--llm) requires --model <path>".into(),
            )
        })?;

        let sha256_hex = match expected_sha256 {
            Some(h) => h,
            None => lpdf_core_types::compute_sha256_hex(&m_path)?,
        };

        let format = if m_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("onnx"))
            .unwrap_or(false)
        {
            "ONNX".to_string()
        } else {
            "GGUF".to_string()
        };

        let manifest = AiModelManifest {
            model_path: m_path,
            sha256_hex,
            format,
            license_spdx: license.unwrap_or_else(|| "Apache-2.0".to_string()),
            license_acknowledged: acknowledge_license,
        };

        let options = LlmSummaryOptions {
            manifest,
            ai_local_enabled: acknowledge_license,
            length,
            chunk_word_limit: chunk_words.unwrap_or(400),
            max_tokens: 512,
            temperature: 0.2,
        };

        let report = engine.summarize_llm_map_reduce(input, output, options)?;
        Ok(format!(
            "Map-Reduce LLM summary generated successfully -> {}\n- Chunks processed: {}\n- Source words: {}\n- Summary words: {}\n- Model: {}",
            report.output_path.display(),
            report.chunks_processed,
            report.source_word_count,
            report.summary_word_count,
            report.model_sha256,
        ))
    } else {
        let report = engine.summarize_extractive(input, output, length)?;
        Ok(format!(
            "Extractive summary generated successfully -> {}\n- Source words: {}\n- Summary words: {}\n- Key points: {}\n- Keywords: {}",
            report.output_path.display(),
            report.source_word_count,
            report.summary_word_count,
            report.key_points.len(),
            if report.keywords.is_empty() {
                "none".to_string()
            } else {
                report.keywords.join(", ")
            }
        ))
    }
}
