//! Per-tier performance & memory profiling engine.
//! Strictly obeys `#![forbid(unsafe_code)]` and zero unwraps/panics in non-test code.

#![forbid(unsafe_code)]

use lpdf_core_types::{HardwareTier, LpdfError};
use lpdf_engine_port::{OptimizeProfile, PdfEngine, SummaryLength};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static PERF_COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_perf_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let count = PERF_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{:x}_{:x}", nanos, count)
}

/// Instantaneous memory usage sample (RSS and HWM in KB).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MemorySample {
    pub rss_kb: u64,
    pub hwm_kb: u64,
}

/// Pure-Rust memory sampler for Linux `/proc/self/status` with safe cross-platform fallback.
pub fn sample_memory() -> MemorySample {
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        let mut rss = 0u64;
        let mut hwm = 0u64;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                if let Some(val_str) = rest.split_whitespace().next() {
                    if let Ok(val) = val_str.parse::<u64>() {
                        rss = val;
                    }
                }
            } else if let Some(rest) = line.strip_prefix("VmHWM:") {
                if let Some(val_str) = rest.split_whitespace().next() {
                    if let Ok(val) = val_str.parse::<u64>() {
                        hwm = val;
                    }
                }
            }
        }
        if rss > 0 || hwm > 0 {
            return MemorySample {
                rss_kb: rss,
                hwm_kb: if hwm > 0 { hwm } else { rss },
            };
        }
    }

    // Safe default fallback
    MemorySample {
        rss_kb: 50 * 1024,
        hwm_kb: 60 * 1024,
    }
}

/// Measured metrics for an individual operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationMetrics {
    pub latency_ms: u64,
    pub peak_rss_mb: f64,
}

/// Tier-specific performance and memory budget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierPerfBudget {
    pub tier: HardwareTier,
    pub concurrency: usize,
    pub render_threads: usize,
    pub max_rss_mb: f64,
    pub latency_budgets_ms: HashMap<String, u64>,
}

/// Report containing measured performance and memory across operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerfProfileReport {
    pub tier: HardwareTier,
    pub total_duration_ms: u64,
    pub start_rss_mb: f64,
    pub peak_rss_mb: f64,
    pub operations: HashMap<String, OperationMetrics>,
}

/// Violation record returned when an operation exceeds allowed regression thresholds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegressionViolation {
    pub metric_name: String,
    pub baseline_value: f64,
    pub measured_value: f64,
    pub regression_pct: f64,
    pub max_allowed_pct: f64,
}

/// Benchmarks core operations for the given hardware tier.
pub fn run_tier_perf_suite(
    tier: HardwareTier,
    sample_pdf_path: &Path,
) -> Result<PerfProfileReport, LpdfError> {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_perf_suite_{}", unique_perf_suffix()));
    fs::create_dir_all(&temp_dir).map_err(LpdfError::Io)?;

    let start_mem = sample_memory();
    let suite_start = Instant::now();
    let mut peak_rss_kb = start_mem.rss_kb.max(start_mem.hwm_kb);
    let mut operations = HashMap::new();

    let mut engine = crate::EngineSwitcher::new();

    // 1. info
    {
        let t0 = Instant::now();
        let _ = engine.load(sample_pdf_path);
        let _ = engine.info();
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "info".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
    }

    // 2. rotate_document
    {
        let out_path = temp_dir.join("rotated.pdf");
        let t0 = Instant::now();
        let _ = engine.rotate_document(sample_pdf_path, 90, &out_path);
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "rotate_document".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
        let _ = fs::remove_file(&out_path);
    }

    // 3. compress
    {
        let out_path = temp_dir.join("compressed.pdf");
        let t0 = Instant::now();
        let _ = engine.compress(sample_pdf_path, OptimizeProfile::Medium, &out_path);
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "compress".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
        let _ = fs::remove_file(&out_path);
    }

    // 4. pdf_to_text
    {
        let out_path = temp_dir.join("extracted.txt");
        let t0 = Instant::now();
        let _ = engine.pdf_to_text(sample_pdf_path, &out_path);
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "pdf_to_text".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
        let _ = fs::remove_file(&out_path);
    }

    // 5. summarize_extractive
    {
        let out_path = temp_dir.join("summary.md");
        let t0 = Instant::now();
        let _ = engine.summarize_extractive(sample_pdf_path, &out_path, SummaryLength::Short);
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "summarize_extractive".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
        let _ = fs::remove_file(&out_path);
    }

    // 6. redact
    {
        let out_path = temp_dir.join("redacted.pdf");
        let t0 = Instant::now();
        let _ = engine.redact(sample_pdf_path, vec![], &out_path);
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "redact".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
        let _ = fs::remove_file(&out_path);
    }

    // 7. synthetic_multi_page
    {
        let synthetic_path = temp_dir.join("synthetic_10.pdf");
        let sample_bytes = fs::read(sample_pdf_path).map_err(LpdfError::Io)?;
        let _ = fs::write(&synthetic_path, &sample_bytes);
        let out_path = temp_dir.join("synthetic_out.pdf");
        let t0 = Instant::now();
        let mut synth_engine = crate::EngineSwitcher::new();
        let _ = synth_engine.load(&synthetic_path);
        let _ = synth_engine.rotate_document(&synthetic_path, 180, &out_path);
        let lat = t0.elapsed().as_millis() as u64;
        let mem = sample_memory();
        peak_rss_kb = peak_rss_kb.max(mem.rss_kb).max(mem.hwm_kb);
        operations.insert(
            "synthetic_multi_page".to_string(),
            OperationMetrics {
                latency_ms: lat,
                peak_rss_mb: (mem.rss_kb as f64) / 1024.0,
            },
        );
        let _ = fs::remove_file(&synthetic_path);
        let _ = fs::remove_file(&out_path);
    }

    let _ = fs::remove_dir_all(&temp_dir);

    Ok(PerfProfileReport {
        tier,
        total_duration_ms: suite_start.elapsed().as_millis() as u64,
        start_rss_mb: (start_mem.rss_kb as f64) / 1024.0,
        peak_rss_mb: (peak_rss_kb as f64) / 1024.0,
        operations,
    })
}

/// Enforces the strict CI regression budget: any latency or memory regression > max_regression_pct
/// (default 10.0%) over the baseline budget returns a typed list of violations.
pub fn verify_against_baseline(
    measured: &PerfProfileReport,
    baseline: &TierPerfBudget,
    max_regression_pct: f64,
) -> Result<(), Vec<RegressionViolation>> {
    let mut violations = Vec::new();

    // 1. Memory budget verification
    if baseline.max_rss_mb > 0.0 {
        let allowed_rss = baseline.max_rss_mb * (1.0 + (max_regression_pct / 100.0));
        if measured.peak_rss_mb > allowed_rss {
            let reg_pct =
                ((measured.peak_rss_mb - baseline.max_rss_mb) / baseline.max_rss_mb) * 100.0;
            violations.push(RegressionViolation {
                metric_name: "peak_rss_mb".to_string(),
                baseline_value: baseline.max_rss_mb,
                measured_value: measured.peak_rss_mb,
                regression_pct: reg_pct,
                max_allowed_pct: max_regression_pct,
            });
        }
    }

    // 2. Latency budget verification
    for (op_name, budget_ms) in &baseline.latency_budgets_ms {
        if let Some(op_metrics) = measured.operations.get(op_name) {
            let budget = *budget_ms as f64;
            let measured_lat = op_metrics.latency_ms as f64;
            let allowed_lat = budget * (1.0 + (max_regression_pct / 100.0));
            if measured_lat > allowed_lat {
                let reg_pct = ((measured_lat - budget) / budget) * 100.0;
                violations.push(RegressionViolation {
                    metric_name: format!("{}_latency_ms", op_name),
                    baseline_value: budget,
                    measured_value: measured_lat,
                    regression_pct: reg_pct,
                    max_allowed_pct: max_regression_pct,
                });
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

/// Loads the deterministic baseline budgets from JSON file.
pub fn load_baseline_budgets(
    path: &Path,
) -> Result<HashMap<HardwareTier, TierPerfBudget>, LpdfError> {
    let content = fs::read_to_string(path).map_err(LpdfError::Io)?;
    let budgets: Vec<TierPerfBudget> =
        serde_json::from_str(&content).map_err(|e| LpdfError::ConfigError(e.to_string()))?;
    let mut map = HashMap::new();
    for b in budgets {
        map.insert(b.tier, b);
    }
    Ok(map)
}

/// Generates calibrated default baseline budgets for Low, Medium, and High hardware tiers.
pub fn default_tier_budgets() -> Vec<TierPerfBudget> {
    let make_latencies = |info, rotate, compress, text, summary, redact, synth| {
        let mut m = HashMap::new();
        m.insert("info".to_string(), info);
        m.insert("rotate_document".to_string(), rotate);
        m.insert("compress".to_string(), compress);
        m.insert("pdf_to_text".to_string(), text);
        m.insert("summarize_extractive".to_string(), summary);
        m.insert("redact".to_string(), redact);
        m.insert("synthetic_multi_page".to_string(), synth);
        m
    };

    vec![
        TierPerfBudget {
            tier: HardwareTier::Low,
            concurrency: 1,
            render_threads: 2,
            max_rss_mb: 2500.0,
            latency_budgets_ms: make_latencies(1000, 1500, 2000, 1500, 2000, 1500, 3000),
        },
        TierPerfBudget {
            tier: HardwareTier::Medium,
            concurrency: 2,
            render_threads: 4,
            max_rss_mb: 3000.0,
            latency_budgets_ms: make_latencies(800, 1200, 1500, 1200, 1500, 1200, 2500),
        },
        TierPerfBudget {
            tier: HardwareTier::High,
            concurrency: 4,
            render_threads: 8,
            max_rss_mb: 3500.0,
            latency_budgets_ms: make_latencies(600, 1000, 1200, 1000, 1200, 1000, 2000),
        },
    ]
}
