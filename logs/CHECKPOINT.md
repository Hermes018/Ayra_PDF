# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-13
- Phase / sub-phase: 11.4 Threat-model review, dependency re-vet, SBOM
- Last green tag: phase-11.3-green
- In progress: None
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Conduct threat-model review across untrusted PDF ingress and sandboxing 2) Perform dependency re-vetting and update 08_DEPENDENCY_VETTING.md 3) Generate reproducible Software Bill of Materials (SBOM) and verify licenses
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
