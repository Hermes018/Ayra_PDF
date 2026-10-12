use lpdf_core_types::Result;
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::PdfEngine;
use std::path::PathBuf;

fn get_test_pdf() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-text.pdf")
}

#[test]
fn test_pdf_to_images() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_images_{}", std::process::id()));
    adapter.pdf_to_images(&pdf, &temp_dir)?;
    assert!(temp_dir.exists());
    let entries: Vec<_> = std::fs::read_dir(&temp_dir)?.collect();
    assert!(!entries.is_empty());
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_extract_all_images() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let temp_dir =
        std::env::temp_dir().join(format!("lpdf_test_extract_images_{}", std::process::id()));
    adapter.extract_all_images(&pdf, &temp_dir)?;
    assert!(temp_dir.exists());
    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}

#[test]
fn test_pdf_to_text() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_text_{}.txt", std::process::id()));
    adapter.pdf_to_text(&pdf, &out)?;
    assert!(out.exists());
    let text = std::fs::read_to_string(&out)?;
    assert!(!text.is_empty());
    let _ = std::fs::remove_file(&out);
    Ok(())
}

#[test]
fn test_pdf_to_structural_docx() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_doc_{}.docx", std::process::id()));
    adapter.pdf_to_structural_docx(&pdf, &out)?;
    assert!(out.exists());
    let bytes = std::fs::read(&out)?;
    assert!(!bytes.is_empty());
    let _ = std::fs::remove_file(&out);
    Ok(())
}

#[test]
fn test_pdf_to_html() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_doc_{}.html", std::process::id()));
    adapter.pdf_to_html(&pdf, &out)?;
    assert!(out.exists());
    let html = std::fs::read_to_string(&out)?;
    assert!(html.contains("<!DOCTYPE html>"));
    let _ = std::fs::remove_file(&out);
    Ok(())
}

#[test]
fn test_pdf_to_rtf() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_doc_{}.rtf", std::process::id()));
    adapter.pdf_to_rtf(&pdf, &out)?;
    assert!(out.exists());
    let bytes = std::fs::read(&out)?;
    assert!(!bytes.is_empty());
    let _ = std::fs::remove_file(&out);
    Ok(())
}
