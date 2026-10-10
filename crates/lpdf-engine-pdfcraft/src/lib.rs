#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{PdfEngine, PdfInfo, RenderOptions};
use std::path::Path;
use std::sync::Arc;

pub struct PdfcraftAdapter {
    session: pdfcraft_engine::Session,
    current_doc: Option<pdfcraft_engine::DocId>,
}

impl Default for PdfcraftAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfcraftAdapter {
    pub fn new() -> Self {
        Self {
            session: pdfcraft_engine::Session::new(),
            current_doc: None,
        }
    }
}

impl PdfEngine for PdfcraftAdapter {
    fn load(&mut self, path: &Path) -> Result<()> {
        let bytes = std::fs::read(path).map_err(LpdfError::Io)?;
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let path_str = path.to_string_lossy().to_string();

        let doc_id = self
            .session
            .open(name, Some(path_str), Arc::new(bytes), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        self.current_doc = Some(doc_id);
        Ok(())
    }

    fn info(&self) -> Result<PdfInfo> {
        let doc_id = self
            .current_doc
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;
        let doc = self
            .session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        Ok(PdfInfo {
            page_count: doc.info.pages.len(),
            title: doc.info_value("Title"),
            author: doc.info_value("Author"),
        })
    }

    fn render(
        &self,
        page_index: usize,
        _options: &RenderOptions,
        output_path: &Path,
    ) -> Result<()> {
        let doc_id = self
            .current_doc
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;
        let doc = self
            .session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        // Extract first image as a basic stub for 'render'
        let mut images = doc.page_images(page_index);
        if let Some(_img) = images.pop() {
            if let Ok((_ext, bytes)) = doc.page_image_file(page_index, 0) {
                std::fs::write(output_path, bytes).map_err(LpdfError::Io)?;
            }
        }
        Ok(())
    }

    fn page_ops_delete(&mut self, pages: &[usize]) -> Result<()> {
        let doc_id = self
            .current_doc
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;

        let edit = pdfcraft_engine::Edit::DeletePages {
            pages: pages.to_vec(),
        };

        self.session
            .apply(doc_id, edit)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;
        Ok(())
    }

    fn save(&self, output_path: &Path) -> Result<()> {
        let doc_id = self
            .current_doc
            .ok_or_else(|| LpdfError::Engine("No document loaded".into()))?;
        let doc = self
            .session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        std::fs::write(output_path, &*doc.bytes).map_err(LpdfError::Io)?;
        Ok(())
    }
}

lpdf_engine_port::define_contract_tests!(PdfcraftAdapter);
