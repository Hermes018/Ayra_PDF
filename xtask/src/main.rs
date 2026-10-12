use std::env;
use std::fs;
use std::path::Path;
use std::process::{exit, Command};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: xtask <command>");
        exit(1);
    }

    match args[1].as_str() {
        "protocol-lint" => run_protocol_lint(),
        "gate" => run_gate(),
        "test-offline" => run_test_offline(),
        "fuzz" | "hostile" => run_fuzz(&args[2..]),
        "perf-check" => run_perf_check(),
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

    println!("Gate passed green!");
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
