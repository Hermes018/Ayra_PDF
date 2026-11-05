# tree.md — LocalPDF Studio repository map

The repository is split into seven top-level folders (decision: ADR-016 in `master/logs/DECISIONS_ADR.md`).

| Folder | What lives there |
|---|---|
| `master/` | Everything shared by every platform: Rust crates, the Svelte/Tauri app, tests, protocol, logs, cross-platform docs. **All day-to-day development happens here.** |
| `ubuntu/` | Ubuntu/Debian packaging (`.deb`) and Ubuntu notes. Current development machine. |
| `fedora/` | Fedora packaging (`.rpm`) and Fedora install/QA/release docs. |
| `windows/` | Windows 10/11 packaging (NSIS/MSI) and Windows plans. |
| `macos/` | macOS packaging (not started). |
| `ios/` | iOS packaging (not in the phase plan yet). |
| `android/` | Android packaging (not started). |

## Rules

1. Develop in `master/`. Run `cargo`, `pnpm` and `cargo xtask` from `master/`.
2. A platform folder holds only files that apply to that platform: packaging config, install scripts, platform docs. Files go there only when work on that platform needs them. Shared code stays in `master/`, even when part of it is `#[cfg(target_os = ...)]`.
3. Platform build config is a **Tauri config overlay**: `<os>/packaging/tauri.<os>.conf.json` is merged over `master/app-tauri/src-tauri/tauri.conf.json` with `tauri build --config <overlay>`. Paths inside an overlay are relative to `master/app-tauri/src-tauri/`.
4. Some files must stay at the repository root because tools look for them there: `.github/` (GitHub Actions), `.gitignore`, `.gitmodules` and `AGENTS.md` (agent boot file).
5. When you add, move or delete a file: add/edit its line in `master/tools/tree/descriptions.tsv` and run `python3 master/tools/tree/render_tree.py`. This regenerates this map and every `<folder>/<folder>.md`; `--check` fails if a file is unmapped. Do not edit the generated maps by hand.
