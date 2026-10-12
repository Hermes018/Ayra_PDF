use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    AnnotationDef, MarkdownBatchOptions, MarkdownBatchReport, MarkdownDocFormat, OfficeInputFormat,
    OfficeOutputFormat, PdfAConformance, PdfAPreflightReport, PdfEngine, PdfInfo, Permissions,
    RedactionRegion, RenderOptions, WatermarkSettings,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

struct MockEngine {
    last_called: RefCell<String>,
}

impl MockEngine {
    fn new() -> Self {
        Self {
            last_called: RefCell::new(String::new()),
        }
    }
}

impl PdfEngine for MockEngine {
    fn load(&mut self, _path: &Path) -> Result<()> {
        Ok(())
    }
    fn info(&self) -> Result<PdfInfo> {
        Ok(PdfInfo {
            page_count: 1,
            title: None,
            author: None,
        })
    }
    fn render(
        &self,
        _page_index: usize,
        _options: &RenderOptions,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn page_ops_delete(&mut self, _pages: &[usize]) -> Result<()> {
        Ok(())
    }
    fn save(&self, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn merge(&self, _input_paths: &[&Path], _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn extract_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn delete_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn rotate_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _degrees: i32,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn rotate_document(
        &self,
        _input_path: &Path,
        _degrees: i32,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn compress(
        &self,
        _input_path: &Path,
        _profile: lpdf_engine_port::OptimizeProfile,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn repair(&self, _input_path: &Path, _output_path: &Path) -> Result<Vec<String>> {
        Ok(vec![])
    }
    fn ocr(&self, _input_path: &Path, _lang: &str, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn protect(
        &self,
        _input_path: &Path,
        _password: &str,
        _permissions: Permissions,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn unlock(&self, _input_path: &Path, _password: &str, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn add_annotation(
        &self,
        _input_path: &Path,
        _annotation: &AnnotationDef,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn add_watermark(
        &self,
        _input_path: &Path,
        _settings: &WatermarkSettings,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn flatten_comments(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn flatten_forms(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn fill_form(
        &self,
        _input_path: &Path,
        _fields: &std::collections::HashMap<String, String>,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn sign_document(
        &self,
        _input_path: &Path,
        _p12_path: &Path,
        _password: &str,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn redact(
        &self,
        _input_path: &Path,
        _regions: Vec<RedactionRegion>,
        _output_path: &Path,
    ) -> Result<()> {
        Ok(())
    }
    fn image_to_pdf(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn text_to_pdf(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn structural_docx_to_pdf(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn html_to_pdf(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_images(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn extract_all_images(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_text(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_structural_docx(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_html(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_rtf(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_pdfa(
        &self,
        _input_path: &Path,
        _conformance: PdfAConformance,
        _output_path: &Path,
    ) -> Result<PdfAPreflightReport> {
        Ok(PdfAPreflightReport {
            conformance: _conformance,
            is_compliant: true,
            fixed_issues: vec![],
            remaining_issues: vec![],
        })
    }
    fn office_to_pdf(
        &self,
        _input_path: &Path,
        _format: OfficeInputFormat,
        output_path: &Path,
    ) -> Result<PathBuf> {
        Ok(output_path.to_path_buf())
    }
    fn pdf_to_office(
        &self,
        _input_path: &Path,
        _format: OfficeOutputFormat,
        output_path: &Path,
    ) -> Result<PathBuf> {
        Ok(output_path.to_path_buf())
    }
    fn markdown_to_pdf(&self, _input_path: &Path, output_path: &Path) -> Result<PathBuf> {
        *self.last_called.borrow_mut() = "markdown_to_pdf".into();
        Ok(output_path.to_path_buf())
    }
    fn pdf_to_markdown(&self, _input_path: &Path, output_path: &Path) -> Result<PathBuf> {
        *self.last_called.borrow_mut() = "pdf_to_markdown".into();
        Ok(output_path.to_path_buf())
    }
    fn markdown_to_doc(
        &self,
        _input_path: &Path,
        _format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<PathBuf> {
        *self.last_called.borrow_mut() = format!("markdown_to_doc:{}", _format.extension());
        Ok(output_path.to_path_buf())
    }
    fn doc_to_markdown(
        &self,
        _input_path: &Path,
        _format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<PathBuf> {
        *self.last_called.borrow_mut() = format!("doc_to_markdown:{}", _format.extension());
        Ok(output_path.to_path_buf())
    }
    fn batch_markdown_convert(
        &self,
        _input_dir: &Path,
        output_dir: &Path,
        options: MarkdownBatchOptions,
    ) -> Result<MarkdownBatchReport> {
        *self.last_called.borrow_mut() = format!("batch_markdown_convert:{:?}", options.target);
        Ok(MarkdownBatchReport {
            total_files: 5,
            converted_files: 5,
            failed_files: 0,
            output_files: vec![output_dir.join("file1.pdf")],
            errors: vec![],
        })
    }
}

#[test]
fn test_cli_help() {
    let engine = MockEngine::new();
    let args = vec!["localpdf".into(), "--help".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    assert!(res.is_ok());
    let msg = res.unwrap();
    assert!(msg.contains("USAGE:"));
    assert!(msg.contains("localpdf <COMMAND>"));
}

#[test]
fn test_cli_markdown_help() {
    let engine = MockEngine::new();
    let args = vec!["localpdf".into(), "md".into(), "--help".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    assert!(res.is_ok());
    let msg = res.unwrap();
    assert!(msg.contains("LocalPDF Studio Markdown Toolset"));
    assert!(msg.contains("to-pdf"));
    assert!(msg.contains("from-pdf"));
    assert!(msg.contains("batch"));
}

#[test]
fn test_cli_markdown_to_pdf() {
    let engine = MockEngine::new();
    let args = vec![
        "localpdf".into(),
        "md".into(),
        "to-pdf".into(),
        "notes.md".into(),
        "notes.pdf".into(),
    ];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    assert!(res.is_ok());
    assert_eq!(*engine.last_called.borrow(), "markdown_to_pdf");
}

#[test]
fn test_cli_markdown_from_pdf() {
    let engine = MockEngine::new();
    let args = vec![
        "localpdf".into(),
        "md".into(),
        "from-pdf".into(),
        "document.pdf".into(),
        "document.md".into(),
    ];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    assert!(res.is_ok());
    assert_eq!(*engine.last_called.borrow(), "pdf_to_markdown");
}

#[test]
fn test_cli_markdown_to_doc() {
    let engine = MockEngine::new();
    let args = vec![
        "localpdf".into(),
        "md".into(),
        "to-doc".into(),
        "notes.md".into(),
        "docx".into(),
    ];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    assert!(res.is_ok());
    assert_eq!(*engine.last_called.borrow(), "markdown_to_doc:docx");
}

#[test]
fn test_cli_markdown_batch() {
    let engine = MockEngine::new();
    let args = vec![
        "localpdf".into(),
        "md".into(),
        "batch".into(),
        "/path/to/in".into(),
        "/path/to/out".into(),
        "--target".into(),
        "docx".into(),
        "--no-front-matter".into(),
    ];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    assert!(res.is_ok());
    assert!(res.unwrap().contains("Batch Markdown Conversion Complete"));
    assert_eq!(*engine.last_called.borrow(), "batch_markdown_convert:Docx");
}

#[test]
fn test_cli_unknown_command() {
    let engine = MockEngine::new();
    let args = vec!["localpdf".into(), "unknown_tool".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &engine);
    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("Unknown command 'unknown_tool'"));
        }
        other => panic!("Unexpected result: {:?}", other),
    }
}
