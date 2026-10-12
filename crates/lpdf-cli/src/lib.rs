#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    MarkdownBatchOptions, MarkdownBatchReport, MarkdownBatchTarget, MarkdownDocFormat, PdfEngine,
};
use lpdf_engine_switcher::EngineSwitcher;
use std::path::Path;

/// Creates a standard engine instance wired to primary (pdfcraft) and fallback.
pub fn create_default_engine() -> EngineSwitcher {
    EngineSwitcher::new()
}

/// Dispatches CLI arguments to the appropriate tool command.
pub fn run_cli(args: &[String]) -> Result<String> {
    let engine = create_default_engine();
    run_cli_with_engine(args, &engine)
}

/// Dispatches CLI arguments using an explicit engine implementation (useful for testing).
pub fn run_cli_with_engine<E: PdfEngine>(args: &[String], engine: &E) -> Result<String> {
    if args.len() <= 1 {
        return Ok(usage_summary());
    }

    match args[1].as_str() {
        "-h" | "--help" | "help" => Ok(usage_summary()),
        "md" | "markdown" => handle_markdown_command(&args[2..], engine),
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
    md              Markdown toolset operations (Pandoc sidecar)
    help, --help    Print help information

Run 'localpdf md --help' for Markdown subcommands.
"#
    .to_string()
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
