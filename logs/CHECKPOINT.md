# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-12
- Phase / sub-phase: 9.4 (Batch folder conversion, front-matter, link/asset rewriting, CLI `localpdf md ...`)
- Last green tag: phase-9.3-green
- In progress: None
- Open risks: E2E Playwright with `tauri-driver` needs Tauri driver setup. Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization. The canvas needs an actual PDF renderer for full WYSIWYG interactivity in future phases.
- Next 3 actions: 1) Plan Phase 9.4 batch folder conversion and front-matter handling 2) Implement link/asset rewriting in Pandoc pipelines 3) Add CLI `localpdf md ...` command handling
- Active ADRs: 001-010, 004, 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
