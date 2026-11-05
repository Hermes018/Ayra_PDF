# windows/ — Windows 10/11 build (NSIS / MSI)

**Status:** the Windows code from phases W1–W4 lives in `master/` behind `#[cfg(target_os = "windows")]`: NTFS-safe atomic writes, the Job Object sandbox, window vibrancy, the RAM governor and installer config. **It has never been compiled or run on Windows.** CI (`.github/workflows/ci.yml`, `release-windows.yml`) is the only planned Windows build so far.

Changes on 2026-11-05:
- The W3 memory budget now uses `sysinfo`, which needs no `unsafe` code.
- The unused `WorkerQosGuard` was removed.
- `lpdf-fs-safety` is the only crate allowed `unsafe` on Windows (ADR-015).

## Build the installers (on Windows)

```powershell
cd master
corepack pnpm --dir app-tauri install --frozen-lockfile
corepack pnpm --dir app-tauri tauri build --bundles nsis,msi --config ..\windows\packaging\tauri.windows.conf.json
# output in master\target\release\bundle\nsis\ and \msi\
```

Plan and decision record: `docs/WINDOWS_MASTER_PLAN.md`, `docs/ADR-011-windows-platform-architecture.md` (registered as ADR-013).

## Map of `windows/`

Every entry has a one-line description. `master/vendor/pdfcraft` is an upstream submodule and is listed as one entry. Build output (`target/`, `node_modules/`, `build/`) and downloaded corpus PDFs are not tracked and not listed.

```text
windows/
├── docs/                                         # Windows plans and decisions
│   ├── ADR-011-windows-platform-architecture.md  # Windows architecture decision (registered as ADR-013 in DECISIONS_ADR)
│   └── WINDOWS_MASTER_PLAN.md                    # Windows engineering plan (phases W1–W4)
├── packaging/                                    # Files used to build the Windows installers
│   ├── icons/                                    # Microsoft Store / tile logos (not used by NSIS/MSI today)
│   │   ├── Square107x107Logo.png                 # Store tile logo 107 px
│   │   ├── Square142x142Logo.png                 # Store tile logo 142 px
│   │   ├── Square150x150Logo.png                 # Store tile logo 150 px
│   │   ├── Square284x284Logo.png                 # Store tile logo 284 px
│   │   ├── Square30x30Logo.png                   # Store tile logo 30 px
│   │   ├── Square310x310Logo.png                 # Store tile logo 310 px
│   │   ├── Square44x44Logo.png                   # Store tile logo 44 px
│   │   ├── Square71x71Logo.png                   # Store tile logo 71 px
│   │   ├── Square89x89Logo.png                   # Store tile logo 89 px
│   │   └── StoreLogo.png                         # Store listing logo
│   └── tauri.windows.conf.json                   # Tauri config overlay: nsis/msi targets, WebView2 bootstrapper, install mode
└── windows.md                                    # Map of windows/ with build instructions and status
```
