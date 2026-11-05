# LocalPDF Studio v0.1.0 Release Notes

**Release:** `v0.1.0-rc5`  
**Date:** 2026-10-29  
**Target Platform:** Linux x86_64 (Fedora 40+, RHEL 9+, openSUSE)  
**Status:** RC5 Build Verified & Approved for Host Installation  

---

## 1. Release Artifact & Checksums

| Field | Value |
|---|---|
| **Package** | `LocalPDF Studio-0.1.0-1.x86_64.rpm` |
| **Path** | `target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm` |
| **Size** | `35.19 MiB` (36,898,844 bytes) |
| **SHA-256** | `748ebc961e063b441cd24abc1ea86dd5018d49159697ac364bbd3767376836b0` |

### Packaged Binaries & Payload

The RPM bundle places all executables in `/usr/bin/`:

- `/usr/bin/localpdf-studio`: High-performance Tauri v2 GUI application with Apple HIG frosted liquid glass design, global high-contrast form controls, floating toast notifications, and responsive fluid spatial layouts.
- `/usr/bin/localpdf`: Comprehensive command-line interface supporting headless document inspection, merge, split, compress, protect, watermark, OCR, and automated batch pipelines.
- `/usr/bin/lpdf-worker`: Dedicated background worker process for heavy PDF manipulation and sidecar task execution.
- `/usr/share/applications/LocalPDF Studio.desktop`: XDG desktop entry configured with `%F` for native file-manager integration.
- `/usr/share/icons/hicolor/*/apps/localpdf-studio.png`: Desktop icon assets (32x32, 128x128, 256x256@2).

---

## 2. Security & Architecture Highlights

- **Apple HIG Frosted Liquid Glass Design System (RC5)**: Complete token system with dark and light vibrancy layers, specular highlights, translucent surfaces paired with `backdrop-filter: blur(24px) saturate(180%)`, and responsive layouts.
- **Global High-Contrast Form Controls (RC5)**: Strict opaque styling on native `<option>` dropdown popups and `color-scheme` synchronization, guaranteeing zero white-on-white text across Linux WebKitGTK and desktop themes.
- **Apple-Style Floating Glass Toast System (RC5)**: Global floating notifications with spring animation physics, badge indicators, path diagnostic displays, and native event integration with background worker jobs.
- **Windowed-Mode Fluid Spatial Layout (RC5)**: Fluid auto-fit grid and flexbox wrapping across two-column views (`ConvertFromPdfView`, `ConvertToPdfView`, `OrganizeView`, `BatchQueueView`, `SettingsView`), ensuring zero right-edge clipping at 800px-1080px viewport widths.
- **`#![forbid(unsafe_code)]`**: Unconditional compile-time guarantee across core and UI application layers.
- **Native OS File & Directory Dialogs**: Cross-platform native pickers (`pick_files_dialog`, `pick_folder_dialog`, `save_file_dialog`) utilizing KDE Dolphin (`kdialog`) and GTK (`zenity`) on Linux.
- **Linux Landlock Worker Sandbox**: Restricts worker file system access strictly to required input and output staging directories.
- **Network-Isolated Sidecars (F20)**: External sidecars (e.g., LibreOffice, Pandoc) execute inside an unprivileged network namespace (`unshare -n -r`), ensuring zero external network egress.
- **True Content-Stream Redaction (F25)**: Destructive, physical content-stream removal of text and graphical vectors—never superficial visual overlays.
- **KDE Dolphin / GNOME Files `%F` Integration**: Seamless right-click "Open With..." support via `extract_pdf_args` CLI filtering and Tauri single-instance forwarding over IPC (`lpdf://open-files`).
- **Tokio Runtime Guard & Clean Startup**: Tauri's managed Tokio async runtime context (`tauri::async_runtime::handle().inner().enter()`) guarantees clean, panic-free startup with single-instance D-Bus daemon.
- **Original Document Immutability**: All operations write atomically to new files; original files remain byte-for-byte untouched.

---

## 3. End-to-End Smoke Test Verification

Tested against release binary (`target/release/localpdf`) on Linux x86_64:

| Operation | Command Executed | Result / Output | Validation Status |
|---|---|---|---|
| **Info / Inspect** | `localpdf info input1.pdf` | `Pages: 1, PDF Version: 1.4, Encrypted: No` | **PASS** ✅ |
| **Document Merge** | `localpdf merge merged.pdf input1.pdf input2.pdf` | `Merged 2 file(s) into 'merged.pdf'` | **PASS** ✅ (`qpdf --check` 0 errors, `pdfinfo` 2 pages) |
| **Compression** | `localpdf compress input1.pdf compressed.pdf` | `Compressed 'input1.pdf' with Medium profile` | **PASS** ✅ (`qpdf --check` 0 errors, valid PDF 1.5) |
| **Empty File Defense** | `localpdf info empty.pdf` | `Error: couldn't parse input` (Exit code: 1) | **PASS** ✅ (Clean error, no panic) |
| **Corrupt File Defense** | `localpdf info corrupt.pdf` | `Error: couldn't parse input` (Exit code: 1) | **PASS** ✅ (Clean error, no panic) |
| **Immutability Check** | `sha256sum input1.pdf` | `3df79d34abbca99308e79cb94461c189...` | **PASS** ✅ (Identical pre/post hash) |
| **Live GUI Startup Smoke Test** | `RUST_BACKTRACE=1 timeout 5s ./target/release/localpdf-studio` | `EXIT_CODE=124` | **PASS** ✅ (Clean execution) |

---

## 4. Installation & Verification Command

```bash
sudo dnf reinstall -y "./target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm"
```
