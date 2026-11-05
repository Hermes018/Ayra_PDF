# ADR-019 — Android as a first-class target (tablet-first) — **PROPOSED**

> **Status: DRAFT (incomplete; Android track paused 2026-11-05, resumes Sunday).** Intended final status: PROPOSED (user approval needed before any Android code).
> - **Sections:** §1–§12 are all written as a first pass. Not done: a word-count trim; an independent review; no build or on-device test has been run.
> - **Still UNVERIFIED:**
>   - WebView loads the local origin without INTERNET.
>   - `http://tauri.localhost` counts as a secure context for `getUserMedia`/`getCoalescedEvents`.
>   - Real stylus tilt/pressure on device.
>   - Multi-window on Android.
>   - Android print-to-file for HTML→PDF.
>   - Tauri auto-adding `http://lpdf.localhost` to the CSP.
>   - Android font matching.
>   - JDK minimum for AGP 9.3.
>   - All component sizes marked "est." (NDK, cmdline-tools, Rust targets, Gradle caches, Android cargo artefacts, APK/AAB size, OCR model sizes).
>   - Tier budget numbers.
>   - `am send-trim-memory` on API 34+.
>   - x86_64 `aws-lc-sys` needing cmake/libclang (inferred from the builder source, not built).

- **Date:** 2026-11-05 · **Author:** Builder B (track/android) · **Status:** DRAFT (see above).
- **Relates to:** ADR-004 (AI), ADR-005 (offline scope), ADR-006 (reconstruction), ADR-015 (unsafe allow-list), ADR-017 (no HTTP client), ADR-020 (cfg modules behind traits; capability module), protocol 02/03/06/07, phase 14.
- **Citation keys:** `[R]` = `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`, `[G]` = `~/.cargo/git/checkouts/`. Paths without a prefix are relative to `master/`. Web sources were fetched on 2026-11-05. **UNVERIFIED** marks anything not confirmed from source or official docs.

---

## 1. Context, decision summary, options

**Context.** LocalPDF Studio is a Tauri v2 + Svelte 5 + Rust app. It currently runs on desktop only. Phase 14 asks for an Android build with content-URI/SAF streaming, tight memory tiers, a touch UI and packaging. Three current desktop assumptions do not hold on Android:
- Untrusted work runs in child processes (F3/F19/F20).
- Outputs are written by temp file + rename in the destination directory (F1).
- External sidecars (LibreOffice, Pandoc) are available.

**Decision (proposed).**
1. Use **Tauri v2 mobile** with the same Rust core crates and the same Svelte engines (`app-tauri/src/lib/core/*`), under a tablet-first layout shell.
2. Put every platform difference behind four Rust traits (fs, jobs, memory, window effects) plus one frontend capability module (ADR-020).
3. On Android, jobs run **in-process** behind a hardened panic/memory boundary. A separate `:worker` process is a later, optional phase.
4. Sidecar features are **desktop-only**. Their tiles show greyed out on Android (07 §1.3).
5. The release manifest has **no INTERNET permission**.
6. OCR and AI models stay **user-imported** (DON'T-19).

| Option | Pros | Cons |
|---|---|---|
| **A. Tauri v2 mobile (chosen)** | Reuses all Rust crates, the IPC contract, the Svelte engines and the CSP. One codebase. Plugins already ship Android Kotlin (dialog, fs). | WebView differences. JNI/Kotlin plugin work for SAF and memory signals. No sidecars. `tauri` pulls `reqwest` on mobile (§8). |
| B. Native Kotlin/Compose UI + Rust via JNI (uniffi) | Best platform feel, full SAF/PrintManager access. | Rewrites the whole UI (109 Studio items). Two UIs to keep in parity. Contradicts ADR-003. |
| C. Flutter/React Native + Rust FFI | Mature mobile toolkits. | New language and runtime. React is banned (ADR-003). Large bundle. |
| D. PWA in Chrome | No install. | No native file access guarantees. Service worker and fetch model conflict with DON'T-1/18. Rust would need wasm. |
| E. Do not support Android | No cost. | Drops phase 14 and the user's tablet/stylus use case. |

## 2. Platform capability matrix

Legend: **S** supported · **D** desktop-only (tile greyed on Android, with reason) · **G** degraded (how) · **?** unknown until verified on device. "Windows S" means the code is present but unverified on Windows hardware (ADR-015).

### 2.1 Tool columns (07_FEATURE_MAP)

| Family | Linux | Windows | Android |
|---|---|---|---|
| **Organize**: Merge, Split, Remove/Extract pages, Organize grid | S | S | **G**. Single-output tools are S via the SAF picker and `ACTION_CREATE_DOCUMENT`. Tools with many outputs (split, one file per page) need a ZIP output or a tree grant (`ACTION_OPEN_DOCUMENT_TREE`, own plugin, §3). |
| Organize: Scan to PDF (7.4a, images from files) | S | S | S. Images come from the picker. Camera capture goes through the WebView (§2.2). |
| Scan to PDF 7.4b (SANE/WIA) | stretch, D | stretch, D | D |
| **Optimize**: Compress, Repair | S | S | S. Pure Rust (pdfcraft, lopdf). |
| Optimize: OCR (ocrs) | S (models must be present) | S | **G**. The user imports the two model files, which are copied into app-private storage. Memory-heavy, so Tier L runs one page at a time. Tesseract (ext) = D. See §7. |
| **Convert to PDF**: JPG/PNG/… → PDF | S | S | S |
| Word/PowerPoint/Excel → PDF (LibreOffice sidecar) | S if installed | S | **D**. Child processes plus `execve` from writable dirs are blocked for targetSdk ≥ 29 ("Removed execute permission for app home directory", developer.android.com/about/versions/10/behavior-changes-10). No LibreOffice build either. |
| HTML → PDF (`HtmlPrinter`) | stretch | stretch | **?** No Android print-to-file path is verified. Greyed. |
| **Convert from PDF**: PDF→JPG, PDF/A, Text/HTML/RTF | S | S | S |
| PDF→Word/PPT/Excel, native `lpdf-layout` path (ADR-006) | S | S | S. Quality report as on desktop. |
| PDF→Office, LibreOffice sidecar path (phase 9.2) | S | S | D |
| PDF→Markdown | S (native / Pandoc) | S | **G**. Native heuristic path only. Pandoc = D. |
| **Edit**: Rotate, Page numbers, Watermark, Crop, Edit, Forms | S | S | **G** (fonts only). Pure Rust. System fonts come from Android's set, so font matching can differ (UNVERIFIED). |
| **Security**: Unlock, Protect, Redact, Compare | S | S | S |
| Security: Sign (PAdES, PKCS#12) | S | S (+ cert store via rustls-cng) | S, with build risk. `pdfcraft-sign` → `aws-lc-rs` → `aws-lc-sys` needs the NDK C toolchain (§10). The .p12 file arrives via SAF. No OS keystore. |
| **Intelligence**: Summarizer Basic (extractive) | S | S | S |
| Summarizer LLM / Translate PDF | ? | ? | **?** `crates/lpdf-ai-worker/Cargo.toml` has no inference backend, despite the checklist note "candle-core". No sandboxed process exists on Android. RAM is tight. Greyed until a later phase. |
| **Extras**: Markdown → PDF / MD↔DOCX (Pandoc sidecar, phase 9.3) | S | S | D, unless the Typst path from ADR-010 is implemented natively. |
| Workflows / Pipelines | S | S | G. Only steps whose tools are S on Android. Sidecar steps are greyed. |

### 2.2 Studio checklist sections (docs/DRAWBOARD_FEATURE_CHECKLIST.md)

| Section (items) | Linux | Windows | Android |
|---|---|---|---|
| Canvas & navigation (6.1–6.3, 6.5–6.7, 6.10, 6.11, 6.13, 6.14) | S | S | S. Pinch and pan already use `pointerType === 'touch'` (`SpatialCanvas.svelte:946`, `touch-action:none` at :2341). |
| 6.4 Split view / multi-window | S | S | **G**. The in-window `SplitLayout.svelte` is S. Extra OS windows (`capabilities/studio-split.json`) are treated as unsupported on Android (UNVERIFIED in Tauri source). |
| 6.8 Dockable toolbar, 6.9 Radial menu | S | S | **G**. The radial flyouts open on `onpointerenter` (`RadialMenu.svelte:218,268`), which is hover-dependent. They need a press-drag-release or tap path. |
| 6.12 Keyboard shortcuts | S | S | G. Hardware keyboard only. Every command must also be reachable by touch. |
| Inking (1.1–1.13) incl. pressure/tilt/palm rejection | S | S | S/?. PointerEvent `pressure`/`tiltX`/`tiltY` are in WebView Android 55+, `twist` 57+, `getCoalescedEvents` 58+, `altitudeAngle`/`azimuthAngle` 86+ (MDN BCD `api/PointerEvent.json`, entries marked "mirror", so WebView was not tested separately). Whether a given stylus actually reports tilt needs a device: **?** |
| Shapes / hatching (1.14–1.17, 2.15, 2.17) | S | S | S |
| CAD measurement (4.1–4.11, 7.4 takeoff export) | S | S | S. The CSV is written via SAF. |
| Insert / media: image (2.9), stamps, signature | S | S | S |
| 2.10 Webcam capture, 8.1 Audio notes (`getUserMedia`/`MediaRecorder`) | S | S | S, with permissions. wry maps WebView `RESOURCE_VIDEO_CAPTURE`/`AUDIO_CAPTURE` to `CAMERA`/`RECORD_AUDIO` runtime requests (`[R]wry-0.57.0/src/android/kotlin/RustWebChromeClient.kt:60-112`). We must declare both permissions. A secure context is needed (`http://tauri.localhost`; UNVERIFIED that the WebView treats it as secure). |
| 2.16 LaTeX, 8.10 Handwriting (PARTIAL) | PARTIAL | PARTIAL | PARTIAL (same code) |
| Structural text edit (2.8) | S | S | G (fonts, as for Edit) |
| Ledger / workspace: 7.10 recents, tags | S | S | **G**. Recents survive a reboot only for persistable grants (§3). |
| 8.7 Watch folder | S (`read_dir`, `studio.rs:1398`) | S | **G**. On-demand scan of a tree URI the user granted. No background watching. |
| 7.7 Print with/without markup | S | S | G. Export, then share or save. `craft-print` uses CUPS/lp on unix (`[G]craft-libs-*/e11f9d9/crates/print/src/lib.rs:299`), which fails gracefully on Android. Android PrintManager is a later plugin. |
| 9.10 Hardware tiers, 9.11 Privacy dashboard | S | S | S. Android memory signals (§4). The dashboard adds an "INTERNET permission absent" row. |
| 8.2/8.5/8.6/8.8/8.13/8.14 (DEFERRED) | none | none | none |

On adoption, every checklist item that is S/G on Android gets `android` added to its `[platforms: …]` tag. D items keep `linux, windows`.

## 3. File I/O on Android (SAF)

**What the plugins really do (verified):**
- **Dialog plugin.** `showFilePicker` uses `Intent.ACTION_GET_CONTENT` (`[R]tauri-plugin-dialog-2.8.1/android/src/main/java/DialogPlugin.kt:62-63`, with a `TODO: ACTION_OPEN_DOCUMENT ??`). It returns `content://` URI strings (:116-121) and never calls `takePersistableUriPermission`.
- **Save dialog.** Uses `ACTION_CREATE_DOCUMENT` (:204).
- **Folder pickers.** `pick_folder`/`pick_folders` are `#[cfg(desktop)]` (`src/lib.rs` ~:618-645), so there is no folder picker on Android.
- **Fs plugin.** `Fs::open(FilePath::Url(content://…), OpenOptions)` calls Kotlin `getFileDescriptor`, which runs `contentResolver.openAssetFileDescriptor(uri, mode)` and `detachFd()`. Rust gets a real `std::fs::File` back (`[R]tauri-plugin-fs-2.6.0/src/android.rs:37-80`, `android/src/main/java/FsPlugin.kt:40-70`). Modes: `r`, `w`, `wt`, `rw`, `rwt`, `wa` (`src/lib.rs:370-381`).
  - The plugin calls `unimplemented!()` when no fd comes back (android.rs:78), which panics. We must call it inside our panic boundary or check for the null fd first.
- **Persistent grants.** These need `ACTION_OPEN_DOCUMENT`/`CREATE_DOCUMENT`/`OPEN_DOCUMENT_TREE` plus `takePersistableUriPermission`. Without that, a grant lasts until reboot, and it is lost if the document moves (developer.android.com/training/data-storage/shared/documents-files). `ACTION_CREATE_DOCUMENT` cannot overwrite; it appends "(1)" (same page).

**Consequences.** We need a small own plugin, `tauri-plugin-lpdf-android` (Kotlin + Rust, registered via `PluginApi::register_android_plugin`, `[R]tauri-2.12.2/src/plugin/mobile.rs:208`). Its commands:
- `openDocuments(mime[], multiple)`: `ACTION_OPEN_DOCUMENT`, then take persistable read permission.
- `createDocument(name, mime)`: take persistable read/write permission.
- `openTree()`: for the watch folder and batch output.
- `listTree(uri)`: `DocumentsContract` children.
- `deleteDocument(uri)`: cleanup.
- `queryMeta(uri)`: display name, size, flags.
- `releasePersisted(uri)`
- `memoryState()` and the pressure channel (§4).

The dialog plugin stays for message boxes only.

**Android backend for lpdf-fs-safety** (our guarantees, mapped):

| Guarantee | Desktop today | Android design |
|---|---|---|
| New file by default (DO-5) | Save dialog plus collision suffix | `createDocument` always makes a new document. The provider resolves collisions. |
| Original untouched | Read-only open | Inputs are opened with mode `r` only, and are never handed to writers. |
| F1 atomic | temp in same dir → rename (`lpdf-fs-safety/src/lib.rs:229-285`) | SAF has no rename-over. Steps: (1) stage the output in `app_cache_dir()/jobs/<id>/` (`[R]tauri-2.12.2/src/path/android.rs:152-161`); (2) fsync; (3) **verify** by re-opening it as a PDF (F6, plus F25 for redaction); (4) open the destination fd with `wt`, run `std::io::copy` in a bounded buffer, `sync_all`, then compare size and SHA-256 by reading the destination back; (5) on failure, call `deleteDocument(dest)`. Result: the destination is either complete and verified, or deleted. |
| F2 journal | (planned) | The journal records the dest URI **before** the copy. Startup reconcile deletes half-written destinations and GCs the staging dirs. |
| Overwrite (F8) | backup + replace | **Disabled in A-phases** (open question Q3). It would mean backup + `rwt` in place, which is non-atomic. |
| F5 disk preflight | free space at dest | Free space of the staging volume (`statvfs` on the cache dir). The destination provider's free space is often unknown (`COLUMN_SIZE`/root info is optional), so the user is told when it is unknown. |
| Large files, DON'T-6 | — | Never send bytes over IPC. The engine reads from a `File`/fd; it may `memmap2` the staged copy. Thumbnails need the planned `lpdf://` scheme (06 §3), which on Android is served at `http://lpdf.localhost` (`[R]tauri-2.12.2/src/protocol/mod.rs:28-38`; register via `register_asynchronous_uri_scheme_protocol`, `src/app.rs:2335`). Bulk binary command results use `tauri::ipc::Response` (`src/ipc/mod.rs:190`). |
| Cancel cleanup | temp removal | The job's staging dir is removed. A dest created but not committed is deleted. Partial copies are never left behind. |

**Blocking bugs for Android in current code (they also violate desktop rules):**
1. `read_file_bytes` returns `Vec<u8>` (a JSON number array) of the whole file to pdf.js (`app-tauri/src-tauri/src/lib.rs:88`, used in `PdfViewer.svelte:32` and `GridEditor.svelte:34`). This is DON'T-6 and it will OOM on tablets.
2. Settings and logs use `directories::ProjectDirs` (lib.rs:20-28). On Android `dirs-sys` `home_dir()` returns `None` unless `HOME` is set (`[R]dirs-sys-0.4.1/src/lib.rs:33-42`), so settings fall back to a relative `settings.toml`. Use Tauri's `PathResolver` instead.
3. 22 uses of `std::env::temp_dir()` in crates and app. On Android this resolves to `/data/local/tmp` when `TMPDIR` is unset (Rust std `library/std/src/sys/paths/unix.rs:420-426`), which apps cannot write. Pass staging dirs explicitly.
4. The dialogs are `kdialog`/`zenity` child processes, cfg-gated per OS (lib.rs:277-575). Android needs the plugin path.
5. Minor: `FormsView.svelte` imports `@tauri-apps/plugin-dialog` but no capability grants `dialog:*` (`capabilities/default.json`).

## 4. Jobs, isolation and memory

**Desktop reality.**
- Jobs run **in-process** on `tokio::task::spawn_blocking` (`crates/lpdf-jobs/src/queue.rs:785`).
- `spawn_sandboxed_worker` (`unshare -n -r` at :1012-1019) has no caller outside `lpdf-jobs`.
- The release profile sets `panic = "abort"` (`Cargo.toml` `[profile.release]`). That means the `JoinError` mapping (`queue.rs:815`) and the OCR `catch_unwind` (`lpdf-engine-pdfcraft/src/lib.rs:470`) catch nothing in release builds.
- So F3/F13 are not met on desktop either. This ADR does not fix desktop, but the jobs trait (§6) makes it fixable.

**Android options.**

| Option | Notes |
|---|---|
| Child process + `exec` of a worker binary | Blocked for files in writable app dirs (W^X, API 29+ link above). Shipping the worker as `lib*.so` in `nativeLibraryDir` and exec'ing it is a known workaround, but it is fragile and outside the official docs. **Rejected.** |
| `<service android:process=":worker">` | Officially supported. A name starting with `:` creates a private process (developer.android.com/guide/topics/manifest/service-element). It needs a Kotlin Service that loads a second Rust `.so` entry point, plus Binder/Messenger IPC and fd passing. Gives crash isolation (an LMK or abort kills only the worker). **Phase A6, optional.** |
| `android:isolatedProcess="true"` | "has no permissions of its own"; it can only talk through the Service API (same page). It cannot open SAF URIs itself, so the parent must pass fds. Strongest sandbox. Later. |
| **In-process (A1–A5)** | The Android app sandbox (per-app UID, SELinux) isolates us from other apps, and the absent INTERNET permission blocks sockets (§8). Within the app, a hostile PDF that triggers a panic is contained only if `panic = "unwind"`. An abort or OOM kills the app, and the F2 journal plus relaunch recovery handle that. |

**In-process hardening (required for Android, recommended for desktop).**
- A dedicated Cargo profile `[profile.android-release]` that inherits `release` but sets `panic = "unwind"`.
- `catch_unwind` at the single job boundary, in `JobIsolation::run`.
- Per-page render watchdog and cancel token (F4).
- Bounded buffers.
- Memory **pre-checks** before an allocation-heavy step, because Rust aborts on alloc failure and the abort cannot be caught.
- Concurrency of 1 on Tier L.

**Memory signals.**
- `ActivityManager.getMemoryClass()` is the per-app **Java heap** class in MB (developer.android.com/reference/android/app/ActivityManager). It does not bound native (Rust) memory, so we use it only as a device-class hint.
- Use `getMemoryInfo()` (`availMem`, `threshold`, `lowMemory`, `totalMem`) and `isLowRamDevice()` ("generally … 1GB or less").
- `onTrimMemory`: since API 34 only `TRIM_MEMORY_UI_HIDDEN` (20) and `TRIM_MEMORY_BACKGROUND` (40) are delivered. The `RUNNING_*`, `MODERATE` and `COMPLETE` levels are deprecated in API 35, and `onLowMemory` is never called on 34+ (developer.android.com/reference/android/content/ComponentCallbacks2).
- Consequence: there is **no foreground "running low" signal on modern Android**. We must poll `getMemoryInfo` (for example every 2 s while a job runs) and compare `availMem` with `threshold`.

**Bridge.**
- Tauri's Kotlin `Plugin` has lifecycle hooks (`onPause`/`onResume`/`onStop`, `[R]tauri-2.12.2/mobile/android/src/main/java/app/tauri/plugin/Plugin.kt:62-151`) but no `onTrimMemory`.
- Our plugin calls `activity.registerComponentCallbacks(...)` in `load()`.
- Pressure is pushed to Rust through a `tauri::ipc::Channel` created on the Rust side with a closure (`[R]tauri-2.12.2/src/ipc/channel.rs:378`) and handed to Kotlin, which calls `Channel.send` (`…/plugin/Channel.kt:25`). `memoryState()` is the pull path.
- Rust stores the latest value in `lpdf-hw` (pure, atomics only).

**Android tier budgets (initial; calibrate in A4).**

| Tier | Detect | Jobs | Threads | Thumb cache | Job memory budget |
|---|---|---|---|---|---|
| L | `isLowRamDevice` or totalMem ≤ 4 GB | 1 | 1–2 | 32 MB | min(25 % availMem, availMem − threshold − 256 MB, 512 MB) |
| M | 4–8 GB | 1 | 2–4 | 96 MB | same formula, cap 1.5 GB |
| H | > 8 GB | 2 | cores/2 | 192 MB | same formula, cap 3 GB |

Response to pressure:
- On `UI_HIDDEN`: drop the thumbnail and tile caches.
- On `BACKGROUND`: also pause queued jobs at the next chunk.
- On `availMem < threshold × 1.5`: switch the running job to chunked/slow mode and tell the user (DON'T-7).

## 5. UI

- **Same engines.** The Svelte engines in `app-tauri/src/lib/core/*.svelte.ts` (ink, snap, measure, selection, shape/hatch, viewport, command, insert) are reused unchanged. Only the layout shell differs: `src/lib/shells/DesktopShell.svelte` and `src/lib/shells/TabletShell.svelte`, chosen once at boot from the capability module.
- **Tablet shell rules.**
  - Touch targets ≥ 48 dp.
  - No hover-only affordances. Replace the radial `onpointerenter` flyouts with press-drag-release plus a tap fallback. Toast pause (`ToastContainer.svelte:23` `onmouseenter`) also pauses on touch.
  - Every keyboard shortcut has a visible control.
  - Side panels become bottom sheets on narrow widths.
  - Palm rejection: when `pointerType === 'pen'` has been seen, finger input pans and zooms only (already the model, `SpatialCanvas.svelte:946-973`).
  - Stylus samples via `getCoalescedEvents` (:1201), with `getPredictedEvents` as an optional low-latency preview (WebView 77+, MDN BCD).
- **Capability module (ADR-020), single source.**
  - Rust: `crates/lpdf-core-types/src/platform.rs` defines `enum Platform`, `enum Availability { Supported, Degraded{reason_key}, DesktopOnly, Unavailable{reason_key} }`, and `fn feature_availability(platform, probes) -> BTreeMap<FeatureId, Availability>`, exported to TS with ts-rs.
  - It is served by the planned `tool_availability` command (06 §3). Runtime probes such as "LibreOffice found" or "OCR models imported" refine the static table.
  - Frontend: **`app-tauri/src/lib/platform/capabilities.svelte.ts`** with:
    ```ts
    export const platform: { os: 'linux'|'windows'|'macos'|'android'|'ios'; formFactor: 'desktop'|'tablet'|'phone'; };
    export function availability(id: FeatureId): Availability;   // drives greyed tiles + reason text (07 §1.3)
    export function can(id: FeatureId): boolean;                  // Supported | Degraded
    export async function refresh(): Promise<void>;               // re-run probes (after model import etc.)
    ```
  - Components may not test the OS anywhere else. protocol-lint bans `navigator.userAgent`, `navigator.platform` and `@tauri-apps/plugin-os` outside `lib/platform/`.
  - `feature-map-check` cross-checks the Rust table against the checklist `[platforms:]` tags.
  - There is no tool registry yet (`registry.ts` from 07 §1.4 does not exist). It should consume `availability()`.

## 6. Rust traits per concern (ADR-020)

Core crates stay Tauri-free (DO-3). Android implementations that need JNI live in `app-tauri` or the plugin crate and are injected.

```rust
// crates/lpdf-fs-safety/src/store/{mod.rs, local.rs (linux/macos), windows.rs, android.rs}
pub enum DocRef { Path(PathBuf), Uri(String) }            // content:// on Android
pub trait UriOpener: Send + Sync {                          // impl in app-tauri via tauri-plugin-fs Fs::open
    fn open(&self, uri: &str, mode: OpenMode) -> io::Result<std::fs::File>;
    fn delete(&self, uri: &str) -> io::Result<()>;
}
pub trait DocumentStore: Send + Sync {
    fn open_read(&self, src: &DocRef) -> Result<std::fs::File>;
    fn stage(&self, job: &JobId, name_hint: &str) -> Result<Staged>;          // temp path + journal entry
    fn commit(&self, staged: Staged, dest: &DocRef, verify: &dyn Fn(&Path) -> Result<()>) -> Result<Committed>;
    fn discard(&self, staged: Staged);
    fn free_space(&self, at: &DocRef) -> Result<Option<u64>>;
}
// crates/lpdf-jobs/src/isolation/{mod.rs, linux.rs, windows.rs, inproc.rs}
pub trait JobIsolation: Send + Sync {
    fn run(&self, req: &JobRequest, ctx: &JobCtx) -> Result<JobOutput>;    // ctx: cancel, progress, budget, store
}
//   ChildProcess (linux: unshare -n + Landlock worker; windows: Job Object) | InProcess (android; catch_unwind)
// crates/lpdf-hw/src/memory/{mod.rs, sysinfo.rs, android.rs}
pub trait MemoryGovernor: Send + Sync {
    fn snapshot(&self) -> MemSnapshot;                 // total, avail, threshold, low_ram
    fn tier(&self) -> Tier;
    fn job_budget(&self) -> u64;
    fn pressure(&self) -> Pressure;                    // Normal | Elevated | Critical | Hidden | Background
}
// android.rs: AtomicU64/AtomicU8 fields set by app-tauri from the plugin channel (no JNI in lpdf-hw).
// app-tauri/src-tauri/src/platform/{mod.rs, linux.rs, windows.rs, android.rs}
pub trait WindowEffects { fn apply_theme(&self, w: &tauri::Window, theme: &str) -> Result<(), String>; }  // android: Ok(()) no-op
```

**Scattered cfg code that moves behind these traits:**
- `lpdf-fs-safety/src/lib.rs:8-190`: `win_fs`, `win_sandbox`, `ChildProcessSandbox` → `store/windows.rs`, `isolation/windows.rs`.
- `lpdf-fs-safety/src/lib.rs:196-285`: path helpers and `atomic_write` cfgs → `store/*`.
- `lpdf-jobs/src/queue.rs:988-1036` (`spawn_sandboxed_worker`) → `isolation/linux.rs`/`windows.rs`.
- `lpdf-jobs/src/lib.rs:25-60` (sysinfo budget) → `MemoryGovernor`.
- `lpdf-worker/src/main.rs:9-94` (Landlock) stays in the worker bin but is called through `isolation/linux.rs`.
- `lpdf-engine-pdfcraft/src/office_sidecar.rs:9-32` and `pandoc_sidecar.rs:11-34,125-132` (binary names, `unshare`) → a `SidecarRunner` inside the jobs concern, compiled only `#[cfg(not(target_os = "android"))]`.
- `app-tauri/src-tauri/src/lib.rs:277-575` (dialogs) → `platform::*` pickers. Lines 653-675 (vibrancy) → `WindowEffects`.
- Plugin registration of `tauri-plugin-single-instance`/`window-state` (lib.rs:705-711) must become `#[cfg(desktop)]`. Both crates compile to nothing on Android (`#![cfg(not(any(target_os = "android", target_os = "ios")))]`, `[R]tauri-plugin-single-instance-2.5.2/src/lib.rs:16`, `[R]tauri-plugin-window-state-2.5.0/src/lib.rs:11`), so **app-tauri does not compile for Android today**.

Note: `target_os = "linux"` is false on Android, so the existing Landlock/`unshare` cfgs already compile out there.

## 7. Bundle, ABIs and models

**Google Play limits** (compressed download size; support.google.com/googleplay/android-developer/answer/9859372):

| Item | Limit |
|---|---|
| Base module | 500 MB |
| Each feature module | 500 MB |
| Each asset pack | 1.5 GB |
| Base + install-time packs | 4 GB |
| On-demand / fast-follow packs | 30 GB |
| App total | 34 GB |
| Legacy APK | 100 MB |

Our DON'T-9 budget of 1 GB is stricter than Play's total, so it governs. Target: AAB base ≤ 150 MB per ABI split.

**Size drivers** (all UNVERIFIED until A2 measures them):
- `libapp_tauri_lib.so` per ABI (LTO + strip already in the release profile; try `opt-level="s"` for the Android profile).
- hayro/pdfcraft/rten code size.
- Embedded fonts if `CRAFT_FONTS_DIR` is set at build time (`[G]pdfcraft-*/75360d1/crates/fonts/build.rs:1-25`): CJK/Arabic fonts. Keep it unset or measure.
- Frontend: about 3.5 MB (`app-tauri/build` 2.1 MB + `static` 1.4 MB incl. pdf.js).
- AndroidX/Kotlin runtime.

**ABIs:**
- `arm64-v8a` (devices) and `x86_64` (emulator; this machine's AVD is x86_64) via `abiFilters` (developer.android.com/ndk/guides/abis).
- Drop `armeabi-v7a`/`x86` unless the user asks.
- The Play 16 KB page-size requirement applies to apps targeting API 35+. Updates without 16 KB support are blocked from 2027-02-01. NDK r28+ aligns to 16 KB by default, and AGP ≥ 8.5.1 is needed for packaging (developer.android.com/guide/practices/page-sizes). The Tauri template uses AGP 9.3.1 (`tauri-cli/templates/mobile/android/build.gradle.kts`), so pin NDK ≥ r28.

**OCR models.**
- pdfcraft finds models through `PDFCRAFT_MODELS`, then dirs beside the executable, then `CARGO_MANIFEST_DIR/../../assets/models` (`[G]pdfcraft-*/75360d1/crates/ocr/src/lib.rs:60-87`). It loads them with `rten::Model::load_file(path)` (:115-119), and the engine caches one global instance (`crates/engine/src/ocr.rs:81-90`).
- **There is no runtime download.** Models are fetched only by pdfcraft's dev command `cargo xtask models` (`xtask/src/assets.rs:371-383`), which is not part of our runtime. pdfcraft's own release packages **bundle** them, which DON'T-19 forbids for us.
- On Android there is no sensible "beside exe" path. Plan:
  1. The user imports the two files via SAF (`text-detection.onnx`, `text-recognition.onnx`, SHA-256 from `ATTRIBUTION.toml:3360-3382`).
  2. We copy them to `app_data_dir()/models/ocrs/` and verify the hash.
  3. We show the CC-BY-SA-4.0 attribution (protocol 01 §6).
  4. We set `PDFCRAFT_MODELS` once at startup, before any thread spawns. `std::env::set_var` is safe in edition 2021, but racy with concurrent getenv.
  5. Better: an upstream API that takes `Models::in_dir(dir)` directly.
- Play Asset Delivery would count as bundling, so it is **not** used. Model file sizes are UNVERIFIED.

## 8. Network and privacy

- **INTERNET permission.** The Tauri app template declares exactly one permission, `android.permission.INTERNET` (`tauri-cli/templates/mobile/android/app/src/main/AndroidManifest.xml`, dev branch). The library manifests of `tauri`, `tauri-plugin-dialog`, `-fs` and `-opener` declare none (`[R]…/android/src/main/AndroidManifest.xml`). INTERNET "Allows applications to open network sockets" (Manifest.permission).
- **Removing it.** Add `gen/android/app/src/release/AndroidManifest.xml` containing `<uses-permission android:name="android.permission.INTERNET" tools:node="remove"/>`. Build-type manifests have the highest merge priority, and `tools:node="remove"` drops the element (developer.android.com/build/manage-manifests). Debug keeps INTERNET because `tauri android dev` loads `devUrl` over the network. The template already sets `usesCleartextTraffic=false` for release, `true` for debug (`app/build.gradle.kts`).
- **UNVERIFIED:** that the WebView loads `http://tauri.localhost` (intercepted by wry, no socket) without INTERNET. This is the first A1 on-device test.
- **HTTP client in the Android tree** (lead-verified and re-checked):
  - `tauri` 2.12.2 has `reqwest` 0.13 (`json`, `stream`, `default-features=false`) as a normal dependency on Android/iOS (`[R]tauri-2.12.2/Cargo.toml:325-331`).
  - `rustls` 0.23 is listed there too, but **optional** and only on with tauri's `rustls-tls` feature (:333-337). It is absent from our Android tree.
  - All uses in `src/protocol/tauri.rs` are `#[cfg(all(dev, mobile))]` (lines 16-195, dev-server proxy).
  - `dev` = no `custom-protocol` feature (`[R]tauri-2.12.2/build.rs:250-255`).
  - `cargo tree --target aarch64-linux-android -p app-tauri -i reqwest` shows `tauri → reqwest`, and `hyper` comes with it. Desktop targets show none.
  - So release builds compile the crate but do not call it. This conflicts with the letter of ADR-017's "no HTTP client in the tree".
  - Proposal: (a) a `cargo deny` ban list for HTTP crates with a scoped `wrappers = ["tauri"]` exception for `reqwest`, recorded in an ADR-017 addendum; (b) gate that release Android builds enable `custom-protocol` (tauri-cli does this) and that the APK has no INTERNET permission, so the OS blocks sockets even if the code ran; (c) file an upstream request to make the dependency dev-only.
  - Note: `deny.toml` currently has an empty `[bans] deny = []` (lines 187-193). The F9 "cargo-deny bans network crates" claim is not enforced on any platform yet.
- **Telemetry and CSP.** No telemetry. CSP stays unchanged: `connect-src ipc: http://ipc.localhost` already matches Android, because IPC is served at `http://ipc.localhost` there (`protocol/mod.rs:33`). A future `lpdf://` thumbnail scheme becomes `http://lpdf.localhost` on Android and Windows. `img-src` must list it explicitly; that Tauri injects it automatically is UNVERIFIED.
- **No other permissions** beyond `CAMERA` and `RECORD_AUDIO` (requested at runtime, only when the user starts webcam or audio notes). No storage permissions are needed with SAF (same SAF doc).

## 9. Toolchain and this machine

**Required** (v2.tauri.app/start/prerequisites):
- Android SDK Platform, Platform-Tools, Build-Tools, Command-line Tools and NDK (Side by side), installed through Android Studio.
- `JAVA_HOME` = Android Studio's `jbr`.
- `ANDROID_HOME` and `NDK_HOME`.
- `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`. We only need `aarch64` and `x86_64`.

The page names no NDK or JDK version. Template facts: compile/targetSdk 37, Java 1.8 source/target, AGP 9.3.1, Kotlin 2.2.10. Default `minSdkVersion` is 24 (`[R]tauri-utils-2.10.1/src/config.rs:3690`). JDK minimum for AGP 9.3 is UNVERIFIED; the installed JBR is 25.

**This machine (measured 2026-11-05):**
- 35 GB free of 938 GB (97 %).
- 31 GB RAM (about 15 GB available while working).
- i5-12400, 12 threads.
- `/dev/kvm` present, user in the `kvm` group.

| Component | Status here | Size |
|---|---|---|
| Android Studio (snap 2026.2.2, JBR 25.0.4) | installed | — |
| `~/Android/Sdk` (build-tools 36.0.0, platform android-36.1, platform-tools, emulator) | installed | 9.6 GB total, measured (emulator 821 MB; build-tools, platforms and platform-tools under 0.5 GB) |
| System image android-37.1 `google_apis_playstore_ps16k` x86_64 (16 KB pages) | installed | 8.2 GB, measured (included in the 9.6 GB) |
| AVD `Pixel_7_Pro` (2 GB RAM, 10 GB data partition) | exists | 9.1 GB, measured |
| NDK r28+ | **missing** | UNVERIFIED, est. 2–3 GB |
| cmdline-tools | missing | UNVERIFIED, est. ≤ 0.2 GB |
| Rust std for aarch64 + x86_64 android | missing | UNVERIFIED, est. 0.3–0.5 GB |
| Gradle wrapper + AGP 9.3 caches (`~/.gradle` does not exist yet) | missing | UNVERIFIED, est. 1.5–3 GB |
| Cargo target artefacts for Android, per ABI, debug | — | UNVERIFIED. The desktop `target/debug` is **45 GB** today, so an Android debug tree could add 10–25 GB per ABI |
| Emulator recommendation | — | "16 GB RAM … 16 GB disk space" (developer.android.com/studio/run/emulator) |

**Verdict:**
- **RAM:** fine for emulator + Gradle + `CARGO_BUILD_JOBS=2`.
- **CPU/KVM:** fine.
- **Disk:** the binding constraint. NDK + Gradle + Rust targets fit (about 5–7 GB), but Android cargo artefacts in the shared `master/target` do not fit safely next to the 45 GB desktop debug tree.
- The emulator is already installed, so it is feasible for x86_64 smoke tests. However, `aws-lc-sys` 0.45 has pregenerated bindings for `aarch64-linux-android` but **not** `x86_64-linux-android` (`[R]aws-lc-sys-0.45.0/builder/main.rs:915-935`). The x86_64 emulator build would fall back to the CMake builder (:808-811), which needs `cmake` and likely libclang/bindgen; neither is installed.
- The emulator also has no real pen pressure or tilt.
- **Recommendation:** a USB arm64 tablet with a stylus is the primary test device. The emulator is secondary, for 16 KB and layout checks.

**What to free before A1** (user decision; ADR-020 says "never `cargo clean`"):
- Prune stale artefacts from `master/target/debug` (45 GB), for example by removing old incremental dirs only.
- Or give Android builds `CARGO_PROFILE_DEV_DEBUG=0` plus a size-capped separate target dir.
- Wipe the AVD's user data (part of the 9.1 GB).
- Remove disabled snap revisions.
- Goal: ≥ 60 GB free before the first full Android build.

## 10. Gate additions (desktop gate unchanged)

**Offline `cargo tree --locked --offline --target aarch64-linux-android -e normal,build`** (run in the main checkout; the worktree lacks the pdfcraft submodule):

| Crate | Crates | Native / sys build deps | Expected `cargo check` result |
|---|---|---|---|
| lpdf-core-types / -engine-port / -fs-safety / -hw / -control-bus (current `ANDROID_CORE_CRATES`, `xtask/src/main.rs:157-163`) | 24 / 25 / 25 / 34 / 18 | none | pass once the Rust target is installed (no NDK needed) |
| lpdf-engine-fallback, lpdf-ai-worker, lpdf-tools-convert, -markdown, -optimize | 85 / 29 / 26 / 26 / 26 | none | expected pass. **Add these to the list now** |
| lpdf-engine-pdfcraft, lpdf-engine-switcher, lpdf-jobs, lpdf-worker | ~342–357 | `aws-lc-sys` (+ cc, cmake, pkg-config), via `pdfcraft-sign` | fails without the NDK (build scripts run even for `check`). aarch64 should pass with NDK clang. x86_64 needs cmake/bindgen |
| app-tauri | 514 | the above + `jni-sys`, `ndk-sys`, `dirs-sys` | **fails today**: single-instance/window-state items are missing on Android (§6), plus the NDK requirement |

**Proposed gate steps:**
1. **A0 (no installs):** extend `ANDROID_CORE_CRATES` with the five pure crates above. It stays SKIPPED until `rustup target add aarch64-linux-android`.
2. **A1:** `cargo check --target aarch64-linux-android -p lpdf-jobs -p lpdf-engine-pdfcraft -p app-tauri` with NDK env. `lpdf-worker` is excluded on Android (no child processes).
3. **A2:** `cargo tauri android build --apk --target aarch64` through `tools/build-lock.sh`, then:
   - (a) the merged release manifest has **no** `android.permission.INTERNET` (check `build/outputs/logs/manifest-merger-release-report.txt` or `apkanalyzer manifest permissions`);
   - (b) `zipalign -c -P 16 -v 4` passes (16 KB);
   - (c) APK/AAB size ≤ budget;
   - (d) `cargo deny --target aarch64-linux-android check bans` with the HTTP ban list plus the `wrappers=["tauri"]` exception;
   - (e) protocol-lint: no `Vec<u8>` whole-file command returns, no OS sniffing outside `lib/platform/`.
4. **A3+:** on-device instrumentation smoke test (`adb`): open, merge, save via SAF; kill during copy, then confirm the journal reconcile deleted the partial output; open under memory pressure (`adb shell am send-trim-memory <pkg> BACKGROUND`, UNVERIFIED on API 34+).

## 11. Phased plan

| Phase | Scope | Gate |
|---|---|---|
| **A0** | This ADR approved. Extend `ANDROID_CORE_CRATES`. Add `android` to the platform enum in the capability table (Rust) with all values UNKNOWN. No installs. | desktop gate + SKIPPED android check |
| **A1** | Install the Rust targets + NDK r28+ (user approval, disk freed). Refactor behind the traits: `WindowEffects`, `#[cfg(desktop)]` plugins, `PathResolver` dirs, explicit staging dirs. `tauri android init` into `android/packaging` (ADR-016 overlay). Release manifest without INTERNET. | `cargo check` aarch64 on all crates except the worker. Hello-world APK launches on device without INTERNET |
| **A2** | `DocumentStore` (desktop + Android), own plugin (OPEN/CREATE_DOCUMENT, persisted grants, delete, tree). Replace `read_file_bytes` with an fd/`lpdf.localhost` streaming path (also fixes desktop DON'T-6). | open → merge → save on device; kill-during-copy test; APK manifest/16 KB/size checks |
| **A3** | `JobIsolation::InProcess` with `panic="unwind"` Android profile, watchdog, cancel. Journal reconcile. | injected-panic test leaves the UI alive; hostile corpus subset on device |
| **A4** | `MemoryGovernor` Android + plugin channel. Tier calibration on one low and one mid device. | pressure tests; no LMK kill on the 500-page scroll at Tier L |
| **A5** | `TabletShell`, capability module wired into the greyed tiles, hover-free radial menu, stylus checks, CAMERA/RECORD_AUDIO flows, OCR model import. Checklist tags gain `android`. | screenshot diffs (tablet viewport), a11y touch audit, stylus on-device checklist |
| **A6 (optional)** | `:worker` Service process (or `isolatedProcess` with fd passing) for crash isolation. | crasher corpus kills the worker only |
| **A7** | Packaging: AAB, signing, Play listing or sideload APK, SBOM. | release gate |

**Risks:**
- `aws-lc-sys` cross-build (PAdES). Mitigation: aarch64 pregenerated; x86_64 needs cmake.
- `panic=abort` leaves no in-process isolation. Mitigation: unwind profile + journal.
- Hostile PDFs abort the app (OOM or abort cannot be caught).
- SAF provider quirks: `w` vs `wt` truncation, unknown sizes, cloud-virtual files (`FLAG_VIRTUAL_DOCUMENT`).
- Stylus tilt not reported by the device.
- WebView needing INTERNET for the local origin (UNVERIFIED).
- Disk space.
- pdfcraft reads whole files (06 §6) on low-RAM tablets.

## 12. Open questions for the user

1. **Sideload APK only, or Google Play** (signing, Play policies, 16 KB deadline 2027-02-01)?
2. **Test hardware:** which Android tablet and stylus (S-Pen?) can you connect over USB? Is emulator-only acceptable for A1?
3. **Overwrite mode on Android:** keep it off (non-atomic on SAF)?
4. **Accept the `reqwest`-via-`tauri` exception** (scoped `wrappers=["tauri"]` ban + INTERNET-less manifest), or block Android until upstream makes it dev-only?
5. **Disk:** may we prune `master/target/debug` (45 GB) or use a separate size-capped Android target dir, contrary to "never `cargo clean`"?
6. **Minimum Android version:** Tauri's default `minSdk` 24, or 29+ (simplifies SAF/W^X assumptions)?
7. **x86_64 emulator builds:** worth installing `cmake` + libclang for `aws-lc-sys`, or arm64 device only?
8. **Phase A6** (separate worker process): required for parity with F3, or is in-process + journal acceptable on Android?
