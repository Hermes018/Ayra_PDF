# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-15
- Phase / sub-phase: 12.2 RC2 Fixes (argv forwarding, CLI packaging)
- Last green tag: phase-12.1-rc1
- In progress: F-RC1-001 argv startup file forwarding, F-RC1-002 CLI/worker RPM packaging, GitHub credential setup
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization. GitHub push blocked pending credential config (F-RC1-005).
- Next 3 actions: 1) Implement std::env::args forwarding + single-instance argv in lib.rs 2) Configure GitHub PAT/SSH and push all commits+tags 3) Map CLI/worker binaries in bundle.linux.rpm.files
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check), 008 (SvelteKit SPA), 009 (Vendored PDF.js)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6

