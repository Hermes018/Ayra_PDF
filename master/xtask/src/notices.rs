//! `cargo xtask notices [--check]` — writes `THIRD_PARTY_NOTICES.md` (ADR-018 R3).
//!
//! Sources, all local (no network):
//! - Rust: `cargo metadata --filter-platform x86_64-unknown-linux-gnu --offline`, walking only
//!   normal (shipped) dependencies from every workspace member except `xtask`. For each package
//!   the licence/notice files in its directory are included; when it has none (crates inside a git
//!   checkout or a vendored repo, e.g. cadcraft or pdfcraft), the nearest parent directory that has
//!   them is used, so repository-level `NOTICE` files (Apache-2.0 §4(d)) are carried.
//! - npm: `pnpm --dir app-tauri licenses list --json` (build and runtime packages: a superset of
//!   what the bundle contains), with each package's licence files.
//!
//! Identical licence texts are printed once, listing every package that uses them. Absolute local
//! paths never appear in the output, so it is reproducible. `--check` fails if the committed file
//! differs from a fresh render.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const OUT: &str = "THIRD_PARTY_NOTICES.md";
const TARGET: &str = "x86_64-unknown-linux-gnu";

struct Pkg {
    ecosystem: &'static str,
    name: String,
    version: String,
    license: String,
    repository: String,
    texts: Vec<(String, String)>, // (file name, contents)
}

fn is_licence_file(name: &str) -> bool {
    let n = name.to_ascii_uppercase();
    [
        "LICENSE",
        "LICENCE",
        "COPYING",
        "NOTICE",
        "COPYRIGHT",
        "UNLICENSE",
    ]
    .iter()
    .any(|p| n.starts_with(p))
}

fn licence_files_in(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return out;
    };
    let mut names: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    names.sort();
    for p in names {
        let Some(name) = p.file_name().and_then(|n| n.to_str()).map(str::to_string) else {
            continue;
        };
        if p.is_file() && is_licence_file(&name) {
            if let Ok(text) = fs::read_to_string(&p) {
                out.push((name, text.replace("\r\n", "\n")));
            }
        }
    }
    out
}

/// Licence files of `dir`, else of the nearest parent (at most 3 levels up) that has some.
fn licence_files_near(dir: &Path) -> Vec<(String, String)> {
    let mut d = Some(dir);
    for _ in 0..4 {
        let Some(cur) = d else { break };
        let found = licence_files_in(cur);
        if !found.is_empty() {
            return found;
        }
        d = cur.parent();
    }
    Vec::new()
}

fn run_json(cmd: &mut Command) -> Value {
    let out = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            eprintln!("notices: cannot run {:?}: {e}", cmd);
            exit(1);
        }
    };
    if !out.status.success() {
        eprintln!(
            "notices: {:?} failed:\n{}",
            cmd,
            String::from_utf8_lossy(&out.stderr)
        );
        exit(1);
    }
    match serde_json::from_slice(&out.stdout) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("notices: {:?} printed invalid JSON: {e}", cmd);
            exit(1);
        }
    }
}

fn s(v: &Value, k: &str) -> String {
    v.get(k).and_then(Value::as_str).unwrap_or("").to_string()
}

fn rust_packages(root: &Path) -> Vec<Pkg> {
    let meta = run_json(Command::new(env_cargo()).current_dir(root).args([
        "metadata",
        "--format-version",
        "1",
        "--offline",
        "--filter-platform",
        TARGET,
    ]));
    let empty = Vec::new();
    let packages = meta
        .get("packages")
        .and_then(Value::as_array)
        .unwrap_or(&empty);
    let by_id: HashMap<String, &Value> = packages.iter().map(|p| (s(p, "id"), p)).collect();
    let members: BTreeSet<String> = meta
        .get("workspace_members")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let nodes: HashMap<String, &Value> = meta
        .pointer("/resolve/nodes")
        .and_then(Value::as_array)
        .map(|a| a.iter().map(|n| (s(n, "id"), n)).collect())
        .unwrap_or_default();

    // Walk normal dependencies from every member except xtask.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = members
        .iter()
        .filter(|id| by_id.get(*id).map(|p| s(p, "name")) != Some("xtask".into()))
        .cloned()
        .collect();
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let Some(node) = nodes.get(&id) else { continue };
        for dep in node.get("deps").and_then(Value::as_array).unwrap_or(&empty) {
            let normal = dep
                .get("dep_kinds")
                .and_then(Value::as_array)
                .is_some_and(|ks| ks.iter().any(|k| k.get("kind").is_none_or(Value::is_null)));
            if normal {
                stack.push(s(dep, "pkg"));
            }
        }
    }

    let mut out = Vec::new();
    for id in seen {
        if members.contains(&id) {
            continue;
        }
        let Some(p) = by_id.get(&id) else { continue };
        let dir = PathBuf::from(s(p, "manifest_path"));
        let dir = dir.parent().map(Path::to_path_buf).unwrap_or_default();
        let mut license = s(p, "license");
        if license.is_empty() && !s(p, "license_file").is_empty() {
            license = "see licence file".into();
        }
        out.push(Pkg {
            ecosystem: "Rust crate",
            name: s(p, "name"),
            version: s(p, "version"),
            license,
            repository: s(p, "repository"),
            texts: licence_files_near(&dir),
        });
    }
    out
}

fn npm_packages(root: &Path) -> Vec<Pkg> {
    let v = run_json(Command::new("pnpm").current_dir(root).args([
        "--offline",
        "--dir",
        "app-tauri",
        "licenses",
        "list",
        "--json",
    ]));
    let mut out = Vec::new();
    let Some(groups) = v.as_object() else {
        return out;
    };
    for list in groups.values() {
        for p in list.as_array().into_iter().flatten() {
            let versions: Vec<String> = p
                .get("versions")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            let dir = p
                .get("paths")
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .unwrap_or_default();
            out.push(Pkg {
                ecosystem: "npm package",
                name: s(p, "name"),
                version: versions.join(", "),
                license: s(p, "license"),
                repository: s(p, "homepage"),
                texts: licence_files_in(&dir),
            });
        }
    }
    out
}

fn env_cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

/// Standard SPDX licence texts (`xtask/spdx/<id>.txt`) for packages that ship no licence file.
fn spdx_texts(root: &Path, expression: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let ids: BTreeSet<&str> = expression
        .split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '/')
        .filter(|t| !t.is_empty() && !matches!(*t, "OR" | "AND" | "WITH"))
        .collect();
    for id in ids {
        let p = root.join("xtask/spdx").join(format!("{id}.txt"));
        match fs::read_to_string(&p) {
            Ok(t) => out.push((
                format!("SPDX {id} standard text; the package ships no licence file"),
                t,
            )),
            Err(_) => {
                eprintln!("notices: no standard text for SPDX id '{id}' (add xtask/spdx/{id}.txt)");
                exit(1);
            }
        }
    }
    out
}

fn render(mut pkgs: Vec<Pkg>) -> String {
    pkgs.sort_by(|a, b| {
        (a.ecosystem, &a.name, &a.version).cmp(&(b.ecosystem, &b.name, &b.version))
    });
    // Unique licence texts, numbered in first-use order.
    let mut text_ids: BTreeMap<String, usize> = BTreeMap::new();
    let mut texts: Vec<(String, Vec<String>)> = Vec::new(); // (text, users)
    let mut rows = String::new();
    let mut missing = Vec::new();
    for p in &pkgs {
        let mut refs = Vec::new();
        for (file, text) in &p.texts {
            let id = match text_ids.get(text) {
                Some(&i) => i,
                None => {
                    texts.push((text.clone(), Vec::new()));
                    text_ids.insert(text.clone(), texts.len());
                    texts.len()
                }
            };
            if let Some(t) = texts.get_mut(id - 1) {
                t.1.push(format!("{} {} ({file})", p.name, p.version));
            }
            refs.push(format!("[T{id}](#t{id})"));
        }
        if p.texts.is_empty() {
            missing.push(format!("{} {}", p.name, p.version));
        }
        let lic = if p.license.is_empty() {
            "unspecified"
        } else {
            &p.license
        };
        rows.push_str(&format!(
            "| {} | {} | {} | `{}` | {} | {} |\n",
            p.ecosystem,
            p.name,
            p.version,
            lic,
            if p.repository.is_empty() {
                "—".to_string()
            } else {
                p.repository.clone()
            },
            if refs.is_empty() {
                "SPDX text by identifier".to_string()
            } else {
                refs.join(" ")
            }
        ));
    }
    let mut out = String::new();
    out.push_str("# THIRD_PARTY_NOTICES — LocalPDF Studio (Linux build)\n\n");
    out.push_str("This file covers the **Linux x86_64 build**. Windows, macOS and Android builds link a different dependency set and need their own notices before they ship.\n\n");
    out.push_str("Generated by `cargo xtask notices` (do not edit by hand; `cargo xtask notices --check` verifies it).\n");
    out.push_str(&format!(
        "Rust: shipped (normal) dependencies of the workspace for `{TARGET}`, from `Cargo.lock`. npm: every package in `app-tauri`'s lockfile \
         (build and runtime; a superset of what the bundle contains). Repository-level `NOTICE` files of git and vendored sources \
         (for example storytold/cadcraft, Apache-2.0 §4(d)) are included with their crates.\n\n"
    ));
    out.push_str(&format!(
        "{} packages, {} distinct licence/notice texts.\n\n",
        pkgs.len(),
        texts.len()
    ));
    out.push_str("## Packages\n\n| Kind | Name | Version | Licence | Source | Texts |\n|---|---|---|---|---|---|\n");
    out.push_str(&rows);
    if !missing.is_empty() {
        out.push_str(&format!(
            "\n{} packages ship no licence file in their published source; their licence is the SPDX identifier above: {}.\n",
            missing.len(),
            missing.join(", ")
        ));
    }
    out.push_str("\n## Licence and notice texts\n");
    for (i, (text, users)) in texts.iter().enumerate() {
        let n = i + 1;
        out.push_str(&format!(
            "\n### T{n}\n<a id=\"t{n}\"></a>\nUsed by: {}\n\n```text\n{}\n```\n",
            users.join("; "),
            text.trim_end()
        ));
    }
    out
}

pub fn run(root: &Path, check: bool) {
    let mut pkgs = rust_packages(root);
    pkgs.extend(npm_packages(root));
    for p in &mut pkgs {
        if p.texts.is_empty() && !p.license.is_empty() {
            p.texts = spdx_texts(root, &p.license);
        }
    }
    let rendered = render(pkgs);
    let path = root.join(OUT);
    if check {
        let current = fs::read_to_string(&path).unwrap_or_default();
        if current != rendered {
            eprintln!(
                "notices: {} is out of date; run `cargo xtask notices`",
                path.display()
            );
            exit(1);
        }
        println!("notices: {} is up to date.", OUT);
    } else if let Err(e) = fs::write(&path, rendered) {
        eprintln!("notices: cannot write {}: {e}", path.display());
        exit(1);
    } else {
        println!("notices: wrote {}", path.display());
    }
}
