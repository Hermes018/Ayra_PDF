# LocalPDF Studio v0.1.0 Release Notes

**Release:** `v0.1.0-rc2`  
**Date:** 2026-10-15  
**Target Platform:** Linux x86_64 (Fedora 40+, RHEL 9+, openSUSE)  
**Status:** RC2 Build Verified & Approved for Host Installation  

---

## 1. Release Artifact & Checksums

| Field | Value |
|---|---|
| **Package** | `LocalPDF Studio-0.1.0-1.x86_64.rpm` |
| **Path** | `target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm` |
| **Size** | `35.18 MiB` (36,888,674 bytes) |
| **SHA-256** | `e36150a2afee39c60cd57f513c42beb32c56f2ea8a84d38e26494069da130003` |

### Packaged Binaries & Payload

The RPM bundle places all executables in `/usr/bin/`:

- `/usr/bin/localpdf-studio`: High-performance Tauri v2 GUI application with dark/light themes, drag-and-drop studio, and real-time page reordering.
- `/usr/bin/localpdf`: Comprehensive command-line interface supporting headless document inspection, merge, split, compress, protect, watermark, OCR, and automated batch pipelines.
- `/usr/bin/lpdf-worker`: Dedicated background worker process for heavy PDF manipulation and sidecar task execution.
- `/usr/share/applications/LocalPDF Studio.desktop`: XDG desktop entry configured with `%F` for native file-manager integration.
- `/usr/share/icons/hicolor/*/apps/localpdf-studio.png`: Desktop icon assets (32x32, 128x128, 256x256@2).

---

## 2. Security & Architecture Highlights

- **`#![forbid(unsafe_code)]`**: Unconditional compile-time guarantee across core and UI application layers.
- **Linux Landlock Worker Sandbox**: Restricts worker file system access strictly to required input and output staging directories.
- **Network-Isolated Sidecars (F20)**: External sidecars (e.g., LibreOffice, Pandoc) execute inside an unprivileged network namespace (`unshare -n -r`), ensuring zero external network egress.
- **True Content-Stream Redaction (F25)**: Destructive, physical content-stream removal of text and graphical vectors—never superficial visual overlays.
- **KDE Dolphin / GNOME Files `%F` Integration**: Seamless right-click "Open With..." support via `extract_pdf_args` CLI filtering and Tauri single-instance forwarding over IPC (`lpdf://open-files`).
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

---

## 4. Host Installation Instructions

Install the package directly using `dnf` on Fedora / RHEL-compatible distributions:

```bash
sudo dnf install "./target/release/bundle/rpm/LocalPDF Studio-0.1.0-1.x86_64.rpm"
```

To run the application:
```bash
# Launch GUI Studio
localpdf-studio

# Or launch CLI
localpdf --help
```
