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
