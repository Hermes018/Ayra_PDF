use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{PdfAConformance, PdfEngine};
use std::path::PathBuf;

fn get_test_pdf() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-text.pdf")
}

#[test]
fn test_pdf_to_pdfa_2b() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_pdfa2b_{}.pdf", std::process::id()));

    let report = adapter.pdf_to_pdfa(&pdf, PdfAConformance::PdfA2b, &out)?;
    assert_eq!(report.conformance, PdfAConformance::PdfA2b);
    assert!(out.exists());

    let bytes = std::fs::read(&out)?;
    let content = String::from_utf8_lossy(&bytes);

    // Verify PDF/A-2b XMP metadata packet and output intent presence
    assert!(content.contains("<pdfaid:part>2</pdfaid:part>"));
    assert!(content.contains("<pdfaid:conformance>B</pdfaid:conformance>"));
    assert!(content.contains("GTS_PDFA1"));

    // Verify preflight report fixed items
    assert!(!report.fixed_issues.is_empty());

    let _ = std::fs::remove_file(&out);
    Ok(())
}

#[test]
fn test_pdf_to_pdfa_3b() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_pdfa3b_{}.pdf", std::process::id()));

    let report = adapter.pdf_to_pdfa(&pdf, PdfAConformance::PdfA3b, &out)?;
    assert_eq!(report.conformance, PdfAConformance::PdfA3b);
    assert!(out.exists());

    let bytes = std::fs::read(&out)?;
    let content = String::from_utf8_lossy(&bytes);

    // Verify PDF/A-3b XMP metadata packet presence
    assert!(content.contains("<pdfaid:part>3</pdfaid:part>"));
    assert!(content.contains("<pdfaid:conformance>B</pdfaid:conformance>"));

    let _ = std::fs::remove_file(&out);
    Ok(())
}

#[test]
fn test_pdf_to_pdfa_1b_unsupported() {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let out = std::env::temp_dir().join(format!("lpdf_test_pdfa1b_{}.pdf", std::process::id()));

    let res = adapter.pdf_to_pdfa(&pdf, PdfAConformance::PdfA1b, &out);
    match res {
        Err(LpdfError::Unsupported(msg)) => {
            assert!(msg.contains("PDF/A-1b"));
        }
        _ => panic!("Expected Unsupported error for PDF/A-1b"),
    }
}

#[test]
fn test_pdf_to_pdfa_dir_target() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let pdf = get_test_pdf();
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_pdfa_dir_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir)?;

    let report = adapter.pdf_to_pdfa(&pdf, PdfAConformance::PdfA2b, &temp_dir)?;
    assert_eq!(report.conformance, PdfAConformance::PdfA2b);

    let expected_file = temp_dir.join("standard-text_pdfa.pdf");
    assert!(expected_file.exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
    Ok(())
}
