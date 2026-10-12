#![forbid(unsafe_code)]

pub use lpdf_core_types::HardwareTier;
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub tier: HardwareTier,
    pub total_memory_mb: u64,
    pub logical_cores: usize,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        let total_memory_bytes = sys.total_memory();
        let total_memory_mb = total_memory_bytes / 1024 / 1024;
        let logical_cores = sys.cpus().len();

        let tier = if total_memory_mb <= 4096 || logical_cores <= 2 {
            HardwareTier::Low
        } else if total_memory_mb >= 16384 && logical_cores >= 8 {
            HardwareTier::High
        } else {
            HardwareTier::Medium
        };

        Self {
            tier,
            total_memory_mb,
            logical_cores,
        }
    }

    pub fn concurrent_jobs(&self) -> usize {
        match self.tier {
            HardwareTier::Low => 1,
            HardwareTier::Medium => 2,
            HardwareTier::High => std::cmp::max(2, self.logical_cores / 4),
        }
    }

    pub fn render_threads(&self) -> usize {
        match self.tier {
            HardwareTier::Low => std::cmp::min(2, self.logical_cores),
            HardwareTier::Medium => std::cmp::max(1, self.logical_cores / 2),
            HardwareTier::High => self.logical_cores,
        }
    }

    pub fn tile_cache_mb(&self) -> u64 {
        match self.tier {
            HardwareTier::Low => 64,
            HardwareTier::Medium => 256,
            HardwareTier::High => 512,
        }
    }

    pub fn memory_budget_mb(&self) -> u64 {
        let mut sys = System::new_all();
        sys.refresh_memory();
        let free_memory_mb = sys.free_memory() / 1024 / 1024;

        match self.tier {
            HardwareTier::Low => std::cmp::min(free_memory_mb * 40 / 100, 1024),
            HardwareTier::Medium => std::cmp::min(free_memory_mb * 50 / 100, 4096),
            HardwareTier::High => std::cmp::min(free_memory_mb * 50 / 100, 16384),
        }
    }
}
