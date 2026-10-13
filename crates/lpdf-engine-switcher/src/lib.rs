//! Engine switcher: tries PdfcraftAdapter first, falls back to FallbackAdapter
//! on parse/corruption errors (Failsafe F7).

#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    ExtractiveSummaryReport, LlmSummaryOptions, LlmSummaryReport, MarkdownBatchOptions,
    MarkdownBatchReport, MarkdownDocFormat, OfficeInputFormat, OfficeOutputFormat, PdfAConformance,
    PdfAPreflightReport, PdfEngine, PdfInfo, RedactionRegion, RenderOptions, SummaryLength,
    TranslateOptions, TranslatePdfReport,
};
use std::path::{Path, PathBuf};

#[cfg(any(test, feature = "test-utils"))]
pub mod fuzz;
#[cfg(any(test, feature = "test-utils"))]
pub mod hostile;
#[cfg(any(test, feature = "test-utils"))]
pub mod perf;
#[cfg(any(test, feature = "test-utils"))]
pub mod sbom;
#[cfg(any(test, feature = "test-utils"))]
pub mod soak;

/// Tracks which engine is active after a load attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveEngine {
    Primary,
    Fallback,
}

/// Routes PDF operations through the primary engine (pdfcraft) first.
/// If the primary fails to load a document due to a parse or engine error,
/// it transparently retries with the fallback engine (lopdf) and flags
/// degraded mode.
pub struct EngineSwitcher {
    primary: lpdf_engine_pdfcraft::PdfcraftAdapter,
    fallback: lpdf_engine_fallback::FallbackAdapter,
    active: Option<ActiveEngine>,
    loaded_path: Option<PathBuf>,
}

impl Default for EngineSwitcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineSwitcher {
    pub fn new() -> Self {
        Self {
            primary: lpdf_engine_pdfcraft::PdfcraftAdapter::new(),
            fallback: lpdf_engine_fallback::FallbackAdapter::new(),
            active: None,
            loaded_path: None,
        }
    }

    /// Returns which engine is currently handling the loaded document, if any.
    pub fn active_engine(&self) -> Option<ActiveEngine> {
        self.active
    }

    /// Returns true when operating in degraded (fallback) mode.
    pub fn is_degraded(&self) -> bool {
        self.active == Some(ActiveEngine::Fallback)
    }

    /// Returns whether the given error is a parse/corruption error that should
    /// trigger a fallback attempt.
    fn is_fallback_worthy(err: &LpdfError) -> bool {
        matches!(err, LpdfError::Engine(_) | LpdfError::ParseError(_))
    }
}

impl PdfEngine for EngineSwitcher {
    fn load(&mut self, path: &Path) -> Result<()> {
        self.loaded_path = Some(path.to_path_buf());

        // Try primary first
        match self.primary.load(path) {
            Ok(()) => {
                tracing::info!("EngineSwitcher: loaded via primary (pdfcraft)");
                self.active = Some(ActiveEngine::Primary);
                Ok(())
            }
            Err(e) if Self::is_fallback_worthy(&e) => {
                tracing::warn!(
                    "EngineSwitcher: primary load failed ({}), attempting fallback",
                    e
                );
                self.fallback.load(path).map_err(|fallback_err| {
                    tracing::error!("EngineSwitcher: fallback also failed: {}", fallback_err);
                    fallback_err
                })?;
                self.active = Some(ActiveEngine::Fallback);
                tracing::info!("EngineSwitcher: loaded via fallback (lopdf) — degraded mode");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn info(&self) -> Result<PdfInfo> {
        match self.active {
            Some(ActiveEngine::Primary) => self.primary.info(),
            Some(ActiveEngine::Fallback) => self.fallback.info(),
            None => Err(LpdfError::Engine("No document loaded".into())),
        }
    }

    fn render(&self, page_index: usize, options: &RenderOptions, output_path: &Path) -> Result<()> {
        match self.active {
            Some(ActiveEngine::Primary) => self.primary.render(page_index, options, output_path),
            Some(ActiveEngine::Fallback) => self.fallback.render(page_index, options, output_path),
            None => Err(LpdfError::Engine("No document loaded".into())),
        }
    }

    fn page_ops_delete(&mut self, pages: &[usize]) -> Result<()> {
        match self.active {
            Some(ActiveEngine::Primary) => self.primary.page_ops_delete(pages),
            Some(ActiveEngine::Fallback) => self.fallback.page_ops_delete(pages),
            None => Err(LpdfError::Engine("No document loaded".into())),
        }
    }

    fn save(&self, output_path: &Path) -> Result<()> {
        match self.active {
            Some(ActiveEngine::Primary) => self.primary.save(output_path),
            Some(ActiveEngine::Fallback) => self.fallback.save(output_path),
            None => Err(LpdfError::Engine("No document loaded".into())),
        }
    }

    fn merge(&self, input_paths: &[&Path], output_path: &Path) -> Result<()> {
        // Merge operations are static/standalone and go through primary.
        self.primary.merge(input_paths, output_path)
    }

    fn extract_pages(&self, input_path: &Path, pages: &[usize], output_path: &Path) -> Result<()> {
        // Extract operations are static/standalone and go through primary.
        self.primary.extract_pages(input_path, pages, output_path)
    }

    fn delete_pages(&self, input_path: &Path, pages: &[usize], output_path: &Path) -> Result<()> {
        self.primary.delete_pages(input_path, pages, output_path)
    }

    fn rotate_pages(
        &self,
        input_path: &Path,
        pages: &[usize],
        degrees: i32,
        output_path: &Path,
    ) -> Result<()> {
        self.primary
            .rotate_pages(input_path, pages, degrees, output_path)
    }

    fn rotate_document(&self, input_path: &Path, degrees: i32, output_path: &Path) -> Result<()> {
        self.primary
            .rotate_document(input_path, degrees, output_path)
    }

    fn compress(
        &self,
        input_path: &Path,
        profile: lpdf_engine_port::OptimizeProfile,
        output_path: &Path,
    ) -> Result<()> {
        self.primary.compress(input_path, profile, output_path)
    }

    fn repair(&self, input_path: &Path, output_path: &Path) -> Result<Vec<String>> {
        self.primary.repair(input_path, output_path)
    }

    fn ocr(&self, input_path: &Path, lang: &str, output_path: &Path) -> Result<()> {
        self.primary.ocr(input_path, lang, output_path)
    }

    fn protect(
        &self,
        input_path: &Path,
        password: &str,
        permissions: lpdf_engine_port::Permissions,
        output_path: &Path,
    ) -> Result<()> {
        self.primary
            .protect(input_path, password, permissions, output_path)
    }

    fn unlock(&self, input_path: &Path, password: &str, output_path: &Path) -> Result<()> {
        self.primary.unlock(input_path, password, output_path)
    }

    fn add_annotation(
        &self,
        input_path: &Path,
        annotation: &lpdf_engine_port::AnnotationDef,
        output_path: &Path,
    ) -> Result<()> {
        self.primary
            .add_annotation(input_path, annotation, output_path)
    }

    fn add_watermark(
        &self,
        input_path: &Path,
        settings: &lpdf_engine_port::WatermarkSettings,
        output_path: &Path,
    ) -> Result<()> {
        self.primary
            .add_watermark(input_path, settings, output_path)
    }

    fn flatten_comments(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.flatten_comments(input_path, output_path)
    }

    fn flatten_forms(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.flatten_forms(input_path, output_path)
    }

    fn fill_form(
        &self,
        input_path: &Path,
        fields: &std::collections::HashMap<String, String>,
        output_path: &Path,
    ) -> Result<()> {
        self.primary.fill_form(input_path, fields, output_path)
    }

    fn sign_document(
        &self,
        input_path: &Path,
        p12_path: &Path,
        password: &str,
        output_path: &Path,
    ) -> Result<()> {
        self.primary
            .sign_document(input_path, p12_path, password, output_path)
    }

    fn redact(
        &self,
        input_path: &Path,
        regions: Vec<RedactionRegion>,
        output_path: &Path,
    ) -> Result<()> {
        self.primary.redact(input_path, regions, output_path)
    }

    fn image_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.image_to_pdf(input_path, output_path)
    }

    fn text_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.text_to_pdf(input_path, output_path)
    }

    fn structural_docx_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.structural_docx_to_pdf(input_path, output_path)
    }

    fn html_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.html_to_pdf(input_path, output_path)
    }

    fn pdf_to_images(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.pdf_to_images(input_path, output_path)
    }

    fn extract_all_images(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.extract_all_images(input_path, output_path)
    }

    fn pdf_to_text(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.pdf_to_text(input_path, output_path)
    }

    fn pdf_to_structural_docx(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.pdf_to_structural_docx(input_path, output_path)
    }

    fn pdf_to_html(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.pdf_to_html(input_path, output_path)
    }

    fn pdf_to_rtf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.primary.pdf_to_rtf(input_path, output_path)
    }

    fn pdf_to_pdfa(
        &self,
        input_path: &Path,
        conformance: PdfAConformance,
        output_path: &Path,
    ) -> Result<PdfAPreflightReport> {
        self.primary
            .pdf_to_pdfa(input_path, conformance, output_path)
    }

    fn office_to_pdf(
        &self,
        input_path: &Path,
        format: OfficeInputFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        self.primary.office_to_pdf(input_path, format, output_path)
    }

    fn pdf_to_office(
        &self,
        input_path: &Path,
        format: OfficeOutputFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        self.primary.pdf_to_office(input_path, format, output_path)
    }

    fn markdown_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<std::path::PathBuf> {
        self.primary.markdown_to_pdf(input_path, output_path)
    }

    fn pdf_to_markdown(&self, input_path: &Path, output_path: &Path) -> Result<std::path::PathBuf> {
        self.primary.pdf_to_markdown(input_path, output_path)
    }

    fn markdown_to_doc(
        &self,
        input_path: &Path,
        format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        self.primary
            .markdown_to_doc(input_path, format, output_path)
    }

    fn doc_to_markdown(
        &self,
        input_path: &Path,
        format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        self.primary
            .doc_to_markdown(input_path, format, output_path)
    }

    fn batch_markdown_convert(
        &self,
        input_dir: &Path,
        output_dir: &Path,
        options: MarkdownBatchOptions,
    ) -> Result<MarkdownBatchReport> {
        self.primary
            .batch_markdown_convert(input_dir, output_dir, options)
    }

    fn summarize_extractive(
        &self,
        input_path: &Path,
        output_path: &Path,
        length: SummaryLength,
    ) -> Result<ExtractiveSummaryReport> {
        self.primary
            .summarize_extractive(input_path, output_path, length)
    }

    fn summarize_llm_map_reduce(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: LlmSummaryOptions,
    ) -> Result<LlmSummaryReport> {
        self.primary
            .summarize_llm_map_reduce(input_path, output_path, options)
    }

    fn translate_pdf_experimental(
        &self,
        input_path: &Path,
        output_pdf_path: &Path,
        options: TranslateOptions,
    ) -> Result<TranslatePdfReport> {
        self.primary
            .translate_pdf_experimental(input_path, output_pdf_path, options)
    }
}
