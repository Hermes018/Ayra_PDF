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
