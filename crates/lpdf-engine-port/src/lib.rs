#![forbid(unsafe_code)]

use lpdf_core_types::Result;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfInfo {
    pub page_count: usize,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub dpi: u32,
    pub format: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum OptimizeProfile {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct Permissions {
    pub allow_print: bool,
    pub allow_copy: bool,
    pub allow_edit: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct WatermarkSettings {
    pub text: String,
    pub opacity: f64,
    pub rotation: f64,
    pub color: [f64; 3],
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum AnnotationShape {
    StickyNote { x: f64, y: f64, text: String },
    Highlight { quads: Vec<[f64; 8]> },
    Underline { quads: Vec<[f64; 8]> },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct AnnotationDef {
    pub page: usize,
    pub shape: AnnotationShape,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct RedactionRegion {
    pub page: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum PdfAConformance {
    PdfA1b,
    PdfA2b,
    PdfA3b,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct PdfAPreflightReport {
    pub conformance: PdfAConformance,
    pub is_compliant: bool,
    pub fixed_issues: Vec<String>,
    pub remaining_issues: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum OfficeInputFormat {
    Docx,
    Xlsx,
    Pptx,
    Odt,
    Ods,
    Odp,
}

impl OfficeInputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Odt => "odt",
            Self::Ods => "ods",
            Self::Odp => "odp",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "docx" => Some(Self::Docx),
            "xlsx" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            "odt" => Some(Self::Odt),
            "ods" => Some(Self::Ods),
            "odp" => Some(Self::Odp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum OfficeOutputFormat {
    Docx,
    Xlsx,
    Pptx,
    Odt,
    Ods,
    Odp,
}

impl OfficeOutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Xlsx => "xlsx",
            Self::Pptx => "pptx",
            Self::Odt => "odt",
            Self::Ods => "ods",
            Self::Odp => "odp",
        }
    }

    pub fn infilter(&self) -> &'static str {
        match self {
            Self::Docx | Self::Odt => "writer_pdf_import",
            Self::Xlsx | Self::Ods => "calc_pdf_import",
            Self::Pptx | Self::Odp => "impress_pdf_import",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "docx" => Some(Self::Docx),
            "xlsx" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            "odt" => Some(Self::Odt),
            "ods" => Some(Self::Ods),
            "odp" => Some(Self::Odp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum MarkdownDocFormat {
    Docx,
    Html,
}

impl MarkdownDocFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Html => "html",
        }
    }

    pub fn pandoc_format(&self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Html => "html5",
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "docx" => Some(Self::Docx),
            "html" | "htm" => Some(Self::Html),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum MarkdownBatchTarget {
    Pdf,
    Docx,
    Html,
    Markdown,
}

impl MarkdownBatchTarget {
    pub fn default_extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Docx => "docx",
            Self::Html => "html",
            Self::Markdown => "md",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct MarkdownBatchOptions {
    pub target: MarkdownBatchTarget,
    pub inject_front_matter: bool,
    pub preserve_front_matter: bool,
    pub rewrite_assets: bool,
}

impl Default for MarkdownBatchOptions {
    fn default() -> Self {
        Self {
            target: MarkdownBatchTarget::Pdf,
            inject_front_matter: true,
            preserve_front_matter: true,
            rewrite_assets: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct MarkdownBatchReport {
    pub total_files: usize,
    pub converted_files: usize,
    pub failed_files: usize,
    pub output_files: Vec<std::path::PathBuf>,
    pub errors: Vec<String>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, ts_rs::TS,
)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum SummaryLength {
    Short,
    #[default]
    Medium,
    Detailed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct ExtractiveSummaryReport {
    pub output_path: std::path::PathBuf,
    pub summary_text: String,
    pub key_points: Vec<String>,
    pub keywords: Vec<String>,
    pub source_word_count: usize,
    pub summary_word_count: usize,
}

pub trait PdfEngine {
    fn load(&mut self, path: &Path) -> Result<()>;
    fn info(&self) -> Result<PdfInfo>;
    fn render(&self, page_index: usize, options: &RenderOptions, output_path: &Path) -> Result<()>;
    fn page_ops_delete(&mut self, pages: &[usize]) -> Result<()>;
    fn save(&self, output_path: &Path) -> Result<()>;
    fn merge(&self, input_paths: &[&Path], output_path: &Path) -> Result<()>;
    fn extract_pages(&self, input_path: &Path, pages: &[usize], output_path: &Path) -> Result<()>;
    fn delete_pages(&self, input_path: &Path, pages: &[usize], output_path: &Path) -> Result<()>;
    fn rotate_pages(
        &self,
        input_path: &Path,
        pages: &[usize],
        degrees: i32,
        output_path: &Path,
    ) -> Result<()>;
    fn rotate_document(&self, input_path: &Path, degrees: i32, output_path: &Path) -> Result<()>;
    fn compress(
        &self,
        input_path: &Path,
        profile: OptimizeProfile,
        output_path: &Path,
    ) -> Result<()>;
    fn repair(&self, input_path: &Path, output_path: &Path) -> Result<Vec<String>>;
    fn ocr(&self, input_path: &Path, lang: &str, output_path: &Path) -> Result<()>;
    fn protect(
        &self,
        input_path: &Path,
        password: &str,
        permissions: Permissions,
        output_path: &Path,
    ) -> Result<()>;
    fn unlock(&self, input_path: &Path, password: &str, output_path: &Path) -> Result<()>;
    fn add_annotation(
        &self,
        input_path: &Path,
        annotation: &AnnotationDef,
        output_path: &Path,
    ) -> Result<()>;

    fn add_watermark(
        &self,
        input_path: &Path,
        settings: &WatermarkSettings,
        output_path: &Path,
    ) -> Result<()>;
    fn flatten_comments(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn flatten_forms(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn fill_form(
        &self,
        input_path: &Path,
        fields: &std::collections::HashMap<String, String>,
        output_path: &Path,
    ) -> Result<()>;
    fn sign_document(
        &self,
        input_path: &Path,
        p12_path: &Path,
        password: &str,
        output_path: &Path,
    ) -> Result<()>;
    fn redact(
        &self,
        input_path: &Path,
        regions: Vec<RedactionRegion>,
        output_path: &Path,
    ) -> Result<()>;

    // Conversions to PDF
    fn image_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn text_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn structural_docx_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn html_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()>;

    // Conversions from PDF (Phase 8.1)
    fn pdf_to_images(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn extract_all_images(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn pdf_to_text(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn pdf_to_structural_docx(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn pdf_to_html(&self, input_path: &Path, output_path: &Path) -> Result<()>;
    fn pdf_to_rtf(&self, input_path: &Path, output_path: &Path) -> Result<()>;

    // Conversions from PDF (Phase 8.2 PDF/A Archival & Preflight)
    fn pdf_to_pdfa(
        &self,
        input_path: &Path,
        conformance: PdfAConformance,
        output_path: &Path,
    ) -> Result<PdfAPreflightReport>;

    // Advanced Conversion via Headless Sidecar (Phase 9.1)
    fn office_to_pdf(
        &self,
        input_path: &Path,
        format: OfficeInputFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf>;

    // Advanced Conversion via Headless Sidecar (Phase 9.2)
    fn pdf_to_office(
        &self,
        input_path: &Path,
        format: OfficeOutputFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf>;

    // Markdown Toolset via Pandoc Sidecar (Phase 9.3)
    fn markdown_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<std::path::PathBuf>;

    fn pdf_to_markdown(&self, input_path: &Path, output_path: &Path) -> Result<std::path::PathBuf>;

    fn markdown_to_doc(
        &self,
        input_path: &Path,
        format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf>;

    fn doc_to_markdown(
        &self,
        input_path: &Path,
        format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf>;

    // Batch Markdown Folder Conversion (Phase 9.4)
    fn batch_markdown_convert(
        &self,
        input_dir: &Path,
        output_dir: &Path,
        options: MarkdownBatchOptions,
    ) -> Result<MarkdownBatchReport>;

    // Extractive PDF Summarizer (Phase 10.5a)
    fn summarize_extractive(
        &self,
        input_path: &Path,
        output_path: &Path,
        length: SummaryLength,
    ) -> Result<ExtractiveSummaryReport>;
}

#[macro_export]
macro_rules! define_contract_tests {
    ($adapter:ty) => {
        #[cfg(test)]
        mod contract_tests {
            use super::*;
            use lpdf_engine_port::{PdfEngine, PdfInfo};
            use std::fs;
            use std::path::PathBuf;

            fn get_test_pdf() -> PathBuf {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-text.pdf")
            }

            #[test]
            fn test_load_and_info() -> Result<()> {
                let mut engine: $adapter = Default::default();
                let path = get_test_pdf();
                engine.load(&path)?;
                let info = engine.info()?;
                assert!(info.page_count >= 1);
                Ok(())
            }

            #[test]
            fn test_delete_and_save() -> Result<()> {
                let mut engine: $adapter = Default::default();
                let path = get_test_pdf();
                engine.load(&path)?;
                // We're deleting page 0. We wrap it in a condition just in case it has only 1 page.
                let info = engine.info()?;
                if info.page_count > 1 {
                    engine.page_ops_delete(&[0])?;
                }

                let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
                    "../../corpus/test_contract_out_{}.pdf",
                    stringify!($adapter)
                ));
                engine.save(&out_path)?;
                assert!(out_path.exists());
                Ok(())
            }
        }
    };
}
