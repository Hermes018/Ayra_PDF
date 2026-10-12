# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-13
- Phase / sub-phase: 11.3 Perf/memory profiling per tier (systemd-run -p MemoryMax=2G -p CPUQuota=200%)
- Last green tag: phase-11.2-green
- In progress: None
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Implement automated benchmark harness for memory consumption per hardware tier (Tier 1..3) under systemd-run MemoryMax=2G 2) Verify memory budget compliance and CPU quotas on 1k-page synthetic files 3) Integrate automated budget regression thresholds (>10% regression fails CI)
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
