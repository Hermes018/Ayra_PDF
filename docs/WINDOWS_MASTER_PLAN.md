# LocalPDF Studio — Enterprise Windows 10 & 11 Engineering Master Plan

- **Document ID:** `WINDOWS-MASTER-PLAN-v1.0`
- **Target OS Matrix:** Windows 10 (1809 Build 17763+) & Windows 11 (21H2 Build 22000 through 24H2 Build 26100+)
- **Architecture Authority:** [ADR-011](file:///home/ayra/localpdf-studio-protocol/docs/adr/ADR-011-windows-platform-architecture.md), `protocol/02_RULES_DOS_DONTS.md`, `protocol/03_FAILSAFES.md`
- **Pinned Engine:** `storytold/pdfcraft` rev `75360d1d7db06fed974f48c914d4c04132757187`
- **Baseline Tag:** `v0.1.0-rc5`

---

## SECTION 1: Cross-Platform Invariants & Pre-Session Protocol Rules

Every autonomous agent or engineer executing a Windows port phase (Phases W1–W4) MUST obey the following cross-platform invariants so our production Linux build (Fedora 44 / KDE Plasma 6 Wayland) never regresses.

### 1.1 Mandatory Pre-Session Boot Sequence
Before modifying any file in any phase, the executing agent must run this single verification block:

```bash
# 1. Read protocol state and verify clean git working tree
cat logs/CHECKPOINT.md
git status --short

# 2. Verify line counts of critical source files to establish baseline guards
wc -l Cargo.toml \
      crates/lpdf-fs-safety/src/lib.rs \
      crates/lpdf-hw/src/lib.rs \
      crates/lpdf-jobs/src/queue.rs \
      crates/lpdf-engine-pdfcraft/src/lib.rs \
      crates/lpdf-engine-pdfcraft/src/office_sidecar.rs \
      crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs \
      app-tauri/src-tauri/Cargo.toml \
      app-tauri/src-tauri/src/lib.rs \
      app-tauri/src-tauri/tauri.conf.json

# 3. Verify baseline Linux compilation and zero-hex UI invariant
cargo check --workspace
grep -rn -E "#[0-9a-fA-F]{3,6}" app-tauri/src/lib/views/ | wc -l
```

### 1.2 Strict Conditional Compilation (`#[cfg]`) Isolation Rules
1. **Never Break Linux Semantics:** Every Linux-specific mechanism (`unshare -n -r` network isolation in `office_sidecar.rs`, `pandoc_sidecar.rs`, and `queue.rs`; `landlock` in `lpdf-worker`; `kdialog`/`zenity` in `app-tauri/src-tauri/src/lib.rs`) MUST remain guarded under `#[cfg(target_os = "linux")]`.
2. **Explicit Windows Submodules:** All Win32 FFI bindings (`windows-sys`) MUST be encapsulated inside `#[cfg(target_os = "windows")]` submodules with `#![allow(unsafe_code)]` scoped strictly to that submodule per ADR-011. Top-level crate roots keep `#![forbid(unsafe_code)]` (or scope the exception to the Windows FFI module) and never expose raw Win32 `HANDLE` or pointers across public crate APIs.
3. **Target-Specific Dependencies Only:** Win32 crates (`windows-sys`, `window-vibrancy`, `dunce`) MUST be declared under `[target.'cfg(target_os = "windows")'.dependencies]` in `Cargo.toml` so Linux compile times and dependency graphs remain untouched.
4. **No `.unwrap()` / `.expect()` / `todo!()`:** Per `protocol/04_INSTRUCTION_FADE_PREVENTION.md`, `cargo xtask protocol-lint` scans all non-test `src/` files for `.unwrap(`, `.expect(`, `todo!(`, `unimplemented!(`, and `panic!(`. All Windows code must use `?`, `.unwrap_or(...)`, `.unwrap_or_else(...)`, or explicit `match` expressions.

---

## SECTION 2: Extreme Rust & MSVC Compiler Optimization Matrix

### 2.1 Root `Cargo.toml` Release Profile
The workspace root `Cargo.toml` defines the release optimization profile for all 17 crates:

```toml
[profile.release]
codegen-units = 1
lto = "fat"
opt-level = 3
panic = "abort"
strip = "symbols"
overflow-checks = false
incremental = false
```

### 2.2 `.cargo/config.toml` Target & Linker Matrix (`x86_64-pc-windows-msvc`)
On `x86_64-pc-windows-msvc`, we configure MSVC linker flags to enforce PE exploit mitigations, COMDAT folding, and dynamic CRT linkage:

```toml
[alias]
xtask = "run --package xtask --"

[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "target-feature=-crt-static",
    "-C", "link-arg=/OPT:REF",
    "-C", "link-arg=/OPT:ICF",
    "-C", "link-arg=/HIGHENTROPYVA",
    "-C", "link-arg=/DYNAMICBASE",
    "-C", "link-arg=/NXCOMPAT",
    "-C", "link-arg=/GUARD:CF",
    "-C", "link-arg=/CETCOMPAT",
]
```

### 2.3 Dynamic vs. Static CRT Trade-Off Analysis
- **Why `-crt-static` + `"bundleVcRuntime": true` Wins:** Tauri v2 links against `WebView2Loader.dll` and Windows COM (`ole32.dll`, `combase.dll`, `dwmapi.dll`). Linking `+crt-static` embeds a private copy of `libucrt.lib` into the binary while `WebView2Loader.dll` uses `ucrtbase.dll`, increasing binary size and risking heap mismatches across FFI boundaries. Using `-crt-static` with `"bundleVcRuntime": true` in `tauri.conf.json` ships `vcruntime140.dll`, `vcruntime140_1.dll`, and `msvcp140.dll` directly in the application folder so clean Windows 10 1809 Virtual Machines run out-of-the-box without requiring the user to install `vc_redist.x64.exe`.

---

## SECTION 3: RAM, Thread-Pool, & Resource Utilization Engineering

### 3.1 Bounded Memory Architecture (`crates/lpdf-hw`)
On Windows, `GlobalMemoryStatusEx` provides zero-allocation kernel memory telemetry (`ullTotalPhys`, `ullAvailPhys`, `dwMemoryLoad`). We augment `crates/lpdf-hw` with a Windows-native hardware probe and working-set trimmer so LocalPDF Studio scales smoothly from 8 GB budget laptops to 64 GB workstations.

#### Hardware Tier Scaling Matrix
| Tier | Physical RAM | Concurrent Jobs | Rayon Render Threads | Raster Page Buffer Cap | Tile Cache Budget |Per-Job Memory Cap |
|---|---|---|---|---|---|---|
| **Low (`Tier::Low`)** | $\le 8192\text{ MB}$ or $\le 2$ cores | `1` | $\min(2, \text{cores})$ | `4` pages in-flight | `64 MB` | $\min(40\%\text{ avail}, 1024\text{ MB})$ |
| **Medium (`Tier::Medium`)** | $8193\text{ MB} - 16383\text{ MB}$ | `2` | $\max(2, \text{cores} / 2)$ | `8` pages in-flight | `256 MB` | $\min(50\%\text{ avail}, 4096\text{ MB})$ |
| **High (`Tier::High`)** | $\ge 16384\text{ MB}$ & $\ge 8$ cores | $\max(2, \text{cores} / 4)$ | $\max(4, \text{cores} - 2)$ | `16` pages in-flight | `512 MB` | $\min(50\%\text{ avail}, 16384\text{ MB})$ |

### 3.2 Complete Implementation for `crates/lpdf-hw/src/lib.rs`

```rust
#![cfg_attr(not(target_os = "windows"), forbid(unsafe_code))]

pub use lpdf_core_types::HardwareTier;
use sysinfo::System;

#[cfg(target_os = "windows")]
mod win_mem {
    #![allow(unsafe_code)]
    use windows_sys::Win32::System::ProcessStatus::K32EmptyWorkingSet;
    use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, GetCurrentThread, SetThreadInformation, SetThreadPriority,
        ThreadPowerThrottling, THREAD_MODE_BACKGROUND_BEGIN, THREAD_MODE_BACKGROUND_END,
        THREAD_POWER_THROTTLING_CURRENT_VERSION, THREAD_POWER_THROTTLING_EXECUTION_SPEED,
        THREAD_POWER_THROTTLING_STATE, THREAD_PRIORITY_BELOW_NORMAL, THREAD_PRIORITY_NORMAL,
    };

    #[derive(Debug, Clone, Copy)]
    pub struct WinMemorySnapshot {
        pub total_phys_mb: u64,
        pub avail_phys_mb: u64,
        pub memory_load_percent: u32,
    }

    pub fn query_windows_memory() -> Option<WinMemorySnapshot> {
        let mut status = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailVirtual: 0,
            ullAvailExtendedVirtual: 0,
        };
        let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
        if ok == 0 {
            return None;
        }
        Some(WinMemorySnapshot {
            total_phys_mb: status.ullTotalPhys / (1024 * 1024),
            avail_phys_mb: status.ullAvailPhys / (1024 * 1024),
            memory_load_percent: status.dwMemoryLoad,
        })
    }

    /// Trims unused committed pages from the current process working set after heavy PDF jobs.
    pub fn trim_process_working_set() -> bool {
        let handle = unsafe { GetCurrentProcess() };
        let res = unsafe { K32EmptyWorkingSet(handle) };
        res != 0
    }

    /// Lowers the calling worker thread priority and enables Windows 11 EcoQoS power throttling
    /// so heavy PDF batch processing never stutters the WebView2 UI thread or system audio.
    pub fn enter_eco_qos_background_mode() {
        unsafe {
            let thread = GetCurrentThread();
            let _ = SetThreadPriority(thread, THREAD_PRIORITY_BELOW_NORMAL);
            let _ = SetThreadPriority(thread, THREAD_MODE_BACKGROUND_BEGIN);

            let throttling = THREAD_POWER_THROTTLING_STATE {
                Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
                ControlMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
                StateMask: THREAD_POWER_THROTTLING_EXECUTION_SPEED,
            };
            let _ = SetThreadInformation(
                thread,
                ThreadPowerThrottling,
                &throttling as *const THREAD_POWER_THROTTLING_STATE as *const std::ffi::c_void,
                std::mem::size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
            );
        }
    }

    /// Restores normal scheduling priority on worker thread exit.
    pub fn exit_eco_qos_background_mode() {
        unsafe {
            let thread = GetCurrentThread();
            let _ = SetThreadPriority(thread, THREAD_MODE_BACKGROUND_END);
            let _ = SetThreadPriority(thread, THREAD_PRIORITY_NORMAL);
        }
    }
}

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub tier: HardwareTier,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub logical_cores: usize,
}

impl HardwareProfile {
    pub fn detect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();
        sys.refresh_cpu_usage();

        let logical_cores = std::cmp::max(1, sys.cpus().len());

        #[cfg(target_os = "windows")]
        let (total_memory_mb, available_memory_mb) = match win_mem::query_windows_memory() {
            Some(snap) => (snap.total_phys_mb, snap.avail_phys_mb),
            None => (
                sys.total_memory() / 1024 / 1024,
                sys.available_memory() / 1024 / 1024,
            ),
        };

        #[cfg(not(target_os = "windows"))]
        let (total_memory_mb, available_memory_mb) = (
            sys.total_memory() / 1024 / 1024,
            sys.available_memory() / 1024 / 1024,
        );

        let tier = if total_memory_mb <= 8192 || logical_cores <= 2 {
            HardwareTier::Low
        } else if total_memory_mb >= 16384 && logical_cores >= 8 {
            HardwareTier::High
        } else {
            HardwareTier::Medium
        };

        Self {
            tier,
            total_memory_mb,
            available_memory_mb,
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
            HardwareTier::Medium => std::cmp::max(2, self.logical_cores / 2),
            HardwareTier::High => std::cmp::max(4, self.logical_cores.saturating_sub(2)),
        }
    }

    pub fn max_raster_pages_in_flight(&self) -> usize {
        match self.tier {
            HardwareTier::Low => 4,
            HardwareTier::Medium => 8,
            HardwareTier::High => 16,
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
        #[cfg(target_os = "windows")]
        let free_memory_mb = win_mem::query_windows_memory()
            .map(|s| s.avail_phys_mb)
            .unwrap_or(self.available_memory_mb);

        #[cfg(not(target_os = "windows"))]
        let free_memory_mb = {
            let mut sys = System::new_all();
            sys.refresh_memory();
            sys.available_memory() / 1024 / 1024
        };

        match self.tier {
            HardwareTier::Low => std::cmp::min(free_memory_mb * 40 / 100, 1024),
            HardwareTier::Medium => std::cmp::min(free_memory_mb * 50 / 100, 4096),
            HardwareTier::High => std::cmp::min(free_memory_mb * 50 / 100, 16384),
        }
    }
}

/// RAII guard for worker thread QoS (enters EcoQoS background mode on Windows, no-op elsewhere).
pub struct WorkerQosGuard;

impl WorkerQosGuard {
    pub fn enter() -> Self {
        #[cfg(target_os = "windows")]
        win_mem::enter_eco_qos_background_mode();
        Self
    }
}

impl Drop for WorkerQosGuard {
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        win_mem::exit_eco_qos_background_mode();
    }
}

/// Reclaims unused physical pages in the current process working set after heavy jobs.
pub fn reclaim_working_set_memory() {
    #[cfg(target_os = "windows")]
    {
        let _ = win_mem::trim_process_working_set();
    }
}
```

---

## SECTION 4: Windows 10 & 11 Native Glass Vibrancy & DWM Failsafes

### 4.1 Multi-Tiered Window Composition Engine (`app-tauri/src-tauri/src/win_vibrancy.rs`)
We implement a dedicated vibrancy module that detects DWM composition and OS transparency settings (`EnableTransparency` in Registry and `DwmIsCompositionEnabled`) and cascades gracefully across three tiers:

1. **Tier 1 (Windows 11 Build $\ge$ 22000):** `window_vibrancy::apply_mica(&window, Some(dark))`
2. **Tier 2 (Windows 10 1809 Build $\ge$ 17763):** `window_vibrancy::apply_acrylic(&window, Some(tint))`
3. **Tier 3 (DWM Disabled / RDP / Battery Saver / High Contrast):** Clears all transparency effects and returns `VibrancyTier::OpaqueFallback`, signaling the Svelte frontend to apply solid surface backgrounds (`data-vibrancy="opaque"`).

### 4.2 Complete Implementation for `app-tauri/src-tauri/src/win_vibrancy.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum VibrancyTier {
    Mica,
    Acrylic,
    OpaqueFallback,
}

#[cfg(target_os = "windows")]
mod imp {
    #![allow(unsafe_code)]
    use super::VibrancyTier;
    use tauri::WebviewWindow;
    use windows_sys::Win32::Graphics::Dwm::DwmIsCompositionEnabled;
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    use windows_sys::Win32::System::Registry::{
        RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_REMOTESESSION};

    /// Checks whether DWM composition is active, the session is local (not RDP),
    /// Battery Energy Saver is not active, and OS "Transparency effects" are enabled.
    pub fn is_transparency_supported_and_enabled() -> bool {
        unsafe {
            // 1. Check Remote Desktop session (SM_REMOTESESSION != 0)
            if GetSystemMetrics(SM_REMOTESESSION) != 0 {
                return false;
            }

            // 2. Check DWM composition enabled
            let mut dwm_enabled: i32 = 0;
            if DwmIsCompositionEnabled(&mut dwm_enabled) != 0 || dwm_enabled == 0 {
                return false;
            }

            // 3. Check Battery Saver status (SystemStatusFlag == 1 means Energy Saver ON)
            let mut power_status = SYSTEM_POWER_STATUS {
                ACLineStatus: 0,
                BatteryFlag: 0,
                BatteryLifePercent: 0,
                SystemStatusFlag: 0,
                BatteryLifeTime: 0,
                BatteryFullLifeTime: 0,
            };
            if GetSystemPowerStatus(&mut power_status) != 0 && power_status.SystemStatusFlag == 1 {
                return false;
            }

            // 4. Query HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\EnableTransparency
            let subkey: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
                .encode_utf16()
                .collect();
            let val_name: Vec<u16> = "EnableTransparency\0".encode_utf16().collect();
            let mut data: u32 = 1;
            let mut data_len: u32 = std::mem::size_of::<u32>() as u32;
            let status = RegGetValueW(
                HKEY_CURRENT_USER,
                subkey.as_ptr(),
                val_name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                &mut data as *mut u32 as *mut std::ffi::c_void,
                &mut data_len,
            );
            if status == 0 && data == 0 {
                return false;
            }

            true
        }
    }

    pub fn apply_window_vibrancy(window: &WebviewWindow, dark: bool) -> VibrancyTier {
        // Clear any previous backdrop effects before applying new ones
        let _ = window_vibrancy::clear_mica(window);
        let _ = window_vibrancy::clear_acrylic(window);
        let _ = window_vibrancy::clear_blur(window);

        if !is_transparency_supported_and_enabled() {
            return VibrancyTier::OpaqueFallback;
        }

        // Tier 1: Try Windows 11 Mica (Build 22000+)
        if window_vibrancy::apply_mica(window, Some(dark)).is_ok() {
            return VibrancyTier::Mica;
        }

        // Tier 2: Fallback to Windows 10 1809+ Acrylic with calibrated RGBA tint
        let acrylic_tint = if dark {
            (18, 18, 24, 215)
        } else {
            (245, 245, 250, 220)
        };
        if window_vibrancy::apply_acrylic(window, Some(acrylic_tint)).is_ok() {
            return VibrancyTier::Acrylic;
        }

        // Tier 3: Opaque high-contrast fallback
        VibrancyTier::OpaqueFallback
    }
}

pub fn configure_window_vibrancy(window: &tauri::WebviewWindow, dark: bool) -> VibrancyTier {
    #[cfg(target_os = "windows")]
    {
        imp::apply_window_vibrancy(window, dark)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window, dark);
        VibrancyTier::OpaqueFallback
    }
}
```

### 4.3 Cold-Start White-Flash Elimination Strategy
1. **`tauri.conf.json` Window Pre-Configuration:**
   - Set `"visible": false` and `"transparent": true` on the main window definition, with `"backgroundColor": [15, 15, 20, 0]`.
   - `wry` 0.57 automatically translates `backgroundColor` with `A == 0` into `ICoreWebView2ControllerOptions3::SetDefaultBackgroundColor(COREWEBVIEW2_COLOR { R: 15, G: 15, B: 20, A: 0 })` prior to navigation.
2. **Coordinated First-Paint Reveal:**
   - Expose two Tauri commands in `app-tauri/src-tauri/src/lib.rs`:
     - `set_window_vibrancy_theme(window: tauri::WebviewWindow, dark: bool) -> String`
     - `show_main_window(window: tauri::WebviewWindow) -> Result<(), String>`
   - In `app-tauri/src/lib/state.svelte.ts`, `applyTheme()` invokes `IpcClient.setWindowVibrancyTheme(resolved === 'dark')`, sets `document.documentElement.setAttribute('data-vibrancy', tier)`, and in `MainLayout.svelte` `onMount`, calls `IpcClient.showMainWindow()` once the Svelte 5 DOM tree and theme styles are mounted.

---

## SECTION 5: Enterprise File System, Atomic I/O, & Sidecar Sandboxing on NTFS

### 5.1 NTFS Atomic Write with Retry Backoff & Safe Recovery (`crates/lpdf-fs-safety/src/lib.rs`)
On Windows NTFS, `std::fs::rename` fails with `ERROR_SHARING_VIOLATION` (OS Error 32) or `ERROR_ACCESS_DENIED` (OS Error 5) whenever an antivirus scanner (Windows Defender `MsMpEng.exe`), Windows Search Indexer (`SearchIndexer.exe`), or a PDF viewer holds a handle to the destination file.

We upgrade `atomic_write` in `crates/lpdf-fs-safety/src/lib.rs` with:
1. Write-through disk flushing (`File::sync_all()`).
2. Win32 `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`.
3. Exponential backoff (6 attempts: `25ms`, `50ms`, `100ms`, `200ms`, `400ms`, `800ms`) on OS Errors `5` (`ERROR_ACCESS_DENIED`), `32` (`ERROR_SHARING_VIOLATION`), and `33` (`ERROR_LOCK_VIOLATION`).
4. **Extended-Length Path (`\\?\`) Helper (`simplify_path_for_sidecar`):** Uses `dunce::simplified` on Windows so external sidecar CLIs (`soffice.exe`, `pandoc.exe`) receive standard Win32 paths (`C:\Users\...`) instead of `\\?\C:\Users\...` verbatim paths that break their command-line parsers, and formats LibreOffice profile URLs (`file:///C:/...`) with forward slashes.

```rust
#[cfg(target_os = "windows")]
mod win_fs {
    #![allow(unsafe_code)]
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::time::Duration;
    use windows_sys::Win32::Foundation::{
        GetLastError, ERROR_ACCESS_DENIED, ERROR_LOCK_VIOLATION, ERROR_SHARING_VIOLATION,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    fn to_wide_null(p: &Path) -> Vec<u16> {
        p.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn atomic_replace_with_backoff(temp_path: &Path, target_path: &Path) -> std::io::Result<()> {
        let src_w = to_wide_null(temp_path);
        let dst_w = to_wide_null(target_path);
        let retry_delays_ms = [25_u64, 50, 100, 200, 400, 800];

        for (attempt, delay_ms) in retry_delays_ms.iter().enumerate() {
            let ok = unsafe {
                MoveFileExW(
                    src_w.as_ptr(),
                    dst_w.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            };
            if ok != 0 {
                return Ok(());
            }

            let err_code = unsafe { GetLastError() };
            let is_transient_lock = err_code == ERROR_SHARING_VIOLATION
                || err_code == ERROR_LOCK_VIOLATION
                || err_code == ERROR_ACCESS_DENIED;

            if !is_transient_lock || attempt == retry_delays_ms.len() - 1 {
                return Err(std::io::Error::from_raw_os_error(err_code as i32));
            }

            std::thread::sleep(Duration::from_millis(*delay_ms));
        }

        Err(std::io::Error::other("Exhausted atomic replace retries"))
    }
}
```

### 5.2 Win32 Job Object Sandbox & `CREATE_NO_WINDOW` Sidecar Containment
To enforce Failsafes F3, F20, and F21 on Windows, we create a reusable Win32 Job Object sandbox in `crates/lpdf-fs-safety/src/win_sandbox.rs` (exported on Windows and no-op on Unix) and wire it into `office_sidecar.rs`, `pandoc_sidecar.rs`, and `lpdf-jobs/src/queue.rs`:

```rust
#[cfg(target_os = "windows")]
pub mod win_sandbox {
    #![allow(unsafe_code)]
    use std::os::windows::io::AsRawHandle;
    use std::os::windows::process::CommandExt;
    use std::process::{Child, Command};
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION, JOB_OBJECT_LIMIT_JOB_MEMORY,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    pub const CREATE_NO_WINDOW_FLAG: u32 = 0x0800_0000;
    pub const SIDECAR_MEMORY_LIMIT_BYTES: usize = 1024 * 1024 * 1024; // 1 GiB per ADR-001

    /// Configures `std::process::Command` on Windows to suppress console window flashing.
    pub fn apply_no_window_flags(cmd: &mut Command) {
        cmd.creation_flags(CREATE_NO_WINDOW_FLAG);
    }

    /// RAII Win32 Job Object that enforces a 1 GiB memory ceiling and guarantees
    /// all assigned child/grandchild processes are terminated when dropped or if the host crashes.
    pub struct WinJobSandbox {
        job_handle: HANDLE,
    }

    // Safety: Win32 kernel job handles are thread-safe.
    unsafe impl Send for WinJobSandbox {}
    unsafe impl Sync for WinJobSandbox {}

    impl WinJobSandbox {
        pub fn new_with_memory_cap(memory_limit_bytes: usize) -> std::io::Result<Self> {
            let job_handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            if job_handle.is_null() {
                return Err(std::io::Error::last_os_error());
            }

            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_JOB_MEMORY
                | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
            info.JobMemoryLimit = memory_limit_bytes;

            let ok = unsafe {
                SetInformationJobObject(
                    job_handle,
                    JobObjectExtendedLimitInformation,
                    &info as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION as *const std::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };

            if ok == 0 {
                let err = std::io::Error::last_os_error();
                unsafe {
                    let _ = CloseHandle(job_handle);
                }
                return Err(err);
            }

            Ok(Self { job_handle })
        }

        pub fn assign_child(&self, child: &Child) -> std::io::Result<()> {
            let process_handle = child.as_raw_handle() as HANDLE;
            let ok = unsafe { AssignProcessToJobObject(self.job_handle, process_handle) };
            if ok == 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        }
    }

    impl Drop for WinJobSandbox {
        fn drop(&mut self) {
            if !self.job_handle.is_null() {
                unsafe {
                    let _ = CloseHandle(self.job_handle);
                }
            }
        }
    }
}
```

---

## SECTION 6: Enterprise Security, Binary Hardening, & Offline WebView2 Lockdown

### 6.1 PE Binary Exploit Mitigations
Every shipped Windows executable (`localpdf-studio.exe`, `localpdf.exe`, `lpdf-worker.exe`) is compiled with the following PE header security flags:
- **`/HIGHENTROPYVA` & `/DYNAMICBASE`:** 64-bit High-Entropy Address Space Layout Randomization (ASLR) providing 24 bits of entropy (1 TB variance) for stack, heap, and image base addresses.
- **`/NXCOMPAT`:** Hardware-enforced Data Execution Prevention (DEP / XD / NX bit), marking all data pages non-executable.
- **`/GUARD:CF`:** Control Flow Guard (CFG) indirect-call target bitmap verification at runtime.
- **`/CETCOMPAT`:** Hardware-enforced Stack Protection (Intel CET / AMD Shadow Stack) compatibility flag.

### 6.2 Offline WebView2 Browser Lockdown & Tauri v2 Capabilities
On Windows, `wry` initializes Chromium-based Microsoft Edge WebView2. By default, `wry` sets `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection`. When overriding `additionalBrowserArgs` in `tauri.conf.json`, we MUST preserve those three flags and append strict offline/anti-telemetry flags so WebView2 never phones home or leaks document metadata:

```text
--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,Translate,OptimizationHints,MediaRouter,DialMediaRouteProvider --disable-background-networking --disable-breakpad --disable-client-side-phishing-detection --disable-component-update --disable-default-apps --disable-domain-reliability --disable-sync --metrics-recording-only --no-default-browser-check --no-first-run --no-pings
```

Combined with our strict Content Security Policy:
```json
"csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src ipc: http://ipc.localhost"
```
this guarantees 100% offline compliance (Failsafe F9) even when Windows has an active internet connection.

---

## SECTION 7: Phased Execution Roadmap (Phases W1 – W4)

Each phase below is self-contained, atomic, and designed for sequential execution by Gemini 3.8 Flash.

---

### PHASE W1: Rust Core, NTFS Atomic I/O, Long-Path Sanitization, & Win32 Job Object Sandbox

#### 1. Pre-Phase Backup & State Verification
```bash
cat logs/CHECKPOINT.md
git status --short
mkdir -p /tmp/lpdf_backup_w1
cp Cargo.toml \
   crates/lpdf-fs-safety/Cargo.toml \
   crates/lpdf-fs-safety/src/lib.rs \
   crates/lpdf-engine-pdfcraft/src/office_sidecar.rs \
   crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs \
   crates/lpdf-jobs/src/queue.rs \
   /tmp/lpdf_backup_w1/
```

#### 2. Exact File-by-File Implementation Spec

##### File 1: `crates/lpdf-fs-safety/Cargo.toml`
Add target-specific `windows-sys` and `dunce` dependencies for Windows:
```toml
[package]
name = "lpdf-fs-safety"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[dependencies]
lpdf-core-types = { path = "../lpdf-core-types" }
toml = "0.8"

[target.'cfg(target_os = "windows")'.dependencies]
dunce = "1.0.5"
windows-sys = { version = "0.61.2", features = [
    "Win32_Foundation",
    "Win32_Storage_FileSystem",
    "Win32_System_JobObjects",
] }

[dev-dependencies]
tempfile = "3.10"
```

##### File 2: `crates/lpdf-fs-safety/src/lib.rs`
Upgrade `crates/lpdf-fs-safety/src/lib.rs` to include:
- `#![cfg_attr(not(target_os = "windows"), forbid(unsafe_code))]` at crate root.
- `win_fs::atomic_replace_with_backoff` (from Section 5.1) called inside `atomic_write` on `#[cfg(target_os = "windows")]` after `File::sync_all()`, with fallback to safe recovery path `{stem}_saved_{uuid}.{ext}` if `ERROR_SHARING_VIOLATION` persists after all 6 retries.
- `pub fn simplify_path_for_sidecar(path: &Path) -> PathBuf` (uses `dunce::simplified(path).to_path_buf()` on Windows; `path.to_path_buf()` on Unix).
- `pub fn format_file_url_from_path(path: &Path) -> String` (converts Windows backslashes `C:\foo\bar` into `file:///C:/foo/bar` and Unix `/tmp/foo` into `file:///tmp/foo`).
- `pub mod sandbox` wrapping `WinJobSandbox` (Section 5.2) with a cross-platform `ChildProcessSandbox` struct:
```rust
pub struct ChildProcessSandbox {
    #[cfg(target_os = "windows")]
    inner: Option<win_sandbox::WinJobSandbox>,
}

impl ChildProcessSandbox {
    pub fn new_sidecar_limit() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self {
                inner: win_sandbox::WinJobSandbox::new_with_memory_cap(
                    win_sandbox::SIDECAR_MEMORY_LIMIT_BYTES,
                )
                .ok(),
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self {}
        }
    }

    pub fn configure_command(cmd: &mut std::process::Command) {
        #[cfg(target_os = "windows")]
        win_sandbox::apply_no_window_flags(cmd);
        #[cfg(not(target_os = "windows"))]
        let _ = cmd;
    }

    pub fn attach(&self, child: &std::process::Child) {
        #[cfg(target_os = "windows")]
        if let Some(ref job) = self.inner {
            let _ = job.assign_child(child);
        }
        #[cfg(not(target_os = "windows"))]
        let _ = child;
    }
}
```

##### File 3: `crates/lpdf-engine-pdfcraft/src/office_sidecar.rs` & `crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs`
- Replace `let canonical_input = input_path.canonicalize().map_err(LpdfError::Io)?;` with:
  ```rust
  let canonical_raw = input_path.canonicalize().map_err(LpdfError::Io)?;
  let canonical_input = lpdf_fs_safety::simplify_path_for_sidecar(&canonical_raw);
  ```
- Replace `let profile_url = format!("file://{}", temp_profile.display());` in `office_sidecar.rs` with:
  ```rust
  let profile_url = lpdf_fs_safety::format_file_url_from_path(&temp_profile);
  ```
- Before spawning `cmd.spawn()`, invoke `lpdf_fs_safety::ChildProcessSandbox::configure_command(&mut cmd);`, instantiate `let job_sandbox = lpdf_fs_safety::ChildProcessSandbox::new_sidecar_limit();`, and immediately after `cmd.spawn()` succeeds, call `job_sandbox.attach(&child);`. Keep `job_sandbox` alive for the duration of the child execution scope.

##### File 4: `crates/lpdf-jobs/src/queue.rs`
In `JobManager::spawn_sandboxed_worker`:
- Resolve `lpdf-worker.exe` on Windows (`#[cfg(target_os = "windows")] let worker_FileName = "lpdf-worker.exe"; #[cfg(not(target_os = "windows"))] let worker_FileName = "lpdf-worker";`).
- Call `lpdf_fs_safety::ChildProcessSandbox::configure_command(&mut cmd);` before `cmd.spawn()`.

#### 3. Cross-Check & Validation Gate
```bash
cargo check --workspace
cargo test -p lpdf-fs-safety -p lpdf-engine-pdfcraft -p lpdf-jobs
cargo xtask protocol-lint
```
All 3 commands must exit with code `0` (`TRUE`).

#### 4. Rollback Failsafe
```bash
cp /tmp/lpdf_backup_w1/Cargo.toml Cargo.toml
cp /tmp/lpdf_backup_w1/lib.rs crates/lpdf-fs-safety/src/lib.rs
cp /tmp/lpdf_backup_w1/office_sidecar.rs crates/lpdf-engine-pdfcraft/src/office_sidecar.rs
cp /tmp/lpdf_backup_w1/pandoc_sidecar.rs crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs
cp /tmp/lpdf_backup_w1/queue.rs crates/lpdf-jobs/src/queue.rs
git checkout -- crates/lpdf-fs-safety/Cargo.toml
```

#### 5. Log Update & Git Commit Protocol
- Update `logs/CHECKPOINT.md` to `Phase W1 COMPLETE — NTFS Atomic I/O, Verbatim Path Sanitization, & Win32 Job Object Sandbox`.
- Append Phase W1 entry to `logs/PROGRESS_LOG.md`.
- Commit & push:
```bash
git add Cargo.lock crates/lpdf-fs-safety crates/lpdf-engine-pdfcraft crates/lpdf-jobs logs/CHECKPOINT.md logs/PROGRESS_LOG.md
git commit -m "feat(windows-w1): implement NTFS retry atomic_write, verbatim path sanitizer, and Win32 Job Object sandbox"
git push origin main
```

---

### PHASE W2: Window Vibrancy Engine (Mica / Acrylic / Opaque Failsafe), Theme Sync IPC, & Cold-Start Flash Elimination

#### 1. Pre-Phase Backup & State Verification
```bash
cat logs/CHECKPOINT.md
git status --short
mkdir -p /tmp/lpdf_backup_w2
cp app-tauri/src-tauri/Cargo.toml \
   app-tauri/src-tauri/src/lib.rs \
   app-tauri/src/lib/ipc.ts \
   app-tauri/src/lib/state.svelte.ts \
   app-tauri/src/lib/components/MainLayout.svelte \
   /tmp/lpdf_backup_w2/
```

#### 2. Exact File-by-File Implementation Spec

##### File 1: `app-tauri/src-tauri/Cargo.toml`
Add target-specific Windows vibrancy and DWM/Power/Registry crates:
```toml
[target.'cfg(target_os = "windows")'.dependencies]
window-vibrancy = "0.8.1"
windows-sys = { version = "0.61.2", features = [
    "Win32_Foundation",
    "Win32_Graphics_Dwm",
    "Win32_System_Power",
    "Win32_System_Registry",
    "Win32_UI_WindowsAndMessaging",
] }
```

##### File 2: `app-tauri/src-tauri/src/win_vibrancy.rs`
Create `app-tauri/src-tauri/src/win_vibrancy.rs` with the exact code specified in Section 4.2.

##### File 3: `app-tauri/src-tauri/src/lib.rs`
1. Declare `mod win_vibrancy;` at the top of `app-tauri/src-tauri/src/lib.rs`.
2. Replace PowerShell dialog spawns on `#[cfg(target_os = "windows")]` in `pick_files_dialog`, `pick_folder_dialog`, and `save_file_dialog` with `lpdf_fs_safety::ChildProcessSandbox::configure_command(&mut cmd)` so PowerShell file dialogs never flash a blue console window.
3. Add the Tauri commands:
```rust
#[tauri::command]
fn set_window_vibrancy_theme(window: tauri::WebviewWindow, dark: bool) -> String {
    match win_vibrancy::configure_window_vibrancy(&window, dark) {
        win_vibrancy::VibrancyTier::Mica => "mica".to_string(),
        win_vibrancy::VibrancyTier::Acrylic => "acrylic".to_string(),
        win_vibrancy::VibrancyTier::OpaqueFallback => "opaque".to_string(),
    }
}

#[tauri::command]
fn show_main_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    let _ = window.set_focus();
    Ok(())
}
```
4. Register `set_window_vibrancy_theme` and `show_main_window` in `tauri::generate_handler![...]` and invoke `win_vibrancy::configure_window_vibrancy` on the main window inside `.setup(|app| { ... })` using `tauri::Manager::get_webview_window(app, "main")`.

##### File 4: `app-tauri/src/lib/ipc.ts`, `state.svelte.ts`, & `MainLayout.svelte`
- In `app-tauri/src/lib/ipc.ts`, add:
```typescript
    static async setWindowVibrancyTheme(dark: boolean): Promise<string> {
        try {
            return await invoke<string>('set_window_vibrancy_theme', { dark });
        } catch {
            return 'opaque';
        }
    }

    static async showMainWindow(): Promise<void> {
        try {
            await invoke<void>('show_main_window');
        } catch {
            // no-op in browser test environments
        }
    }
```
- In `app-tauri/src/lib/state.svelte.ts`, at the end of `applyTheme()`, call:
```typescript
        void IpcClient.setWindowVibrancyTheme(resolved === 'dark').then((tier) => {
            if (typeof document !== 'undefined') {
                document.documentElement.setAttribute('data-vibrancy', tier);
            }
        });
```
- In `app-tauri/src/lib/components/MainLayout.svelte`, inside `onMount` (or `$effect` on mount), invoke `void IpcClient.showMainWindow();`.

#### 3. Cross-Check & Validation Gate
```bash
cargo check --manifest-path app-tauri/src-tauri/Cargo.toml
tools/sb.sh pnpm --dir app-tauri check
pnpm --dir app-tauri exec vitest run
grep -rn -E "#[0-9a-fA-F]{3,6}" app-tauri/src/lib/views/ | wc -l
```
All checks must pass with 0 errors and hex count `0`.

#### 4. Rollback Failsafe
```bash
rm -f app-tauri/src-tauri/src/win_vibrancy.rs
cp /tmp/lpdf_backup_w2/Cargo.toml app-tauri/src-tauri/Cargo.toml
cp /tmp/lpdf_backup_w2/lib.rs app-tauri/src-tauri/src/lib.rs
cp /tmp/lpdf_backup_w2/ipc.ts app-tauri/src/lib/ipc.ts
cp /tmp/lpdf_backup_w2/state.svelte.ts app-tauri/src/lib/state.svelte.ts
cp /tmp/lpdf_backup_w2/MainLayout.svelte app-tauri/src/lib/components/MainLayout.svelte
```

#### 5. Log Update & Git Commit Protocol
- Update `logs/CHECKPOINT.md` to `Phase W2 COMPLETE — Windows 10/11 Vibrancy Engine, Theme Sync IPC, & Cold-Start Reveal`.
- Append Phase W2 entry to `logs/PROGRESS_LOG.md`.
- Commit & push:
```bash
git add Cargo.lock app-tauri logs/CHECKPOINT.md logs/PROGRESS_LOG.md
git commit -m "feat(windows-w2): add 3-tier Mica/Acrylic/Opaque vibrancy engine and cold-start flash elimination"
git push origin main
```

---

### PHASE W3: Hardware-Adaptive Memory Governor, EcoQoS Worker Priority, & Compiler Release Profile Tuning

#### 1. Pre-Phase Backup & State Verification
```bash
cat logs/CHECKPOINT.md
git status --short
mkdir -p /tmp/lpdf_backup_w3
cp Cargo.toml \
   .cargo/config.toml \
   crates/lpdf-hw/Cargo.toml \
   crates/lpdf-hw/src/lib.rs \
   crates/lpdf-jobs/src/queue.rs \
   crates/lpdf-worker/src/main.rs \
   /tmp/lpdf_backup_w3/
```

#### 2. Exact File-by-File Implementation Spec

##### File 1: `Cargo.toml` & `.cargo/config.toml`
Update `[profile.release]` in `Cargo.toml` and `[target.x86_64-pc-windows-msvc]` in `.cargo/config.toml` with the exact configurations from Section 2.1 and Section 2.2.

##### File 2: `crates/lpdf-hw/Cargo.toml` & `crates/lpdf-hw/src/lib.rs`
In `crates/lpdf-hw/Cargo.toml`, add:
```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.61.2", features = [
    "Win32_Foundation",
    "Win32_System_ProcessStatus",
    "Win32_System_SystemInformation",
    "Win32_System_Threading",
] }
```
In `crates/lpdf-hw/src/lib.rs`, apply the complete `GlobalMemoryStatusEx`, `K32EmptyWorkingSet`, and `WorkerQosGuard` implementation from Section 3.2.

##### File 3: `crates/lpdf-jobs/src/queue.rs` & `crates/lpdf-worker/src/main.rs`
- In `crates/lpdf-jobs/src/queue.rs`:
  - Inside `tokio::task::spawn_blocking` before calling `Self::execute_job(...)`, bind `let _qos_guard = lpdf_hw::WorkerQosGuard::enter();`.
  - Immediately after `Self::execute_job(...)` completes, call `lpdf_hw::reclaim_working_set_memory();`.
- In `crates/lpdf-worker/src/main.rs`:
  - At the start of `execute_sandboxed_request`, bind `let _qos_guard = lpdf_hw::WorkerQosGuard::enter();` (adding `lpdf-hw = { path = "../lpdf-hw" }` to `crates/lpdf-worker/Cargo.toml`).
- In `app-tauri/src-tauri/src/lib.rs`:
  - Replace hardcoded `JobManager::new(2, event_tx)` with hardware-adaptive concurrency:
    ```rust
    let hw_profile = lpdf_hw::HardwareProfile::detect();
    let job_manager = Arc::new(JobManager::new(hw_profile.concurrent_jobs(), event_tx));
    ```

#### 3. Cross-Check & Validation Gate
```bash
cargo check --workspace
cargo test -p lpdf-hw -p lpdf-jobs -p lpdf-worker
cargo xtask protocol-lint
```
All commands must exit with code `0` (`TRUE`).

#### 4. Rollback Failsafe
```bash
cp /tmp/lpdf_backup_w3/Cargo.toml Cargo.toml
cp /tmp/lpdf_backup_w3/config.toml .cargo/config.toml
cp /tmp/lpdf_backup_w3/lib.rs crates/lpdf-hw/src/lib.rs
cp /tmp/lpdf_backup_w3/queue.rs crates/lpdf-jobs/src/queue.rs
cp /tmp/lpdf_backup_w3/main.rs crates/lpdf-worker/src/main.rs
git checkout -- crates/lpdf-hw/Cargo.toml crates/lpdf-worker/Cargo.toml app-tauri/src-tauri/src/lib.rs
```

#### 5. Log Update & Git Commit Protocol
- Update `logs/CHECKPOINT.md` to `Phase W3 COMPLETE — Hardware-Adaptive Memory Governor, EcoQoS Priority, & MSVC Linker Hardening`.
- Append Phase W3 entry to `logs/PROGRESS_LOG.md`.
- Commit & push:
```bash
git add Cargo.toml Cargo.lock .cargo/config.toml crates/lpdf-hw crates/lpdf-jobs crates/lpdf-worker app-tauri/src-tauri/src/lib.rs logs/CHECKPOINT.md logs/PROGRESS_LOG.md
git commit -m "feat(windows-w3): implement GlobalMemoryStatusEx governor, EcoQoS worker priority, and MSVC security flags"
git push origin main
```

---

### PHASE W4: Tauri Windows Bundler (`nsis` + `msi`), WebView2 Hardening, & GitHub Actions CI/CD Matrix

#### 1. Pre-Phase Backup & State Verification
```bash
cat logs/CHECKPOINT.md
git status --short
mkdir -p /tmp/lpdf_backup_w4
cp app-tauri/src-tauri/tauri.conf.json /tmp/lpdf_backup_w4/
cp .github/workflows/ci.yml /tmp/lpdf_backup_w4/ 2>/dev/null || true
```

#### 2. Exact File-by-File Implementation Spec

##### File 1: `app-tauri/src-tauri/tauri.conf.json`
Update `app-tauri/src-tauri/tauri.conf.json` to include transparent/hidden-until-ready cold-start config, offline WebView2 browser args, cross-platform bundle targets (`["rpm", "nsis", "msi"]`), and the `windows` bundler configuration (`embedBootstrapper`, `bundleVcRuntime`, `nsis`, and `wix`):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "LocalPDF Studio",
  "mainBinaryName": "localpdf-studio",
  "version": "0.1.0",
  "identifier": "com.ayra.localpdf-studio",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "LocalPDF Studio",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "visible": true,
        "transparent": true,
        "backgroundColor": [15, 15, 20, 0],
        "additionalBrowserArgs": "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,Translate,OptimizationHints,MediaRouter,DialMediaRouteProvider --disable-background-networking --disable-breakpad --disable-client-side-phishing-detection --disable-component-update --disable-default-apps --disable-domain-reliability --disable-sync --metrics-recording-only --no-default-browser-check --no-first-run --no-pings"
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["rpm", "nsis", "msi"],
    "category": "Utility",
    "shortDescription": "Offline, private, enterprise-grade PDF studio and toolchain",
    "longDescription": "LocalPDF Studio is a high-performance offline PDF manipulation suite featuring native organize, edit, annotate, compress, protect, redact, OCR, and document conversion tools.",
    "fileAssociations": [
      {
        "ext": ["pdf"],
        "mimeType": "application/pdf",
        "name": "PDF Document",
        "description": "Portable Document Format",
        "role": "Editor"
      }
    ],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "linux": {
      "rpm": {
        "desktopTemplate": "desktop.template",
        "depends": [
          "libreoffice-core",
          "pandoc"
        ],
        "files": {
          "/usr/bin/localpdf": "../../target/release/localpdf",
          "/usr/bin/lpdf-worker": "../../target/release/lpdf-worker"
        }
      }
    },
    "windows": {
      "digestAlgorithm": "sha256",
      "certificateThumbprint": null,
      "timestampUrl": null,
      "tsp": false,
      "allowDowngrades": true,
      "bundleVcRuntime": true,
      "webviewInstallMode": {
        "type": "embedBootstrapper",
        "silent": true
      },
      "nsis": {
        "installMode": "both",
        "compression": "lzma",
        "displayLanguageSelector": false,
        "startMenuFolder": "LocalPDF Studio"
      },
      "wix": {
        "language": "en-US",
        "fipsCompliant": true
      }
    }
  }
}
```
*(Note: `"visible": true` is kept as the cross-platform default so Linux headless `timeout 5s ./target/release/localpdf-studio` smoke tests map the GTK window immediately, while `backgroundColor: [15, 15, 20, 0]` eliminates WebView2's white background flash on Windows.)*

##### File 2: `.github/workflows/windows-release.yml`
Create `.github/workflows/windows-release.yml` to build, test, package (`nsis` + `msi`), and smoke-test LocalPDF Studio on `windows-latest`:

```yaml
name: Windows 10/11 Enterprise Build & Smoke Gate

on:
  push:
    branches: [main]
    tags: ["v*"]
  pull_request:
    branches: [main]
  workflow_dispatch:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  windows-x64-release:
    name: Windows x64 (MSVC) Build, Gate, & Bundle
    runs-on: windows-latest
    steps:
      - name: Checkout repository with submodules
        uses: actions/checkout@v4
        with:
          submodules: recursive

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Setup Node.js 22
        uses: actions/setup-node@v4
        with:
          node-version: "22"
          cache: "pnpm"
          cache-dependency-path: app-tauri/pnpm-lock.yaml

      - name: Install Rust stable toolchain (x86_64-pc-windows-msvc)
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-pc-windows-msvc
          components: rustfmt, clippy

      - name: Cache Cargo registry and target build artifacts
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: ". -> target"

      - name: Install frontend dependencies & run strict type/unit checks
        working-directory: app-tauri
        run: |
          pnpm install --frozen-lockfile
          pnpm check
          pnpm exec vitest run

      - name: Run Rust workspace unit tests & protocol-lint
        run: |
          cargo xtask protocol-lint
          cargo test --workspace --exclude app-tauri

      - name: Build CLI and Worker binaries (Release)
        run: |
          cargo build --release -p lpdf-cli -p lpdf-worker

      - name: Build Tauri Windows Installers (NSIS .exe + WiX .msi)
        working-directory: app-tauri
        run: |
          pnpm tauri build --bundles nsis,msi

      - name: Execute 5-Second Live Windows GUI Smoke Test
        shell: pwsh
        run: |
          $exePath = "target\release\localpdf-studio.exe"
          if (-not (Test-Path $exePath)) {
            Write-Error "Binary not found at $exePath"
            exit 1
          }
          $proc = Start-Process -FilePath $exePath -PassThru
          Start-Sleep -Seconds 5
          if ($proc.HasExited) {
            Write-Error "localpdf-studio.exe crashed prematurely with exit code $($proc.ExitCode)"
            exit 1
          }
          Stop-Process -Id $proc.Id -Force
          Write-Host "SMOKE_TEST_PASSED: Process stayed alive for 5s cleanly (equivalent to EXIT_CODE=124)"

      - name: Generate SHA-256 Checksums for Windows Bundles
        shell: pwsh
        run: |
          Get-ChildItem -Path "target\release\bundle\nsis\*.exe", "target\release\bundle\msi\*.msi" | ForEach-Object {
            $hash = (Get-FileHash -Path $_.FullName -Algorithm SHA256).Hash
            "$hash  $($_.Name)" | Out-File -FilePath "target\release\bundle\SHA256SUMS-WINDOWS.txt" -Append -Encoding utf8
          }
          Get-Content "target\release\bundle\SHA256SUMS-WINDOWS.txt"

      - name: Upload Windows Release Installers
        uses: actions/upload-artifact@v4
        with:
          name: LocalPDF-Studio-Windows-x64-Installers
          path: |
            target/release/bundle/nsis/*.exe
            target/release/bundle/msi/*.msi
            target/release/bundle/SHA256SUMS-WINDOWS.txt
            target/release/localpdf.exe
            target/release/lpdf-worker.exe
          if-no-files-found: error
```

#### 3. Cross-Check & Validation Gate (Including Linux Regression Smoke Test)
```bash
cargo check --workspace
tools/sb.sh pnpm --dir app-tauri check
pnpm --dir app-tauri exec vitest run
SB_RELEASE=1 tools/sb.sh pnpm --dir app-tauri tauri build --bundles rpm
RUST_BACKTRACE=1 timeout 5s ./target/release/localpdf-studio 2>&1 ; echo "EXIT_CODE=$?"
```
Verify that `EXIT_CODE=124` and the RPM bundle builds cleanly on Fedora Linux without regression.

#### 4. Rollback Failsafe
```bash
cp /tmp/lpdf_backup_w4/tauri.conf.json app-tauri/src-tauri/tauri.conf.json
rm -f .github/workflows/windows-release.yml
```

#### 5. Log Update & Git Commit Protocol
- Update `logs/CHECKPOINT.md` to `Phase W4 COMPLETE — Windows 10/11 Port, NSIS/MSI Bundler, & CI/CD Matrix Ready`.
- Append Phase W4 entry to `logs/PROGRESS_LOG.md`.
- Commit & push:
```bash
git add app-tauri/src-tauri/tauri.conf.json .github/workflows/windows-release.yml logs/CHECKPOINT.md logs/PROGRESS_LOG.md
git commit -m "feat(windows-w4): configure NSIS/WiX Windows bundlers, offline WebView2 flags, and CI release workflow"
git push origin main
```
