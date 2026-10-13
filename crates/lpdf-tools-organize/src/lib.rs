#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use lpdf_engine_port::PdfEngine;
use std::path::Path;

/// Merges multiple PDF files into a single document.
pub fn merge_pdfs<E: PdfEngine + ?Sized>(
    engine: &E,
    input_paths: &[&Path],
    output_path: &Path,
) -> Result<()> {
    engine.merge(input_paths, output_path)
}

/// Extracts specified pages into a new PDF document.
pub fn extract_pages<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    pages: &[usize],
    output_path: &Path,
) -> Result<()> {
    engine.extract_pages(input_path, pages, output_path)
}

/// Deletes specified pages and outputs a new PDF document.
pub fn delete_pages<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    pages: &[usize],
    output_path: &Path,
) -> Result<()> {
    engine.delete_pages(input_path, pages, output_path)
}

/// Rotates specified pages by given degrees (90, 180, 270).
pub fn rotate_pages<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    pages: &[usize],
    degrees: i32,
    output_path: &Path,
) -> Result<()> {
    engine.rotate_pages(input_path, pages, degrees, output_path)
}

/// Rotates all pages in the document.
pub fn rotate_document<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    degrees: i32,
    output_path: &Path,
) -> Result<()> {
    engine.rotate_document(input_path, degrees, output_path)
}
