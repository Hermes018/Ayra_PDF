//! Async job queue for long-running PDF operations.
//!
//! Design rules:
//! - The Tauri main thread is NEVER blocked (DO-8: command handlers < 10ms).
//! - Every job runs on a tokio blocking task to avoid starving the async runtime.
//! - Progress, completion, and errors are communicated via typed events.
//! - Every job supports cancellation via a `CancellationToken`.

#![cfg_attr(not(target_os = "windows"), forbid(unsafe_code))]

pub mod events;
pub mod pipeline;
pub mod queue;

pub use events::{JobEvent, JobEventSender};
pub use pipeline::{builtin_pipeline_presets, PipelinePreset, PipelineStep};
pub use queue::{
    spawn_sandboxed_worker, JobId, JobManager, JobPriority, JobRecord, JobRequest, JobStatus,
};

/// Phase W3: Hardware-adaptive memory budget detection (`GlobalMemoryStatusEx` on Windows,
/// `/proc/meminfo` `MemAvailable` on Linux). Returns safe working RAM budget in MiB (`>= 256`).
pub fn detect_system_memory_budget_mb() -> u64 {
    #[cfg(target_os = "windows")]
    {
        #![allow(unsafe_code)]
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
        if ok != 0 {
            let avail_mb = status.ullAvailPhys / (1024 * 1024);
            let total_mb = status.ullTotalPhys / (1024 * 1024);
            let budget = if total_mb <= 4096 {
                (avail_mb * 40 / 100).min(1024)
            } else if total_mb <= 8192 {
                (avail_mb * 50 / 100).min(2560)
            } else {
                (avail_mb * 50 / 100).min(8192)
            };
            return budget.max(256);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
            let mut avail_kb: u64 = 0;
            let mut total_kb: u64 = 0;
            for line in meminfo.lines() {
                if let Some(rest) = line.strip_prefix("MemAvailable:") {
                    avail_kb = rest
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                } else if let Some(rest) = line.strip_prefix("MemTotal:") {
                    total_kb = rest
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u64>().ok())
                        .unwrap_or(0);
                }
            }
            if avail_kb > 0 {
                let avail_mb = avail_kb / 1024;
                let total_mb = total_kb / 1024;
                let budget = if total_mb <= 4096 {
                    (avail_mb * 40 / 100).min(1024)
                } else if total_mb <= 8192 {
                    (avail_mb * 50 / 100).min(2560)
                } else {
                    (avail_mb * 50 / 100).min(8192)
                };
                return budget.max(256);
            }
        }
    }

    512
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
        assert!(budget >= 256, "Memory budget should be at least 256 MiB, got {}", budget);
        let concurrency = adaptive_worker_concurrency();
        assert!(
            (1..=4).contains(&concurrency),
            "Concurrency should be clamped in 1..=4, got {}",
            concurrency
        );
    }
}
