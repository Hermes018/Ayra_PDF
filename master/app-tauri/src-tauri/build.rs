//! Build script: embeds the short git hash so the About screen can show which build this is.
//! Offline and dependency-free: it runs `git rev-parse` once and falls back to "unknown" when the
//! source is not a git checkout (a tarball, a packaging sandbox) or git is missing. The release
//! scripts may pin it with `LPDF_BUILD_HASH`.

use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let text = text.trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// A short hash is 7 to 40 hex digits; anything else is not trusted.
fn clean(raw: &str) -> Option<String> {
    let raw = raw.trim();
    (raw.len() >= 7 && raw.len() <= 40 && raw.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| raw.to_ascii_lowercase())
}

fn main() {
    println!("cargo:rerun-if-env-changed=LPDF_BUILD_HASH");
    let hash = std::env::var("LPDF_BUILD_HASH")
        .ok()
        .and_then(|v| clean(&v))
        .or_else(|| git(&["rev-parse", "--short=10", "HEAD"]).and_then(|v| clean(&v)))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=LPDF_BUILD_HASH={hash}");

    // Re-run when the checked-out commit changes. `--git-path` also resolves inside worktrees.
    // Only existing paths are registered (a missing path would force a re-run every build).
    for name in ["HEAD", "refs/heads"] {
        if let Some(p) = git(&["rev-parse", "--git-path", name]) {
            let p = PathBuf::from(p);
            if p.exists() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }

    tauri_build::build()
}
