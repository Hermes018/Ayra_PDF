# KNOWN ISSUES — v0.2.0-prototype (Linux)

Cuts and known limitations shipped in this release. Every item here is also reported in the milestone report.

| ID | Area | Issue | Workaround |
|---|---|---|---|
| KI-001 | Tabs | Switching tabs clears the Undo history of the tab you leave. A tooltip on the tab says so. | Save before switching if you may want to undo. |
| KI-002 | Organize | When pages are both reordered/deleted **and** rotated, one intermediate `*.organizing-*.pdf` file is left next to the output (cleanup IPC pending in engines CP2b). | Delete the intermediate file by hand. |
| KI-003 | Labs | LaTeX insert and ink-to-text are partial and sit behind Labs (off by default). | — |
| KI-004 | Split view | **Cut to Labs (off by default).** Multi-pane split view shares one set of markup engines between panes; three review rounds found data-loss races. In the default UI, extra documents open as tabs. | Use tabs. |
| KI-005 | Packaging | Prototype packages are built with LTO off (RAM rule while model training runs). The version string still says 0.1.0 (to be bumped to 0.2.0 before rc1). | Rebuild with LTO in a RAM window; larger and slightly slower binary until then. |
| KI-006 | Packaging (build-time only) | The AppImage build downloads linuxdeploy. The app itself never uses the network. | Pre-fetch linuxdeploy for offline builds. |
| KI-007 | Studio | There is no "Close document" action other than closing its tab. | Close the tab. |
| KI-008 | Protect | Protect sets only an open password; PDF permission flags (print/copy) and an owner password are not set. The FAQ no longer claims otherwise. | Use a dedicated tool if you need permission flags. |
