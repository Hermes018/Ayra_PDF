#![forbid(unsafe_code)]

use lpdf_core_types::{LpdfError, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Safely writes bytes to `target_path` atomically using a temporary file
/// in the same directory, verifying byte count and renaming.
pub fn atomic_write(target_path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(LpdfError::Io)?;

    let file_stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = target_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("tmp");

    let temp_name = format!(".{}_{}_{}.tmp", file_stem, ext, uuid_simple());
    let temp_path = parent.join(temp_name);

    fs::write(&temp_path, bytes).map_err(LpdfError::Io)?;

    let meta = fs::metadata(&temp_path).map_err(LpdfError::Io)?;
    if meta.len() != bytes.len() as u64 {
        let _ = fs::remove_file(&temp_path);
        return Err(LpdfError::Io(std::io::Error::other(
            "Written byte count mismatch",
        )));
    }

    fs::rename(&temp_path, target_path).map_err(LpdfError::Io)?;
    Ok(())
}

/// Create a safely named, newly created output folder inside `parent_dir` based on `prefix`.
/// If the folder already exists, appends numbers (_1, _2, etc.) to prevent collisions.
pub fn create_safe_output_dir(parent_dir: &Path, prefix: &str) -> Result<PathBuf> {
    fs::create_dir_all(parent_dir).map_err(LpdfError::Io)?;
    let sanitized_prefix = sanitize_filename(prefix);

    let mut candidate = parent_dir.join(&sanitized_prefix);
    let mut counter = 1;
    while candidate.exists() {
        candidate = parent_dir.join(format!("{}_{}", sanitized_prefix, counter));
        counter += 1;
    }

    fs::create_dir_all(&candidate).map_err(LpdfError::Io)?;
    Ok(candidate)
}

/// Sanitize filename to avoid directory traversal or reserved characters.
pub fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('_').trim();
    if trimmed.is_empty() {
        "export".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Generate a short random alphanumeric string for temp file suffixes without extra external deps.
pub fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}
