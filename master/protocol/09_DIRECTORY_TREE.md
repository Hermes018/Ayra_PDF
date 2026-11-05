# 09 — DIRECTORY TREE (keep current: update in the same commit as any structural change)

Since 2026-11-05 (ADR-016) the repository has seven top-level folders. The full, described map of every file is generated:
`/tree.md` (whole repo) and `<folder>/<folder>.md` (one per folder), built by `master/tools/tree/render_tree.py` from `master/tools/tree/descriptions.tsv`.
Structural change = add the path to `descriptions.tsv` and rerun the script (`--check` fails when a file is unmapped).

```
Ayra_PDF/                         # repo root: only files tools require at the root
├── AGENTS.md  tree.md  .gitignore  .gitmodules  .github/workflows/
├── master/                       # shared workspace for ALL platforms (run cargo / pnpm / xtask here)
│   ├── protocol/  logs/  ux-spec/  docs/  corpus/  tools/  fuzz/  xtask/
│   ├── crates/                   # lpdf-* engine, jobs, fs-safety, worker, cli, ...
│   ├── app-tauri/                # src/ (Svelte 5 SPA) + src-tauri/ (Tauri v2 shell, shared tauri.conf.json)
│   ├── vendor/pdfcraft/          # pinned submodule (commit in ENGINE_NOTES / CHECKPOINT)
│   └── Cargo.toml  Cargo.lock  deny.toml  rust-toolchain.toml  .cargo/
├── ubuntu/   packaging/ (tauri.ubuntu.conf.json overlay, build-deb.sh)  ubuntu.md
├── fedora/   packaging/ (tauri.fedora.conf.json overlay, build-rpm.sh)  docs/  fedora.md
├── windows/  packaging/ (tauri.windows.conf.json overlay, store icons)  docs/  windows.md
├── macos/  ios/  android/        # not started; each has its <folder>.md
```
