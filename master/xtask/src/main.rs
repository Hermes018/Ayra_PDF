use std::env;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};

mod notices;
mod ui_audit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xtask <command>");
        exit(1);
    }

    match args[1].as_str() {
        "protocol-lint" => run_protocol_lint(),
        "feature-map-check" => run_feature_map_check(),
        "studio-audit" => run_studio_audit(&args[2..]),
        "gate" => run_gate(),
        "test-offline" => run_test_offline(),
        "fuzz" | "hostile" => run_fuzz(&args[2..]),
        "perf-check" => run_perf_check(),
        "sbom" => run_sbom(),
        "a11y" => run_a11y(),
        "soak" => run_soak(&args[2..]),
        "notices" => notices::run(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap_or(Path::new(".")),
            args.get(2).map(|s| s.as_str()) == Some("--check"),
        ),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            exit(1);
        }
    }
}

fn run_protocol_lint() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut failed = false;

    // Scan for banned code patterns in src/**/*.rs
    let banned_patterns = vec![
        ".unwrap(",
        ".expect(",
        "todo!(",
        "unimplemented!(",
        "panic!(",
    ];
    let crates_dir = workspace_root.join("crates");

    if crates_dir.exists() {
        for entry in walkdir::WalkDir::new(&crates_dir) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Ignore test files or test modules if we could, but let's just do a simple scan
                let content = fs::read_to_string(path).unwrap();
                let mut in_test_cfg = false;
                for (i, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed == "#[cfg(test)]" {
                        in_test_cfg = true;
                    }
                    if trimmed == "}" && in_test_cfg {
                        // Very naive, but for now we'll just scan all lines not starting with //
                    }

                    if !trimmed.starts_with("//") {
                        for pattern in &banned_patterns {
                            if trimmed.contains(pattern) {
                                // Exclude test files entirely for now?
                                if path.to_string_lossy().contains("tests/") {
                                    continue;
                                }
                                // Naive check: if #[test] is nearby? Actually, just forbid in all non-test crates
                                eprintln!(
                                    "protocol-lint error: banned pattern '{}' found in {}:{}",
                                    pattern,
                                    path.display(),
                                    i + 1
                                );
                                failed = true;
                            }
                        }
                    }
                }
            }
        }
    }

    // Scan Cargo.toml for banned dependencies
    let banned_crates = vec!["reqwest", "hyper", "ureq", "isahc", "surf"];
    let mut cargo_tomls = vec![workspace_root.join("Cargo.toml")];
    if crates_dir.exists() {
        for entry in walkdir::WalkDir::new(&crates_dir) {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap() == "Cargo.toml" {
                cargo_tomls.push(path.to_path_buf());
            }
        }
    }

    for toml_path in cargo_tomls {
        let content = fs::read_to_string(&toml_path).unwrap();
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('#') {
                for banned in &banned_crates {
                    // Check if it's a dependency line like `reqwest = ...`
                    if trimmed.starts_with(&format!("{} ", banned))
                        || trimmed.starts_with(&format!("{}=", banned))
                    {
                        eprintln!(
                            "protocol-lint error: banned dependency '{}' found in {}",
                            banned,
                            toml_path.display()
                        );
                        failed = true;
                    }
                }
            }
        }
    }

    if failed {
        eprintln!("protocol-lint failed!");
        exit(1);
    } else {
        println!("protocol-lint passed.");
    }
}

fn run_gate() {
    let steps = vec![
        vec!["cargo", "fmt", "--", "--check"],
        vec!["cargo", "clippy", "--workspace", "--", "-D", "warnings"],
        vec!["cargo", "nextest", "run"],
        vec!["cargo", "deny", "check"],
        vec!["cargo", "audit"],
        vec!["cargo", "xtask", "protocol-lint"],
    ];

    for step in steps {
        println!("Running: {:?}", step);
        let status = Command::new(step[0])
            .args(&step[1..])
            .status()
            .expect("Failed to execute command");

        if !status.success() {
            eprintln!("Gate failed at step: {:?}", step);
            exit(1);
        }
    }

    run_android_core_check();

    println!("Gate passed green!");
}

/// Core crates that must keep compiling for Android (ADR-020 / ADR-019).
const ANDROID_CORE_CRATES: &[&str] = &[
    "lpdf-core-types",
    "lpdf-engine-port",
    "lpdf-fs-safety",
    "lpdf-hw",
    "lpdf-control-bus",
];
const ANDROID_TARGET: &str = "aarch64-linux-android";

/// Gate step: `cargo check --target aarch64-linux-android` on the core crates, once the Rust
/// target is installed. Until then it reports itself as skipped (never silently passes).
fn run_android_core_check() {
    let installed = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.trim() == ANDROID_TARGET)
        })
        .unwrap_or(false);
    if !installed {
        println!(
            "Android core check: SKIPPED ({} target not installed; see ADR-019)",
            ANDROID_TARGET
        );
        return;
    }
    let mut cmd = Command::new("cargo");
    cmd.args(["check", "--target", ANDROID_TARGET]);
    for c in ANDROID_CORE_CRATES {
        cmd.args(["-p", c]);
    }
    println!(
        "Running: cargo check --target {} -p {}",
        ANDROID_TARGET,
        ANDROID_CORE_CRATES.join(" -p ")
    );
    let status = cmd
        .status()
        .expect("Failed to execute cargo check for Android");
    if !status.success() {
        eprintln!(
            "Gate failed at step: Android core check ({})",
            ANDROID_TARGET
        );
        exit(1);
    }
}

fn run_test_offline() {
    let workspace_root = env::current_dir().unwrap();
    let bin_path = workspace_root
        .join("target")
        .join("debug")
        .join("app-tauri");
    if !bin_path.exists() {
        eprintln!(
            "Binary not found at {}. Please run `cargo build -p app-tauri` first.",
            bin_path.display()
        );
        exit(1);
    }

    println!("Testing offline launch via unshare...");
    let status = Command::new("timeout")
        .args(["5s", "unshare", "-n", "-r", bin_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute unshare command");

    // timeout exits with 124 if the command times out, which is success for us since it means the app stayed alive
    if status.code() == Some(124) || status.success() {
        println!("Offline test passed (app launched and stayed alive without network).");
    } else {
        eprintln!(
            "Offline test failed: app crashed or failed to launch. Exit code: {:?}",
            status.code()
        );
        exit(1);
    }
}

fn run_fuzz(_extra_args: &[String]) {
    println!("Running deterministic adapter input fuzz & hostile-file tests...");
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "lpdf-engine-switcher",
            "--test",
            "fuzz_smoke_tests",
            "--",
            "--test-threads=1",
        ])
        .status()
        .expect("Failed to execute fuzz & hostile tests");

    if !status.success() {
        eprintln!("Fuzz & hostile tests failed!");
        exit(1);
    }
    println!("Fuzz & hostile tests passed green!");
}

fn run_perf_check() {
    println!("Running perf & memory profiling regression check under tier budgets...");
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "lpdf-engine-switcher",
            "--test",
            "fuzz_smoke_tests",
            "--",
            "test_tier_perf_suite",
            "test_perf_regression",
            "test_load_baseline",
            "--test-threads=1",
        ])
        .status()
        .expect("Failed to execute perf check tests");

    if !status.success() {
        eprintln!("Perf & memory profiling regression check failed!");
        exit(1);
    }
    println!("Perf & memory profiling regression check passed green!");
}

fn run_sbom() {
    println!("Generating CycloneDX 1.5 SBOM and verifying supply chain invariants...");
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "lpdf-engine-switcher",
            "--test",
            "fuzz_smoke_tests",
            "--",
            "test_cyclonedx_sbom_generation",
            "test_supply_chain_forbid_unsafe_revet",
            "test_threat_model_matrix_coverage",
            "--test-threads=1",
        ])
        .status()
        .expect("Failed to execute SBOM generation tests");

    if !status.success() {
        eprintln!("SBOM generation or supply chain verification failed!");
        exit(1);
    }
    println!("CycloneDX SBOM generated at logs/sbom.cdx.json and supply chain verified green!");
}

fn run_a11y() {
    println!("Running accessibility (a11y) protocol audit and Svelte landmark verification...");
    let status = Command::new("cargo")
        .args([
            "test",
            "-p",
            "lpdf-engine-switcher",
            "--test",
            "fuzz_smoke_tests",
            "--",
            "test_a11y_protocol_audit_and_svelte_landmarks",
            "--test-threads=1",
        ])
        .status()
        .expect("Failed to execute a11y tests");

    if !status.success() {
        eprintln!("A11y verification failed!");
        exit(1);
    }
    println!("A11y protocol audit and Svelte landmark verification passed green!");
}

fn run_soak(args: &[String]) {
    let mut duration_secs = 0u64;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--duration-secs" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<u64>() {
                duration_secs = val;
            }
            i += 2;
        } else {
            i += 1;
        }
    }

    println!(
        "Running soak testing harness (target duration: {}s, minimum 150 cycles)...",
        duration_secs
    );
    let mut cmd = Command::new("cargo");
    cmd.args([
        "test",
        "-p",
        "lpdf-engine-switcher",
        "--test",
        "fuzz_smoke_tests",
        "--",
        "test_soak_suite_zero_memory_fd_and_temp_leaks",
        "test_soak_leak_detector_catches_violations",
        "--test-threads=1",
    ]);
    if duration_secs > 0 {
        cmd.env("LPDF_SOAK_DURATION_SECS", duration_secs.to_string());
    }

    let status = cmd.status().expect("Failed to execute soak test suite");
    if !status.success() {
        eprintln!("Soak test suite failed!");
        exit(1);
    }
    println!("Soak test suite completed green with zero leaks!");
}

/// Allowed values in a checklist item's `[platforms: ...]` tag (ADR-020).
const PLATFORMS: &[&str] = &["linux", "windows", "macos", "android", "ios", "none"];

/// Parses the `` `[platforms: a, b]` `` tag of a checklist line into its platform names.
fn parse_platforms(line: &str) -> Result<Vec<String>, String> {
    let start = line
        .find("`[platforms:")
        .ok_or_else(|| "missing `[platforms: ...]` declaration (ADR-020)".to_string())?;
    let rest = &line[start + "`[platforms:".len()..];
    let end = rest
        .find("]`")
        .ok_or_else(|| "unterminated `[platforms: ...]` tag".to_string())?;
    let list: Vec<String> = rest[..end]
        .split(',')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    if list.is_empty() {
        return Err("empty `[platforms: ...]` tag".to_string());
    }
    for p in &list {
        if !PLATFORMS.contains(&p.as_str()) {
            return Err(format!(
                "unknown platform `{}` (allowed: {})",
                p,
                PLATFORMS.join(", ")
            ));
        }
    }
    if list.len() > 1 && list.iter().any(|p| p == "none") {
        return Err("`none` cannot be combined with other platforms".to_string());
    }
    Ok(list)
}

fn run_feature_map_check() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let checklist_path = workspace_root.join("docs/DRAWBOARD_FEATURE_CHECKLIST.md");
    let feature_map_path = workspace_root.join("protocol/07_FEATURE_MAP.md");

    if !checklist_path.exists() {
        eprintln!(
            "feature-map-check error: missing {}",
            checklist_path.display()
        );
        exit(1);
    }
    if !feature_map_path.exists() {
        eprintln!(
            "feature-map-check error: missing {}",
            feature_map_path.display()
        );
        exit(1);
    }

    let content =
        fs::read_to_string(&checklist_path).expect("Failed to read DRAWBOARD_FEATURE_CHECKLIST.md");
    let mut done_count = 0usize;
    let mut in_progress_count = 0usize;
    let mut partial_count = 0usize;
    let mut not_started_count = 0usize;
    let mut deferred_count = 0usize;
    let mut errors = Vec::new();
    let mut platform_counts: Vec<(&str, usize)> = PLATFORMS.iter().map(|p| (*p, 0usize)).collect();

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let is_checked = trimmed.starts_with("- [x] **");
        let is_unchecked = trimmed.starts_with("- [ ] **");
        if !is_checked && !is_unchecked {
            continue;
        }

        // ADR-020: every feature declares its target platforms, e.g. `[platforms: linux, windows]`.
        match parse_platforms(trimmed) {
            Err(msg) => errors.push(format!("Line {}: {}: {}", line_idx + 1, msg, trimmed)),
            Ok(list) => {
                let is_deferred = trimmed.contains("`DEFERRED");
                if list == ["none"] && !is_deferred {
                    errors.push(format!(
                        "Line {}: only DEFERRED items may declare `[platforms: none]`: {}",
                        line_idx + 1,
                        trimmed
                    ));
                }
                for p in &list {
                    if let Some(entry) = platform_counts.iter_mut().find(|(n, _)| n == p) {
                        entry.1 += 1;
                    }
                }
            }
        }

        if trimmed.contains("`DONE`") {
            done_count += 1;
            if !is_checked {
                errors.push(format!(
                    "Line {}: marked `DONE` but checkbox is `[ ]`: {}",
                    line_idx + 1,
                    trimmed
                ));
            }
            if !trimmed.contains(".rs") && !trimmed.contains(".svelte") && !trimmed.contains(".ts")
            {
                errors.push(format!(
                    "Line {}: marked `DONE` without a verifying file reference (.rs/.svelte/.ts): {}",
                    line_idx + 1,
                    trimmed
                ));
            }
        } else if trimmed.contains("`PARTIAL`") {
            // Something real ships, but it does not meet the feature as specified. The line must
            // point at the code and say what the code actually does ("Actually: ...").
            partial_count += 1;
            if is_checked {
                errors.push(format!(
                    "Line {}: marked `PARTIAL` but checkbox is `[x]`: {}",
                    line_idx + 1,
                    trimmed
                ));
            }
            if !trimmed.contains("Actually:") {
                errors.push(format!(
                    "Line {}: marked `PARTIAL` without an \"Actually:\" note describing what the code does: {}",
                    line_idx + 1,
                    trimmed
                ));
            }
        } else if trimmed.contains("`IN PROGRESS`") {
            in_progress_count += 1;
            if is_checked {
                errors.push(format!(
                    "Line {}: marked `IN PROGRESS` but checkbox is `[x]`: {}",
                    line_idx + 1,
                    trimmed
                ));
            }
        } else if trimmed.contains("`NOT STARTED`") {
            not_started_count += 1;
            if is_checked {
                errors.push(format!(
                    "Line {}: marked `NOT STARTED` but checkbox is `[x]`: {}",
                    line_idx + 1,
                    trimmed
                ));
            }
        } else if trimmed.contains("`DEFERRED") {
            deferred_count += 1;
            if is_checked {
                errors.push(format!(
                    "Line {}: marked `DEFERRED` but checkbox is `[x]`: {}",
                    line_idx + 1,
                    trimmed
                ));
            }
        } else {
            errors.push(format!(
                "Line {}: feature item missing canonical status tag (`DONE`, `PARTIAL`, `IN PROGRESS`, `NOT STARTED`, `DEFERRED`): {}",
                line_idx + 1,
                trimmed
            ));
        }
    }

    let total = done_count + partial_count + in_progress_count + not_started_count + deferred_count;
    println!(
        "feature-map-check: {} features audited (DONE: {}, PARTIAL: {}, IN PROGRESS: {}, NOT STARTED: {}, DEFERRED: {})",
        total, done_count, partial_count, in_progress_count, not_started_count, deferred_count
    );

    let summary: Vec<String> = platform_counts
        .iter()
        .filter(|(n, _)| *n != "none")
        .map(|(n, c)| format!("{}: {}", n, c))
        .collect();
    println!(
        "feature-map-check: platform declarations ({})",
        summary.join(", ")
    );

    if total < 109 {
        errors.push(format!(
            "Expected at least 109 spec features in DRAWBOARD_FEATURE_CHECKLIST.md, found {}",
            total
        ));
    }

    if !errors.is_empty() {
        for err in &errors {
            eprintln!("feature-map-check error: {}", err);
        }
        exit(1);
    }
    println!("feature-map-check passed! All {} features cross-checked with valid status tags and evidence.", total);
}

/// `studio-audit [--strict [<glob-list-file>]] [--list] [--write-inventory]`
///
/// - Studio checks (keydown / placeholder / deliverables): unchanged; `--strict` fails on any.
/// - UI-consistency scan of every `.svelte` (ui_audit.rs): report-only by default (totals per rule
///   and per file); `--list` prints every `file:line rule message`; `--strict <file>` additionally
///   fails on any violation in the files matched by the glob list; `--write-inventory` writes
///   `docs/UI_CONSISTENCY_INVENTORY.md`.
fn run_studio_audit(args: &[String]) {
    let strict = args.iter().any(|a| a == "--strict");
    let default_strict = "xtask/ui-strict.txt".to_string();
    let strict_file = if strict {
        Some(
            args.iter()
                .position(|a| a == "--strict")
                .and_then(|i| args.get(i + 1))
                .filter(|a| !a.starts_with("--"))
                .unwrap_or(&default_strict),
        )
    } else {
        None
    };
    let list = args.iter().any(|a| a == "--list");
    let write_inventory = args.iter().any(|a| a == "--write-inventory");
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut warnings = Vec::new();

    // 1. Check ADR-012 no-local-keydown rule outside commandEngine.svelte.ts
    let frontend_src = workspace_root.join("app-tauri/src/lib");
    if frontend_src.exists() {
        for entry in walkdir::WalkDir::new(&frontend_src) {
            let entry = entry.unwrap();
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if file_name.ends_with(".test.ts") {
                continue;
            }
            let rel = path
                .strip_prefix(workspace_root)
                .unwrap_or(path)
                .to_string_lossy();
            let is_studio_scope = rel.contains("views/studio/")
                || rel.contains("components/studio/")
                || rel.contains("lib/core/");
            let content = fs::read_to_string(path).unwrap_or_default();
            for (idx, line) in content.lines().enumerate() {
                if is_studio_scope
                    && file_name != "commandEngine.svelte.ts"
                    && (line.contains("addEventListener('keydown'")
                        || line.contains("addEventListener(\"keydown\"")
                        || line.contains("onkeydown="))
                {
                    warnings.push(format!(
                        "[no-local-keydown] {}:{} attaches keydown outside commandEngine.svelte.ts",
                        rel,
                        idx + 1
                    ));
                }
                if (line.contains("Spatial Studio PDF Canvas")
                    || line.contains("placeholder-preview")
                    || line.contains("canvas-placeholder"))
                    && (file_name.ends_with(".svelte"))
                {
                    warnings.push(format!(
                        "[ui-placeholder] {}:{} contains static placeholder markup instead of live PDF canvas",
                        rel,
                        idx + 1
                    ));
                }
            }
        }
    }

    // 2. Check Opus 5.5 Phase D1/D2 required crates and IPC files
    let required_paths = [
        (
            "crates/lpdf-control-bus/Cargo.toml",
            "Opus 5.5 Phase D1 control-bus crate",
        ),
        (
            "app-tauri/src-tauri/src/commands/studio.rs",
            "Opus 5.5 Phase D1 prefs_save/prefs_load IPC module",
        ),
        (
            "app-tauri/src/lib/components/studio/FavoritesBar.svelte",
            "Opus 5.5 Phase D2 FavoritesBar component",
        ),
    ];
    for (rel_path, desc) in &required_paths {
        if !workspace_root.join(rel_path).exists() {
            warnings.push(format!(
                "[missing-deliverable] {} ({}) not yet created",
                rel_path, desc
            ));
        }
    }

    println!(
        "studio-audit: found {} compliance findings:",
        warnings.len()
    );
    for w in &warnings {
        println!("  - {}", w);
    }

    let ui_failed = run_ui_audit(workspace_root, strict_file, list, write_inventory);

    if strict && !warnings.is_empty() {
        eprintln!(
            "studio-audit failed in --strict mode with {} findings.",
            warnings.len()
        );
        exit(1);
    }
    if ui_failed {
        exit(1);
    }
}

/// UI-consistency part of studio-audit. Returns true when `--strict <file>` found violations in
/// swept files.
fn run_ui_audit(
    workspace_root: &Path,
    strict_file: Option<&String>,
    list: bool,
    write_inventory: bool,
) -> bool {
    use std::collections::BTreeMap;
    use ui_audit::Rule;

    let patterns = match strict_file {
        Some(f) => {
            let path = Path::new(f);
            let path = if path.is_absolute() || path.exists() {
                path.to_path_buf()
            } else {
                workspace_root.join(f)
            };
            match fs::read_to_string(&path) {
                Ok(c) => ui_audit::parse_patterns(&c),
                Err(e) => {
                    eprintln!(
                        "studio-audit: cannot read strict list {}: {}",
                        path.display(),
                        e
                    );
                    exit(2);
                }
            }
        }
        None => Vec::new(),
    };

    let scan = ui_audit::scan_tree(workspace_root);
    let mut per_file: BTreeMap<&str, usize> = BTreeMap::new();
    for v in &scan.violations {
        *per_file.entry(v.file.as_str()).or_insert(0) += 1;
    }
    println!(
        "ui-audit: {} violations in {} of {} .svelte files (report-only unless --strict <list>)",
        scan.violations.len(),
        per_file.len(),
        scan.files.len()
    );
    println!("  per rule:");
    for r in Rule::ALL {
        let n = scan.violations.iter().filter(|v| v.rule == r).count();
        println!("    {:<18} {}", r.id(), n);
    }
    let mut ranked: Vec<(&&str, &usize)> = per_file.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("  per file:");
    for (f, n) in &ranked {
        println!("    {:>5}  {}", n, f);
    }
    if list {
        for v in &scan.violations {
            println!("{}", v.render());
        }
    }

    if write_inventory {
        let md = ui_audit::render_inventory_with(
            &scan.violations,
            &scan.files,
            &ui_audit::inventory_notes(&scan),
        );
        let out = workspace_root.join("docs/UI_CONSISTENCY_INVENTORY.md");
        match fs::write(&out, md) {
            Ok(()) => println!("ui-audit: wrote {}", out.display()),
            Err(e) => {
                eprintln!("ui-audit: cannot write {}: {}", out.display(), e);
                exit(2);
            }
        }
    }

    if strict_file.is_some() {
        let swept: Vec<&ui_audit::Violation> = scan
            .violations
            .iter()
            .filter(|v| ui_audit::is_swept(&patterns, &v.file))
            .collect();
        println!(
            "ui-audit strict: {} swept pattern(s), {} violation(s) in swept files",
            patterns.len(),
            swept.len()
        );
        if !swept.is_empty() {
            for v in &swept {
                eprintln!("{}", v.render());
            }
            eprintln!(
                "studio-audit failed: {} violation(s) in swept files.",
                swept.len()
            );
            return true;
        }
    }
    false
}
