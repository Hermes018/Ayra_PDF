#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareTier {
    L,
    M,
    H,
}

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
            HardwareTier::L
        } else if total_memory_mb >= 16384 && logical_cores >= 8 {
            HardwareTier::H
        } else {
            HardwareTier::M
        };

        Self {
            tier,
            total_memory_mb,
            logical_cores,
        }
    }

    pub fn concurrent_jobs(&self) -> usize {
        match self.tier {
            HardwareTier::L => 1,
            HardwareTier::M => 2,
            HardwareTier::H => std::cmp::max(2, self.logical_cores / 4),
        }
    }

    pub fn render_threads(&self) -> usize {
        match self.tier {
            HardwareTier::L => std::cmp::min(2, self.logical_cores),
            HardwareTier::M => std::cmp::max(1, self.logical_cores / 2),
            HardwareTier::H => self.logical_cores,
        }
    }

    pub fn tile_cache_mb(&self) -> u64 {
        match self.tier {
            HardwareTier::L => 64,
            HardwareTier::M => 256,
            HardwareTier::H => 512,
        }
    }

    pub fn memory_budget_mb(&self) -> u64 {
        let mut sys = System::new_all();
        sys.refresh_memory();
        let free_memory_mb = sys.free_memory() / 1024 / 1024;

        match self.tier {
            HardwareTier::L => std::cmp::min(free_memory_mb * 40 / 100, 1024),
            HardwareTier::M => std::cmp::min(free_memory_mb * 50 / 100, 4096),
            HardwareTier::H => std::cmp::min(free_memory_mb * 50 / 100, 16384),
        }
    }
}
