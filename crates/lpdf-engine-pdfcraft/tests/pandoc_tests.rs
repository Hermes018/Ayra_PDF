use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_pdfcraft::pandoc_sidecar::{
    batch_convert_markdown, convert_doc_to_markdown, convert_markdown_to_doc,
    convert_markdown_to_pdf, convert_pdf_to_markdown, get_target_sidecar_name,
    inject_or_preserve_yaml_front_matter, parse_yaml_front_matter, resolve_pandoc_binary,
    rewrite_markdown_asset_links,
};
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{MarkdownBatchOptions, MarkdownBatchTarget, MarkdownDocFormat, PdfEngine};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_pandoc_target_name() {
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
fn test_missing_pandoc_binary_graceful_error() {
    let fake_binary = PathBuf::from("/nonexistent/bin/fake_pandoc_12345");
    let res = resolve_pandoc_binary(Some(&fake_binary));
    match res {
        Err(LpdfError::ExternalDependencyError(msg)) => {
            assert!(msg.contains("does not exist"));
        }
        _ => panic!("Expected ExternalDependencyError for missing custom binary"),
    }
}

#[test]
fn test_mock_pandoc_failure() -> Result<()> {
    let script_path =
        std::env::temp_dir().join(format!("mock_pandoc_fail_{}.sh", std::process::id()));
    fs::write(
        &script_path,
        "#!/bin/sh\necho \"Fatal pandoc execution error\" >&2\nexit 42\n",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    let dummy_md = std::env::temp_dir().join(format!("dummy_md_{}.md", std::process::id()));
    fs::write(&dummy_md, b"# Test Markdown Header")?;

    let out = std::env::temp_dir().join("mock_fail_out.docx");
    let res = convert_markdown_to_doc(&dummy_md, MarkdownDocFormat::Docx, &out, Some(&script_path));

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_md);

    match res {
        Err(LpdfError::ExternalDependencyError(msg)) => {
            assert!(msg.contains("42"));
            assert!(msg.contains("Fatal pandoc execution error"));
        }
        _ => panic!("Expected ExternalDependencyError on non-zero exit"),
    }

    Ok(())
}

#[test]
fn test_mock_markdown_to_pdf() -> Result<()> {
    // Mock pandoc script that writes standalone HTML or direct PDF
    let script_path =
        std::env::temp_dir().join(format!("mock_pandoc_md2pdf_{}.sh", std::process::id()));
    let script_body = r#"#!/bin/sh
outfile=""
for arg in "$@"; do
    if [ "$prev" = "-o" ]; then
        outfile="$arg"
    fi
    prev="$arg"
done
if [ -n "$outfile" ]; then
    printf '%%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%%%EOF' > "$outfile"
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

    let dummy_md = std::env::temp_dir().join(format!("mock_doc_{}.md", std::process::id()));
    fs::write(&dummy_md, b"# Heading\n\nSome body text here.")?;

    let out = std::env::temp_dir().join(format!("mock_md2pdf_out_{}.pdf", std::process::id()));
    let res = convert_markdown_to_pdf(&dummy_md, &out, Some(&script_path), |_html, _pdf| Ok(()))?;

    assert!(res.exists());
    let bytes = fs::read(&res)?;
    assert!(bytes.starts_with(b"%PDF-"));

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_md);
    let _ = fs::remove_file(&out);

    Ok(())
}

#[test]
fn test_mock_pdf_to_markdown_with_assets() -> Result<()> {
    let script_path =
        std::env::temp_dir().join(format!("mock_pandoc_pdf2md_{}.sh", std::process::id()));
    let script_body = r#"#!/bin/sh
outfile=""
for arg in "$@"; do
    if [ "$prev" = "-o" ]; then
        outfile="$arg"
    fi
    prev="$arg"
done
if [ -n "$outfile" ]; then
    printf '# Converted Markdown\n\n![Image](assets/test.png)\n' > "$outfile"
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

    let dummy_pdf = std::env::temp_dir().join(format!("mock_source_{}.pdf", std::process::id()));
    fs::write(
        &dummy_pdf,
        b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF",
    )?;

    let out_dir = std::env::temp_dir().join(format!("mock_pdf2md_dir_{}", std::process::id()));
    fs::create_dir_all(&out_dir)?;
    let out_md = out_dir.join("document.md");

    let res = convert_pdf_to_markdown(
        &dummy_pdf,
        &out_md,
        Some(&script_path),
        |_pdf, html_path| {
            fs::write(html_path, b"<html><body><h1>Test</h1></body></html>")?;
            Ok(())
        },
        |_pdf, assets_dir| {
            fs::write(assets_dir.join("test.png"), b"mock png image")?;
            Ok(())
        },
    )?;

    assert!(res.exists());
    let md_text = fs::read_to_string(&res)?;
    assert!(md_text.contains("# Converted Markdown"));
    assert!(md_text.contains("assets/test.png"));

    // Verify assets/ directory exists and contains the image
    let assets_file = out_dir.join("assets").join("test.png");
    assert!(assets_file.exists());

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_pdf);
    let _ = fs::remove_dir_all(&out_dir);

    Ok(())
}

#[test]
fn test_mock_markdown_to_doc_and_doc_to_markdown() -> Result<()> {
    let script_path = std::env::temp_dir().join(format!(
        "mock_pandoc_bidirectional_{}.sh",
        std::process::id()
    ));
    let script_body = r#"#!/bin/sh
outfile=""
for arg in "$@"; do
    if [ "$prev" = "-o" ]; then
        outfile="$arg"
    fi
    prev="$arg"
done
case "$outfile" in
    *.docx)
        printf 'PK\x03\x04mock_docx_stream' > "$outfile"
        ;;
    *.html)
        printf '<html><body><p>HTML</p></body></html>' > "$outfile"
        ;;
    *.md)
        printf '# Markdown Result\n' > "$outfile"
        ;;
esac
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

    let dummy_md = std::env::temp_dir().join(format!("test_bi_{}.md", std::process::id()));
    fs::write(&dummy_md, b"# Bi test")?;

    // 1. MD -> DOCX
    let out_docx = std::env::temp_dir().join(format!("test_bi_{}.docx", std::process::id()));
    let res_docx = convert_markdown_to_doc(
        &dummy_md,
        MarkdownDocFormat::Docx,
        &out_docx,
        Some(&script_path),
    )?;
    assert!(res_docx.exists());
    assert!(fs::read(&res_docx)?.starts_with(b"PK\x03\x04"));

    // 2. MD -> HTML
    let out_html = std::env::temp_dir().join(format!("test_bi_{}.html", std::process::id()));
    let res_html = convert_markdown_to_doc(
        &dummy_md,
        MarkdownDocFormat::Html,
        &out_html,
        Some(&script_path),
    )?;
    assert!(res_html.exists());
    assert!(fs::read_to_string(&res_html)?.contains("<html>"));

    // 3. DOCX -> MD
    let out_md = std::env::temp_dir().join(format!("test_from_docx_{}.md", std::process::id()));
    let res_md = convert_doc_to_markdown(
        &res_docx,
        MarkdownDocFormat::Docx,
        &out_md,
        Some(&script_path),
    )?;
    assert!(res_md.exists());
    assert!(fs::read_to_string(&res_md)?.contains("# Markdown Result"));

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&dummy_md);
    let _ = fs::remove_file(&out_docx);
    let _ = fs::remove_file(&out_html);
    let _ = fs::remove_file(&out_md);

    Ok(())
}

#[test]
fn test_adapter_markdown_methods_integration() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let dummy_md = std::env::temp_dir().join(format!("adapter_test_{}.md", std::process::id()));
    fs::write(&dummy_md, b"# Title\n\nContent paragraph.")?;
    let out_pdf = std::env::temp_dir().join(format!("adapter_out_{}.pdf", std::process::id()));

    let res = adapter.markdown_to_pdf(&dummy_md, &out_pdf);
    match res {
        Ok(path) => {
            assert!(path.exists());
            let _ = fs::remove_file(&path);
        }
        Err(LpdfError::ExternalDependencyError(_)) => {
            // Expected if Pandoc is not installed on this test host
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }

    let _ = fs::remove_file(&dummy_md);
    Ok(())
}

#[test]
fn test_yaml_front_matter_parsing_and_injection() {
    // 1. Check content without front-matter
    let raw = "# Hello World\nParagraph text.";
    let (fm, body) = parse_yaml_front_matter(raw);
    assert!(fm.is_none());
    assert_eq!(body, raw);

    // 2. Inject front matter
    let injected = inject_or_preserve_yaml_front_matter(
        raw,
        "Test Document",
        "doc.pdf",
        "2026-10-12T00:00:00Z",
        true,
    );
    assert!(injected.starts_with("---\n"));
    assert!(injected.contains("title: \"Test Document\""));
    assert!(injected.contains("source: \"doc.pdf\""));
    assert!(injected.contains("# Hello World"));

    // 3. Verify parse on injected
    let (parsed_fm, parsed_body) = parse_yaml_front_matter(&injected);
    assert!(parsed_fm.is_some());
    assert!(parsed_body.contains("# Hello World"));

    // 4. Preserving existing front-matter
    let preserve_res = inject_or_preserve_yaml_front_matter(
        &injected,
        "Different Title",
        "different.pdf",
        "2026-10-12T01:00:00Z",
        true,
    );
    assert_eq!(preserve_res, injected);

    // 5. Overwriting existing front-matter if preserve is false
    let overwrite_res = inject_or_preserve_yaml_front_matter(
        &injected,
        "Overwritten Title",
        "overwritten.pdf",
        "2026-10-12T02:00:00Z",
        false,
    );
    assert!(overwrite_res.contains("title: \"Overwritten Title\""));
    assert!(!overwrite_res.contains("title: \"Test Document\""));
}

#[test]
fn test_rewrite_markdown_asset_links() {
    let md = r#"# Sample
Here is an image: ![Alt Text](subfolder/diagram.png "Diagram Title")
Here is another: ![Photo](/var/data/tmp_123/photo.jpg)
Here is an html image: <img src="assets/nested/icon.svg" alt="icon" />
Here is a web link that should not change: ![Web](https://example.com/logo.png)
And a regular link: [Doc](files/document.pdf)
"#;

    let rewritten = rewrite_markdown_asset_links(md);
    assert!(rewritten.contains("![Alt Text](assets/diagram.png \"Diagram Title\")"));
    assert!(rewritten.contains("![Photo](assets/photo.jpg)"));
    assert!(rewritten.contains("<img src=\"assets/icon.svg\" alt=\"icon\" />"));
    assert!(rewritten.contains("![Web](https://example.com/logo.png)"));
    assert!(rewritten.contains("[Doc](files/document.pdf)"));
}

#[test]
fn test_mock_batch_convert_markdown_folder() -> Result<()> {
    let script_path =
        std::env::temp_dir().join(format!("mock_pandoc_batch_{}.sh", std::process::id()));
    let script_body = r#"#!/bin/sh
outfile=""
for arg in "$@"; do
  if [ "$prev" = "-o" ]; then
    outfile="$arg"
  fi
  prev="$arg"
done
if [ -n "$outfile" ]; then
  case "$outfile" in
    *.pdf)
      printf '%%PDF-1.4 Mock Batch PDF\n' > "$outfile"
      ;;
    *.docx)
      printf 'PK\x03\x04Mock Batch DOCX' > "$outfile"
      ;;
    *.html)
      printf '<html><body>Mock Batch HTML</body></html>' > "$outfile"
      ;;
    *.md)
      printf '# Mock Batch MD\n![Extracted](media/img1.png)\n' > "$outfile"
      ;;
    *)
      printf 'Mock Output\n' > "$outfile"
      ;;
  esac
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

    let input_dir = std::env::temp_dir().join(format!("test_batch_in_{}", std::process::id()));
    let output_dir = std::env::temp_dir().join(format!("test_batch_out_{}", std::process::id()));
    fs::create_dir_all(&input_dir)?;
    fs::create_dir_all(&output_dir)?;

    let doc1 = input_dir.join("chapter1.md");
    let doc2 = input_dir.join("chapter2.markdown");
    fs::write(
        &doc1,
        b"# Chapter 1\nSome text with ![Figure](figures/fig1.png)",
    )?;
    fs::write(&doc2, b"# Chapter 2\nAnother chapter.")?;

    let options = MarkdownBatchOptions {
        target: MarkdownBatchTarget::Pdf,
        inject_front_matter: true,
        preserve_front_matter: true,
        rewrite_assets: true,
    };

    let report = batch_convert_markdown(
        &input_dir,
        &output_dir,
        options,
        Some(&script_path),
        |_in, _out| Ok(()),
        |_in, _out| Ok(()),
        |_in, out| {
            fs::write(out, b"%PDF-1.4 Mock Rendered")?;
            Ok(())
        },
    )?;

    assert_eq!(report.total_files, 2);
    assert_eq!(report.converted_files, 2);
    assert_eq!(report.failed_files, 0);
    assert_eq!(report.output_files.len(), 2);
    assert!(output_dir.join("chapter1.pdf").exists());
    assert!(output_dir.join("chapter2.pdf").exists());

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_dir_all(&input_dir);
    let _ = fs::remove_dir_all(&output_dir);
    Ok(())
}

#[test]
fn test_adapter_batch_markdown_convert_integration() -> Result<()> {
    let adapter = PdfcraftAdapter::new();
    let input_dir = std::env::temp_dir().join(format!("adapter_batch_in_{}", std::process::id()));
    let output_dir = std::env::temp_dir().join(format!("adapter_batch_out_{}", std::process::id()));
    fs::create_dir_all(&input_dir)?;

    let doc1 = input_dir.join("notes.md");
    fs::write(&doc1, b"# Quick Notes\nBullet points.")?;

    let options = MarkdownBatchOptions {
        target: MarkdownBatchTarget::Html,
        inject_front_matter: true,
        preserve_front_matter: true,
        rewrite_assets: true,
    };

    let res = adapter.batch_markdown_convert(&input_dir, &output_dir, options);
    match res {
        Ok(report) => {
            assert_eq!(report.total_files, 1);
        }
        Err(LpdfError::ExternalDependencyError(_)) => {
            // Expected if Pandoc is not installed on test host
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }

    let _ = fs::remove_dir_all(&input_dir);
    let _ = fs::remove_dir_all(&output_dir);
    Ok(())
}
