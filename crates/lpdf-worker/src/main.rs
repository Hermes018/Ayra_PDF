#![forbid(unsafe_code)]

use lpdf_engine_port::PdfEngine;
use lpdf_engine_switcher::EngineSwitcher;
use lpdf_jobs::JobRequest;
use std::io::Read;
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
use landlock::{
    Access, AccessFs, PathBeneath, PathFd, Ruleset, RulesetAttr, RulesetCreatedAttr, RulesetStatus,
    ABI,
};

#[cfg(target_os = "linux")]
fn apply_sandbox(read_paths: &[PathBuf], write_paths: &[PathBuf]) -> Result<(), String> {
    let abi = ABI::V1;
    let mut ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(abi))
        .map_err(|e| format!("Landlock initialization error: {e}"))?
        .create()
        .map_err(|e| format!("Landlock create ruleset error: {e}"))?;

    // Allow read-only access to input files and directories
    for path in read_paths {
        if path.exists() {
            let target_path = if path.is_dir() {
                path.clone()
            } else {
                path.parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| path.clone())
            };
            if let Ok(fd) = PathFd::new(&target_path) {
                ruleset = ruleset
                    .add_rule(PathBeneath::new(fd, AccessFs::from_read(abi)))
                    .map_err(|e| {
                        format!(
                            "Landlock read rule error for {}: {e}",
                            target_path.display()
                        )
                    })?;
            }
        }
    }

    // Allow read-write access to destination directories
    for path in write_paths {
        let target_dir = if path.is_dir() {
            path.clone()
        } else {
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };
        if target_dir.exists() {
            if let Ok(fd) = PathFd::new(&target_dir) {
                ruleset = ruleset
                    .add_rule(PathBeneath::new(fd, AccessFs::from_all(abi)))
                    .map_err(|e| {
                        format!(
                            "Landlock write rule error for {}: {e}",
                            target_dir.display()
                        )
                    })?;
            }
        }
    }

    // Always permit temp directory for intermediate staging
    let tmp = Path::new("/tmp");
    if tmp.exists() {
        if let Ok(fd) = PathFd::new(tmp) {
            ruleset = ruleset
                .add_rule(PathBeneath::new(fd, AccessFs::from_all(abi)))
                .map_err(|e| format!("Landlock /tmp rule error: {e}"))?;
        }
    }

    let status = ruleset
        .restrict_self()
        .map_err(|e| format!("Landlock restrict_self error: {e}"))?;

    // Failsafe F3/F19: Fail closed if the sandbox cannot be enforced
    if status.ruleset == RulesetStatus::NotEnforced {
        return Err(
            "Landlock sandbox could not be enforced by the host kernel; failing closed".into(),
        );
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn apply_sandbox(_read_paths: &[PathBuf], _write_paths: &[PathBuf]) -> Result<(), String> {
    Ok(())
}

fn extract_paths(request: &JobRequest) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut read_paths = Vec::new();
    let mut write_paths = Vec::new();
    let (_, in_opt, out_opt) = request.summary();
    if let Some(inp) = in_opt {
        read_paths.push(PathBuf::from(inp));
    }
    if let Some(outp) = out_opt {
        write_paths.push(PathBuf::from(outp));
    }
    if let JobRequest::Merge { input_paths, .. } = request {
        for p in input_paths {
            read_paths.push(PathBuf::from(p));
        }
    }
    (read_paths, write_paths)
}

fn execute_request(request: &JobRequest) -> Result<String, String> {
    let mut engine = EngineSwitcher::new();
    match request {
        JobRequest::GetInfo { input_path } => {
            engine
                .load(Path::new(input_path))
                .map_err(|e| e.to_string())?;
            let info = engine.info().map_err(|e| e.to_string())?;
            Ok(format!("Page count: {}", info.page_count))
        }
        JobRequest::Merge {
            input_paths,
            output_path,
        } => {
            let in_refs: Vec<&Path> = input_paths.iter().map(Path::new).collect();
            engine
                .merge(&in_refs, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::DeletePages {
            input_path,
            output_path,
            pages,
        } => {
            engine
                .delete_pages(Path::new(input_path), pages, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::ExtractPages {
            input_path,
            output_path,
            pages,
        } => {
            engine
                .extract_pages(Path::new(input_path), pages, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::RotatePages {
            input_path,
            output_path,
            pages,
            degrees,
        } => {
            engine
                .rotate_pages(
                    Path::new(input_path),
                    pages,
                    *degrees,
                    Path::new(output_path),
                )
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::RotateDocument {
            input_path,
            output_path,
            degrees,
        } => {
            engine
                .rotate_document(Path::new(input_path), *degrees, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::Compress {
            input_path,
            output_path,
            profile,
        } => {
            engine
                .compress(Path::new(input_path), *profile, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::Repair {
            input_path,
            output_path,
        } => {
            let fixes = engine
                .repair(Path::new(input_path), Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(format!("Repaired: {} fixes", fixes.len()))
        }
        JobRequest::Ocr {
            input_path,
            output_path,
            lang,
        } => {
            engine
                .ocr(Path::new(input_path), lang, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::ProtectDocument {
            input_path,
            output_path,
            password,
            permissions,
        } => {
            engine
                .protect(
                    Path::new(input_path),
                    password,
                    permissions.clone(),
                    Path::new(output_path),
                )
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::UnlockDocument {
            input_path,
            output_path,
            password,
        } => {
            engine
                .unlock(Path::new(input_path), password, Path::new(output_path))
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        JobRequest::RedactDocument {
            input_path,
            output_path,
            regions,
        } => {
            engine
                .redact(
                    Path::new(input_path),
                    regions.clone(),
                    Path::new(output_path),
                )
                .map_err(|e| e.to_string())?;
            Ok(output_path.clone())
        }
        _ => Err("Unsupported operation in worker sandbox".into()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let raw_payload = if args.len() > 1 {
        args[1].clone()
    } else {
        let mut buf = String::new();
        if let Err(e) = std::io::stdin().read_to_string(&mut buf) {
            eprintln!("Worker error: failed to read stdin: {e}");
            std::process::exit(1);
        }
        buf
    };

    let request: JobRequest = match serde_json::from_str(&raw_payload) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Worker error: invalid JSON JobRequest: {e}");
            std::process::exit(1);
        }
    };

    let (read_paths, write_paths) = extract_paths(&request);

    // Apply strict Landlock filesystem sandboxing before performing work
    if let Err(e) = apply_sandbox(&read_paths, &write_paths) {
        eprintln!("Worker sandbox enforcement failed (FAIL CLOSED): {e}");
        std::process::exit(1);
    }

    match execute_request(&request) {
        Ok(out) => {
            println!("{}", out);
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Worker execution error: {e}");
            std::process::exit(1);
        }
    }
}
