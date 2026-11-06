//! Async job queue for long-running PDF operations.
//!
//! Design rules:
//! - The Tauri main thread is NEVER blocked (DO-8: command handlers < 10ms).
//! - Every job runs on a tokio blocking task to avoid starving the async runtime.
//! - Progress, completion, and errors are communicated via typed events.
//! - Every job supports cancellation via a `CancellationToken`.

#![forbid(unsafe_code)]

pub mod events;
pub mod output_guard;
pub mod pipeline;
pub mod queue;

pub use events::{JobEvent, JobEventSender};
pub use pipeline::{builtin_pipeline_presets, PipelinePreset, PipelineStep};
pub use queue::{
    is_organizing_intermediate, spawn_sandboxed_worker, JobId, JobManager, JobPriority, JobRecord,
    JobRequest, JobStatus,
};

/// Phase W3: Hardware-adaptive memory budget detection. Uses `sysinfo` (safe, cross-platform:
/// `MemAvailable` on Linux, `GlobalMemoryStatusEx` on Windows, host statistics on macOS) so this
/// crate keeps `#![forbid(unsafe_code)]` on every target. Returns a safe working RAM budget in
/// MiB (`>= 256`); falls back to 512 MiB when the OS reports no memory figures.
pub fn detect_system_memory_budget_mb() -> u64 {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();
    let avail_mb = sys.available_memory() / (1024 * 1024);
    let total_mb = sys.total_memory() / (1024 * 1024);
    if avail_mb == 0 {
        return 512;
    }
    let budget = if total_mb <= 4096 {
        (avail_mb * 40 / 100).min(1024)
    } else if total_mb <= 8192 {
        (avail_mb * 50 / 100).min(2560)
    } else {
        (avail_mb * 50 / 100).min(8192)
    };
    budget.max(256)
}

/// Phase W3: Computes adaptive background worker concurrency (`1..=4`) from available system RAM
/// and logical CPU cores so 4GB–8GB machines never swap-thrash.
pub fn adaptive_worker_concurrency() -> usize {
    let budget_mb = detect_system_memory_budget_mb();
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    if budget_mb <= 1024 || cores <= 2 {
        1
    } else if budget_mb <= 2560 || cores <= 4 {
        2
    } else {
        (cores / 2).clamp(2, 4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_system_memory_budget_mb_and_adaptive_concurrency() {
        let budget = detect_system_memory_budget_mb();
        assert!(
            budget >= 256,
            "Memory budget should be at least 256 MiB, got {}",
            budget
        );
        let concurrency = adaptive_worker_concurrency();
        assert!(
            (1..=4).contains(&concurrency),
            "Concurrency should be clamped in 1..=4, got {}",
            concurrency
        );
    }
}
