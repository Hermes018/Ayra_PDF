# ADR-011: Windows 10 & 11 Platform Architecture, Vibrancy, NTFS I/O, and Process Containment

- **Date:** 2026-10-30
- **Status:** ACCEPTED
- **Phase:** Phase 13.0 (Windows Platform Engineering)
- **Authors:** Principal Windows Rust & Systems Architect
- **Supersedes / Amends:** Extends ADR-001 (Core Stack), ADR-002 (Engine), ADR-003 (Svelte 5 UI), Failsafes F1–F22 (`protocol/03_FAILSAFES.md`)

---

## 1. Context & Problem Statement

LocalPDF Studio has achieved production readiness on Linux (Fedora 44 / KDE Plasma 6 Wayland, `v0.1.0-rc5`). Porting and hardening the suite for Windows 10 (1809 / Build 17763+) and Windows 11 (21H2 Build 22000 through 24H2 Build 26100+) introduces four fundamental systems-engineering challenges that differ sharply from POSIX/Linux semantics:

1. **Toolchain, Binary Mitigation, & Runtime ABI:** Windows executables must link cleanly against the Microsoft Visual C++ (MSVC) ABI required by WebView2 while enforcing enterprise PE exploit mitigations (High-Entropy ASLR, DEP, Control Flow Guard) and aggressive dead-code elimination without breaking bare Windows 10 installations.
2. **DWM Window Composition & Cold-Start Flash:** Windows 11 supports Mica (`DWMWA_SYSTEMBACKDROP_TYPE`), Windows 10 1809+ supports Acrylic (`SetWindowCompositionAttribute`), and Remote Desktop / Battery Saver / low-end GPUs disable DWM transparency altogether. Furthermore, WebView2 defaults to an opaque white background during initialization, causing a jarring white flash on dark-themed launches unless coordinated with window visibility.
3. **NTFS File Locking, Verbatim Paths, & Sidecar Zombie Processes:** Unlike Linux VFS `rename(2)` (which atomically replaces open files), NTFS returns `ERROR_SHARING_VIOLATION` (OS Error 32) or `ERROR_ACCESS_DENIED` (OS Error 5) when a PDF is inspected by Windows Defender, Search Indexer, Adobe Reader, or Edge. Additionally, `std::fs::canonicalize` on Windows prepends the `\\?\` extended-length verbatim prefix, which crashes legacy Win32 CLI parsers in LibreOffice (`soffice.exe`) and Pandoc (`pandoc.exe`). Finally, Linux `unshare -n -r` is unavailable on Windows, and spawning console binaries flashes transient `cmd.exe`/conhost windows or leaves orphan processes if the host crashes.
4. **Memory Fragmentation & UI Thread Starvation:** Heavy PDF rasterization and OCR passes on 8 GB laptops can exhaust commit charge or starve the WebView2 UI thread and system audio unless bounded by physical RAM telemetry (`GlobalMemoryStatusEx`) and scheduled under Windows 11 EcoQoS / background thread priority.

---

## 2. Binding Architectural Decisions

### Decision 1: MSVC Compiler, Linker, & PE Hardening Matrix (`x86_64-pc-windows-msvc`)

We standardize on the `x86_64-pc-windows-msvc` target with the following release profile and linker configuration:

- **Cargo Release Profile (`Cargo.toml`):**
  - `lto = "fat"` (full cross-crate link-time optimization across all 17 workspace members and `pdfcraft`).
  - `codegen-units = 1` (maximizes LLVM inlining and vectorization).
  - `panic = "abort"` (eliminates `.pdata`/`.xdata` SEH unwinding tables across non-worker boundaries; worker job boundaries retain `catch_unwind` in process isolation via `lpdf-worker.exe`).
  - `strip = "symbols"` (strips DWARF/PDB symbol tables from shipped binaries).
  - `opt-level = 3` (maximum throughput optimization for PDF parsing, rendering, and cryptography).
- **CRT Linkage Strategy:** Dynamic CRT (`-C target-feature=-crt-static`) paired with `"bundleVcRuntime": true` in `tauri.conf.json`. Static CRT (`+crt-static`) conflicts with `WebView2Loader.dll` and causes heap corruption across DLL boundaries when allocator pointers cross into COM interfaces. Bundling the redistributable VC++ runtime DLLs guarantees zero-dependency execution on clean Windows 10 1809 machines.
- **MSVC Linker Flags (`.cargo/config.toml`):**
  - `/OPT:REF` and `/OPT:ICF` (identical COMDAT folding and unreferenced section elimination).
  - `/HIGHENTROPYVA` and `/DYNAMICBASE` (64-bit High-Entropy ASLR).
  - `/NXCOMPAT` (hardware Data Execution Prevention / NX bit enforcement).
  - `/GUARD:CF` (Control Flow Guard indirect-call validation).
  - `/CETCOMPAT` (Intel CET / AMD Shadow Stack compatibility).

### Decision 2: 3-Tier DWM Window Composition & Cold-Start White-Flash Elimination

We implement a deterministic 3-tier vibrancy engine in `app-tauri/src-tauri` coupled with our Svelte 5 design token architecture:

- **Tier 1 — Windows 11 (Build $\ge$ 22000):** Apply native `window_vibrancy::apply_mica(&window, Some(is_dark))`. Synchronize dynamically at runtime whenever the user switches between Light, Dark, or System themes via a dedicated Tauri IPC command `set_window_vibrancy_theme(dark: bool)`.
- **Tier 2 — Windows 10 (1809 Build 17763 to 21999):** Automatic `.or_else()` fallback to `window_vibrancy::apply_acrylic(&window, Some(rgba))` using theme-calibrated RGBA tints (`(18, 18, 24, 215)` for Dark; `(245, 245, 250, 220)` for Light) matching `--glass-bg-canvas` in `app-tauri/src/app.css`.
- **Tier 3 — DWM Disabled / RDP / Battery Saver / Low-End GPU Failsafe:** Query Windows transparency availability (`DwmIsCompositionEnabled` and `HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\EnableTransparency`). When disabled or when Tier 1/Tier 2 return `Err`, clear all native backdrop effects (`clear_mica`, `clear_acrylic`, `clear_blur`) and emit a `vibrancy-fallback` state so the frontend renders opaque high-contrast surface tokens (`--bg-canvas-opaque`), preventing black or invisible window bugs.
- **Cold-Start White-Flash Elimination:**
  - Configure `"visible": false` and `"transparent": true` in `app-tauri/src-tauri/tauri.conf.json`.
  - Pre-seed WebView2 controller default background color to transparent `(0, 0, 0, 0)` (`COREWEBVIEW2_COLOR { A: 0, R: 0, G: 0, B: 0 }`) during window setup.
  - Reveal and focus the main window (`window.show()`, `window.set_focus()`) from the Svelte `onMount` lifecycle via an IPC command `show_main_window` after the initial DOM frame and theme tokens are painted.

### Decision 3: NTFS Atomic I/O Retry Wrapper, Verbatim Path Sanitization, & Win32 Job Object Containment

To satisfy Failsafes F1, F3, F17, F20, F21, and F22 on Windows:

- **NTFS Atomic Write with Exponential Backoff (`crates/lpdf-fs-safety`):**
  - Write output payload to `.{stem}_{ext}_{uuid}.tmp` in the same target directory, followed by `File::sync_all()` (`FlushFileBuffers`).
  - Execute atomic replacement via Win32 `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH`.
  - If `MoveFileExW` fails with `ERROR_SHARING_VIOLATION` (32) or `ERROR_ACCESS_DENIED` (5) (commonly caused by transient Windows Defender real-time scans or Search Indexer), retry up to 6 times with exponential backoff ($25\text{ ms}, 50\text{ ms}, 100\text{ ms}, 200\text{ ms}, 400\text{ ms}, 800\text{ ms}$; total $1.575\text{ s}$).
  - If the target remains locked after all retries (e.g., open in Adobe Acrobat Reader with an exclusive read lock), never discard the user's output: automatically write to a safe sibling recovery path `{stem}_saved_{timestamp}.{ext}` and surface a clear warning toast via IPC.
- **Extended-Length Path (`\\?\`) Sanitization:**
  - Internal Rust `std::fs` operations support paths $> 260$ characters via `\\?\` verbatim prefixes returned by `std::fs::canonicalize()`.
  - Before passing any path to `soffice.exe` or `pandoc.exe`, strip the `\\?\` prefix using `dunce::simplified(path)` (when path length $< 260$) or convert `\\?\UNC\` appropriately, preventing sidecar CLI argument parser failures. Also format LibreOffice `-env:UserInstallation=file:///C:/...` with forward slashes as required by UNO URL parsing on Windows.
- **Zero-Console & Win32 Job Object Sandbox (`crates/lpdf-engine-pdfcraft`, `crates/lpdf-jobs`):**
  - Apply `.creation_flags(0x08000000)` (`CREATE_NO_WINDOW`) via `std::os::windows::process::CommandExt` on all child `Command` spawns (`lpdf-worker.exe`, `soffice.exe`, `pandoc.exe`, PowerShell dialogs) so console windows never flash on screen.
  - Encapsulate every spawned worker and sidecar process inside a Win32 Job Object (`CreateJobObjectW`, `SetInformationJobObject` with `JobObjectExtendedLimitInformation`, `AssignProcessToJobObject`) enforcing:
    1. `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (`0x00002000`): Kernel-guaranteed immediate termination of all child and grandchild processes (`soffice.bin`) if the parent process exits, panics, or is killed via Task Manager.
    2. `JOB_OBJECT_LIMIT_JOB_MEMORY` (`0x00000200`): Hard 1 GiB (`1_073_741_824` bytes) committed memory ceiling per sidecar job in compliance with ADR-001 and Rule DO-20.
    3. `JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION` (`0x00000400`): Suppresses the Windows Error Reporting (`WerFault.exe`) modal crash dialog if a sidecar faults on a hostile document.

### Decision 4: Hardware-Adaptive Memory Governor & Windows 11 EcoQoS Scheduling

- **Win32 Physical Memory Telemetry (`crates/lpdf-hw`):**
  - Query total and available physical memory directly via `GlobalMemoryStatusEx` (`MEMORYSTATUSEX`) on Windows (with `sysinfo` retained as cross-platform fallback) to classify hardware into Low ($\le 8\text{ GB}$), Medium ($> 8\text{ GB}$ to $< 16\text{ GB}$), and High ($\ge 16\text{ GB}$) tiers.
  - Dynamically size the Rayon global/per-job thread pool (`rayon::ThreadPoolBuilder`) and cap concurrent raster page buffers so 8 GB laptops never page-thrash.
  - Invoke `K32EmptyWorkingSet(GetCurrentProcess())` after completing heavy batch conversions or OCR jobs to return unused committed pages to the Windows Standby List immediately.
- **Process Priority & EcoQoS (`crates/lpdf-jobs`, `crates/lpdf-worker`):**
  - Worker threads executing batch jobs invoke `SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL)` and enable Windows 11 EcoQoS (`SetThreadInformation` with `ThreadPowerThrottling` and `THREAD_POWER_THROTTLING_EXECUTION_SPEED`) so background PDF crunching never degrades WebView2 60fps/120fps scrolling or system audio.

---

## 3. Compliance with Protocol Rules (`protocol/02_RULES_DOS_DONTS.md`)

- **Rule DON'T-3 (`#![forbid(unsafe_code)]` default; exceptions need an ADR):**
  - Core domain crates (`lpdf-core-types`, `lpdf-tools-*`, `lpdf-jobs`) remain `#![forbid(unsafe_code)]`.
  - Win32 FFI calls (`CreateJobObjectW`, `SetInformationJobObject`, `MoveFileExW`, `GlobalMemoryStatusEx`, `SetThreadInformation`, `DwmIsCompositionEnabled`) are strictly isolated inside `#[cfg(target_os = "windows")]` submodules with `#![allow(unsafe_code)]` explicitly authorized by this ADR (ADR-011), using Microsoft's official `windows-sys` crate (already present in `Cargo.lock`).
- **Rule DON'T-1 & Failsafe F9 (100% Offline Guarantee):**
  - WebView2 is locked down with `--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --disable-background-networking --disable-sync --disable-translate --disable-client-side-phishing-detection --no-pings` in `additionalBrowserArgs`, alongside strict Tauri v2 CSP (`default-src 'self'; connect-src ipc: http://ipc.localhost`).

---

## 4. Consequences

- **Positive:** Zero white-flash cold starts; native Windows 11 Mica and Windows 10 Acrylic aesthetics with bulletproof RDP/Battery-Saver fallbacks; immune to NTFS transient file-lock crashes; zero zombie LibreOffice/Pandoc processes; predictable RAM footprint across 8 GB to 64 GB machines.
- **Negative / Trade-offs:** Win32 FFI requires `#[cfg(target_os = "windows")]` conditional compilation modules that must be validated via cross-checking on Linux (`cargo check`) and native execution on `windows-latest` CI runners.
