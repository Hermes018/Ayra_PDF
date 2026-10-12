# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-12
- Phase / sub-phase: 9.2 (Advanced Convert - PDF to Office sidecars)
- Last green tag: phase-9.1-green
- In progress: None
- Open risks: E2E Playwright with `tauri-driver` needs Tauri driver setup. Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization. The canvas needs an actual PDF renderer for full WYSIWYG interactivity in future phases.
- Next 3 actions: 1) Plan Phase 9.2 PDF to Office (Word/Excel/PowerPoint) sidecar export 2) Extend sidecar runner to support PDF-to-DOCX/XLSX/PPTX filters 3) Update ConvertFromPdfView with high-fidelity sidecar export options
- Active ADRs: 001-010, 004, 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
