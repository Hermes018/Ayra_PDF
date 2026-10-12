# CHECKPOINT (OVERWRITE this file; single source of "where am I")
- Date: 2026-10-13
- Phase / sub-phase: 12.1 Fedora packaging & KDE integration
- Last green tag: phase-11.6-green
- In progress: None
- Open risks: Memory spikes on large rasterized render passes across low tiers (ADR-002). Rendering performance on >1000 pages for the grid editor might require IntersectionObserver virtualization.
- Next 3 actions: 1) Package Fedora release artifacts (RPM spec, Flatpak manifest, AppImage recipe) 2) Test execution and sandboxing under SELinux-enforcing mode 3) Implement and verify KDE desktop integration (desktop file, MIME types, portals, HiDPI, Wayland)
- Active ADRs: 001-010, 004 (Local LLM Architecture & Candle Selection), 005 (Native Feature Expansion), 006 (Document Reconstruction Reality Check)
- Pinned pdfcraft commit: dc4b3eee84f73bad8564906c141bdda9caad0fd6
