# ADR-009: Vendored Mozilla pdf.js Webview Rendering Exception

- **Date:** 2026-10-13
- **Status:** Approved Exception (Retroactive Architectural Reconciliation)
- **Context:**
  The frontend client integrates vendored versions of Mozilla's `pdf.min.js` and `pdf.worker.min.js` in `static/` to render visual PDF previews inside `PdfViewer.svelte` and `GridEditor.svelte`. Protocol 02 prioritizes native Rust rendering (`hayro`/`pdfcraft`) and forbids untrusted client execution.
- **Decision:**
  Vendoring Mozilla's `pdf.min.js` and `pdf.worker.min.js` strictly for visual webview canvas preview rendering is formally approved as an architectural exception under strict constraints.
- **Mandatory Invariants:**
  1. **Strict CSP Enforcement:** Content Security Policy in `tauri.conf.json` must strictly block external network access and unauthorized scripts (`default-src 'self'; connect-src ipc: http://ipc.localhost`).
  2. **Zero Modification in JS:** No structural PDF modifications, mutations, redactions, or cryptographic operations may take place in JavaScript; `pdf.js` is restricted solely to rendering visual pixels from byte arrays provided by the local Rust backend.
  3. **Licensing Compliance:** Mozilla's Apache License 2.0 attribution notices must be maintained intact in all asset headers and bundled licensing manifests.
- **Approved by:** Staff Software Engineer & Enterprise Security Auditor.
