use lpdf_engine_pdfcraft::summarizer::*;
use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{PdfEngine, SummaryLength};
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
