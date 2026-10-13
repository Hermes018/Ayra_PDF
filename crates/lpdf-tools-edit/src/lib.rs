#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use lpdf_engine_port::{AnnotationDef, PdfEngine, WatermarkSettings};
use std::collections::HashMap;
use std::path::Path;

/// Adds a watermark to the PDF document.
pub fn add_watermark<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    settings: &WatermarkSettings,
    output_path: &Path,
) -> Result<()> {
    engine.add_watermark(input_path, settings, output_path)
}

/// Adds an annotation (e.g. sticky note, highlight) to the document.
pub fn add_annotation<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    annotation: &AnnotationDef,
    output_path: &Path,
) -> Result<()> {
    engine.add_annotation(input_path, annotation, output_path)
}

/// Flattens reviewer comments and annotations into the page content stream.
pub fn flatten_comments<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.flatten_comments(input_path, output_path)
}

/// Flattens interactive form fields into static page content.
pub fn flatten_forms<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    output_path: &Path,
) -> Result<()> {
    engine.flatten_forms(input_path, output_path)
}

/// Fills interactive form fields with key-value data.
pub fn fill_form<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    fields: &HashMap<String, String>,
    output_path: &Path,
) -> Result<()> {
    engine.fill_form(input_path, fields, output_path)
}
