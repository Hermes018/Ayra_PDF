# macos/ — macOS build

**Status:** not started. Protocol phase 13 covers building, signing and notarizing, sandbox entitlements and WKWebView quirks.

When macOS work begins, its overlay goes here: `packaging/tauri.macos.conf.json` (dmg/app targets, signing identity, entitlements), plus any macOS-only docs. Shared files that macOS already uses stay in `master/`, for example the icon set `master/app-tauri/src-tauri/icons/icon.icns`.

## Map of `macos/`

Every entry has a one-line description. `master/vendor/pdfcraft` is an upstream submodule and is listed as one entry. Build output (`target/`, `node_modules/`, `build/`) and downloaded corpus PDFs are not tracked and not listed.

```text
macos/
└── macos.md  # Map of macos/ and what will go here
```
