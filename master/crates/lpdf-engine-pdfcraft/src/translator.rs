#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{TranslateOptions, TranslatePdfReport};
use std::path::{Path, PathBuf};

/// Splits raw text into paragraphs / layout blocks.
pub fn split_into_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    for part in text.split("\n\n") {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            blocks.push(trimmed.to_string());
        }
    }

    if blocks.is_empty() {
        for line in text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                blocks.push(trimmed.to_string());
            }
        }
    }

    blocks
}

/// Executes experimental block-level translation.
pub fn execute_experimental_translation(
    input_path: &Path,
    output_pdf_path: &Path,
    pages_text: &[String],
    page_count: usize,
    options: &TranslateOptions,
) -> Result<TranslatePdfReport> {
    // Failsafe F19 Rule 1: Refuse when ai_local_enabled is false
    if !options.ai_local_enabled {
        return Err(LpdfError::InvalidInput(
            "Local AI execution refused: ai_local_enabled is false.".to_string(),
        ));
    }

    // Failsafe F19 Rule 2: Refuse when model license is unacknowledged
    if !options.manifest.license_acknowledged {
        return Err(LpdfError::InvalidInput(format!(
            "Local AI execution refused: License '{}' for model has not been acknowledged.",
            options.manifest.license_spdx
        )));
    }

    // Failsafe F19 Rule 3 & 4: Pre-flight model header and SHA-256 validation via worker session
    let mut session = lpdf_ai_worker::AiWorkerSession::new(true, Some(options.manifest.clone()));
    let validation_resp = session.handle_request(&lpdf_ai_worker::AiWorkerRequest::ValidateModel {
        path: options.manifest.model_path.clone(),
        expected_sha256: options.manifest.sha256_hex.clone(),
    });

    match validation_resp {
        lpdf_ai_worker::AiWorkerResponse::ModelValidated { .. } => {}
        lpdf_ai_worker::AiWorkerResponse::Error { code, message } => {
            return Err(LpdfError::InvalidInput(format!(
                "Local AI model pre-flight check failed [{}]: {}",
                code, message
            )));
        }
        _ => {
            return Err(LpdfError::Engine(
                "Unexpected response during model pre-flight validation.".to_string(),
            ));
        }
    }

    // Check if there's any extractable text
    let total_chars: usize = pages_text.iter().map(|s| s.trim().len()).sum();
    if total_chars == 0 {
        return Err(LpdfError::InvalidInput(
            "Source document contains no extractable text for translation.".to_string(),
        ));
    }

    let mut all_translated_page_texts = Vec::new();
    let mut block_mappings = Vec::new();
    let mut total_blocks_translated = 0;

    for (page_idx, page_raw) in pages_text.iter().enumerate() {
        let blocks = split_into_blocks(page_raw);
        let mut translated_blocks_for_page = Vec::new();

        for (block_idx, block) in blocks.iter().enumerate() {
            total_blocks_translated += 1;
            let prompt = format!(
                "Translate the following text from {} to {}:\n\n{}",
                options.source_lang, options.target_lang, block
            );

            let resp = session.handle_request(&lpdf_ai_worker::AiWorkerRequest::InferChunk {
                prompt,
                max_tokens: 512,
                temperature: 0.1,
            });

            let translated_block = match resp {
                lpdf_ai_worker::AiWorkerResponse::ChunkInferred { text, .. } => text,
                lpdf_ai_worker::AiWorkerResponse::Error { code, message } => {
                    return Err(LpdfError::Engine(format!(
                        "Inference error translating page {} block {} [{}]: {}",
                        page_idx + 1,
                        block_idx + 1,
                        code,
                        message
                    )));
                }
                _ => {
                    return Err(LpdfError::Engine(
                        "Unexpected response during block translation.".to_string(),
                    ));
                }
            };

            block_mappings.push((
                page_idx + 1,
                block_idx + 1,
                block.clone(),
                translated_block.clone(),
            ));
            translated_blocks_for_page.push(translated_block);
        }

        all_translated_page_texts.push(translated_blocks_for_page.join("\n\n"));
    }

    let combined_translated_text = all_translated_page_texts.join("\n\n");

    // Re-typeset the translated blocks into a valid output PDF document (%PDF-) via pdfcraft Session
    let doc_title = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let craft_session = pdfcraft_engine::Session::new();
    let pdf_bytes = craft_session
        .create_from_text(doc_title, &combined_translated_text)
        .map_err(|e| LpdfError::Engine(format!("Failed to re-typeset translated PDF: {}", e)))?;

    // Determine target PDF path
    let target_pdf_file = if output_pdf_path.is_dir() {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        output_pdf_path.join(format!("{}_translated_{}.pdf", stem, options.target_lang))
    } else {
        output_pdf_path.to_path_buf()
    };

    let report_file = PathBuf::from(format!("{}.report.md", target_pdf_file.display()));

    // Generate companion report markdown
    let file_name = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.pdf");
    let model_name = options
        .manifest
        .model_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("model.gguf");
    let sha_snippet = if options.manifest.sha256_hex.len() >= 16 {
        &options.manifest.sha256_hex[0..16]
    } else {
        &options.manifest.sha256_hex
    };

    let experimental_notice = "Notice: PDF block-level translation is experimental (ADR-006). Layout structure, fonts, and word reflow may deviate from the original geometry. Consult the companion report for untranslated fragments and mapping verification.".to_string();

    let mut report_md = String::new();
    report_md.push_str("# Translation Companion Report (Experimental Block-Level Re-Typeset)\n\n");
    report_md.push_str(&format!("- **Source Document:** `{}`\n", file_name));
    report_md.push_str(&format!(
        "- **Source Language:** `{}`\n",
        options.source_lang
    ));
    report_md.push_str(&format!(
        "- **Target Language:** `{}`\n",
        options.target_lang
    ));
    report_md.push_str(&format!("- **Pages Processed:** {}\n", page_count));
    report_md.push_str(&format!(
        "- **Blocks Translated:** {}\n",
        total_blocks_translated
    ));
    report_md.push_str(&format!(
        "- **Model Provenance:** `{}` (Format: `{}`, SHA-256: `{}...`)\n",
        model_name, options.manifest.format, sha_snippet
    ));
    report_md.push_str(&format!(
        "- **Licence (SPDX):** `{}` (Acknowledged for offline execution)\n",
        options.manifest.license_spdx
    ));
    report_md.push_str(&format!(
        "- **Preserve Layout Blocks:** {}\n\n",
        options.preserve_layout_blocks
    ));

    report_md.push_str("## Experimental Notice\n\n");
    report_md.push_str(&format!("{}\n\n", experimental_notice));

    report_md.push_str("## Block Translation Mappings\n\n");
    for (page_num, block_num, orig, trans) in &block_mappings {
        report_md.push_str(&format!(
            "### Page {}, Block {}\n\n- **Original ({}):**\n> {}\n\n- **Translated ({}):**\n> {}\n\n",
            page_num, block_num, options.source_lang, orig, options.target_lang, trans
        ));
    }

    // Atomic write of both target PDF and companion report
    lpdf_fs_safety::atomic_write(&target_pdf_file, &pdf_bytes)?;
    lpdf_fs_safety::atomic_write(&report_file, report_md.as_bytes())?;

    Ok(TranslatePdfReport {
        output_pdf_path: target_pdf_file,
        report_path: report_file,
        blocks_translated: total_blocks_translated,
        pages_processed: page_count,
        source_lang: options.source_lang.clone(),
        target_lang: options.target_lang.clone(),
        model_sha256: options.manifest.sha256_hex.clone(),
        experimental_notice,
    })
}
