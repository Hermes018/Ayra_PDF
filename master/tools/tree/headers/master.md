# master/ — shared workspace (all platforms)

Everything every platform build needs lives here: the Rust crates, the desktop app (Svelte 5 frontend plus Tauri v2 shell), tests, the project protocol and logs, and the cross-platform docs. **All general development happens in this folder.** Platform folders (`../ubuntu`, `../fedora`, `../windows`, ...) receive only the files specific to their platform, when work on that platform needs them. See `../tree.md` for the full repository map.

## Everyday commands (run from `master/`)

| Task | Command |
|---|---|
| First checkout on a new machine | `git submodule update --init --recursive` (pdfcraft engine) and `bash corpus/fetch.sh` (test PDFs) |
| Frontend dependencies | `corepack enable` once (Tauri builds call `pnpm`), then `pnpm --dir app-tauri install --frozen-lockfile` |
| Run the app (dev) | `corepack pnpm --dir app-tauri tauri dev` |
| Frontend type check / tests | `corepack pnpm --dir app-tauri check` · `corepack pnpm --dir app-tauri exec vitest run` |
| Rust tests | `cargo test --workspace` (the `app-tauri` crate needs the webkit2gtk dev packages; see `../ubuntu/ubuntu.md`) |
| Project checks | `cargo xtask protocol-lint` · `cargo xtask feature-map-check` · `cargo xtask studio-audit --strict` |
| Full gate | `cargo xtask gate` (fmt, clippy, nextest, deny, audit, protocol-lint) |
| Platform package | `../ubuntu/packaging/build-deb.sh` · `../fedora/packaging/build-rpm.sh` · see `../windows/windows.md` |

Start every agent session by reading `../AGENTS.md`.
