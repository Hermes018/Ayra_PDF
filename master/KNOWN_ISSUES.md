# KNOWN ISSUES — v0.2.0-prototype (Linux)

Cuts and known limitations shipped in this release. Every item here is also reported in the milestone report.

| ID | Area | Issue | Workaround |
|---|---|---|---|
| KI-001 | Tabs | Switching tabs clears the Undo history of the tab you leave. A tooltip on the tab says so. | Save before switching if you may want to undo. |
| KI-002 | Organize | When pages are both reordered/deleted **and** rotated, one intermediate `*.organizing-*.pdf` file is left next to the output (cleanup IPC pending in engines CP2b). | Delete the intermediate file by hand. |
| KI-003 | Labs | LaTeX insert and ink-to-text are partial and sit behind Labs (off by default). | — |
| KI-004 | Split view | **Cut to Labs (off by default).** Multi-pane split view shares one set of markup engines between panes; three review rounds found data-loss races. In the default UI, extra documents open as tabs. | Use tabs. |
