//! BUG-050 / BUG-032: a job or save must never write over its own input or over an existing file.
//!
//! `check_output_path` is the one check, used by the job queue (every job) and by the Studio save command.
//! # Scope
//! Applies to every writer that takes a path the USER (or a screen) chose: job outputs, Studio saves, the takeoff
//! CSV, the diagnostics report, `convert_pdf` and `extract_text`.
//! Exempt on purpose: files the app owns and manages itself (the page-scales sidecar, settings, preferences, the
//! markup library, workspace documents and the log). They are internal state, are rewritten atomically by design,
//! and their paths never come from the user.
//!
//! `check_output_path_confirmed` is for a writer whose user explicitly confirmed replacing an existing FILE (the
//! diagnostics report); it still refuses any protected path (an open document or an input).
//!
//! It compares canonical paths (symlinks and `..` resolved), so a different spelling of the input path is
//! still refused. It never touches the disk beyond `canonicalize` / `metadata`.

use std::fmt;
use std::path::{Path, PathBuf};

/// Why an output path was refused. Typed so callers can tell the cases apart; the text is plain words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputPathError {
    /// The output is the same file as an input: writing it would modify the original.
    SameAsInput { path: String },
    /// A file already exists at the output path: writing it would replace that file.
    AlreadyExists { path: String },
}

impl fmt::Display for OutputPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputPathError::SameAsInput { path } => write!(
                f,
                "OUTPUT_PATH_UNSAFE: the output file would overwrite the original ({path}). Nothing was written."
            ),
            OutputPathError::AlreadyExists { path } => write!(
                f,
                "OUTPUT_PATH_UNSAFE: a file already exists at the output path ({path}). Nothing was written."
            ),
        }
    }
}

impl std::error::Error for OutputPathError {}

/// Canonical form of a path that may not exist yet: the parent is canonicalized and the file name re-attached.
fn canonical_target(path: &Path) -> PathBuf {
    if let Ok(c) = path.canonicalize() {
        return c;
    }
    match (path.parent(), path.file_name()) {
        (Some(parent), Some(name)) => {
            let base = if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            };
            base.canonicalize()
                .map(|p| p.join(name))
                .unwrap_or_else(|_| path.to_path_buf())
        }
        _ => path.to_path_buf(),
    }
}

/// True when the path names a PDF, either as written or after resolving symlinks (a link `x.txt` that points at
/// `a.pdf` resolves to a PDF). Used by writers of non-PDF text (diagnostics report, takeoff CSV).
pub fn targets_pdf(path: &Path) -> bool {
    let is_pdf = |p: &Path| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
    is_pdf(path) || is_pdf(&canonical_target(path))
}

/// Like `check_output_path`, but an existing file is allowed (the user confirmed replacing it). A protected path
/// (an input or an open document) is still always refused.
pub fn check_output_path_confirmed(
    protected: &[PathBuf],
    output: &Path,
) -> Result<(), OutputPathError> {
    let target = canonical_target(output);
    for p in protected {
        if canonical_target(p) == target {
            return Err(OutputPathError::SameAsInput {
                path: output.display().to_string(),
            });
        }
    }
    Ok(())
}

/// Refuse an output that is one of the inputs, or an existing file. An existing directory is not a collision
/// (some jobs export into a folder).
pub fn check_output_path(inputs: &[PathBuf], output: &Path) -> Result<(), OutputPathError> {
    let target = canonical_target(output);
    for input in inputs {
        if canonical_target(input) == target {
            return Err(OutputPathError::SameAsInput {
                path: output.display().to_string(),
            });
        }
    }
    if output.is_file() {
        return Err(OutputPathError::AlreadyExists {
            path: output.display().to_string(),
        });
    }
    Ok(())
}
