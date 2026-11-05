#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    ExtractiveSummaryReport, LlmSummaryOptions, LlmSummaryReport, MarkdownBatchOptions,
    MarkdownBatchReport, MarkdownDocFormat, OfficeInputFormat, OfficeOutputFormat, PdfAConformance,
    PdfAPreflightReport, PdfEngine, PdfInfo, RedactionRegion, RenderOptions, SummaryLength,
    TranslateOptions, TranslatePdfReport,
};
use std::path::Path;
use std::sync::Arc;

pub mod office_sidecar;
pub mod pandoc_sidecar;
pub mod summarizer;
pub mod translator;

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

    fn merge(&self, input_paths: &[&Path], output_path: &Path) -> Result<()> {
        let mut sources = Vec::new();
        for path in input_paths {
            let bytes = std::fs::read(path).map_err(LpdfError::Io)?;
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            sources.push((name, Arc::new(bytes)));
        }

        let combined_bytes = self
            .session
            .combine(&sources)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*combined_bytes).map_err(LpdfError::Io)?;

        // Verify output bytes
        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != combined_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        // Rename
        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn extract_pages(&self, input_path: &Path, pages: &[usize], output_path: &Path) -> Result<()> {
        let mut session = pdfcraft_engine::Session::new();
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let name = input_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let doc_id = session
            .open(name, None, Arc::new(bytes), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let zero_indexed: Vec<usize> = pages.iter().map(|p| p.saturating_sub(1)).collect();
        let extracted_bytes = session
            .extract(doc_id, &zero_indexed)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*extracted_bytes).map_err(LpdfError::Io)?;

        // Verify output bytes
        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != extracted_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        // Rename
        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn delete_pages(&self, input_path: &Path, pages: &[usize], output_path: &Path) -> Result<()> {
        let mut session = pdfcraft_engine::Session::new();
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let name = input_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let doc_id = session
            .open(name, None, Arc::new(bytes), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let zero_indexed: Vec<usize> = pages.iter().map(|p| p.saturating_sub(1)).collect();

        session
            .apply(
                doc_id,
                pdfcraft_engine::Edit::DeletePages {
                    pages: zero_indexed,
                },
            )
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let doc = session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*doc.bytes).map_err(LpdfError::Io)?;

        // Verify output bytes
        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != doc.bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        // Rename
        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn rotate_pages(
        &self,
        input_path: &Path,
        pages: &[usize],
        degrees: i32,
        output_path: &Path,
    ) -> Result<()> {
        let mut session = pdfcraft_engine::Session::new();
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let name = input_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let doc_id = session
            .open(name, None, Arc::new(bytes), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let zero_indexed: Vec<usize> = pages.iter().map(|p| p.saturating_sub(1)).collect();

        session
            .apply(
                doc_id,
                pdfcraft_engine::Edit::RotatePages {
                    pages: zero_indexed,
                    degrees: degrees as i64,
                },
            )
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let doc = session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*doc.bytes).map_err(LpdfError::Io)?;

        // Verify output bytes
        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != doc.bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        // Rename
        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn rotate_document(&self, input_path: &Path, degrees: i32, output_path: &Path) -> Result<()> {
        let mut session = pdfcraft_engine::Session::new();
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let name = input_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let doc_id = session
            .open(name, None, Arc::new(bytes), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let doc = session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        let all_pages: Vec<usize> = (0..doc.info.pages.len()).collect();

        session
            .apply(
                doc_id,
                pdfcraft_engine::Edit::RotatePages {
                    pages: all_pages,
                    degrees: degrees as i64,
                },
            )
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let doc = session
            .get(doc_id)
            .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*doc.bytes).map_err(LpdfError::Io)?;

        // Verify output bytes
        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != doc.bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        // Rename
        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn compress(
        &self,
        input_path: &Path,
        profile: lpdf_engine_port::OptimizeProfile,
        output_path: &Path,
    ) -> Result<()> {
        use pdfcraft_engine::optimize::{Compression, ImageSettings, Settings};

        let mut session = pdfcraft_engine::Session::new();
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let name = input_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let doc_id = session
            .open(name, None, Arc::new(bytes.clone()), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let mut settings = Settings::default();

        match profile {
            lpdf_engine_port::OptimizeProfile::Low => {
                settings.color = ImageSettings {
                    downsample: true,
                    target_ppi: 200.0,
                    above_ppi: 300.0,
                    compression: Compression::Jpeg(80),
                };
                settings.gray = ImageSettings {
                    downsample: true,
                    target_ppi: 200.0,
                    above_ppi: 300.0,
                    compression: Compression::Jpeg(80),
                };
            }
            lpdf_engine_port::OptimizeProfile::Medium => {
                // defaults are exactly acrobat's medium settings
            }
            lpdf_engine_port::OptimizeProfile::High => {
                settings.color = ImageSettings {
                    downsample: true,
                    target_ppi: 72.0,
                    above_ppi: 100.0,
                    compression: Compression::Jpeg(40),
                };
                settings.gray = ImageSettings {
                    downsample: true,
                    target_ppi: 72.0,
                    above_ppi: 100.0,
                    compression: Compression::Jpeg(40),
                };
            }
        }

        let (optimized_bytes, _) = session
            .optimized_bytes(doc_id, &settings, &[])
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        let final_bytes = if optimized_bytes.len() < bytes.len() {
            &optimized_bytes
        } else {
            &bytes
        };

        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &**final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn repair(&self, input_path: &Path, output_path: &Path) -> Result<Vec<String>> {
        let mut session = pdfcraft_engine::Session::new();
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let name = input_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let doc_id = session
            .open(name, None, Arc::new(bytes), None)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        // Extract the log of repairs made during parsing
        let log = session
            .get(doc_id)
            .map(|doc| doc.repair_log())
            .unwrap_or_default();

        let repaired_bytes = session
            .save_bytes(doc_id)
            .map_err(|e| LpdfError::Engine(e.to_string()))?;

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*repaired_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != repaired_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(log)
    }

    fn ocr(&self, input_path: &Path, lang: &str, output_path: &Path) -> Result<()> {
        self.ocr_with_options(input_path, lang, output_path, false)
    }

    fn ocr_with_options(
        &self,
        input_path: &Path,
        lang: &str,
        output_path: &Path,
        native_text_only: bool,
    ) -> Result<()> {
        let input_path = input_path.to_owned();
        let lang = lang.to_owned();

        // Wrap in catch_unwind to satisfy F23 (ML Inference Failure)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Document not found".into()))?;

            let all_pages: Vec<usize> = (0..doc.info.pages.len()).collect();

            let settings = pdfcraft_engine::ocr::OcrSettings {
                language: lang,
                dpi: 300.0,
                skip_text_pages: true,
                native_text_only,
            };

            // This is where ML inference runs and might panic
            session
                .recognize_text(doc_id, &all_pages, settings)
                .map_err(|e| LpdfError::OcrInferenceError(e.to_string()))?;

            let repaired_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(repaired_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown ML Panic".to_string()
                };
                return Err(LpdfError::OcrInferenceError(format!(
                    "Panic during OCR inference: {}",
                    msg
                )));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn protect(
        &self,
        input_path: &Path,
        password: &str,
        permissions: lpdf_engine_port::Permissions,
        output_path: &Path,
    ) -> Result<()> {
        let input_path = input_path.to_owned();
        let password = password.to_owned();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let protection = pdfcraft_engine::Protection {
                open_password: Some(password.clone()),
                permissions_password: None,
                printing: if permissions.allow_print {
                    pdfcraft_engine::Printing::High
                } else {
                    pdfcraft_engine::Printing::None
                },
                changes: if permissions.allow_edit {
                    pdfcraft_engine::Changes::AnyExceptExtract
                } else {
                    pdfcraft_engine::Changes::None
                },
                copy: permissions.allow_copy,
                ..Default::default()
            };

            session
                .apply(doc_id, pdfcraft_engine::Edit::Protect(protection))
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let encrypted_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(encrypted_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Crypto Panic".to_string()
                };
                return Err(LpdfError::Engine(format!(
                    "Panic during cryptographic operation: {}",
                    msg
                )));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn unlock(&self, input_path: &Path, password: &str, output_path: &Path) -> Result<()> {
        let input_path = input_path.to_owned();
        let password = password.to_owned();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), Some(&password))
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            session
                .apply(doc_id, pdfcraft_engine::Edit::RemoveProtection)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let decrypted_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(decrypted_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(_e) => {
                // Do not leak cryptographic stack traces.
                return Err(LpdfError::Engine(
                    "Panic during cryptographic operation (stack trace withheld for security)"
                        .to_string(),
                ));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn add_annotation(
        &self,
        input_path: &Path,
        annotation: &lpdf_engine_port::AnnotationDef,
        output_path: &Path,
    ) -> Result<()> {
        let input_path = input_path.to_owned();
        let annotation = annotation.clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let shape = match &annotation.shape {
                lpdf_engine_port::AnnotationShape::StickyNote { x, y, .. } => {
                    pdfcraft_engine::Shape::Note {
                        at: [*x, *y],
                        icon: pdfcraft_engine::NoteIcon::Comment,
                    }
                }
                lpdf_engine_port::AnnotationShape::Highlight { quads } => {
                    pdfcraft_engine::Shape::TextMarkup {
                        kind: pdfcraft_engine::Markup::Highlight,
                        quads: quads.clone(),
                    }
                }
                lpdf_engine_port::AnnotationShape::Underline { quads } => {
                    pdfcraft_engine::Shape::TextMarkup {
                        kind: pdfcraft_engine::Markup::Underline,
                        quads: quads.clone(),
                    }
                }
            };

            let contents = match &annotation.shape {
                lpdf_engine_port::AnnotationShape::StickyNote { text, .. } => text.clone(),
                _ => String::new(),
            };

            let style = pdfcraft_engine::Style::default_for(&shape);

            let new_annot = pdfcraft_engine::NewAnnotation {
                page: annotation.page,
                shape,
                style,
                contents,
                author: annotation.author,
            };

            session
                .apply(doc_id, pdfcraft_engine::Edit::AddAnnotation(new_annot))
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let modified_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(modified_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Edit Panic".to_string()
                };
                return Err(LpdfError::Engine(format!(
                    "ObjectTreeError: Panic during add_annotation: {}",
                    msg
                )));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn add_watermark(
        &self,
        input_path: &Path,
        settings: &lpdf_engine_port::WatermarkSettings,
        output_path: &Path,
    ) -> Result<()> {
        let input_path = input_path.to_owned();
        let settings = settings.clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let pages = (0..session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("No document found".into()))?
                .info
                .pages
                .len())
                .collect();

            let watermark_settings = pdfcraft_engine::Watermark {
                text: settings.text,
                opacity: settings.opacity,
                rotation: settings.rotation,
                color: settings.color,
                ..Default::default()
            };

            session
                .apply(
                    doc_id,
                    pdfcraft_engine::Edit::AddWatermark {
                        pages,
                        settings: watermark_settings,
                        replace: false,
                        file: None,
                    },
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let modified_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(modified_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Edit Panic".to_string()
                };
                return Err(LpdfError::Engine(format!(
                    "ObjectTreeError: Panic during add_watermark: {}",
                    msg
                )));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn flatten_comments(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let input_path = input_path.to_owned();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            // F26: Flattening comments must not corrupt the tree, if it does, it panics
            // and we catch it here.
            session
                .apply(
                    doc_id,
                    pdfcraft_engine::Edit::Flatten {
                        comments: true,
                        fields: false,
                    },
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let modified_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(modified_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Edit Panic".to_string()
                };
                // Enforce F26: return an ObjectTreeError variant (here as a typed string in Engine).
                return Err(LpdfError::Engine(format!(
                    "ObjectTreeError: Corrupted object tree during flatten: {}",
                    msg
                )));
            }
        };

        // Atomic write - discard temp file if failure occurs before rename.
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn flatten_forms(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let input_path = input_path.to_owned();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            // F26: Wrap in catch_unwind
            session
                .apply(
                    doc_id,
                    pdfcraft_engine::Edit::Flatten {
                        comments: false,
                        fields: true,
                    },
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let modified_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(modified_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Edit Panic".to_string()
                };
                return Err(LpdfError::Engine(format!(
                    "ObjectTreeError: Corrupted object tree during flatten_forms: {}",
                    msg
                )));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn fill_form(
        &self,
        input_path: &Path,
        fields: &std::collections::HashMap<String, String>,
        output_path: &Path,
    ) -> Result<()> {
        let input_path = input_path.to_owned();
        let fields = fields.clone();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let mut session = pdfcraft_engine::Session::new();
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            for (k, v) in fields {
                session
                    .apply(
                        doc_id,
                        pdfcraft_engine::Edit::SetFieldValue {
                            name: k,
                            value: pdfcraft_forms::FieldValue::Text(v),
                        },
                    )
                    .map_err(|e| LpdfError::Engine(e.to_string()))?;
            }

            let modified_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(modified_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Edit Panic".to_string()
                };
                return Err(LpdfError::Engine(format!(
                    "ObjectTreeError: Corrupted object tree during fill_form: {}",
                    msg
                )));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn sign_document(
        &self,
        input_path: &Path,
        p12_path: &Path,
        password: &str,
        output_path: &Path,
    ) -> Result<()> {
        let input_path = input_path.to_owned();
        let p12_path = p12_path.to_owned();
        let password = password.to_string();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let bytes = std::fs::read(&input_path).map_err(LpdfError::Io)?;
            let p12_bytes = std::fs::read(&p12_path).map_err(LpdfError::Io)?;

            let doc = pdfcraft_cos::Document::open(Arc::new(bytes))
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let digital_id = pdfcraft_sign::pkcs12::open(&p12_bytes, &password)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let opts = pdfcraft_sign::SignOptions {
                field: None,
                page: 0,
                rect: None,
                new_field_name: None,
                reason: Some("LocalPDF Studio Signature".to_string()),
                location: None,
                contact: None,
                date: "D:20261010000000Z".to_string(), // placeholder date
                certify: None,
                appearance: pdfcraft_sign::Appearance::default(),
                appearance_labels: pdfcraft_sign::AppearanceLabels::default(),
                graphic: None,
            };

            let signed_bytes = pdfcraft_sign::sign(&doc, &digital_id, &opts)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok(signed_bytes)
        }));

        let final_bytes = match result {
            Ok(Ok(bytes)) => bytes,
            Ok(Err(e)) => return Err(e),
            Err(_e) => {
                // F24: Do not leak stack traces for cryptographic integrity
                return Err(LpdfError::Engine(
                    "Panic during cryptographic signing".to_string(),
                ));
            }
        };

        // Atomic write
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*final_bytes).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.len() != final_bytes.len() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn redact(
        &self,
        input_path: &Path,
        regions: Vec<RedactionRegion>,
        output_path: &Path,
    ) -> Result<()> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
            let mut session = pdfcraft_engine::Session::new();
            let name = input_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let doc_id = session
                .open(name, None, std::sync::Arc::new(bytes), None)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            for region in regions {
                let shape = pdfcraft_engine::Shape::Redact {
                    quads: vec![[
                        region.x,
                        region.y + region.height,
                        region.x + region.width,
                        region.y + region.height,
                        region.x,
                        region.y,
                        region.x + region.width,
                        region.y,
                    ]],
                    overlay: String::new(),
                    look: Default::default(),
                };

                let page_idx = region.page.saturating_sub(1);
                let mark = pdfcraft_engine::Edit::AddAnnotation(pdfcraft_engine::NewAnnotation {
                    page: page_idx,
                    style: pdfcraft_engine::Style::default_for(&shape),
                    shape,
                    contents: String::new(),
                    author: "Admin".into(),
                });
                session
                    .apply(doc_id, mark)
                    .map_err(|e| LpdfError::Engine(e.to_string()))?;
            }

            // F25: Secondary verification pass is performed internally by ApplyRedactions
            session
                .apply(
                    doc_id,
                    pdfcraft_engine::Edit::ApplyRedactions { pages: None },
                )
                .map_err(|e| LpdfError::Engine(format!("RedactionVerificationError: {}", e)))?;

            session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))
        }));

        let saved = match result {
            Ok(Ok(saved)) => saved,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(LpdfError::Engine(
                    "Panic during redaction operations".into(),
                ))
            }
        };

        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, &*saved).map_err(LpdfError::Io)?;

        let written = std::fs::read(&temp_path).map_err(LpdfError::Io)?;
        if written.is_empty() {
            let _ = std::fs::remove_file(&temp_path);
            return Err(LpdfError::Io(std::io::Error::other("Output size mismatch")));
        }

        std::fs::rename(&temp_path, output_path).map_err(LpdfError::Io)?;

        Ok(())
    }

    fn image_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.do_convert_to_pdf(input_path, output_path)
    }

    fn text_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.do_convert_to_pdf(input_path, output_path)
    }

    fn structural_docx_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.do_convert_to_pdf(input_path, output_path)
    }

    fn html_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        self.do_convert_to_pdf(input_path, output_path)
    }

    fn pdf_to_images(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let target_dir = if output_path.is_file() {
            output_path.parent().unwrap_or_else(|| Path::new("."))
        } else {
            std::fs::create_dir_all(output_path).map_err(LpdfError::Io)?;
            output_path
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let page_count = doc.info.pages.len();
            let mut exporter = pdfcraft_engine::export::Exporter::new(doc);

            let mut rendered_pages = Vec::with_capacity(page_count);
            for p in 0..page_count {
                let png_bytes = exporter
                    .png(p, 150.0)
                    .map_err(|e| LpdfError::Engine(e.to_string()))?;
                rendered_pages.push((p + 1, png_bytes));
            }
            Ok::<_, LpdfError>(rendered_pages)
        }));

        let rendered_pages = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(LpdfError::Engine(
                    "Panic during pdf_to_images export".into(),
                ))
            }
        };

        // Strict enforcement: safe directory creation using established file-safety layer
        let folder_prefix = format!("{}_images", stem);
        let out_dir = lpdf_fs_safety::create_safe_output_dir(target_dir, &folder_prefix)?;

        for (page_num, png_bytes) in rendered_pages {
            let page_filename = format!("{}_page_{:04}.png", stem, page_num);
            let page_path = out_dir.join(page_filename);
            lpdf_fs_safety::atomic_write(&page_path, &png_bytes)?;
        }

        Ok(())
    }

    fn extract_all_images(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let target_dir = if output_path.is_file() {
            output_path.parent().unwrap_or_else(|| Path::new("."))
        } else {
            std::fs::create_dir_all(output_path).map_err(LpdfError::Io)?;
            output_path
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let page_count = doc.info.pages.len();
            let pages: Vec<usize> = (0..page_count).collect();
            let src = doc.export_source();
            let extracted = pdfcraft_engine::export::extract_images(&src, &pages, 0)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            Ok::<_, LpdfError>((extracted, page_count))
        }));

        let (extracted, page_count) = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(LpdfError::Engine("Panic during extract_all_images".into())),
        };

        let folder_prefix = format!("{}_extracted_images", stem);
        let out_dir = lpdf_fs_safety::create_safe_output_dir(target_dir, &folder_prefix)?;

        for (idx, img) in extracted.images.into_iter().enumerate() {
            let fname = pdfcraft_engine::export::image_file_name(stem, &img, idx + 1, page_count);
            let img_path = out_dir.join(fname);
            lpdf_fs_safety::atomic_write(&img_path, &img.data)?;
        }

        Ok(())
    }

    fn pdf_to_text(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let page_count = doc.info.pages.len();
            let pages: Vec<usize> = (0..page_count).collect();
            let mut exporter = pdfcraft_engine::export::Exporter::new(doc);
            exporter
                .text_of(&pages)
                .map_err(|e| LpdfError::Engine(e.to_string()))
        }));

        let text = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(LpdfError::Engine("Panic during pdf_to_text".into())),
        };

        let target_file = if output_path.is_dir() {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
            output_path.join(format!("{}.txt", stem))
        } else {
            output_path.to_path_buf()
        };

        lpdf_fs_safety::atomic_write(&target_file, text.as_bytes())?;
        Ok(())
    }

    fn pdf_to_structural_docx(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let docx_bytes = doc.export_office(pdfcraft_engine::compare::OfficeFormat::Docx);
            Ok::<_, LpdfError>(docx_bytes)
        }));

        let docx_bytes = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(LpdfError::Engine("Panic during docx export".into())),
        };

        let target_file = if output_path.is_dir() {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
            output_path.join(format!("{}.docx", stem))
        } else {
            output_path.to_path_buf()
        };

        lpdf_fs_safety::atomic_write(&target_file, &docx_bytes)?;
        Ok(())
    }

    fn pdf_to_html(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let html_bytes = doc.export_office(pdfcraft_engine::compare::OfficeFormat::Html);
            Ok::<_, LpdfError>(html_bytes)
        }));

        let html_bytes = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(LpdfError::Engine("Panic during html export".into())),
        };

        // If target is a directory, place inside a newly created safe folder
        if output_path.is_dir() {
            let folder_prefix = format!("{}_html", stem);
            let out_dir = lpdf_fs_safety::create_safe_output_dir(output_path, &folder_prefix)?;
            let index_file = out_dir.join("index.html");
            lpdf_fs_safety::atomic_write(&index_file, &html_bytes)?;
        } else {
            lpdf_fs_safety::atomic_write(output_path, &html_bytes)?;
        }

        Ok(())
    }

    fn pdf_to_rtf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let rtf_bytes = doc.export_office(pdfcraft_engine::compare::OfficeFormat::Rtf);
            Ok::<_, LpdfError>(rtf_bytes)
        }));

        let rtf_bytes = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(LpdfError::Engine("Panic during rtf export".into())),
        };

        let target_file = if output_path.is_dir() {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
            output_path.join(format!("{}.rtf", stem))
        } else {
            output_path.to_path_buf()
        };

        lpdf_fs_safety::atomic_write(&target_file, &rtf_bytes)?;
        Ok(())
    }

    fn pdf_to_pdfa(
        &self,
        input_path: &Path,
        conformance: PdfAConformance,
        output_path: &Path,
    ) -> Result<PdfAPreflightReport> {
        let level = match conformance {
            PdfAConformance::PdfA1b => {
                return Err(LpdfError::Unsupported(
                    "PDF/A-1b is not supported by the native preflight engine; please use PDF/A-2b or PDF/A-3b.".into(),
                ));
            }
            PdfAConformance::PdfA2b => pdfcraft_engine::pdfa::Level::A2b,
            PdfAConformance::PdfA3b => pdfcraft_engine::pdfa::Level::A3b,
        };

        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let initial_issues = session
                .get(doc_id)
                .map(|d| d.pdfa_verify(level))
                .unwrap_or_default();

            session
                .apply(doc_id, pdfcraft_engine::Edit::ConvertPdfA { level })
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            let remaining_issues_raw = session
                .get(doc_id)
                .map(|d| d.pdfa_verify(level))
                .unwrap_or_default();

            let out_bytes = session
                .save_bytes(doc_id)
                .map_err(|e| LpdfError::Engine(e.to_string()))?;

            Ok::<_, LpdfError>((initial_issues, remaining_issues_raw, out_bytes))
        }));

        let (initial_issues, remaining_issues_raw, out_bytes) = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                return Err(LpdfError::Engine(format!(
                    "Panic during PDF/A conversion or metadata injection: {}",
                    msg
                )));
            }
        };

        let target_file = if output_path.is_dir() {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("document");
            output_path.join(format!("{}_pdfa.pdf", stem))
        } else {
            output_path.to_path_buf()
        };

        // F1: Atomic output
        lpdf_fs_safety::atomic_write(&target_file, &out_bytes)?;

        // F6: Verify-after-write
        let written_bytes = std::fs::read(&target_file).map_err(LpdfError::Io)?;
        if written_bytes.is_empty() || written_bytes.len() != out_bytes.len() {
            let _ = std::fs::remove_file(&target_file);
            return Err(LpdfError::Engine(
                "Post-write verification failed: empty or corrupted output".into(),
            ));
        }

        let mut fixed_issues = Vec::new();
        for init in &initial_issues {
            if !remaining_issues_raw
                .iter()
                .any(|r| r.clause == init.clause && r.message == init.message)
            {
                fixed_issues.push(format!("Fixed: {}", init.message));
            }
        }
        if fixed_issues.is_empty() {
            fixed_issues.push(format!(
                "Applied {} metadata and output intent normalization",
                match conformance {
                    PdfAConformance::PdfA1b => "PDF/A-1b",
                    PdfAConformance::PdfA2b => "PDF/A-2b",
                    PdfAConformance::PdfA3b => "PDF/A-3b",
                }
            ));
        }

        let remaining_issues: Vec<String> = remaining_issues_raw
            .iter()
            .map(|issue| {
                if let Some(pg) = issue.page {
                    format!(
                        "[Clause {}] Page {}: {}",
                        issue.clause,
                        pg + 1,
                        issue.message
                    )
                } else {
                    format!("[Clause {}] {}", issue.clause, issue.message)
                }
            })
            .collect();

        let is_compliant = remaining_issues.is_empty();

        Ok(PdfAPreflightReport {
            conformance,
            is_compliant,
            fixed_issues,
            remaining_issues,
        })
    }

    fn office_to_pdf(
        &self,
        input_path: &Path,
        format: OfficeInputFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        office_sidecar::convert_office_to_pdf(input_path, format, output_path, None)
    }

    fn pdf_to_office(
        &self,
        input_path: &Path,
        format: OfficeOutputFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        office_sidecar::convert_pdf_to_office(input_path, format, output_path, None)
    }

    fn markdown_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<std::path::PathBuf> {
        pandoc_sidecar::convert_markdown_to_pdf(
            input_path,
            output_path,
            None,
            |intermediate_html, target_pdf| self.do_convert_to_pdf(intermediate_html, target_pdf),
        )
    }

    fn pdf_to_markdown(&self, input_path: &Path, output_path: &Path) -> Result<std::path::PathBuf> {
        pandoc_sidecar::convert_pdf_to_markdown(
            input_path,
            output_path,
            None,
            |in_pdf, out_html| self.pdf_to_html(in_pdf, out_html),
            |in_pdf, assets_dir| self.extract_all_images(in_pdf, assets_dir),
        )
    }

    fn markdown_to_doc(
        &self,
        input_path: &Path,
        format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        pandoc_sidecar::convert_markdown_to_doc(input_path, format, output_path, None)
    }

    fn doc_to_markdown(
        &self,
        input_path: &Path,
        format: MarkdownDocFormat,
        output_path: &Path,
    ) -> Result<std::path::PathBuf> {
        pandoc_sidecar::convert_doc_to_markdown(input_path, format, output_path, None)
    }

    fn batch_markdown_convert(
        &self,
        input_dir: &Path,
        output_dir: &Path,
        options: MarkdownBatchOptions,
    ) -> Result<MarkdownBatchReport> {
        pandoc_sidecar::batch_convert_markdown(
            input_dir,
            output_dir,
            options,
            None,
            |in_p, out_p| self.pdf_to_html(in_p, out_p),
            |in_p, out_p| self.extract_all_images(in_p, out_p),
            |in_p, out_p| self.do_convert_to_pdf(in_p, out_p),
        )
    }

    fn summarize_extractive(
        &self,
        input_path: &Path,
        output_path: &Path,
        length: SummaryLength,
    ) -> Result<ExtractiveSummaryReport> {
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("document.pdf");

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_id = session
                .open(
                    name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let page_count = doc.info.pages.len();
            let pages: Vec<usize> = (0..page_count).collect();
            let mut exporter = pdfcraft_engine::export::Exporter::new(doc);
            exporter
                .text_of(&pages)
                .map_err(|e| LpdfError::Engine(e.to_string()))
        }));

        let text = match result {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(LpdfError::Engine(
                    "Panic during text extraction for summarization".into(),
                ))
            }
        };

        let summary_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            summarizer::generate_extractive_summary(input_path, output_path, &text, length)
        }));

        match summary_res {
            Ok(res) => res,
            Err(_) => Err(LpdfError::Engine(
                "Panic during extractive summarization".into(),
            )),
        }
    }

    fn summarize_llm_map_reduce(
        &self,
        input_path: &Path,
        output_path: &Path,
        options: LlmSummaryOptions,
    ) -> Result<LlmSummaryReport> {
        let text_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_name = input_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("doc.pdf");
            let data = std::fs::read(input_path).map_err(LpdfError::Io)?;
            let arc_bytes = Arc::new(data);
            let doc_id = session
                .open(
                    doc_name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let page_count = doc.info.pages.len();
            let pages: Vec<usize> = (0..page_count).collect();
            let mut exporter = pdfcraft_engine::export::Exporter::new(doc);
            exporter
                .text_of(&pages)
                .map_err(|e| LpdfError::Engine(e.to_string()))
        }));

        let text = match text_res {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(LpdfError::Engine(
                    "Panic during text extraction for LLM summarization".into(),
                ))
            }
        };

        let summary_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            summarizer::generate_llm_map_reduce_summary(input_path, output_path, &text, &options)
        }));

        match summary_res {
            Ok(res) => res,
            Err(_) => Err(LpdfError::Engine(
                "Panic during Map-Reduce LLM summarization".into(),
            )),
        }
    }

    fn translate_pdf_experimental(
        &self,
        input_path: &Path,
        output_pdf_path: &Path,
        options: TranslateOptions,
    ) -> Result<TranslatePdfReport> {
        let text_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut session = pdfcraft_engine::Session::new();
            let doc_name = input_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("doc.pdf");
            let data = std::fs::read(input_path).map_err(LpdfError::Io)?;
            let arc_bytes = Arc::new(data);
            let doc_id = session
                .open(
                    doc_name.to_string(),
                    Some(input_path.to_string_lossy().to_string()),
                    arc_bytes,
                    None,
                )
                .map_err(|e| LpdfError::Engine(e.to_string()))?;
            let doc = session
                .get(doc_id)
                .ok_or_else(|| LpdfError::Engine("Failed to get document".into()))?;
            let page_count = doc.info.pages.len();
            let mut exporter = pdfcraft_engine::export::Exporter::new(doc);
            let mut pages_text = Vec::with_capacity(page_count);
            for p in 0..page_count {
                let page_text = exporter
                    .text(p)
                    .map_err(|e| LpdfError::Engine(e.to_string()))?;
                pages_text.push(page_text);
            }
            Ok((pages_text, page_count))
        }));

        let (pages_text, page_count) = match text_res {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                return Err(LpdfError::Engine(
                    "Panic during text extraction for PDF translation".into(),
                ))
            }
        };

        let translate_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            translator::execute_experimental_translation(
                input_path,
                output_pdf_path,
                &pages_text,
                page_count,
                &options,
            )
        }));

        match translate_res {
            Ok(res) => res,
            Err(_) => Err(LpdfError::Engine(
                "Panic during experimental PDF translation".into(),
            )),
        }
    }
}

impl PdfcraftAdapter {
    fn do_convert_to_pdf(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        let name = input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let bytes = std::fs::read(input_path).map_err(LpdfError::Io)?;
        let arc_bytes = Arc::new(bytes);

        // Atomic convert to PDF
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.session
                .convert_to_pdf(name, &arc_bytes)
                .map_err(|e| LpdfError::Engine(e.to_string()))
        }));

        let out_bytes = match result {
            Ok(Ok((_kind, out))) => out,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(LpdfError::Engine("Panic during conversion".into())),
        };

        lpdf_fs_safety::atomic_write(output_path, &out_bytes)?;
        Ok(())
    }
}

lpdf_engine_port::define_contract_tests!(PdfcraftAdapter);
