use lpdf_core_types::compute_sha256_hex;
use lpdf_engine_pdfcraft::summarizer::*;
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{
    AiModelManifest, LlmSummaryOptions, PdfEngine, SummaryLength, TranslateOptions,
};
use std::fs;
use std::path::PathBuf;

fn get_corpus_pdf(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus")
        .join(name)
}

#[test]
fn test_sentence_splitting_and_tokenization() {
    let sample = "LocalPDF Studio is a desktop document utility. It runs completely offline without network connections! Does it support fast text processing? Yes, indeed. Short.";
    let sentences = split_into_sentences(sample);

    assert!(sentences.len() >= 3);
    assert!(sentences[0].contains("LocalPDF Studio is a desktop document utility"));
    assert!(sentences[1].contains("It runs completely offline"));

    let words = tokenize_words("Hello, World! 123 LocalPDF_Studio.");
    assert!(words.contains(&"hello".to_string()));
    assert!(words.contains(&"world".to_string()));
    assert!(words.contains(&"localpdf".to_string()));
}

#[test]
fn test_keywords_and_scoring() {
    let text = "Artificial intelligence and machine learning transform document processing. Machine learning models require significant compute and memory. Local machine learning keeps documents secure on the local device without external cloud access.";
    let words = tokenize_words(text);
    let (freqs, keywords) = extract_keywords(&words, 5);

    assert!(
        keywords.contains(&"learning".to_string()) || keywords.contains(&"machine".to_string())
    );
    assert!(!keywords.contains(&"and".to_string())); // Stopword filtered

    let sentences = split_into_sentences(text);
    let scored = score_sentences(&sentences, &freqs);
    assert_eq!(scored.len(), sentences.len());
    // First sentence has position bonus
    assert!(scored[0].score > 0.0);
}

#[test]
fn test_summary_length_scaling() {
    let text = "The quick brown fox jumps over the lazy dog. Document automation speeds up enterprise legal review. Security and privacy remain paramount for client records. Offline software guarantees zero data exfiltration. Every processing step is verified atomically on disk. Engineers rely on automated tests to maintain quality. Modern user interfaces should feel snappy and responsive. Clean typography enhances readability for long reports. LocalPDF delivers zero-compromise privacy without limits.";

    let (_, _, _, _, words_short) = summarize_text("test.pdf", text, SummaryLength::Short).unwrap();
    let (_, _, _, _, words_med) = summarize_text("test.pdf", text, SummaryLength::Medium).unwrap();
    let (_, _, _, _, words_det) =
        summarize_text("test.pdf", text, SummaryLength::Detailed).unwrap();

    assert!(words_short <= words_med);
    assert!(words_med <= words_det);
}

#[test]
fn test_e2e_pdf_extractive_summarization() {
    let pdf_path = get_corpus_pdf("standard-text.pdf");
    assert!(
        pdf_path.exists(),
        "Corpus file standard-text.pdf must exist"
    );

    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_summary_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);
    let out_md = temp_dir.join("standard_text_summary.md");

    let engine = PdfcraftAdapter::new();
    let report = engine
        .summarize_extractive(&pdf_path, &out_md, SummaryLength::Short)
        .expect("summarize_extractive should succeed on standard-text.pdf");

    assert!(out_md.exists());
    assert_eq!(report.output_path, out_md);
    assert!(report.source_word_count > 0);
    assert!(report.summary_word_count > 0);
    assert!(report.summary_word_count <= report.source_word_count);
    assert!(!report.key_points.is_empty());

    let content = fs::read_to_string(&out_md).unwrap();
    assert!(content.contains("# Executive Document Summary"));
    assert!(content.contains("## Key Takeaways"));
    assert!(content.contains("## Executive Summary"));

    let _ = fs::remove_dir_all(&temp_dir);
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

#[test]
fn test_semantic_chunking_partitions_text_properly() {
    let text = "First sentence of section one. Second sentence provides background context. Third sentence concludes section one. Fourth sentence starts the second topic. Fifth sentence elaborates on implementation details. Sixth sentence summarizes the whole document.";
    let chunks = partition_into_chunks(text, 15);
    assert!(chunks.len() >= 2);
    for chunk in &chunks {
        assert!(!chunk.trim().is_empty());
    }
}

#[test]
fn test_llm_map_reduce_f19_guards() {
    let pdf_path = get_corpus_pdf("standard-text.pdf");
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_llm_f19_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);
    let out_md = temp_dir.join("f19_summary.md");

    let model_file = temp_dir.join("test_model.gguf");
    let model_bytes = b"GGUF\x03\x00\x00\x00_valid_model_header_payload";
    let _ = fs::write(&model_file, model_bytes);
    let sha256_hex = compute_sha256_hex(&model_file).expect("compute sha256");

    let engine = PdfcraftAdapter::new();

    // 1. Guard against ai_local_enabled = false
    let mut opts_disabled = LlmSummaryOptions {
        manifest: AiModelManifest {
            model_path: model_file.clone(),
            sha256_hex: sha256_hex.clone(),
            format: "GGUF".into(),
            license_spdx: "Apache-2.0".into(),
            license_acknowledged: true,
        },
        ai_local_enabled: false,
        length: SummaryLength::Medium,
        chunk_word_limit: 100,
        max_tokens: 256,
        temperature: 0.2,
    };
    let res1 = engine.summarize_llm_map_reduce(&pdf_path, &out_md, opts_disabled.clone());
    assert!(res1.is_err(), "Must reject when ai_local_enabled is false");

    // 2. Guard against license_acknowledged = false
    opts_disabled.ai_local_enabled = true;
    opts_disabled.manifest.license_acknowledged = false;
    let res2 = engine.summarize_llm_map_reduce(&pdf_path, &out_md, opts_disabled.clone());
    assert!(res2.is_err(), "Must reject when license is unacknowledged");

    // 3. Guard against SHA-256 mismatch
    opts_disabled.manifest.license_acknowledged = true;
    opts_disabled.manifest.sha256_hex =
        "0000000000000000000000000000000000000000000000000000000000000000".into();
    let res3 = engine.summarize_llm_map_reduce(&pdf_path, &out_md, opts_disabled);
    assert!(res3.is_err(), "Must reject when SHA-256 mismatches");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_e2e_pdf_llm_map_reduce_summarization() {
    let pdf_path = get_corpus_pdf("standard-text.pdf");
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_llm_e2e_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);
    let out_md = temp_dir.join("llm_e2e_summary.md");

    let model_file = temp_dir.join("valid_mock.gguf");
    let model_bytes = b"GGUF\x03\x00\x00\x00_test_model_weights_and_tensors_dummy_data";
    let _ = fs::write(&model_file, model_bytes);
    let sha256_hex = compute_sha256_hex(&model_file).expect("compute sha256");

    let engine = PdfcraftAdapter::new();
    let opts = LlmSummaryOptions {
        manifest: AiModelManifest {
            model_path: model_file,
            sha256_hex,
            format: "GGUF".into(),
            license_spdx: "Apache-2.0".into(),
            license_acknowledged: true,
        },
        ai_local_enabled: true,
        length: SummaryLength::Medium,
        chunk_word_limit: 50,
        max_tokens: 128,
        temperature: 0.1,
    };

    let report = engine
        .summarize_llm_map_reduce(&pdf_path, &out_md, opts)
        .expect("LLM map-reduce summary should succeed");

    assert!(out_md.exists());
    assert_eq!(report.output_path, out_md);
    assert!(report.chunks_processed >= 1);
    assert!(!report.chunk_summaries.is_empty());
    assert!(report.source_word_count > 0);
    assert!(report.summary_word_count > 0);

    let content = fs::read_to_string(&out_md).expect("read output md");
    assert!(content.contains("# Executive Document Summary (Local LLM Map-Reduce)"));
    assert!(content.contains("## Executive Synthesis"));
    assert!(content.contains("## Section-by-Section Intermediate Summaries"));
    assert!(content.contains("### Section 1"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_translate_pdf_f19_guards() {
    let pdf_path = get_corpus_pdf("standard-text.pdf");
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_trans_f19_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);
    let out_pdf = temp_dir.join("f19_translated.pdf");

    let model_file = temp_dir.join("trans_model.gguf");
    let model_bytes = b"GGUF\x03\x00\x00\x00_valid_model_header_payload";
    let _ = fs::write(&model_file, model_bytes);
    let sha256_hex = compute_sha256_hex(&model_file).expect("compute sha256");

    let engine = PdfcraftAdapter::new();

    // 1. Guard against ai_local_enabled = false
    let mut opts = TranslateOptions {
        manifest: AiModelManifest {
            model_path: model_file.clone(),
            sha256_hex: sha256_hex.clone(),
            format: "GGUF".into(),
            license_spdx: "Apache-2.0".into(),
            license_acknowledged: true,
        },
        ai_local_enabled: false,
        source_lang: "en".into(),
        target_lang: "es".into(),
        preserve_layout_blocks: true,
    };
    let res1 = engine.translate_pdf_experimental(&pdf_path, &out_pdf, opts.clone());
    assert!(res1.is_err(), "Must reject when ai_local_enabled is false");

    // 2. Guard against license_acknowledged = false
    opts.ai_local_enabled = true;
    opts.manifest.license_acknowledged = false;
    let res2 = engine.translate_pdf_experimental(&pdf_path, &out_pdf, opts.clone());
    assert!(res2.is_err(), "Must reject when license is unacknowledged");

    // 3. Guard against SHA-256 mismatch
    opts.manifest.license_acknowledged = true;
    opts.manifest.sha256_hex =
        "0000000000000000000000000000000000000000000000000000000000000000".into();
    let res3 = engine.translate_pdf_experimental(&pdf_path, &out_pdf, opts);
    assert!(res3.is_err(), "Must reject when SHA-256 mismatches");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_e2e_translate_pdf_block_level() {
    let pdf_path = get_corpus_pdf("standard-text.pdf");
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_trans_e2e_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);
    let out_pdf = temp_dir.join("translated_out.pdf");

    let model_file = temp_dir.join("valid_trans_model.gguf");
    let model_bytes = b"GGUF\x03\x00\x00\x00_valid_model_weights_and_tensors_dummy_data";
    let _ = fs::write(&model_file, model_bytes);
    let sha256_hex = compute_sha256_hex(&model_file).expect("compute sha256");

    let engine = PdfcraftAdapter::new();
    let opts = TranslateOptions {
        manifest: AiModelManifest {
            model_path: model_file,
            sha256_hex,
            format: "GGUF".into(),
            license_spdx: "Apache-2.0".into(),
            license_acknowledged: true,
        },
        ai_local_enabled: true,
        source_lang: "en".into(),
        target_lang: "es".into(),
        preserve_layout_blocks: true,
    };

    let report = engine
        .translate_pdf_experimental(&pdf_path, &out_pdf, opts)
        .expect("PDF translation should succeed");

    assert!(out_pdf.exists(), "Output PDF must exist");
    assert!(report.report_path.exists(), "Companion report must exist");
    assert_eq!(report.output_pdf_path, out_pdf);
    assert!(report.blocks_translated >= 1);
    assert!(report.pages_processed >= 1);
    assert_eq!(report.source_lang, "en");
    assert_eq!(report.target_lang, "es");

    let pdf_bytes = fs::read(&out_pdf).expect("read output pdf");
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "Output must be valid PDF starting with %PDF-"
    );

    let report_content = fs::read_to_string(&report.report_path).expect("read report");
    assert!(report_content
        .contains("# Translation Companion Report (Experimental Block-Level Re-Typeset)"));
    assert!(report_content.contains("- **Source Language:** `en`"));
    assert!(report_content.contains("- **Target Language:** `es`"));
    assert!(report_content.contains("## Block Translation Mappings"));
    assert!(report_content.contains("### Page 1, Block 1"));

    let _ = fs::remove_dir_all(&temp_dir);
}
