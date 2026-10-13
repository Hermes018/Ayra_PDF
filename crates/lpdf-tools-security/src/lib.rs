#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use lpdf_engine_port::{PdfEngine, Permissions, RedactionRegion};
use std::path::Path;

/// Encrypts and protects a document with a password and permissions.
pub fn protect_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    password: &str,
    permissions: Permissions,
    output_path: &Path,
) -> Result<()> {
    engine.protect(input_path, password, permissions, output_path)
}

/// Decrypts and removes password security from a document.
pub fn unlock_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    password: &str,
    output_path: &Path,
) -> Result<()> {
    engine.unlock(input_path, password, output_path)
}

/// Digitally signs a document using a local PKCS#12 certificate file.
pub fn sign_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    p12_path: &Path,
    password: &str,
    output_path: &Path,
) -> Result<()> {
    engine.sign_document(input_path, p12_path, password, output_path)
}

/// Applies destructive semantic redaction to remove specified regions.
pub fn redact_pdf<E: PdfEngine + ?Sized>(
    engine: &E,
    input_path: &Path,
    regions: Vec<RedactionRegion>,
    output_path: &Path,
) -> Result<()> {
    engine.redact(input_path, regions, output_path)
}
