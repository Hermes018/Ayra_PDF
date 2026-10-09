# 09 — DIRECTORY TREE (keep current: update in the same commit as any structural change)

```
localpdf-studio/
├── AGENTS.md                      # boot pointer (auto-read)
├── MASTER_PROMPT.md
├── protocol/                      # read-only rules (append-only exceptions: ENGINE_NOTES, ENV)
│   ├── 01_MASTER_CONTEXT.md  02_RULES_DOS_DONTS.md  03_FAILSAFES.md
│   ├── 04_INSTRUCTION_FADE_PREVENTION.md  05_PHASES.md  06_ARCHITECTURE.md
│   ├── 07_FEATURE_MAP.md  08_DEPENDENCY_VETTING.md  09_DIRECTORY_TREE.md
│   ├── ENGINE_NOTES.md            # verified pdfcraft/Tauri API notes (created in 0.4)
│   └── ENV.md                     # toolchain versions (created in 0.2)
├── logs/                          # append-only (CHECKPOINT = overwrite)
│   ├── CHECKPOINT.md  PROGRESS_LOG.md  PLAN_LOG.md  PROMPT_LOG.md  ERROR_LOG.md  DECISIONS_ADR.md
├── ux-spec/                       # per-tool UX specs + reference shots (not shipped)
├── corpus/                        # MANIFEST + fetch scripts (no third-party PDFs committed)
├── crates/
│   ├── lpdf-core-types/  lpdf-engine-port/  lpdf-engine-pdfcraft/  lpdf-engine-fallback/
│   ├── lpdf-tools-organize/  -optimize/  -convert/  -security/  -edit/  -markdown/
│   ├── lpdf-layout/  lpdf-ai/  lpdf-ai-worker/  lpdf-bridge/
│   ├── lpdf-jobs/  lpdf-fs-safety/  lpdf-hw/  lpdf-worker/  lpdf-cli/
├── app-tauri/
│   ├── src/  capabilities/  tauri.conf.json  Cargo.toml
│   └── ui/                        # Svelte 5 SPA (src/, src/tools/registry.ts, src/state/, components/, styles/, i18n/, tests/)
├── vendor/pdfcraft/               # pinned subtree/submodule (commit SHA recorded in ENGINE_NOTES)
├── fuzz/                          # cargo-fuzz targets
├── packaging/                     # rpm/ flatpak/ appimage/ windows/ (macos/ android/ later)
├── xtask/                         # gate, doctor, protocol-lint, feature-map-check, corpus, release
├── .github/workflows/             # ci.yml nightly.yml release.yml
├── deny.toml  rust-toolchain.toml  Cargo.toml  Cargo.lock  pnpm-lock.yaml  CHANGELOG.md
└── docs/                          # user docs, threat model, SBOM notes
```
