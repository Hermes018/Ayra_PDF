#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use lpdf_engine_port::{
    OfficeInputFormat, OfficeOutputFormat, PdfAConformance, PdfAPreflightReport, PdfEngine,
};
use std::path::{Path, PathBuf};

/// Converts an image to PDF.
pub fn image_to_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.image_to_pdf(input_path, output_path)
}

/// Converts plain text to PDF.
pub fn text_to_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.text_to_pdf(input_path, output_path)
}

/// Converts structural Word docx to PDF.
pub fn structural_docx_to_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.structural_docx_to_pdf(input_path, output_path)
}

/// Converts local HTML to PDF.
pub fn html_to_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.html_to_pdf(input_path, output_path)
}

/// Renders all PDF pages to image files.
pub fn pdf_to_images<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.pdf_to_images(input_path, output_path)
}

/// Extracts all embedded images from the PDF document.
pub fn extract_all_images<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.extract_all_images(input_path, output_path)
}

/// Extracts plain text from the PDF document.
pub fn pdf_to_text<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.pdf_to_text(input_path, output_path)
}

/// Converts PDF to structural Word docx.
pub fn pdf_to_structural_docx<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.pdf_to_structural_docx(input_path, output_path)
}

/// Converts PDF to HTML document.
pub fn pdf_to_html<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.pdf_to_html(input_path, output_path)
}

/// Converts PDF to Rich Text Format (RTF).
pub fn pdf_to_rtf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.pdf_to_rtf(input_path, output_path)
}

/// Converts PDF to archival PDF/A standard.
pub fn pdf_to_pdfa<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    conformance: PdfAConformance,
    output_path: &Path,
) -> Result<PdfAPreflightReport> {
    engine.pdf_to_pdfa(input_path, conformance, output_path)
}

/// Converts Office documents to PDF via headless sidecar.
pub fn office_to_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    format: OfficeInputFormat,
    output_path: &Path,
) -> Result<PathBuf> {
    engine.office_to_pdf(input_path, format, output_path)
}

/// Converts PDF to Office formats via headless sidecar.
pub fn pdf_to_office<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    format: OfficeOutputFormat,
    output_path: &Path,
) -> Result<PathBuf> {
    engine.pdf_to_office(input_path, format, output_path)
}
