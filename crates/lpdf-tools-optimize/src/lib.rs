#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use lpdf_engine_port::{OptimizeProfile, PdfEngine};
use std::path::Path;

/// Compresses a PDF document according to the specified profile.
pub fn compress_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    profile: OptimizeProfile,
    output_path: &Path,
) -> Result<()> {
    engine.compress(input_path, profile, output_path)
}

/// Repairs a corrupted PDF cross-reference table and structure.
pub fn repair_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<Vec<String>> {
    engine.repair(input_path, output_path)
}

/// Performs optical character recognition (OCR) and injects searchable text.
pub fn ocr_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    lang: &str,
    output_path: &Path,
) -> Result<()> {
    engine.ocr(input_path, lang, output_path)
}
