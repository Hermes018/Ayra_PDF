# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-12
- Phase / sub-phase: 10.5b Local-LLM spike (llama-cpp-2 vs candle per ADR-004, lpdf-ai-worker skeleton, model import UI, hash + license)
- Last green tag: phase-10.5a-green
- In progress: None
- Open risks: Local AI model inference spikes (ADR-004) depend on hardware tier bounds. E2E Playwright with `tauri-driver` needs Tauri driver setup. Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Benchmark and spike llama-cpp-2 vs candle under unshare -n offline environment 2) Construct lpdf-ai-worker skeleton with typed RPC channel 3) Wire hash + license acknowledgement into model import UI
- Active ADRs: 001-010, 004 (Local LLM Architecture), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
