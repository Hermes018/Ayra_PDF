use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_pdfcraft::office_sidecar::{
    convert_office_to_pdf, get_target_sidecar_name, resolve_office_binary,
};
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{OfficeInputFormat, PdfEngine};
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
