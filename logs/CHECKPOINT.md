# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-12
- Phase / sub-phase: 9.3 (Markdown Toolset - MD to PDF / PDF to MD / Pandoc sidecar)
- Last green tag: phase-9.2-green
- In progress: None
- Open risks: E2E Playwright with `tauri-driver` needs Tauri driver setup. Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization. The canvas needs an actual PDF renderer for full WYSIWYG interactivity in future phases.
- Next 3 actions: 1) Plan Phase 9.3 Pandoc sidecar integration 2) Implement MD -> PDF and PDF -> MD export pipeline with asset extraction 3) Build Markdown conversion UI in app-tauri
- Active ADRs: 001-010, 004, 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
