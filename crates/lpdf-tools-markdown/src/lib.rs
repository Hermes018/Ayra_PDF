#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use lpdf_engine_port::{
    MarkdownBatchOptions, MarkdownBatchReport, MarkdownDocFormat, PdfEngine,
};
use std::path::{Path, PathBuf};

/// Compiles Markdown to PDF via Pandoc sidecar.
pub fn markdown_to_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<PathBuf> {
    engine.markdown_to_pdf(input_path, output_path)
}

/// Converts PDF to Markdown with asset extraction.
pub fn pdf_to_markdown<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<PathBuf> {
    engine.pdf_to_markdown(input_path, output_path)
}

/// Converts Markdown to target document format (DOCX, HTML, etc.).
pub fn markdown_to_doc<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    format: MarkdownDocFormat,
    output_path: &Path,
) -> Result<PathBuf> {
    engine.markdown_to_doc(input_path, format, output_path)
}

/// Converts document format to Markdown.
pub fn doc_to_markdown<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    format: MarkdownDocFormat,
    output_path: &Path,
) -> Result<PathBuf> {
    engine.doc_to_markdown(input_path, format, output_path)
}

/// Batch converts a folder of Markdown or office files.
pub fn batch_markdown_convert<E: PdfEngine + ?Sized>(
    engine: &E,
    input_dir: &Path,
    output_dir: &Path,
    options: MarkdownBatchOptions,
) -> Result<MarkdownBatchReport> {
    engine.batch_markdown_convert(input_dir, output_dir, options)
}
