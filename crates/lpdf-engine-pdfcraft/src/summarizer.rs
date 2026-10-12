#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{
    ExtractiveSummaryReport, LlmSummaryOptions, LlmSummaryReport, SummaryLength,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const STOPWORDS: &[&str] = &[
    "a",
    "about",
    "above",
    "after",
    "again",
    "against",
    "all",
    "am",
    "an",
    "and",
    "any",
    "are",
    "aren't",
    "as",
    "at",
    "be",
    "because",
    "been",
    "before",
    "being",
    "below",
    "between",
    "both",
    "but",
    "by",
    "can",
    "can't",
    "cannot",
    "could",
    "couldn't",
    "did",
    "didn't",
    "do",
    "does",
    "doesn't",
    "doing",
    "don't",
    "down",
    "during",
    "each",
    "few",
    "for",
    "from",
    "further",
    "had",
    "hadn't",
    "has",
    "hasn't",
    "have",
    "haven't",
    "having",
    "he",
    "he'd",
    "he'll",
    "he's",
    "her",
    "here",
    "here's",
    "hers",
    "herself",
    "him",
    "himself",
    "his",
    "how",
    "how's",
    "i",
    "i'd",
    "i'll",
    "i'm",
    "i've",
    "if",
    "in",
    "into",
    "is",
    "isn't",
    "it",
    "it's",
    "its",
    "itself",
    "let's",
    "me",
    "more",
    "most",
    "mustn't",
    "my",
    "myself",
    "no",
    "nor",
    "not",
    "of",
    "off",
    "on",
    "once",
    "only",
    "or",
    "other",
    "ought",
    "our",
    "ours",
    "ourselves",
    "out",
    "over",
    "own",
    "same",
    "shan't",
    "she",
    "she'd",
    "she'll",
    "she's",
    "should",
    "shouldn't",
    "so",
    "some",
    "such",
    "than",
    "that",
    "that's",
    "the",
    "their",
    "theirs",
    "them",
    "themselves",
    "then",
    "there",
    "there's",
    "these",
    "they",
    "they'd",
    "they'll",
    "they're",
    "they've",
    "this",
    "those",
    "through",
    "to",
    "too",
    "under",
    "until",
    "up",
    "very",
    "was",
    "wasn't",
    "we",
    "we'd",
    "we'll",
    "we're",
    "we've",
    "were",
    "weren't",
    "what",
    "what's",
    "when",
    "when's",
    "where",
    "where's",
    "which",
    "while",
    "who",
    "who's",
    "whom",
    "why",
    "why's",
    "with",
    "won't",
    "would",
    "wouldn't",
    "you",
    "you'd",
    "you'll",
    "you're",
    "you've",
    "your",
    "yours",
    "yourself",
    "yourselves",
];

/// Internal sentence scoring structure.
#[derive(Debug, Clone)]
pub struct ScoredSentence {
    pub original_index: usize,
    pub text: String,
    pub word_count: usize,
    pub score: f64,
}

/// Tokenizes text into lowercase alphanumeric words.
pub fn tokenize_words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter_map(|w| {
            let trimmed = w.trim().to_lowercase();
            if trimmed.len() >= 2 {
                Some(trimmed)
            } else {
                None
            }
        })
        .collect()
}

/// Splits text into clean sentences.
pub fn split_into_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    let mut in_whitespace = false;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        let c = chars[i];
        if c.is_whitespace() {
            if !in_whitespace && !current.is_empty() {
                current.push(' ');
                in_whitespace = true;
            }
        } else {
            in_whitespace = false;
            current.push(c);

            // Sentence boundary check: punctuation followed by space or end
            if c == '.' || c == '!' || c == '?' {
                let next_is_boundary = if i + 1 < len {
                    let next = chars[i + 1];
                    next.is_whitespace() || next == '\n' || next == '\r' || next == '"'
                } else {
                    true
                };

                if next_is_boundary {
                    let trimmed = current.trim().to_string();
                    let words = tokenize_words(&trimmed);
                    // Filter out short fragments or numbers
                    if words.len() >= 4 && trimmed.len() >= 15 {
                        sentences.push(trimmed);
                    }
                    current.clear();
                    in_whitespace = true;
                }
            }
        }
        i += 1;
    }

    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        let words = tokenize_words(&remaining);
        if !words.is_empty() {
            sentences.push(remaining);
        }
    }

    if sentences.is_empty() {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_string());
        }
    }

    sentences
}

/// Computes non-stopword term frequencies and extracts top keywords.
pub fn extract_keywords(words: &[String], top_n: usize) -> (HashMap<String, usize>, Vec<String>) {
    let stopwords: HashSet<&str> = STOPWORDS.iter().copied().collect();
    let mut freqs: HashMap<String, usize> = HashMap::new();

    for w in words {
        if !stopwords.contains(w.as_str()) && w.len() >= 3 {
            *freqs.entry(w.clone()).or_insert(0) += 1;
        }
    }

    let mut sorted_words: Vec<(String, usize)> = freqs.clone().into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let keywords: Vec<String> = sorted_words
        .into_iter()
        .take(top_n)
        .map(|(w, _)| w)
        .collect();

    (freqs, keywords)
}

/// Scores and ranks sentences using deterministic TF centrality + position weighting.
pub fn score_sentences(
    sentences: &[String],
    freqs: &HashMap<String, usize>,
) -> Vec<ScoredSentence> {
    let total_sentences = sentences.len();
    let mut scored = Vec::with_capacity(total_sentences);

    for (idx, sent) in sentences.iter().enumerate() {
        let words = tokenize_words(sent);
        let word_count = words.len();
        if word_count == 0 {
            continue;
        }

        let mut raw_score = 0.0f64;
        for w in &words {
            if let Some(&count) = freqs.get(w) {
                raw_score += ((count as f64) + 1.0).ln();
            }
        }

        // Length normalization (penalize extreme brevity or verbosity)
        let len_norm = (word_count as f64).sqrt();
        let mut score = if len_norm > 0.0 {
            raw_score / len_norm
        } else {
            0.0
        };

        // Position weighting: introductory sentences and paragraph starters carry higher centrality
        if idx == 0 {
            score *= 1.5;
        } else if total_sentences > 5 && idx < (total_sentences / 5) {
            score *= 1.25;
        } else if idx == total_sentences - 1 {
            score *= 1.1;
        }

        if !(6..=60).contains(&word_count) {
            score *= 0.75;
        }

        scored.push(ScoredSentence {
            original_index: idx,
            text: sent.clone(),
            word_count,
            score,
        });
    }

    scored
}

/// Result tuple for `summarize_text`: (markdown, key_points, keywords, source_words, summary_words)
pub type ExtractiveTextSummary = (String, Vec<String>, Vec<String>, usize, usize);

/// Selects target sentence count based on `SummaryLength`.
pub fn target_sentence_count(total: usize, length: SummaryLength) -> usize {
    match length {
        SummaryLength::Short => 4.min(total),
        SummaryLength::Medium => 8.min(total),
        SummaryLength::Detailed => 16.min(total),
    }
}

/// Performs zero-model extractive text summarization.
pub fn summarize_text(
    source_name: &str,
    text: &str,
    length: SummaryLength,
) -> Result<ExtractiveTextSummary> {
    let all_words = tokenize_words(text);
    let source_word_count = all_words.len();

    if source_word_count == 0 {
        return Err(LpdfError::InvalidInput(
            "Source document contains no extractable text for summarization.".to_string(),
        ));
    }

    let sentences = split_into_sentences(text);
    if sentences.is_empty() {
        return Err(LpdfError::InvalidInput(
            "No valid sentences could be extracted from document text.".to_string(),
        ));
    }

    let (freqs, keywords) = extract_keywords(&all_words, 8);
    let scored = score_sentences(&sentences, &freqs);

    let target_count = target_sentence_count(scored.len(), length).max(1);

    // Rank by score descending
    let mut by_score = scored.clone();
    by_score.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.original_index.cmp(&b.original_index))
    });

    let top_scored: Vec<ScoredSentence> = by_score.into_iter().take(target_count).collect();

    // Key points from top 3-4 scored sentences
    let key_points_count = 4.min(top_scored.len());
    let key_points: Vec<String> = top_scored
        .iter()
        .take(key_points_count)
        .map(|s| s.text.clone())
        .collect();

    // Re-order top sentences back into original document sequence for coherent reading flow
    let mut document_ordered = top_scored;
    document_ordered.sort_by_key(|s| s.original_index);

    let summary_text = document_ordered
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    let summary_word_count = tokenize_words(&summary_text).len();

    let reduction_pct = if source_word_count > 0 {
        ((summary_word_count as f64 / source_word_count as f64) * 100.0).round() as usize
    } else {
        100
    };

    let mut markdown = String::new();
    markdown.push_str("# Executive Document Summary\n\n");
    markdown.push_str(&format!("- **Source Document:** `{}`\n", source_name));
    markdown.push_str(&format!("- **Length Setting:** {:?}\n", length));
    markdown.push_str(&format!(
        "- **Reduction Ratio:** {}% ({} summary words vs {} source words)\n",
        reduction_pct, summary_word_count, source_word_count
    ));
    if !keywords.is_empty() {
        markdown.push_str(&format!("- **Key Topics:** {}\n", keywords.join(", ")));
    }
    markdown.push_str("\n## Key Takeaways\n\n");
    for point in &key_points {
        markdown.push_str(&format!("- {}\n", point));
    }
    markdown.push_str("\n## Executive Summary\n\n");
    markdown.push_str(&summary_text);
    markdown.push('\n');

    Ok((
        markdown,
        key_points,
        keywords,
        source_word_count,
        summary_word_count,
    ))
}

/// Executes extractive summarization on raw text and commits to `output_path` atomically.
pub fn generate_extractive_summary(
    input_path: &Path,
    output_path: &Path,
    raw_text: &str,
    length: SummaryLength,
) -> Result<ExtractiveSummaryReport> {
    let file_name = input_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("document.pdf");

    let (markdown, key_points, keywords, source_word_count, summary_word_count) =
        summarize_text(file_name, raw_text, length)?;

    let target_file = if output_path.is_dir() {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        output_path.join(format!("{}_summary.md", stem))
    } else {
        output_path.to_path_buf()
    };

    lpdf_fs_safety::atomic_write(&target_file, markdown.as_bytes())?;

    Ok(ExtractiveSummaryReport {
        output_path: target_file,
        summary_text: markdown,
        key_points,
        keywords,
        source_word_count,
        summary_word_count,
    })
}

/// Partitions text into bounded semantic chunks respecting sentence boundaries and `chunk_word_limit`.
pub fn partition_into_chunks(text: &str, chunk_word_limit: usize) -> Vec<String> {
    let limit = if chunk_word_limit == 0 {
        400
    } else {
        chunk_word_limit
    };
    let sentences = split_into_sentences(text);
    if sentences.is_empty() {
        let trimmed = text.trim();
        return if trimmed.is_empty() {
            Vec::new()
        } else {
            vec![trimmed.to_string()]
        };
    }

    let mut chunks = Vec::new();
    let mut current_chunk_sentences = Vec::new();
    let mut current_words = 0;

    for sent in sentences {
        let sent_words = tokenize_words(&sent).len();
        if current_words + sent_words > limit && !current_chunk_sentences.is_empty() {
            chunks.push(current_chunk_sentences.join(" "));
            current_chunk_sentences.clear();
            current_words = 0;
        }

        current_words += sent_words;
        current_chunk_sentences.push(sent);
    }

    if !current_chunk_sentences.is_empty() {
        chunks.push(current_chunk_sentences.join(" "));
    }

    chunks
}

/// Executes Map-Reduce LLM summarization on raw document text.
pub fn generate_llm_map_reduce_summary(
    input_path: &Path,
    output_path: &Path,
    raw_text: &str,
    options: &LlmSummaryOptions,
) -> Result<LlmSummaryReport> {
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

    let all_words = tokenize_words(raw_text);
    let source_word_count = all_words.len();
    if source_word_count == 0 {
        return Err(LpdfError::InvalidInput(
            "Source document contains no extractable text for summarization.".to_string(),
        ));
    }

    let chunks = partition_into_chunks(raw_text, options.chunk_word_limit);
    if chunks.is_empty() {
        return Err(LpdfError::InvalidInput(
            "No valid chunks could be partitioned from document text.".to_string(),
        ));
    }

    // Map pass: summarize each chunk
    let mut chunk_summaries = Vec::with_capacity(chunks.len());
    for (idx, chunk) in chunks.iter().enumerate() {
        let prompt = format!(
            "Summarize the key information, arguments, and data points from Section {} concisely in 2-3 sentences:\n\n{}",
            idx + 1,
            chunk
        );
        let resp = session.handle_request(&lpdf_ai_worker::AiWorkerRequest::InferChunk {
            prompt,
            max_tokens: options.max_tokens as usize,
            temperature: options.temperature,
        });

        let summary_text = match resp {
            lpdf_ai_worker::AiWorkerResponse::ChunkInferred { text, .. } => text,
            lpdf_ai_worker::AiWorkerResponse::Error { code, message } => {
                return Err(LpdfError::Engine(format!(
                    "Inference error on chunk {} [{}]: {}",
                    idx + 1,
                    code,
                    message
                )));
            }
            _ => {
                return Err(LpdfError::Engine(
                    "Unexpected inference response from AI worker.".to_string(),
                ));
            }
        };

        chunk_summaries.push(summary_text);
    }

    // Reduce pass: synthesize all chunk summaries into executive synthesis
    let executive_synthesis = if chunk_summaries.len() == 1 {
        chunk_summaries[0].clone()
    } else {
        let reduce_prompt = format!(
            "Synthesize the following section summaries into a cohesive, executive-level summary scaled to {:?} length:\n\n{}",
            options.length,
            chunk_summaries
                .iter()
                .enumerate()
                .map(|(i, s)| format!("Section {}: {}", i + 1, s))
                .collect::<Vec<_>>()
                .join("\n\n")
        );

        let resp = session.handle_request(&lpdf_ai_worker::AiWorkerRequest::InferChunk {
            prompt: reduce_prompt,
            max_tokens: (options.max_tokens * 2) as usize,
            temperature: options.temperature,
        });

        match resp {
            lpdf_ai_worker::AiWorkerResponse::ChunkInferred { text, .. } => text,
            lpdf_ai_worker::AiWorkerResponse::Error { code, message } => {
                return Err(LpdfError::Engine(format!(
                    "Inference error during reduce step [{}]: {}",
                    code, message
                )));
            }
            _ => {
                return Err(LpdfError::Engine(
                    "Unexpected reduce inference response from AI worker.".to_string(),
                ));
            }
        }
    };

    let summary_word_count = tokenize_words(&executive_synthesis).len();
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

    let mut markdown = String::new();
    markdown.push_str("# Executive Document Summary (Local LLM Map-Reduce)\n\n");
    markdown.push_str(&format!("- **Source Document:** `{}`\n", file_name));
    markdown.push_str(&format!(
        "- **Model Provenance:** `{}` (Format: `{}`, SHA-256: `{}...`)\n",
        model_name, options.manifest.format, sha_snippet
    ));
    markdown.push_str(&format!(
        "- **Licence (SPDX):** `{}` (Acknowledged for offline execution)\n",
        options.manifest.license_spdx
    ));
    markdown.push_str(&format!(
        "- **Map-Reduce Chunks:** {} sections processed\n",
        chunks.len()
    ));
    markdown.push_str(&format!("- **Length Setting:** {:?}\n", options.length));
    markdown.push_str(&format!(
        "- **Word Counts:** {} source words -> {} summary words\n\n",
        source_word_count, summary_word_count
    ));

    markdown.push_str("## Executive Synthesis\n\n");
    markdown.push_str(&executive_synthesis);
    markdown.push_str("\n\n## Section-by-Section Intermediate Summaries\n\n");

    for (i, chunk_sum) in chunk_summaries.iter().enumerate() {
        markdown.push_str(&format!("### Section {}\n\n{}\n\n", i + 1, chunk_sum));
    }

    let target_file = if output_path.is_dir() {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("document");
        output_path.join(format!("{}_llm_summary.md", stem))
    } else {
        output_path.to_path_buf()
    };

    lpdf_fs_safety::atomic_write(&target_file, markdown.as_bytes())?;

    Ok(LlmSummaryReport {
        output_path: target_file,
        summary_text: markdown,
        chunk_summaries,
        chunks_processed: chunks.len(),
        model_sha256: options.manifest.sha256_hex.clone(),
        source_word_count,
        summary_word_count,
    })
}
