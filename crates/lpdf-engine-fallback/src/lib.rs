#![forbid(unsafe_code)]

use lopdf::Document;
use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{PdfEngine, PdfInfo, RenderOptions};
use std::path::Path;

pub struct FallbackAdapter {
    doc: Option<Document>,
}

impl Default for FallbackAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl FallbackAdapter {
    pub fn new() -> Self {
        Self { doc: None }
    }
}

impl PdfEngine for FallbackAdapter {
    fn load(&mut self, path: &Path) -> Result<()> {
        let doc = Document::load(path).map_err(|e| LpdfError::Engine(e.to_string()))?;
        self.doc = Some(doc);
        Ok(())
    }

    fn info(&self) -> Result<PdfInfo> {
        let doc = self
            .doc
            .as_ref()
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;

        // Extracting info from Info dict if needed, for now just basic mapping
        let mut title = None;
        let mut author = None;

        if let Some(info_id) = doc
            .trailer
            .get(b"Info")
            .ok()
            .and_then(|i| i.as_reference().ok())
        {
            if let Ok(info_dict) = doc.get_dictionary(info_id) {
                if let Ok(t) = info_dict.get(b"Title").and_then(|obj| obj.as_str()) {
                    title = Some(String::from_utf8_lossy(t).into_owned());
                }
                if let Ok(a) = info_dict.get(b"Author").and_then(|obj| obj.as_str()) {
                    author = Some(String::from_utf8_lossy(a).into_owned());
                }
            }
        }

        Ok(PdfInfo {
            page_count: doc.get_pages().len(),
            title,
            author,
        })
    }

    fn render(
        &self,
        _page_index: usize,
        _options: &RenderOptions,
        _output_path: &Path,
    ) -> Result<()> {
        // lopdf doesn't render to images natively.
        Err(LpdfError::Engine(
            "Render not supported by FallbackAdapter".into(),
        ))
    }

    fn page_ops_delete(&mut self, pages: &[usize]) -> Result<()> {
        let doc = self
            .doc
            .as_mut()
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;

        // pages is 0-indexed, lopdf uses 1-indexed page numbers.
        let page_numbers: Vec<u32> = pages.iter().map(|&p| (p + 1) as u32).collect();
        doc.delete_pages(&page_numbers);
        Ok(())
    }

    fn save(&self, output_path: &Path) -> Result<()> {
        let mut doc = self
            .doc
            .clone()
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;
        doc.save(output_path)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;
        Ok(())
    }

    fn merge(&self, _input_paths: &[&Path], _output_path: &Path) -> Result<()> {
        Err(LpdfError::Engine(
            "Merge not supported by FallbackAdapter".into(),
        ))
    }

    fn extract_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _output_path: &Path,
    ) -> Result<()> {
        Err(LpdfError::Engine(
            "Extract not supported by FallbackAdapter".into(),
        ))
    }

    fn delete_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _output_path: &Path,
    ) -> Result<()> {
        Err(LpdfError::Engine(
            "Stateless delete not supported by FallbackAdapter".into(),
        ))
    }

    fn rotate_pages(
        &self,
        _input_path: &Path,
        _pages: &[usize],
        _degrees: i32,
        _output_path: &Path,
    ) -> Result<()> {
        Err(LpdfError::Engine(
            "Rotate not supported by FallbackAdapter".into(),
        ))
    }

    fn rotate_document(
        &self,
        _input_path: &Path,
        _degrees: i32,
        _output_path: &Path,
    ) -> Result<()> {
        Err(LpdfError::Engine(
            "Rotate not supported by FallbackAdapter".into(),
        ))
    }
}

lpdf_engine_port::define_contract_tests!(FallbackAdapter);
