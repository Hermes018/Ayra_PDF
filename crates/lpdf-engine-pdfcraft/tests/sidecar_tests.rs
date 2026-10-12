use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_pdfcraft::office_sidecar::{
    convert_office_to_pdf, convert_pdf_to_office, get_target_sidecar_name, resolve_office_binary,
};
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{OfficeInputFormat, OfficeOutputFormat, PdfEngine};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_sidecar_target_name() {
    let name = get_target_sidecar_name();
    assert!(!name.is_empty());
    #[cfg(target_os = "linux")]
    assert!(name.contains("unknown-linux-gnu"));
    #[cfg(target_os = "windows")]
    assert!(name.contains("pc-windows-msvc"));
    #[cfg(target_os = "macos")]
    assert!(name.contains("apple-darwin"));
}

#[test]
fn test_missing_binary_graceful_error() {
    let fake_binary = PathBuf::from("/nonexistent/bin/fake_soffice_12345");
    let res = resolve_office_binary(Some(&fake_binary));
    match res {
        Err(LpdfError::ExternalDependencyError(msg)) => {
            assert!(msg.contains("does not exist"));
        }
        _ => panic!("Expected ExternalDependencyError for missing custom binary"),
    }
}

#[test]
fn test_input_path_validation_nonexistent() {
    let nonexistent_doc = PathBuf::from("/nonexistent/path/doc.docx");
    let out = std::env::temp_dir().join("test_out.pdf");
    let res = convert_office_to_pdf(&nonexistent_doc, OfficeInputFormat::Docx, &out, None);
    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("does not exist"));
        }
        _ => panic!("Expected InvalidInput for nonexistent document"),
    }
}

#[test]
fn test_input_path_validation_directory() {
    let temp_dir = std::env::temp_dir();
    let out = temp_dir.join("test_out.pdf");
    let res = convert_office_to_pdf(&temp_dir, OfficeInputFormat::Docx, &out, None);
    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("not a regular file"));
        }
        _ => panic!("Expected InvalidInput for directory input"),
    }
}

#[test]
fn test_mock_binary_failure() -> Result<()> {
    // Create a mock executable script that fails with exit code 42
    let script_path =
        std::env::temp_dir().join(format!("mock_soffice_fail_{}.sh", std::process::id()));
    fs::write(
        &script_path,
        "#!/bin/sh\necho \"Fatal sidecar error\" >&2\nexit 42\n",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    // Create a dummy input file
    let dummy_doc = std::env::temp_dir().join(format!("dummy_input_{}.docx", std::process::id()));
    fs::write(&dummy_doc, b"dummy content")?;

    let out = std::env::temp_dir().join("mock_fail_out.pdf");
    let res = convert_office_to_pdf(
        &dummy_doc,
        OfficeInputFormat::Docx,
        &out,
        Some(&script_path),
    );

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_doc);

    match res {
        Err(LpdfError::ExternalDependencyError(msg)) => {
            assert!(msg.contains("42"));
            assert!(msg.contains("Fatal sidecar error"));
        }
        _ => panic!("Expected ExternalDependencyError on non-zero exit"),
    }

    Ok(())
}

#[test]
fn test_mock_binary_success() -> Result<()> {
    // Create a mock executable script that produces a valid PDF file in the outdir
    let script_path =
        std::env::temp_dir().join(format!("mock_soffice_succ_{}.sh", std::process::id()));
    let script_body = r#"#!/bin/sh
outdir=""
for arg in "$@"; do
    if [ "$prev" = "--outdir" ]; then
        outdir="$arg"
    fi
    prev="$arg"
done
if [ -n "$outdir" ]; then
    printf '%%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%%%EOF' > "$outdir/mock_doc.pdf"
fi
exit 0
"#;
    fs::write(&script_path, script_body)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    let dummy_doc = std::env::temp_dir().join("mock_doc.docx");
    fs::write(&dummy_doc, b"mock docx content")?;

    let out = std::env::temp_dir().join(format!("mock_success_{}.pdf", std::process::id()));
    let res = convert_office_to_pdf(
        &dummy_doc,
        OfficeInputFormat::Docx,
        &out,
        Some(&script_path),
    )?;

    assert!(res.exists());
    let bytes = fs::read(&res)?;
    assert!(bytes.starts_with(b"%PDF-"));

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_doc);
    let _ = fs::remove_file(&out);

    Ok(())
}

#[test]
fn test_adapter_office_to_pdf_integration() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let dummy_doc =
        std::env::temp_dir().join(format!("integration_doc_{}.odt", std::process::id()));
    fs::write(&dummy_doc, b"sample text")?;
    let out = std::env::temp_dir().join(format!("integration_out_{}.pdf", std::process::id()));

    // If soffice is on the system, this will either succeed or fail cleanly with ExternalDependencyError
    let res = adapter.office_to_pdf(&dummy_doc, OfficeInputFormat::Odt, &out);
    match res {
        Ok(path) => {
            assert!(path.exists());
            let _ = fs::remove_file(&path);
        }
        Err(LpdfError::ExternalDependencyError(_)) => {
            // Valid fallback if headless conversion cannot run on dummy bytes
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }

    let _ = fs::remove_file(&dummy_doc);
    Ok(())
}

#[test]
fn test_office_output_format_infilters() {
    assert_eq!(OfficeOutputFormat::Docx.infilter(), "writer_pdf_import");
    assert_eq!(OfficeOutputFormat::Odt.infilter(), "writer_pdf_import");
    assert_eq!(OfficeOutputFormat::Xlsx.infilter(), "calc_pdf_import");
    assert_eq!(OfficeOutputFormat::Ods.infilter(), "calc_pdf_import");
    assert_eq!(OfficeOutputFormat::Pptx.infilter(), "impress_pdf_import");
    assert_eq!(OfficeOutputFormat::Odp.infilter(), "impress_pdf_import");

    assert_eq!(OfficeOutputFormat::Docx.extension(), "docx");
    assert_eq!(OfficeOutputFormat::Xlsx.extension(), "xlsx");
    assert_eq!(OfficeOutputFormat::Pptx.extension(), "pptx");
    assert_eq!(OfficeOutputFormat::Odt.extension(), "odt");
    assert_eq!(OfficeOutputFormat::Ods.extension(), "ods");
    assert_eq!(OfficeOutputFormat::Odp.extension(), "odp");

    assert_eq!(
        OfficeOutputFormat::from_extension("docx"),
        Some(OfficeOutputFormat::Docx)
    );
    assert_eq!(
        OfficeOutputFormat::from_extension("XLSX"),
        Some(OfficeOutputFormat::Xlsx)
    );
    assert_eq!(OfficeOutputFormat::from_extension("unknown"), None);
}

#[test]
fn test_pdf_to_office_input_validation_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/file_12345.pdf");
    let out = std::env::temp_dir().join("out.docx");
    let res = convert_pdf_to_office(&nonexistent, OfficeOutputFormat::Docx, &out, None);
    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("does not exist"));
        }
        _ => panic!("Expected InvalidInput for nonexistent file"),
    }
}

#[test]
fn test_pdf_to_office_input_validation_not_pdf() -> Result<()> {
    let dummy_non_pdf = std::env::temp_dir().join(format!("not_a_pdf_{}.txt", std::process::id()));
    fs::write(&dummy_non_pdf, b"This is plain text, not a PDF")?;
    let out = std::env::temp_dir().join("out.docx");

    let res = convert_pdf_to_office(&dummy_non_pdf, OfficeOutputFormat::Docx, &out, None);
    let _ = fs::remove_file(&dummy_non_pdf);

    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("missing %PDF- header"));
        }
        _ => panic!("Expected InvalidInput for non-PDF file"),
    }
    Ok(())
}

#[test]
fn test_mock_binary_pdf_to_office_failure() -> Result<()> {
    // Create a mock executable script that exits with code 42
    let script_path =
        std::env::temp_dir().join(format!("mock_soffice_pdf_fail_{}.sh", std::process::id()));
    fs::write(
        &script_path,
        "#!/bin/sh\necho \"Fatal sidecar convert failure\" >&2\nexit 42\n",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    let dummy_pdf = std::env::temp_dir().join(format!("dummy_pdf_fail_{}.pdf", std::process::id()));
    fs::write(
        &dummy_pdf,
        b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF",
    )?;

    let out = std::env::temp_dir().join("mock_fail_out.docx");
    let res = convert_pdf_to_office(
        &dummy_pdf,
        OfficeOutputFormat::Docx,
        &out,
        Some(&script_path),
    );

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_pdf);

    match res {
        Err(LpdfError::ExternalDependencyError(msg)) => {
            assert!(msg.contains("42"));
            assert!(msg.contains("Fatal sidecar convert failure"));
        }
        _ => panic!("Expected ExternalDependencyError on non-zero exit"),
    }

    Ok(())
}

#[test]
fn test_mock_binary_pdf_to_office_success() -> Result<()> {
    // Create a mock executable script that produces a valid zip/Office file
    let script_path =
        std::env::temp_dir().join(format!("mock_soffice_pdf_succ_{}.sh", std::process::id()));
    let script_body = r#"#!/bin/sh
outdir=""
for arg in "$@"; do
    if [ "$prev" = "--outdir" ]; then
        outdir="$arg"
    fi
    prev="$arg"
done
if [ -n "$outdir" ]; then
    printf 'PK\x03\x04mock_zip_content_stream' > "$outdir/mock_test_doc.docx"
fi
exit 0
"#;
    fs::write(&script_path, script_body)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    let dummy_pdf = std::env::temp_dir().join("mock_test_doc.pdf");
    fs::write(
        &dummy_pdf,
        b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF",
    )?;

    let out = std::env::temp_dir().join(format!(
        "mock_pdf_to_office_succ_{}.docx",
        std::process::id()
    ));
    let res = convert_pdf_to_office(
        &dummy_pdf,
        OfficeOutputFormat::Docx,
        &out,
        Some(&script_path),
    )?;

    assert!(res.exists());
    let bytes = fs::read(&res)?;
    assert!(bytes.starts_with(b"PK\x03\x04"));

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_pdf);
    let _ = fs::remove_file(&out);

    Ok(())
}

#[test]
fn test_adapter_pdf_to_office_integration() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let corpus_pdf =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-text.pdf");
    let out = std::env::temp_dir().join(format!("adapter_office_out_{}.docx", std::process::id()));

    let res = adapter.pdf_to_office(&corpus_pdf, OfficeOutputFormat::Docx, &out);
    match res {
        Ok(path) => {
            assert!(path.exists());
            let _ = fs::remove_file(&path);
        }
        Err(LpdfError::ExternalDependencyError(_)) => {
            // Expected if headless Office binary cannot convert in this environment
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }

    Ok(())
}
