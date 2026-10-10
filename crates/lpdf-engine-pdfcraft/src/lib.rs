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
                permissions_password: Some(password.clone()),
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
}

lpdf_engine_port::define_contract_tests!(PdfcraftAdapter);
