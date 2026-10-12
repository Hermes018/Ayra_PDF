//! Soak testing and resource-leak verification engine.
//! Strictly obeys `#![forbid(unsafe_code)]` and zero unwraps/panics in non-test code.

#![forbid(unsafe_code)]

use crate::hostile::generate_broken_xref_pdf;
use crate::perf::sample_memory;
use crate::EngineSwitcher;
use lpdf_core_types::{LpdfError, Result};
use lpdf_engine_port::{OptimizeProfile, PdfEngine, SummaryLength, WatermarkSettings};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static SOAK_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_soak_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let count = SOAK_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("lpdf_soak_{:x}_{:x}", nanos, count))
}

/// Pure-Rust open file-descriptor counter reading `/proc/self/fd` on Linux,
/// with safe cross-platform fallback.
pub fn count_open_fds() -> usize {
    if let Ok(entries) = fs::read_dir("/proc/self/fd") {
        entries.flatten().count()
    } else {
        // Safe cross-platform fallback baseline
        10
    }
}

/// Configuration parameters for soak testing runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoakConfig {
    pub target_duration_secs: u64,
    pub min_iterations: usize,
    pub max_rss_drift_mb: f64,
    pub max_fd_delta: usize,
}

impl Default for SoakConfig {
    fn default() -> Self {
        Self {
            target_duration_secs: 0,
            min_iterations: 150,
            max_rss_drift_mb: 30.0,
            max_fd_delta: 5,
        }
    }
}

/// Report containing measured soak execution and leak statistics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoakReport {
    pub iterations_completed: usize,
    pub operations_executed: usize,
    pub elapsed_ms: u64,
    pub initial_rss_mb: f64,
    pub final_rss_mb: f64,
    pub peak_rss_mb: f64,
    pub rss_drift_mb: f64,
    pub initial_open_fds: usize,
    pub final_open_fds: usize,
    pub orphaned_temp_files: usize,
    pub passed: bool,
}

impl SoakReport {
    /// Evaluates whether the run adhered to configured leak constraints.
    pub fn evaluate_leak_limits(&self, config: &SoakConfig) -> bool {
        let fd_delta = self.final_open_fds.saturating_sub(self.initial_open_fds);
        self.rss_drift_mb <= config.max_rss_drift_mb
            && fd_delta <= config.max_fd_delta
            && self.orphaned_temp_files == 0
    }

    /// Returns human-readable validation violations if any limits are breached.
    pub fn violations(&self, config: &SoakConfig) -> Vec<String> {
        let mut violations = Vec::new();
        if self.rss_drift_mb > config.max_rss_drift_mb {
            violations.push(format!(
                "RSS drift {:.2} MB exceeds allowed ceiling {:.2} MB",
                self.rss_drift_mb, config.max_rss_drift_mb
            ));
        }
        let fd_delta = self.final_open_fds.saturating_sub(self.initial_open_fds);
        if fd_delta > config.max_fd_delta {
            violations.push(format!(
                "Open FD delta {} exceeds allowed ceiling {}",
                fd_delta, config.max_fd_delta
            ));
        }
        if self.orphaned_temp_files > 0 {
            violations.push(format!(
                "Detected {} orphaned temporary files",
                self.orphaned_temp_files
            ));
        }
        violations
    }
}

/// Executes a single mixed-workload cycle in an isolated directory.
/// Returns (number of operations executed, number of orphaned temp files).
fn run_soak_cycle(sample_pdf_path: &Path, cycle_dir: &Path) -> Result<(usize, usize)> {
    fs::create_dir_all(cycle_dir).map_err(LpdfError::Io)?;
    let mut ops = 0usize;

    // 1. info
    let mut switcher = EngineSwitcher::new();
    let _ = switcher.load(sample_pdf_path);
    let _ = switcher.info();
    ops += 1;

    // 2. rotate_document
    let rotated_out = cycle_dir.join("rotated.pdf");
    let _ = switcher.rotate_document(sample_pdf_path, 90, &rotated_out);
    ops += 1;

    // 3. compress
    let compressed_out = cycle_dir.join("compressed.pdf");
    let _ = switcher.compress(sample_pdf_path, OptimizeProfile::Medium, &compressed_out);
    ops += 1;

    // 4. repair
    let repaired_out = cycle_dir.join("repaired.pdf");
    let _ = switcher.repair(sample_pdf_path, &repaired_out);
    ops += 1;

    // 5. watermark
    let wm_out = cycle_dir.join("watermarked.pdf");
    let wm_settings = WatermarkSettings {
        text: "SOAK HARNESS".to_string(),
        opacity: 0.5,
        rotation: 45.0,
        color: [0.0, 0.0, 0.0],
    };
    let _ = switcher.add_watermark(sample_pdf_path, &wm_settings, &wm_out);
    ops += 1;

    // 6. flatten_comments
    let flattened_out = cycle_dir.join("flattened.pdf");
    let _ = switcher.flatten_comments(sample_pdf_path, &flattened_out);
    ops += 1;

    // 7. pdf_to_text
    let text_out = cycle_dir.join("extracted.txt");
    let _ = switcher.pdf_to_text(sample_pdf_path, &text_out);
    ops += 1;

    // 8. summarize_extractive
    let summary_out = cycle_dir.join("summary.md");
    let _ = switcher.summarize_extractive(sample_pdf_path, &summary_out, SummaryLength::Short);
    ops += 1;

    // 9. redact
    let redact_out = cycle_dir.join("redacted.pdf");
    let _ = switcher.redact(sample_pdf_path, vec![], &redact_out);
    ops += 1;

    // 10. hostile-input rejection
    let hostile_bytes = generate_broken_xref_pdf();
    let hostile_input = cycle_dir.join("hostile_xref.pdf");
    let _ = fs::write(&hostile_input, &hostile_bytes);
    let mut hostile_switcher = EngineSwitcher::new();
    let _ = hostile_switcher.load(&hostile_input);
    let hostile_out = cycle_dir.join("hostile_out.pdf");
    let _ = hostile_switcher.repair(&hostile_input, &hostile_out);
    ops += 1;

    // Cleanup expected products
    for file in [
        &rotated_out,
        &compressed_out,
        &repaired_out,
        &wm_out,
        &flattened_out,
        &text_out,
        &summary_out,
        &redact_out,
        &hostile_input,
        &hostile_out,
    ] {
        let _ = fs::remove_file(file);
    }

    // Check for any leftover/orphaned files
    let mut orphaned = 0usize;
    if let Ok(entries) = fs::read_dir(cycle_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                orphaned += 1;
                let _ = fs::remove_file(&path);
            }
        }
    }

    let _ = fs::remove_dir(cycle_dir);
    Ok((ops, orphaned))
}

/// Runs the continuous soak test workload verifying memory, FD, and temp file stability.
pub fn run_soak_suite(config: &SoakConfig, sample_pdf: &[u8]) -> Result<SoakReport> {
    if sample_pdf.is_empty() {
        return Err(LpdfError::InvalidInput(
            "Sample PDF cannot be empty for soak test".to_string(),
        ));
    }

    let base_dir = unique_soak_temp_dir();
    fs::create_dir_all(&base_dir).map_err(LpdfError::Io)?;
    let sample_pdf_path = base_dir.join("sample.pdf");
    fs::write(&sample_pdf_path, sample_pdf).map_err(LpdfError::Io)?;

    // 1. Warmup pass (3 cycles) to prime allocators, caches, and runtime state
    for i in 0..3 {
        let warmup_dir = base_dir.join(format!("warmup_{}", i));
        let _ = run_soak_cycle(&sample_pdf_path, &warmup_dir)?;
    }

    // 2. Record initial baselines
    let initial_mem = sample_memory();
    let initial_rss_mb = (initial_mem.rss_kb as f64) / 1024.0;
    let initial_open_fds = count_open_fds();
    let mut peak_rss_mb = initial_rss_mb;
    let mut operations_executed = 0usize;
    let mut iterations_completed = 0usize;
    let mut orphaned_temp_files = 0usize;

    let start_time = Instant::now();

    // 3. Soak loop: continue until BOTH min_iterations and target_duration_secs are met
    while iterations_completed < config.min_iterations
        || start_time.elapsed().as_secs() < config.target_duration_secs
    {
        let cycle_dir = base_dir.join(format!("cycle_{}", iterations_completed));
        let (ops, orphaned) = run_soak_cycle(&sample_pdf_path, &cycle_dir)?;
        operations_executed += ops;
        orphaned_temp_files += orphaned;

        let cur_mem = sample_memory();
        let cur_rss_mb = (cur_mem.rss_kb as f64) / 1024.0;
        if cur_rss_mb > peak_rss_mb {
            peak_rss_mb = cur_rss_mb;
        }

        iterations_completed += 1;
    }

    let _ = fs::remove_dir_all(&base_dir);

    // 4. Final measurements and report generation
    let final_mem = sample_memory();
    let final_rss_mb = (final_mem.rss_kb as f64) / 1024.0;
    let final_open_fds = count_open_fds();
    let rss_drift_mb = (final_rss_mb - initial_rss_mb).max(0.0);
    let elapsed_ms = start_time.elapsed().as_millis() as u64;

    let mut report = SoakReport {
        iterations_completed,
        operations_executed,
        elapsed_ms,
        initial_rss_mb,
        final_rss_mb,
        peak_rss_mb,
        rss_drift_mb,
        initial_open_fds,
        final_open_fds,
        orphaned_temp_files,
        passed: false,
    };
    report.passed = report.evaluate_leak_limits(config);

    Ok(report)
}
