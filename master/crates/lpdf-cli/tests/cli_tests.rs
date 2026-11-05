use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    AnnotationDef, ExtractiveSummaryReport, MarkdownBatchOptions, MarkdownBatchReport,
    MarkdownDocFormat, OfficeInputFormat, OfficeOutputFormat, PdfAConformance, PdfAPreflightReport,
    PdfEngine, PdfInfo, Permissions, RedactionRegion, RenderOptions, SummaryLength,
    WatermarkSettings,
};
use std::cell::RefCell;
use std::fs;
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

fn write_mock_pdf(output_path: &Path) {
    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(output_path, b"%PDF-1.7\nMock Output\n%%EOF");
}

impl PdfEngine for MockEngine {
    fn load(&mut self, _path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "load".into();
        Ok(())
    }
    fn info(&self) -> Result<PdfInfo> {
        *self.last_called.borrow_mut() = "info".into();
        Ok(PdfInfo {
            page_count: 2,
            title: Some("Mock Title".into()),
            author: Some("Mock Author".into()),
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
    fn merge(&self, _input_paths: &[&Path], output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "merge".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn extract_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        output_path: &Path,
    ) -> Result<()> {
        *self.last_called.borrow_mut() = "extract_pages".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn delete_pages(&self, _input_path: &Path, _pages: &[usize], output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "delete_pages".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn rotate_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _degrees: i32,
        output_path: &Path,
    ) -> Result<()> {
        *self.last_called.borrow_mut() = "rotate_pages".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn rotate_document(&self, _input_path: &Path, _degrees: i32, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "rotate_document".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn compress(
        &self,
        _input_path: &Path,
        _profile: lpdf_engine_port::OptimizeProfile,
        output_path: &Path,
    ) -> Result<()> {
        *self.last_called.borrow_mut() = "compress".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn repair(&self, _input_path: &Path, output_path: &Path) -> Result<Vec<String>> {
        *self.last_called.borrow_mut() = "repair".into();
        write_mock_pdf(output_path);
        Ok(vec!["Fixed xref table".into()])
    }
    fn ocr(&self, _input_path: &Path, _lang: &str, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "ocr".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn protect(
        &self,
        _input_path: &Path,
        _password: &str,
        _permissions: Permissions,
        output_path: &Path,
    ) -> Result<()> {
        *self.last_called.borrow_mut() = "protect".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn unlock(&self, _input_path: &Path, _password: &str, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "unlock".into();
        write_mock_pdf(output_path);
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
        output_path: &Path,
    ) -> Result<()> {
        *self.last_called.borrow_mut() = "watermark".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn flatten_comments(&self, _input_path: &Path, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "flatten_comments".into();
        write_mock_pdf(output_path);
        Ok(())
    }
    fn flatten_forms(&self, _input_path: &Path, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "flatten_forms".into();
        write_mock_pdf(output_path);
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
        output_path: &Path,
    ) -> Result<()> {
        *self.last_called.borrow_mut() = "redact".into();
        write_mock_pdf(output_path);
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
        *self.last_called.borrow_mut() = "pdf_to_images".into();
        Ok(())
    }
    fn extract_all_images(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_text(&self, _input_path: &Path, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "pdf_to_text".into();
        let _ = fs::write(output_path, "Mock Extracted Text");
        Ok(())
    }
    fn pdf_to_structural_docx(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_html(&self, _input_path: &Path, output_path: &Path) -> Result<()> {
        *self.last_called.borrow_mut() = "pdf_to_html".into();
        let _ = fs::write(output_path, "<html><body>Mock HTML</body></html>");
        Ok(())
    }
    fn pdf_to_rtf(&self, _input_path: &Path, _output_path: &Path) -> Result<()> {
        Ok(())
    }
    fn pdf_to_pdfa(
        &self,
        _input_path: &Path,
        _conformance: PdfAConformance,
        output_path: &Path,
    ) -> Result<PdfAPreflightReport> {
        *self.last_called.borrow_mut() = "pdf_to_pdfa".into();
        write_mock_pdf(output_path);
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
    fn summarize_extractive(
        &self,
        _input_path: &Path,
        output_path: &Path,
        length: SummaryLength,
    ) -> Result<ExtractiveSummaryReport> {
        *self.last_called.borrow_mut() = format!("summarize_extractive:{:?}", length);
        let content = "# Mock Summary\n- Key point 1\n- Key point 2\nSummary text";
        let _ = fs::write(output_path, content);
        Ok(ExtractiveSummaryReport {
            output_path: output_path.to_path_buf(),
            summary_text: content.to_string(),
            key_points: vec!["Key point 1".into(), "Key point 2".into()],
            keywords: vec!["mock".into(), "summary".into()],
            source_word_count: 50,
            summary_word_count: 10,
        })
    }
    fn summarize_llm_map_reduce(
        &self,
        _input_path: &Path,
        output_path: &Path,
        options: lpdf_engine_port::LlmSummaryOptions,
    ) -> Result<lpdf_engine_port::LlmSummaryReport> {
        *self.last_called.borrow_mut() = format!("summarize_llm:{:?}", options.length);
        let content = "# Mock LLM Summary\n## Executive Synthesis\nMock Executive synthesis";
        let _ = fs::write(output_path, content);
        Ok(lpdf_engine_port::LlmSummaryReport {
            output_path: output_path.to_path_buf(),
            summary_text: content.to_string(),
            chunk_summaries: vec!["Mock Chunk 1 summary".into()],
            chunks_processed: 1,
            model_sha256: options.manifest.sha256_hex,
            source_word_count: 80,
            summary_word_count: 15,
        })
    }
    fn translate_pdf_experimental(
        &self,
        _input_path: &Path,
        output_pdf_path: &Path,
        options: lpdf_engine_port::TranslateOptions,
    ) -> Result<lpdf_engine_port::TranslatePdfReport> {
        *self.last_called.borrow_mut() =
            format!("translate:{}_{}", options.source_lang, options.target_lang);
        write_mock_pdf(output_pdf_path);
        let report_path = PathBuf::from(format!("{}.report.md", output_pdf_path.display()));
        let _ = fs::write(&report_path, "# Mock Translation Report");
        Ok(lpdf_engine_port::TranslatePdfReport {
            output_pdf_path: output_pdf_path.to_path_buf(),
            report_path,
            blocks_translated: 2,
            pages_processed: 1,
            source_lang: options.source_lang,
            target_lang: options.target_lang,
            model_sha256: options.manifest.sha256_hex,
            experimental_notice: "Mock notice".to_string(),
        })
    }
}

struct TestDirGuard(PathBuf);
impl TestDirGuard {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("lpdf_test_{}_{}", name, uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        Self(dir)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for TestDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn get_corpus_pdf(filename: &str) -> PathBuf {
    let p1 = PathBuf::from(format!("../../corpus/{}", filename));
    if p1.exists() {
        return p1;
    }
    let p2 = PathBuf::from(format!("corpus/{}", filename));
    if p2.exists() {
        return p2;
    }
    panic!(
        "Could not locate corpus/{} in ../../corpus or corpus",
        filename
    );
}

#[test]
fn test_cli_help() {
    let mut engine = MockEngine::new();
    let args = vec!["localpdf".into(), "--help".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &mut engine);
    assert!(res.is_ok());
    let msg = res.unwrap();
    assert!(msg.contains("USAGE:"));
    assert!(msg.contains("localpdf <COMMAND>"));
    assert!(msg.contains("info"));
    assert!(msg.contains("merge"));
    assert!(msg.contains("pipeline"));
}

#[test]
fn test_cli_markdown_help() {
    let mut engine = MockEngine::new();
    let args = vec!["localpdf".into(), "md".into(), "--help".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &mut engine);
    assert!(res.is_ok());
    let msg = res.unwrap();
    assert!(msg.contains("LocalPDF Studio Markdown Toolset"));
    assert!(msg.contains("to-pdf"));
    assert!(msg.contains("from-pdf"));
    assert!(msg.contains("batch"));
}

#[test]
fn test_cli_pipeline_presets_listing() {
    let mut engine = MockEngine::new();
    let args = vec!["localpdf".into(), "pipeline".into(), "presets".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &mut engine);
    assert!(res.is_ok());
    let msg = res.unwrap();
    assert!(msg.contains("Built-in Workflow Presets:"));
    assert!(msg.contains("archive_searchable"));
    assert!(msg.contains("confidential_release"));
    assert!(msg.contains("clean_web_optimize"));
}

#[test]
fn test_cli_unknown_command() {
    let mut engine = MockEngine::new();
    let args = vec!["localpdf".into(), "unknown_tool".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &mut engine);
    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("Unknown command 'unknown_tool'"));
        }
        other => panic!("Unexpected result: {:?}", other),
    }
}

#[test]
fn test_cli_missing_input_file() {
    let mut engine = MockEngine::new();
    let args = vec![
        "localpdf".into(),
        "info".into(),
        "/nonexistent/path/to/missing.pdf".into(),
    ];
    let res = lpdf_cli::run_cli_with_engine(&args, &mut engine);
    match res {
        Err(LpdfError::InvalidInput(msg)) => {
            assert!(msg.contains("does not exist"));
        }
        other => panic!("Expected InvalidInput error, got: {:?}", other),
    }
}

#[test]
fn test_cli_argument_parsing_errors() {
    let mut engine = MockEngine::new();
    let sample_pdf = get_corpus_pdf("standard-text.pdf");

    // rotate missing --degrees
    let args = vec![
        "localpdf".into(),
        "rotate".into(),
        sample_pdf.to_string_lossy().to_string(),
        "out.pdf".into(),
    ];
    assert!(lpdf_cli::run_cli_with_engine(&args, &mut engine).is_err());

    // rotate invalid degrees
    let args = vec![
        "localpdf".into(),
        "rotate".into(),
        sample_pdf.to_string_lossy().to_string(),
        "out.pdf".into(),
        "--degrees".into(),
        "45".into(),
    ];
    assert!(lpdf_cli::run_cli_with_engine(&args, &mut engine).is_err());

    // protect missing --password
    let args = vec![
        "localpdf".into(),
        "protect".into(),
        sample_pdf.to_string_lossy().to_string(),
        "out.pdf".into(),
    ];
    assert!(lpdf_cli::run_cli_with_engine(&args, &mut engine).is_err());

    // watermark missing --text
    let args = vec![
        "localpdf".into(),
        "watermark".into(),
        sample_pdf.to_string_lossy().to_string(),
        "out.pdf".into(),
    ];
    assert!(lpdf_cli::run_cli_with_engine(&args, &mut engine).is_err());
}

#[test]
fn test_e2e_cli_info_text_and_json() {
    let pdf_path = get_corpus_pdf("standard-text.pdf");

    // Standard info
    let args_text = vec![
        "localpdf".into(),
        "info".into(),
        pdf_path.to_string_lossy().to_string(),
    ];
    let res_text = lpdf_cli::run_cli(&args_text);
    assert!(res_text.is_ok(), "info command failed: {:?}", res_text);
    let out_text = res_text.unwrap();
    assert!(out_text.contains("Pages:"));
    assert!(out_text.contains("PDF Version:"));
    assert!(out_text.contains("Encrypted: No"));

    // JSON info
    let args_json = vec![
        "localpdf".into(),
        "info".into(),
        pdf_path.to_string_lossy().to_string(),
        "--json".into(),
    ];
    let res_json = lpdf_cli::run_cli(&args_json);
    assert!(res_json.is_ok(), "info --json failed: {:?}", res_json);
    let out_json = res_json.unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&out_json).expect("valid json");
    assert!(parsed.get("page_count").is_some());
    assert_eq!(parsed["encrypted"], false);
    assert!(parsed.get("version").is_some());
}

#[test]
fn test_e2e_cli_merge() {
    let guard = TestDirGuard::new("merge");
    let pdf1 = get_corpus_pdf("standard-text.pdf");
    let out_pdf = guard.path().join("merged.pdf");

    let args = vec![
        "localpdf".into(),
        "merge".into(),
        out_pdf.to_string_lossy().to_string(),
        pdf1.to_string_lossy().to_string(),
        pdf1.to_string_lossy().to_string(),
    ];
    let res = lpdf_cli::run_cli(&args);
    assert!(res.is_ok(), "merge failed: {:?}", res);
    assert!(out_pdf.exists());
    let bytes = fs::read(&out_pdf).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_e2e_cli_rotate_document_and_pages() {
    let guard = TestDirGuard::new("rotate");
    let pdf = get_corpus_pdf("standard-text.pdf");

    // Document rotation
    let out_doc = guard.path().join("rotated_doc.pdf");
    let args_doc = vec![
        "localpdf".into(),
        "rotate".into(),
        pdf.to_string_lossy().to_string(),
        out_doc.to_string_lossy().to_string(),
        "--degrees".into(),
        "90".into(),
    ];
    let res = lpdf_cli::run_cli(&args_doc);
    assert!(res.is_ok(), "rotate doc failed: {:?}", res);
    assert!(out_doc.exists());

    // Page rotation
    let out_pages = guard.path().join("rotated_pages.pdf");
    let args_pages = vec![
        "localpdf".into(),
        "rotate".into(),
        pdf.to_string_lossy().to_string(),
        out_pages.to_string_lossy().to_string(),
        "--degrees".into(),
        "180".into(),
        "--pages".into(),
        "1".into(),
    ];
    let res = lpdf_cli::run_cli(&args_pages);
    assert!(res.is_ok(), "rotate pages failed: {:?}", res);
    assert!(out_pages.exists());
}

#[test]
fn test_e2e_cli_extract_and_delete() {
    let guard = TestDirGuard::new("extract_delete");
    let pdf = get_corpus_pdf("standard-text.pdf");

    // Extract page 1
    let out_extract = guard.path().join("extracted.pdf");
    let args_extract = vec![
        "localpdf".into(),
        "extract".into(),
        pdf.to_string_lossy().to_string(),
        out_extract.to_string_lossy().to_string(),
        "--pages".into(),
        "1".into(),
    ];
    let res_ext = lpdf_cli::run_cli(&args_extract);
    assert!(res_ext.is_ok(), "extract failed: {:?}", res_ext);
    assert!(out_extract.exists());

    // Delete page 1 on mock or copy
    let mut mock = MockEngine::new();
    let args_del = vec![
        "localpdf".into(),
        "delete".into(),
        pdf.to_string_lossy().to_string(),
        guard
            .path()
            .join("deleted.pdf")
            .to_string_lossy()
            .to_string(),
        "--pages".into(),
        "1".into(),
    ];
    let res_del = lpdf_cli::run_cli_with_engine(&args_del, &mut mock);
    assert!(res_del.is_ok(), "delete failed: {:?}", res_del);
    assert_eq!(*mock.last_called.borrow(), "delete_pages");
}

#[test]
fn test_e2e_cli_split() {
    let guard = TestDirGuard::new("split");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out_dir = guard.path().join("split_output");

    let args = vec![
        "localpdf".into(),
        "split".into(),
        pdf.to_string_lossy().to_string(),
        out_dir.to_string_lossy().to_string(),
    ];
    let res = lpdf_cli::run_cli(&args);
    assert!(res.is_ok(), "split failed: {:?}", res);
    assert!(out_dir.exists());
    let page1 = out_dir.join("page_1.pdf");
    assert!(page1.exists());
}

#[test]
fn test_e2e_cli_compress_and_repair() {
    let guard = TestDirGuard::new("compress_repair");
    let pdf = get_corpus_pdf("standard-text.pdf");

    // Compress
    let out_comp = guard.path().join("compressed.pdf");
    let args_comp = vec![
        "localpdf".into(),
        "compress".into(),
        pdf.to_string_lossy().to_string(),
        out_comp.to_string_lossy().to_string(),
        "--profile".into(),
        "high".into(),
    ];
    let res_comp = lpdf_cli::run_cli(&args_comp);
    assert!(res_comp.is_ok(), "compress failed: {:?}", res_comp);
    assert!(out_comp.exists());

    // Repair
    let out_repair = guard.path().join("repaired.pdf");
    let args_repair = vec![
        "localpdf".into(),
        "repair".into(),
        pdf.to_string_lossy().to_string(),
        out_repair.to_string_lossy().to_string(),
    ];
    let res_repair = lpdf_cli::run_cli(&args_repair);
    assert!(res_repair.is_ok(), "repair failed: {:?}", res_repair);
    assert!(out_repair.exists());
}

#[test]
fn test_e2e_cli_watermark_and_flattens() {
    let guard = TestDirGuard::new("watermark_flattens");
    let pdf = get_corpus_pdf("standard-text.pdf");

    // Watermark
    let out_wm = guard.path().join("watermarked.pdf");
    let args_wm = vec![
        "localpdf".into(),
        "watermark".into(),
        pdf.to_string_lossy().to_string(),
        out_wm.to_string_lossy().to_string(),
        "--text".into(),
        "LOCALPDF CONFIDENTIAL".into(),
        "--opacity".into(),
        "0.4".into(),
        "--rotation".into(),
        "45".into(),
    ];
    let res_wm = lpdf_cli::run_cli(&args_wm);
    assert!(res_wm.is_ok(), "watermark failed: {:?}", res_wm);
    assert!(out_wm.exists());

    // Flatten comments
    let out_fc = guard.path().join("flattened_comments.pdf");
    let args_fc = vec![
        "localpdf".into(),
        "flatten-comments".into(),
        pdf.to_string_lossy().to_string(),
        out_fc.to_string_lossy().to_string(),
    ];
    let res_fc = lpdf_cli::run_cli(&args_fc);
    assert!(res_fc.is_ok(), "flatten-comments failed: {:?}", res_fc);
    assert!(out_fc.exists());

    // Flatten forms
    let form_pdf = get_corpus_pdf("standard-form.pdf");
    let out_ff = guard.path().join("flattened_forms.pdf");
    let args_ff = vec![
        "localpdf".into(),
        "flatten-forms".into(),
        form_pdf.to_string_lossy().to_string(),
        out_ff.to_string_lossy().to_string(),
    ];
    let res_ff = lpdf_cli::run_cli(&args_ff);
    assert!(res_ff.is_ok(), "flatten-forms failed: {:?}", res_ff);
    assert!(out_ff.exists());
}

#[test]
fn test_e2e_cli_protect_and_unlock_roundtrip() {
    let guard = TestDirGuard::new("protect_unlock");
    let pdf = get_corpus_pdf("standard-text.pdf");

    // Protect
    let out_protected = guard.path().join("protected.pdf");
    let args_protect = vec![
        "localpdf".into(),
        "protect".into(),
        pdf.to_string_lossy().to_string(),
        out_protected.to_string_lossy().to_string(),
        "--password".into(),
        "TestPass123!".into(),
        "--allow-print".into(),
    ];
    let res_prot = lpdf_cli::run_cli(&args_protect);
    assert!(res_prot.is_ok(), "protect failed: {:?}", res_prot);
    assert!(out_protected.exists());

    // Unlock
    let out_unlocked = guard.path().join("unlocked.pdf");
    let args_unlock = vec![
        "localpdf".into(),
        "unlock".into(),
        out_protected.to_string_lossy().to_string(),
        out_unlocked.to_string_lossy().to_string(),
        "--password".into(),
        "TestPass123!".into(),
    ];
    let res_unlock = lpdf_cli::run_cli(&args_unlock);
    assert!(res_unlock.is_ok(), "unlock failed: {:?}", res_unlock);
    assert!(out_unlocked.exists());
}

#[test]
fn test_e2e_cli_redact_f25() {
    let guard = TestDirGuard::new("redact");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out_redacted = guard.path().join("redacted.pdf");

    let args = vec![
        "localpdf".into(),
        "redact".into(),
        pdf.to_string_lossy().to_string(),
        out_redacted.to_string_lossy().to_string(),
        "--region".into(),
        "1:100.0,150.0,200.0,50.0".into(),
    ];
    let res = lpdf_cli::run_cli(&args);
    assert!(res.is_ok(), "redact failed: {:?}", res);
    assert!(out_redacted.exists());
}

#[test]
fn test_e2e_cli_pdfa_archival() {
    let guard = TestDirGuard::new("pdfa");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out_pdfa = guard.path().join("archival_pdfa.pdf");

    let args = vec![
        "localpdf".into(),
        "pdfa".into(),
        pdf.to_string_lossy().to_string(),
        out_pdfa.to_string_lossy().to_string(),
        "--conformance".into(),
        "2b".into(),
    ];
    let res = lpdf_cli::run_cli(&args);
    assert!(res.is_ok(), "pdfa conversion failed: {:?}", res);
    assert!(out_pdfa.exists());
    let out_str = res.unwrap();
    assert!(out_str.contains("PdfA2b"));
}

#[test]
fn test_e2e_cli_pipeline_run() {
    let guard = TestDirGuard::new("pipeline_run");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out_pipeline = guard.path().join("pipeline_out.pdf");

    let args = vec![
        "localpdf".into(),
        "pipeline".into(),
        "run".into(),
        "confidential_release".into(),
        pdf.to_string_lossy().to_string(),
        out_pipeline.to_string_lossy().to_string(),
    ];
    let res = lpdf_cli::run_cli(&args);
    assert!(res.is_ok(), "pipeline run failed: {:?}", res);
    assert!(out_pipeline.exists());
    let bytes = fs::read(&out_pipeline).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
fn test_e2e_cli_conversions() {
    let guard = TestDirGuard::new("conversions");
    let pdf = get_corpus_pdf("standard-text.pdf");

    // To text
    let out_txt = guard.path().join("extracted.txt");
    let args_txt = vec![
        "localpdf".into(),
        "to-text".into(),
        pdf.to_string_lossy().to_string(),
        out_txt.to_string_lossy().to_string(),
    ];
    let res_txt = lpdf_cli::run_cli(&args_txt);
    assert!(res_txt.is_ok(), "to-text failed: {:?}", res_txt);
    assert!(out_txt.exists());

    // To HTML
    let out_html = guard.path().join("exported.html");
    let args_html = vec![
        "localpdf".into(),
        "to-html".into(),
        pdf.to_string_lossy().to_string(),
        out_html.to_string_lossy().to_string(),
    ];
    let res_html = lpdf_cli::run_cli(&args_html);
    assert!(res_html.is_ok(), "to-html failed: {:?}", res_html);
    assert!(out_html.exists());

    // To images
    let out_img_dir = guard.path().join("exported_images");
    let args_img = vec![
        "localpdf".into(),
        "to-images".into(),
        pdf.to_string_lossy().to_string(),
        out_img_dir.to_string_lossy().to_string(),
    ];
    let res_img = lpdf_cli::run_cli(&args_img);
    assert!(res_img.is_ok(), "to-images failed: {:?}", res_img);
    assert!(out_img_dir.exists());

    // OCR: handles F23 gracefully when models are not installed
    let out_ocr = guard.path().join("ocr_output.pdf");
    let args_ocr = vec![
        "localpdf".into(),
        "ocr".into(),
        pdf.to_string_lossy().to_string(),
        out_ocr.to_string_lossy().to_string(),
    ];
    let res_ocr = lpdf_cli::run_cli(&args_ocr);
    match res_ocr {
        Ok(_) => assert!(out_ocr.exists()),
        Err(LpdfError::OcrInferenceError(_)) => {}
        Err(other) => panic!("Unexpected OCR error: {:?}", other),
    }

    let mut mock = MockEngine::new();
    let res_mock_ocr = lpdf_cli::run_cli_with_engine(&args_ocr, &mut mock);
    assert!(res_mock_ocr.is_ok());
    assert_eq!(*mock.last_called.borrow(), "ocr");

    // Summarize: extractive summary
    let out_sum = guard.path().join("summary.md");
    let args_sum = vec![
        "localpdf".into(),
        "summarize".into(),
        pdf.to_string_lossy().to_string(),
        out_sum.to_string_lossy().to_string(),
        "--length".into(),
        "short".into(),
    ];
    let res_sum = lpdf_cli::run_cli(&args_sum);
    assert!(res_sum.is_ok(), "summarize failed: {:?}", res_sum);
    assert!(out_sum.exists());

    let mut mock_sum = MockEngine::new();
    let res_mock_sum = lpdf_cli::run_cli_with_engine(&args_sum, &mut mock_sum);
    assert!(res_mock_sum.is_ok());
    assert_eq!(*mock_sum.last_called.borrow(), "summarize_extractive:Short");
}

#[test]
fn test_cli_summarize_options_and_lengths() {
    let guard = TestDirGuard::new("cli_sum_options");
    let pdf = guard.path().join("dummy.pdf");
    write_mock_pdf(&pdf);
    let out = guard.path().join("summary.md");

    let mut mock = MockEngine::new();

    // Default medium
    let args1 = vec![
        "localpdf".into(),
        "summarize".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
    ];
    let res1 = lpdf_cli::run_cli_with_engine(&args1, &mut mock);
    assert!(res1.is_ok());
    assert_eq!(*mock.last_called.borrow(), "summarize_extractive:Medium");

    // Detailed
    let args2 = vec![
        "localpdf".into(),
        "summarize".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
        "--length".into(),
        "detailed".into(),
    ];
    let res2 = lpdf_cli::run_cli_with_engine(&args2, &mut mock);
    assert!(res2.is_ok());
    assert_eq!(*mock.last_called.borrow(), "summarize_extractive:Detailed");

    // Invalid length
    let args_bad = vec![
        "localpdf".into(),
        "summarize".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
        "--length".into(),
        "unknown_len".into(),
    ];
    let res_bad = lpdf_cli::run_cli_with_engine(&args_bad, &mut mock);
    assert!(res_bad.is_err());
}

#[test]
fn test_cli_summarize_llm_map_reduce() {
    let _g = TestDirGuard::new("cli_sum_llm");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out = _g.path().join("llm_summary.md");
    let model = _g.path().join("mock_model.gguf");
    let _ = fs::write(&model, b"GGUF_dummy_bytes");

    let mut mock = MockEngine::new();

    // Missing model error
    let args_missing_model = vec![
        "localpdf".into(),
        "summarize".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
        "--llm".into(),
    ];
    let res_err = lpdf_cli::run_cli_with_engine(&args_missing_model, &mut mock);
    assert!(res_err.is_err());

    // Valid LLM map-reduce execution
    let args_ok = vec![
        "localpdf".into(),
        "summarize".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
        "--llm".into(),
        "--model".into(),
        model.to_string_lossy().to_string(),
        "--sha256".into(),
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
        "--license".into(),
        "Apache-2.0".into(),
        "--acknowledge-license".into(),
        "--chunk-words".into(),
        "250".into(),
    ];
    let res_ok = lpdf_cli::run_cli_with_engine(&args_ok, &mut mock);
    assert!(res_ok.is_ok(), "LLM summarize failed: {:?}", res_ok);
    assert_eq!(*mock.last_called.borrow(), "summarize_llm:Medium");
}

#[test]
fn test_cli_translate_help() {
    let mut mock = MockEngine::new();
    let args = vec!["localpdf".into(), "translate".into(), "--help".into()];
    let res = lpdf_cli::run_cli_with_engine(&args, &mut mock);
    assert!(res.is_ok());
    let msg = res.unwrap();
    assert!(msg.contains("Usage: localpdf translate"));
    assert!(msg.contains("--to <lang>"));
}

#[test]
fn test_cli_translate_missing_args() {
    let _g = TestDirGuard::new("cli_trans_err");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out = _g.path().join("translated.pdf");
    let mut mock = MockEngine::new();

    // Missing --to
    let args1 = vec![
        "localpdf".into(),
        "translate".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
    ];
    let res1 = lpdf_cli::run_cli_with_engine(&args1, &mut mock);
    assert!(res1.is_err());

    // Missing --model
    let args2 = vec![
        "localpdf".into(),
        "translate".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
        "--to".into(),
        "es".into(),
    ];
    let res2 = lpdf_cli::run_cli_with_engine(&args2, &mut mock);
    assert!(res2.is_err());
}

#[test]
fn test_cli_translate_success() {
    let _g = TestDirGuard::new("cli_trans_ok");
    let pdf = get_corpus_pdf("standard-text.pdf");
    let out = _g.path().join("translated.pdf");
    let model = _g.path().join("mock_translate_model.gguf");
    let _ = fs::write(&model, b"GGUF_dummy_model_bytes");

    let mut mock = MockEngine::new();

    let args = vec![
        "localpdf".into(),
        "translate".into(),
        pdf.to_string_lossy().to_string(),
        out.to_string_lossy().to_string(),
        "--from".into(),
        "en".into(),
        "--to".into(),
        "es".into(),
        "--model".into(),
        model.to_string_lossy().to_string(),
        "--sha256".into(),
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
        "--license".into(),
        "MIT".into(),
        "--acknowledge-license".into(),
    ];

    let res = lpdf_cli::run_cli_with_engine(&args, &mut mock);
    assert!(res.is_ok(), "Translate CLI failed: {:?}", res);
    let msg = res.unwrap();
    assert!(msg.contains("PDF translated successfully"));
    assert!(msg.contains("Companion report"));
    assert_eq!(*mock.last_called.borrow(), "translate:en_es");
}
