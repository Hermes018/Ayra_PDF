# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-14
- Phase / sub-phase: 12.1b Clean Install Testing
- Last green tag: phase-12.1-rc1
- In progress: Ready for Clean Install Testing & QA checklist execution
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Test clean RPM install and verification on Fedora 44 2) Execute QA vetting checklist across 31 tools 3) Package Flatpak manifest & AppImage recipe
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check), 008 (SvelteKit SPA), 009 (Vendored PDF.js)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6

