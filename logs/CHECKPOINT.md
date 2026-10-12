# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-12
- Phase / sub-phase: 10.5c AI Summarizer, LLM mode (map-reduce over layout blocks, chunking ≤ 70% context, deterministic T ≤ 0.3, page-cited bullets, cancel at chunk boundary)
- Last green tag: phase-10.5b-green
- In progress: None
- Open risks: Local AI model inference spikes (ADR-004) depend on hardware tier bounds. E2E Playwright with `tauri-driver` needs Tauri driver setup. Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Implement map-reduce chunking over layout blocks in lpdf-ai / lpdf-ai-worker 2) Wire deterministic T <= 0.3 inference generation with page-cited bullets 3) Connect UI progress and cancel boundary for LLM summarization
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6

