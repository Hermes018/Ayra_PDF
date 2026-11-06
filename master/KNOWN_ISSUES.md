# KNOWN ISSUES — LocalPDF Studio (v0.2.0-rc1)

Known limitations and tracked behaviors in this release.

| ID | Area | Issue | Workaround / Mitigation |
|---|---|---|---|
| **KI-001** | Tabs | Switching tabs clears the Undo history of the unmounted tab. | Save document before switching tabs if you need to retain edit history. |
| **KI-002** | Organize | When pages are both reordered/deleted and rotated in a single pass, an intermediate `*.organizing-*.pdf` file may temporarily remain. | Harmless; can be safely deleted or cleaned up. |
| **KI-003** | Labs | LaTeX formula insertion and ink-to-text conversion are experimental and disabled by default under Labs settings. | Enable Labs only if experimenting with early research features. |
| **KI-004** | Split View | Multi-pane split view is gated behind Labs to avoid concurrency races. Default interface uses tabs. | Use multi-tab interface for handling multiple documents. |
| **KI-005** | Packaging Profile | Resolved in v0.2.0-rc1: `[profile.release]` configured with `lto = "thin"` and `codegen-units = 1`. Version 0.2.0 stamped across all crates and packages. | None required (Resolved). |
| **KI-006** | Packaging (Build-time only) | AppImage bundling toolchain may attempt to fetch `linuxdeploy` if not already pre-cached in build environment. The compiled app itself never accesses the network. | Pre-cache `linuxdeploy` in air-gapped packaging environments. |
| **KI-007** | Studio Document Management | There is no separate "Close document" menu action other than clicking the close button on the document tab. | Click the `×` on the document tab to close it. |
| **KI-008** | Security / Protect | Document protection sets AES-256 open/read encryption password. Detailed permission flags (printing/copying restrictions) without an owner password are not set. | Use open password for confidential documents. |
| **KI-009** (BUG-061) | Memory / Soak | Long automated soak runs with hundreds of unclosed tabs exhibit memory growth (~1.75 MB per unclosed document tab). In normal interactive 2-hour sessions, RSS remains well under 1 GiB. | Close unused document tabs during long editing sessions. |
