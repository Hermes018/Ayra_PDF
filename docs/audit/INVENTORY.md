# Enterprise Architectural Audit: V1 Inventory

**Date:** 2026-10-13  
**Auditor:** Staff-Level Software Engineer & Enterprise Security Auditor  
**Scope:** LocalPDF Studio Workspace (Static Analysis Audit)  

---

## PART 1 — Factual Repository Inventory

### 1. Workspace Members & Crate Metrics
Source: [Cargo.toml:1-6](file:///home/ayra/localpdf-studio-protocol/Cargo.toml#L1-L6)

| Crate | Directory Path | Rust LOC (`wc -l`) | Test Count (`#[test]`, `#[tokio::test]`) | `tests/` Directory Present |
|---|---|---|---|---|
| `xtask` | `xtask` | 319 | 0 | No |
| `lpdf-ai-worker` | `crates/lpdf-ai-worker` | 529 | 7 | Yes |
| `lpdf-cli` | `crates/lpdf-cli` | 2,875 | 23 | Yes |
| `lpdf-core-types` | `crates/lpdf-core-types` | 397 | 1 | No |
| `lpdf-engine-fallback` | `crates/lpdf-engine-fallback` | 423 | 0 | No |
| `lpdf-engine-pdfcraft` | `crates/lpdf-engine-pdfcraft` | 5,756 | 43 | Yes |
| `lpdf-engine-port` | `crates/lpdf-engine-port` | 536 | 3 | No |
| `lpdf-engine-switcher` | `crates/lpdf-engine-switcher` | 2,755 | 22 | Yes |
| `lpdf-fs-safety` | `crates/lpdf-fs-safety` | 244 | 5 | Yes |
| `lpdf-hw` | `crates/lpdf-hw` | 73 | 0 | No |
| `lpdf-jobs` | `crates/lpdf-jobs` | 2,725 | 6 | Yes |
| `lpdf-tools-convert` | `crates/lpdf-tools-convert` | 15 | 1 | No |
| `lpdf-tools-edit` | `crates/lpdf-tools-edit` | 15 | 1 | No |
| `lpdf-tools-markdown` | `crates/lpdf-tools-markdown` | 15 | 1 | No |
| `lpdf-tools-optimize` | `crates/lpdf-tools-optimize` | 15 | 1 | No |
| `lpdf-tools-organize` | `crates/lpdf-tools-organize` | 15 | 1 | No |
| `lpdf-tools-security` | `crates/lpdf-tools-security` | 15 | 1 | No |
| `lpdf-worker` | `crates/lpdf-worker` | 4 | 0 | No |
| `app-tauri` | `app-tauri/src-tauri` | 311 | 0 | No |
| **Total** | | **16,038** | **114** | |

---

### 2. Implemented Tools vs Reference Feature Map
Source: [Sidebar.svelte:6-17](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/components/Sidebar.svelte#L6-L17), [app-tauri/src/lib/views/](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/), compared against [protocol/07_FEATURE_MAP.md:22-82](file:///home/ayra/localpdf-studio-protocol/protocol/07_FEATURE_MAP.md#L22-L82).

| Reference Column | Protocol 31 Tools | UI Status | Implementation Location | Notes / Discrepancies |
|---|---|---|---|---|
| **ORGANIZE PDF** | Merge PDF | Implemented | [OrganizeView.svelte:28](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OrganizeView.svelte#L28) | Multi-file merge |
| | Split PDF | Partial | [OrganizeView.svelte:31](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OrganizeView.svelte#L31) | Delete / Extract page ranges implemented; split-by-size/bookmarks missing |
| | Remove pages | Implemented | [OrganizeView.svelte:34](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OrganizeView.svelte#L34) | Deep-link / direct delete action |
| | Extract pages | Implemented | [OrganizeView.svelte:31](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OrganizeView.svelte#L31) | Extract to new PDF |
| | Organize PDF (Grid) | Partial | [GridEditor.svelte](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/components/GridEditor.svelte) | Basic thumbnail grid; worker session missing |
| | Scan to PDF | **MISSING** | NOT FOUND | No scanner integration or image-import scanner view in Organize |
| **OPTIMIZE PDF** | Compress PDF | Implemented | [OptimizeView.svelte:20](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OptimizeView.svelte#L20) | Low/Medium/High compression profiles |
| | Repair PDF | Implemented | [OptimizeView.svelte:36](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OptimizeView.svelte#L36) | Cross-reference table repair |
| | OCR PDF | Implemented | [OptimizeView.svelte:53](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OptimizeView.svelte#L53) | OCR text injection |
| **CONVERT TO PDF** | JPG to PDF | Implemented | [ConvertToPdfView.svelte:115](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertToPdfView.svelte#L115) | Image to PDF |
| | Word to PDF | Implemented | [ConvertToPdfView.svelte:38](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertToPdfView.svelte#L38) | LibreOffice sidecar + Structural Docx |
| | PowerPoint to PDF | Implemented | [ConvertToPdfView.svelte:52](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertToPdfView.svelte#L52) | LibreOffice sidecar |
| | Excel to PDF | Implemented | [ConvertToPdfView.svelte:45](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertToPdfView.svelte#L45) | LibreOffice sidecar |
| | HTML to PDF | Implemented | [ConvertToPdfView.svelte:128](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertToPdfView.svelte#L128) | Local HTML file convert |
| **CONVERT FROM PDF** | PDF to JPG | Implemented | [ConvertFromPdfView.svelte:92](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L92) | PNG render & embedded images export |
| | PDF to Word | Implemented | [ConvertFromPdfView.svelte:38](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L38) | Structural docx & sidecar |
| | PDF to PowerPoint | Implemented | [ConvertFromPdfView.svelte:57](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L57) | Sidecar pptx conversion |
| | PDF to Excel | Implemented | [ConvertFromPdfView.svelte:50](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L50) | Sidecar xlsx conversion |
| | PDF to PDF/A | Implemented | [ConvertFromPdfView.svelte:85](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L85) | Conformance preflight & rewrite |
| | PDF to Markdown | Implemented | [ConvertFromPdfView.svelte:64](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L64) | Markdown export via Pandoc |
| **EDIT PDF** | Rotate PDF | **MERGED** | [OrganizeView.svelte:37](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/OrganizeView.svelte#L37) | Relocated to OrganizeView instead of EditView |
| | Page numbers | **MISSING** | NOT FOUND | Missing from UI and backend jobs |
| | Watermark | Implemented | [EditView.svelte:22](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/EditView.svelte#L22) | WatermarkDocument job |
| | Crop PDF | **MISSING** | NOT FOUND | Missing from UI and backend jobs |
| | Edit PDF | Partial | [EditView.svelte:48,82,212](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/EditView.svelte#L48) | Sticky note/Highlight; canvas is placeholder |
| | PDF Forms | Implemented | [FormsView.svelte:41](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/FormsView.svelte#L41) | Form fill & flattening |
| **PDF SECURITY** | Unlock PDF | Implemented | [SecurityView.svelte:33](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/SecurityView.svelte#L33) | Password removal |
| | Protect PDF | Implemented | [SecurityView.svelte:18](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/SecurityView.svelte#L18) | AES-256 encryption & permission flags |
| | Sign PDF | Implemented | [FormsView.svelte:62](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/FormsView.svelte#L62) | PKCS#12 digital signature (placed in FormsView) |
| | Redact PDF | Implemented | [SecurityView.svelte:49](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/SecurityView.svelte#L49) | Semantic content-stream redaction |
| | Compare PDF | **MISSING** | NOT FOUND | Missing from both UI and backend engine |
| **PDF INTELLIGENCE** | AI Summarizer | Implemented | [AiSummarizeView.svelte:35](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/AiSummarizeView.svelte#L35) | TextRank extractive + local GGUF mode |
| | Translate PDF | **MISSING (UI)** | NOT FOUND in UI | Backend has [translator.rs](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-engine-pdfcraft/src/translator.rs); UI tile missing |

- **Missing Tools (4):** Scan to PDF, Page numbers, Crop PDF, Compare PDF (+ Translate PDF in UI).
- **Merged / Relocated Tools (2):** Rotate PDF (moved to OrganizeView), Sign PDF (moved to FormsView).
- **Extra Tools in UI (6):** Markdown to Word/HTML, Word to Markdown, HTML to Markdown, Batch Markdown Folder Conversion ([ConvertToPdfView.svelte:60-85](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertToPdfView.svelte#L60-L85)), Batch Folder to Markdown, Plain Text (.txt), Rich Text Format (.rtf) ([ConvertFromPdfView.svelte:72,113,120](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/views/ConvertFromPdfView.svelte#L72)).

---

### 3. Tauri Commands & Capabilities Analysis

#### Registered `#[tauri::command]` Handlers
Defined in [app-tauri/src-tauri/src/lib.rs:271-286](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L271-L286):
1. `submit_job` — [lib.rs:29](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L29)
2. `submit_batch` — [lib.rs:35](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L35)
3. `cancel_job` — [lib.rs:45](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L45)
4. `get_job_status` — [lib.rs:51](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L51)
5. `list_jobs` — [lib.rs:57](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L57)
6. `clear_finished_jobs` — [lib.rs:63](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L63)
7. `retry_job` — [lib.rs:69](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L69)
8. `list_pipeline_presets` — [lib.rs:78](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L78)
9. `read_file_bytes` — [lib.rs:84](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L84)
10. `get_settings` — [lib.rs:90](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L90)
11. `save_settings` — [lib.rs:97](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L97)
12. `get_recent_logs` — [lib.rs:105](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L105)
13. `export_diagnostics` — [lib.rs:151](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L151)
14. `import_local_ai_model` — [lib.rs:207](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/src/lib.rs#L207)

#### Capabilities & Security Configuration
- Configuration: [app-tauri/src-tauri/tauri.conf.json](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/tauri.conf.json)
  - `productName`: `"app-tauri"` (Discrepancy: should be `localpdf-studio`)
  - `bundle.targets`: `"all"` (Discrepancy: should target `"rpm"` per Fedora spec)
  - `bundle.fileAssociations`: **None defined** (Discrepancy: `.pdf` MIME association missing)
  - `security.csp`: `"default-src 'self'; connect-src ipc: http://ipc.localhost"` (Strict; compliant with offline policy)
- Capabilities: [app-tauri/src-tauri/capabilities/default.json](file:///home/ayra/localpdf-studio-protocol/app-tauri/src-tauri/capabilities/default.json)
  - Permissions: `"core:default"`, `"opener:default"`.
  - Wildcard check: **Clean.** No `$HOME/**`, `**`, or broad filesystem scopes permitted.

---

### 4. CLI Subcommands
Source: [crates/lpdf-cli/src/lib.rs:29-58](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L29-L58)

The CLI uses a custom hand-written argument parser (`clap` is not in dependencies):
- `info`: Display page count, encryption, metadata ([lib.rs:31](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L31))
- `merge`: Combine multiple PDFs ([lib.rs:32](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L32))
- `split`: Split into individual pages or cuts ([lib.rs:33](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L33))
- `rotate`: Rotate entire document or pages ([lib.rs:34](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L34))
- `extract`: Extract specific pages ([lib.rs:35](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L35))
- `delete`: Remove specified pages ([lib.rs:36](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L36))
- `compress`: Size reduction profiles ([lib.rs:37](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L37))
- `repair`: Rebuild xref table ([lib.rs:38](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L38))
- `ocr`: OCR text injection ([lib.rs:39](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L39))
- `watermark`: Text watermarking ([lib.rs:40](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L40))
- `flatten-comments`: Flatten review markup ([lib.rs:41](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L41))
- `flatten-forms`: Flatten form fields ([lib.rs:42](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L42))
- `protect`: Password encryption & permissions ([lib.rs:43](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L43))
- `unlock`: Password removal ([lib.rs:44](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L44))
- `redact`: Rectangular semantic redaction ([lib.rs:45](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L45))
- `pdfa`: Archival PDF/A conversion ([lib.rs:46](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L46))
- `to-text`: Extract plain text ([lib.rs:47](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L47))
- `to-html`: Render pages to HTML ([lib.rs:48](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L48))
- `to-images`: Render pages to PNG images ([lib.rs:49](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L49))
- `pipeline`: Multi-step workflow presets and execution ([lib.rs:50](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L50))
- `md` / `markdown`: Pandoc sidecar Markdown operations ([lib.rs:51](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L51))
- `summarize`: Extractive zero-model summary ([lib.rs:52](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L52))
- `translate`: Experimental block translation ([lib.rs:53](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-cli/src/lib.rs#L53))

---

### 5. Xtask Subcommands Analysis
Source: [xtask/src/main.rs:13-26](file:///home/ayra/localpdf-studio-protocol/xtask/src/main.rs#L13-L26)

- Subcommands: `protocol-lint`, `gate`, `test-offline`, `fuzz` (`hostile`), `perf-check`, `sbom`, `a11y`, `soak`.
- **`feature-map-check` Status:** **NOT IMPLEMENTED.** Attempting to run `cargo xtask feature-map-check` prints `"Unknown command: feature-map-check"` and exits with code 1 ([xtask/src/main.rs:22-25](file:///home/ayra/localpdf-studio-protocol/xtask/src/main.rs#L22-L25)).
- **`protocol-lint` Functionality:**
  - Scans `crates/**/*.rs` for five banned string patterns: `".unwrap("`, `".expect("`, `"todo!("`, `"unimplemented!("`, `"panic!("` ([xtask/src/main.rs:34-40](file:///home/ayra/localpdf-studio-protocol/xtask/src/main.rs#L34-L40)). Skips lines starting with `//` or paths containing `tests/`.
  - Scans `Cargo.toml` files for banned network dependencies: `reqwest`, `hyper`, `ureq`, `isahc`, `surf` ([xtask/src/main.rs:84](file:///home/ayra/localpdf-studio-protocol/xtask/src/main.rs#L84)).
  - Does NOT check for other protocol rules (e.g. SvelteKit usage, frontend network APIs, bundle size, etc.).

---

## PART 2 — Deep Architectural QA

### Q1: SvelteKit Usage & Mode
**Answer:** `app-tauri` uses `@sveltejs/kit` with `@sveltejs/adapter-static` in SPA mode. `routes/+layout.ts` sets `ssr = false`, but does NOT declare `prerender`.  
**Evidence:**  
- [app-tauri/package.json:23-24](file:///home/ayra/localpdf-studio-protocol/app-tauri/package.json#L23-L24):
```json
"@sveltejs/adapter-static": "^3.0.10",
"@sveltejs/kit": "^2.65.1",
```
- [app-tauri/svelte.config.js:12-14](file:///home/ayra/localpdf-studio-protocol/app-tauri/svelte.config.js#L12-L14):
```javascript
kit: {
  adapter: adapter({ fallback: "index.html" }),
}
```
- [app-tauri/src/routes/+layout.ts:5](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/routes/+layout.ts#L5):
```typescript
export const ssr = false;
```

---

### Q2: PDF.js Loading & Parsing in Webview
**Answer:** Loaded globally in HTML and worker configured in Svelte. The webview directly parses raw PDF bytes in JavaScript. Apache License 2.0 notices are included in file headers.  
**Evidence:**  
- [app-tauri/src/app.html:9](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/app.html#L9):
```html
<script src="/pdf.min.js"></script>
```
- [app-tauri/src/lib/components/PdfViewer.svelte:29-31](file:///home/ayra/localpdf-studio-protocol/app-tauri/src/lib/components/PdfViewer.svelte#L29-L31):
```typescript
pdfjs.GlobalWorkerOptions.workerSrc = '/pdf.worker.min.js';
const loadingTask = pdfjs.getDocument({ data: new Uint8Array(bytes) });
```
- [app-tauri/static/pdf.min.js:6-7](file:///home/ayra/localpdf-studio-protocol/app-tauri/static/pdf.min.js#L6-L7):
```javascript
* Copyright 2023 Mozilla Foundation
* Licensed under the Apache License, Version 2.0 (the "License");
```

---

### Q3: Harness Code Compilation Scope
**Answer:** `fuzz.rs`, `hostile.rs`, `perf.rs`, `sbom.rs`, and `soak.rs` are declared as public unconditional modules in `lpdf-engine-switcher/src/lib.rs`. They are NOT behind `#[cfg(test)]` and are compiled directly into the production library artifact.  
**Evidence:**  
- [crates/lpdf-engine-switcher/src/lib.rs:15-19](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-engine-switcher/src/lib.rs#L15-L19):
```rust
pub mod fuzz;
pub mod hostile;
pub mod perf;
pub mod sbom;
pub mod soak;
```
- [crates/lpdf-engine-switcher/Cargo.toml:1-10](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-engine-switcher/Cargo.toml#L1-L10):
```toml
[package]
name = "lpdf-engine-switcher"
```

---

### Q4: Tool Logic in `lpdf-tools-*` Crates
**Answer:** All six `lpdf-tools-*` crates are completely empty cargo templates containing only `pub fn add(left: u64, right: u64) -> u64`. They have zero domain logic and are not used by the application.  
**Evidence:**  
- [crates/lpdf-tools-convert/src/lib.rs:2-4](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-tools-convert/src/lib.rs#L2-L4) (and identical in edit, markdown, optimize, organize, security):
```rust
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}
```

---

### Q5: Location of Missing Crates' Functionality
**Answer:** PDF→Word, summarizer, translator, and sidecar bridging are implemented directly as submodules within `crates/lpdf-engine-pdfcraft/src/`.  
**Evidence:**  
- [crates/lpdf-engine-pdfcraft/src/lib.rs:13-16](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-engine-pdfcraft/src/lib.rs#L13-L16):
```rust
pub mod office_sidecar;
pub mod pandoc_sidecar;
pub mod summarizer;
pub mod translator;
```
- Module files:
  - `office_sidecar.rs` (526 LOC): Headless LibreOffice bridge for PDF→Word, Word/Excel/PPT→PDF.
  - `pandoc_sidecar.rs` (860 LOC): Pandoc bridge for Markdown transformations.
  - `summarizer.rs` (508 LOC): TextRank extractive algorithm + `lpdf-ai-worker` dispatcher.
  - `translator.rs` (236 LOC): In-place block translator and companion report generator.

---

### Q6: Sidecar Binary Discovery and Process Execution
**Answer:** Sidecars are searched in target-specific binary paths, well-known locations, and system `PATH`. They execute via `std::process::Command::new` with an explicit argument array and a 60-second child watchdog. However, NO network namespace restriction (`unshare -n`) is wrapped around the execution.  
**Evidence:**  
- [crates/lpdf-engine-pdfcraft/src/office_sidecar.rs:180-184,207,214](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-engine-pdfcraft/src/office_sidecar.rs#L180-L184):
```rust
let mut cmd = std::process::Command::new(&binary_path);
cmd.arg(format!("-env:UserInstallation={}", profile_url));
cmd.arg("--headless");
// ...
let timeout = std::time::Duration::from_secs(60);
if start_time.elapsed() > timeout { let _ = child.kill(); let _ = child.wait(); }
```
- [crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs:124,143,150](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-engine-pdfcraft/src/pandoc_sidecar.rs#L124):
```rust
let mut cmd = std::process::Command::new(binary);
let timeout = std::time::Duration::from_secs(60);
if start_time.elapsed() > timeout { let _ = child.kill(); }
```

---

### Q7: Worker Sandbox Security Isolation
**Answer:** Neither Landlock nor seccomp is applied. `crates/lpdf-worker/src/main.rs` is an unlinked placeholder printing `"Hello, world!"`.  
**Evidence:**  
- [crates/lpdf-worker/src/main.rs:1-4](file:///home/ayra/localpdf-studio-protocol/crates/lpdf-worker/src/main.rs#L1-L4):
```rust
#![forbid(unsafe_code)]
fn main() {
    println!("Hello, world!");
}
```
- Search for `landlock` / `seccomp` across workspace returned 0 hits.

---

### Q8: Frontend Views Test Coverage & E2E Tests
**Answer:** 6 views have no unit tests. While `@playwright/test` is a dependency in `package.json`, there are zero Playwright configuration files or e2e test files in the repository.  
**Evidence:**  
- Untested Views:
  - `OptimizeView.svelte`
  - `ConvertView.svelte`
  - `SecurityView.svelte`
  - `EditView.svelte`
  - `MarkdownView.svelte`
  - `FormsView.svelte`
- E2E / Playwright status: NOT FOUND. No `playwright.config.ts`, no `tests/e2e` directory.

---

### Q9: Test Corpus Classification
**Answer:** Input PDFs are specified in `MANIFEST` and downloaded by `fetch.sh`. Three files are transient test outputs left behind on disk after test runs.  
**Evidence:**  
- Inputs ([corpus/MANIFEST:4-12](file:///home/ayra/localpdf-studio-protocol/corpus/MANIFEST#L4-L12)): `standard-text.pdf`, `standard-form.pdf`, `hostile-bombs.pdf`, `hostile-broken-xref.pdf`, `synthetic-1k.pdf`, `test_contract.pdf`.
- Generated Outputs (residue on disk): `test_contract_out.pdf`, `test_contract_out_FallbackAdapter.pdf`, `test_contract_out_PdfcraftAdapter.pdf`.

---

### Q10: Root Artifacts Audit
**Answer:** `deny_out.txt` is a static log output of a previous failed `cargo deny check` run. `install_deps.sh` is an imperative host installation script that installs system packages and curl-pipes Rust/pnpm.  
**Evidence:**  
- [deny_out.txt:12,33](file:///home/ayra/localpdf-studio-protocol/deny_out.txt#L12):
```text
error[unlicensed]: app-tauri = 0.1.0 is unlicensed
licenses FAILED
```
- [install_deps.sh:5,9](file:///home/ayra/localpdf-studio-protocol/install_deps.sh#L5):
```bash
sudo dnf install -y webkit2gtk4.1-devel openssl-devel librsvg2-devel ...
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```
