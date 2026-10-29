# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-29
- Phase / sub-phase: Phase 12.6 — Core Engine Bump (`pdfcraft` 75360d1) & Multilingual FAQ Modal
- Status: Complete, RPM built, passing checks
- Last green tag: v0.1.0-rc5
- In progress: Release packaging verified, ready for deployment
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Run `sudo dnf reinstall` on Fedora host 2) Verify light/dark theme contrast across all views 3) Run UAT on windowed-mode layouts (800px-1080px)
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check), 008 (SvelteKit SPA), 009 (Vendored PDF.js)
- Pinned pdfcraft commit: 75360d1d7db06fed974f48c914d4c04132757187
