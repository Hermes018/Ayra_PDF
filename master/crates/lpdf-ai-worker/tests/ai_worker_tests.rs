use lpdf_ai_worker::{AiModelManifest, AiWorkerRequest, AiWorkerResponse, AiWorkerSession};
use lpdf_core_types::compute_sha256_hex;
use std::fs;
use std::path::PathBuf;

fn create_mock_gguf_file(name: &str, content: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lpdf_test_ai_{}", uuid_simple()));
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(name);
    let mut bytes = b"GGUF".to_vec();
    bytes.extend_from_slice(content);
    fs::write(&path, bytes).unwrap();
    path
}

fn create_corrupt_file(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lpdf_test_ai_{}", uuid_simple()));
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(name);
    fs::write(&path, b"BAD_HEADER_MAGIC").unwrap();
    path
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
fn test_ping_response() {
    let mut session = AiWorkerSession::default();
    let resp = session.handle_request(&AiWorkerRequest::Ping);
    assert_eq!(resp, AiWorkerResponse::Pong);
}

#[test]
fn test_f19_rejects_when_ai_local_disabled() {
    let mock_model = create_mock_gguf_file("model.gguf", b"model weights");
    let sha = compute_sha256_hex(&mock_model).unwrap();

    // Session with ai_local_enabled = false (default)
    let mut session = AiWorkerSession::new(false, None);

    let val_resp = session.handle_request(&AiWorkerRequest::ValidateModel {
        path: mock_model.clone(),
        expected_sha256: sha.clone(),
    });

    match val_resp {
        AiWorkerResponse::Error { code, message } => {
            assert_eq!(code, "F19_DISABLED");
            assert!(message.contains("ai_local_enabled is false"));
        }
        other => panic!("Expected F19_DISABLED error, got {:?}", other),
    }

    let infer_resp = session.handle_request(&AiWorkerRequest::InferChunk {
        prompt: "Summarize this".into(),
        max_tokens: 100,
        temperature: 0.2,
    });

    match infer_resp {
        AiWorkerResponse::Error { code, .. } => assert_eq!(code, "F19_DISABLED"),
        other => panic!("Expected F19_DISABLED error, got {:?}", other),
    }

    let _ = fs::remove_file(mock_model);
}

#[test]
fn test_f19_rejects_unacknowledged_license() {
    let mock_model = create_mock_gguf_file("llama3.gguf", b"llama3 weights payload");
    let sha = compute_sha256_hex(&mock_model).unwrap();

    let manifest = AiModelManifest {
        model_path: mock_model.clone(),
        sha256_hex: sha,
        format: "GGUF".into(),
        license_spdx: "Llama-3-Community".into(),
        license_acknowledged: false, // NOT acknowledged
    };

    let mut session = AiWorkerSession::new(true, Some(manifest));

    let resp = session.handle_request(&AiWorkerRequest::InferChunk {
        prompt: "Test query".into(),
        max_tokens: 100,
        temperature: 0.1,
    });

    match resp {
        AiWorkerResponse::Error { code, message } => {
            assert_eq!(code, "F19_LICENSE_UNACKNOWLEDGED");
            assert!(message.contains("Llama-3-Community"));
        }
        other => panic!("Expected F19_LICENSE_UNACKNOWLEDGED, got {:?}", other),
    }

    let _ = fs::remove_file(mock_model);
}

#[test]
fn test_f19_rejects_sha256_mismatch() {
    let mock_model = create_mock_gguf_file("model.gguf", b"clean payload");
    let mut session = AiWorkerSession::new(true, None);

    let resp = session.handle_request(&AiWorkerRequest::ValidateModel {
        path: mock_model.clone(),
        expected_sha256: "0000000000000000000000000000000000000000000000000000000000000000".into(),
    });

    match resp {
        AiWorkerResponse::Error { code, message } => {
            assert_eq!(code, "F19_HASH_MISMATCH");
            assert!(message.contains("checksum mismatch"));
        }
        other => panic!("Expected F19_HASH_MISMATCH, got {:?}", other),
    }

    let _ = fs::remove_file(mock_model);
}

#[test]
fn test_f19_rejects_corrupted_header_magic() {
    let bad_model = create_corrupt_file("corrupt.bin");
    let mut session = AiWorkerSession::new(true, None);

    let resp = session.handle_request(&AiWorkerRequest::ValidateModel {
        path: bad_model.clone(),
        expected_sha256: "any".into(),
    });

    match resp {
        AiWorkerResponse::Error { code, message } => {
            assert_eq!(code, "F19_INVALID_HEADER");
            assert!(message.contains("Invalid model binary header"));
        }
        other => panic!("Expected F19_INVALID_HEADER, got {:?}", other),
    }

    let _ = fs::remove_file(bad_model);
}

#[test]
fn test_f19_success_when_all_guards_satisfied() {
    let mock_model = create_mock_gguf_file("model.gguf", b"verified gguf content bytes");
    let sha = compute_sha256_hex(&mock_model).unwrap();

    let manifest = AiModelManifest {
        model_path: mock_model.clone(),
        sha256_hex: sha.clone(),
        format: "GGUF".into(),
        license_spdx: "Apache-2.0".into(),
        license_acknowledged: true, // Acknowledged
    };

    let mut session = AiWorkerSession::new(true, Some(manifest));

    // Validation passes
    let val_resp = session.handle_request(&AiWorkerRequest::ValidateModel {
        path: mock_model.clone(),
        expected_sha256: sha,
    });

    match val_resp {
        AiWorkerResponse::ModelValidated { model_info } => {
            assert_eq!(model_info.format, "GGUF");
            assert!(model_info.size_bytes > 4);
        }
        other => panic!("Expected ModelValidated, got {:?}", other),
    }

    // Inference chunk execution passes deterministically
    let infer_resp = session.handle_request(&AiWorkerRequest::InferChunk {
        prompt: "Legal contract confidentiality obligations and duration clauses".into(),
        max_tokens: 50,
        temperature: 0.2,
    });

    match infer_resp {
        AiWorkerResponse::ChunkInferred {
            text,
            tokens_produced,
            ..
        } => {
            assert!(tokens_produced > 0);
            assert!(text.contains("[Local LLM Summary Chunk]"));
            assert!(text.contains("confidentiality"));
        }
        other => panic!("Expected ChunkInferred, got {:?}", other),
    }

    let _ = fs::remove_file(mock_model);
}
