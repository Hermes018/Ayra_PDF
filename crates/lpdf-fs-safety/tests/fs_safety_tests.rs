use lpdf_core_types::{
    compute_sha256_hex_from_reader, inspect_ai_model, AppSettings, HardwareTier, OutputPolicy,
    ThemePreference,
};
use lpdf_fs_safety::*;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_sanitize_filename() {
    assert_eq!(sanitize_filename("my doc (1).pdf"), "my_doc__1__pdf");
    assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
    assert_eq!(sanitize_filename(""), "export");
    assert_eq!(sanitize_filename("..."), "export");
}

#[test]
fn test_atomic_write() {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_fs_{}", uuid_simple()));
    let file_path = temp_dir.join("test.txt");
    let content = b"Hello, World!";

    let res = atomic_write(&file_path, content);
    assert!(res.is_ok());
    assert!(file_path.exists());
    let read = fs::read(&file_path).unwrap();
    assert_eq!(read, content);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_create_safe_output_dir() {
    let base_dir = std::env::temp_dir().join(format!("lpdf_test_safe_dir_{}", uuid_simple()));
    let dir1 = create_safe_output_dir(&base_dir, "export_images").unwrap();
    assert!(dir1.exists());
    assert!(dir1.ends_with("export_images"));

    // Creating second time with same prefix should append _1
    let dir2 = create_safe_output_dir(&base_dir, "export_images").unwrap();
    assert!(dir2.exists());
    assert!(dir2.ends_with("export_images_1"));

    let _ = fs::remove_dir_all(&base_dir);
}

#[test]
fn test_settings_atomic_save_and_load_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_settings_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);
    let settings_path = temp_dir.join("settings.toml");

    // Non-existent path returns default
    let non_existent = temp_dir.join("missing.toml");
    let def = load_settings(&non_existent).unwrap();
    assert_eq!(def, AppSettings::default());
    assert_eq!(def.tier_override, None);
    assert_eq!(def.output_policy, OutputPolicy::SuffixNewFile);
    assert!(def.backups_enabled);
    assert_eq!(def.theme, ThemePreference::System);

    // Save custom settings
    let custom = AppSettings {
        tier_override: Some(HardwareTier::High),
        output_policy: OutputPolicy::OverwriteWithBackup,
        backups_enabled: false,
        language: "de".to_string(),
        theme: ThemePreference::Dark,
        local_ai_model_path: Some(PathBuf::from("/models/llama.gguf")),
        ai_local_enabled: true,
    };

    save_settings_atomic(&settings_path, &custom).unwrap();
    assert!(settings_path.exists());

    // Load back and verify equality
    let loaded = load_settings(&settings_path).unwrap();
    assert_eq!(loaded, custom);
    assert_eq!(loaded.tier_override, Some(HardwareTier::High));
    assert_eq!(loaded.output_policy, OutputPolicy::OverwriteWithBackup);
    assert!(!loaded.backups_enabled);
    assert_eq!(loaded.language, "de");
    assert_eq!(loaded.theme, ThemePreference::Dark);
    assert_eq!(
        loaded.local_ai_model_path,
        Some(PathBuf::from("/models/llama.gguf"))
    );
    assert!(loaded.ai_local_enabled);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_sha256_vector_and_ai_model_inspection() {
    // Known vector: SHA256("hello world") = b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9
    let input = b"hello world";
    let hex = compute_sha256_hex_from_reader(Cursor::new(input)).unwrap();
    assert_eq!(
        hex,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );

    // Test AI model file inspection
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_models_{}", uuid_simple()));
    let _ = fs::create_dir_all(&temp_dir);

    // Mock GGUF model
    let gguf_path = temp_dir.join("test-model.gguf");
    let mut gguf_bytes = b"GGUF".to_vec();
    gguf_bytes.extend_from_slice(&[0u8; 1024]);
    fs::write(&gguf_path, &gguf_bytes).unwrap();

    let gguf_info = inspect_ai_model(&gguf_path).unwrap();
    assert_eq!(gguf_info.file_name, "test-model.gguf");
    assert_eq!(gguf_info.format, "GGUF");
    assert_eq!(gguf_info.size_bytes, gguf_bytes.len() as u64);
    assert_eq!(gguf_info.sha256_hex.len(), 64);

    // Mock ONNX model
    let onnx_path = temp_dir.join("ocr-detector.onnx");
    fs::write(&onnx_path, b"ONNX_MOCK_PAYLOAD").unwrap();
    let onnx_info = inspect_ai_model(&onnx_path).unwrap();
    assert_eq!(onnx_info.file_name, "ocr-detector.onnx");
    assert_eq!(onnx_info.format, "ONNX");

    // Missing file error
    let missing_path = temp_dir.join("non_existent.bin");
    assert!(inspect_ai_model(&missing_path).is_err());

    // Empty file error
    let empty_path = temp_dir.join("empty.gguf");
    fs::write(&empty_path, b"").unwrap();
    assert!(inspect_ai_model(&empty_path).is_err());

    let _ = fs::remove_dir_all(&temp_dir);
}
