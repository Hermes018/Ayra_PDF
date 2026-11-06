# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-07 ~00:43 Asia/Dhaka by Tier 1 Workhorse Builder & Packager (Gemini 3.8 Flash High). Trust `STATE_AUTO.md` for git/process facts.

## STATE: PHASE F3.3 (PACKAGING B1 BUILDS/SBOM & B2 DOCS/HASHES) COMPLETE
- **Current Host:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK) · Intel Core i5-1135G7 · ~8 GiB RAM (7.65 GiB total, ~3.6 GiB available, 32 GiB disk free).
- **8GB RAM Guard Active:** `1` HEAVY slot max (`SLOT_FILES=(/tmp/lpdf-build.lock)`), `MemAvailable >= 2200 MB` guard, `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
- **Phase F3.3 Packaging & Docs (COMPLETE):**
  - **SBOM (B1):** Generated `master/docs/SBOM-v0.2.0.json` (CycloneDX 1.5 format, 173 transitive npm packages from `pnpm-lock.yaml`, sha512 integrity, licensed, version 0.2.0 stamped).
  - **Release Bundles (B1):** Compiled via Tauri v2 with `lto = "thin"`, `codegen-units = 1`:
    - `LocalPDF Studio_0.2.0_amd64.AppImage` (122.08 MiB / 123 MB)
    - `LocalPDF Studio-0.2.0-1.x86_64.rpm` (17.08 MiB / 18 MB)
    - `LocalPDF Studio_0.2.0_amd64.deb` (17.08 MiB / 18 MB)
  - **Documentation & Hashes (B2):** Authored `docs/README.md`, `docs/QUICKSTART.md` (all 12 Golden Flows), `docs/CHANGELOG.md`, `docs/KNOWN_ISSUES.md` (KI-001..009 with BUG-061 memory footprint), and generated `docs/SHA256SUMS.txt`.
- **Goal:** `v0.2.0-prototype` (`v0.2.0-rc1` tag target). **v0.2.0-rc1 is NOT built or tagged yet.**

## Branches (all local / remote tracking)
| Branch | Head | State |
|---|---|---|
| `main` | `b5d6682` | **VERIFIED & PACKAGED:** Phase F1 + F2 + F3.2 + F3.3 (Bundles built, thin LTO, SBOM, Docs & Hashes). |
| `wt/fixes` | `8e1386e` | **MERGED into main.** |
| `wt/release` | `67aaf5c` | **MERGED into main.** |
| `origin/wt/engines` | `13b9bb3` | **WIP, unverified:** CP3 (BUG-029/030 streaming guard moved to `lpdf-engine-pdfcraft/src/guard.rs`, `path_exists` returns `Result`). |
| `origin/track/android` | `644caf3` | Paused (ADR-019 draft). |

## Next 3 Immediate Actions (Follow `master/docs/FLASH_AND_ARGON_EXECUTION_PLAN.md`)
1. **Action 1 (Phase F4 — RC Verifier):** Phase F4 RC Verifier on **Claude Opus 5.5 (`High`)** (or Argon Medium fresh session) for final safety verification & v0.2.0-rc1 sign-off.
2. **Action 2 (Phase F4.1 — Final Verification Gate):** Full `master/tools/verify.sh` + 12 Golden E2E flows + pdftotext redaction proof.
3. **Action 3 (Phase F4.2 — Tagging):** Tag `v0.2.0-rc1` locally on `main` (`owner pushes`).

## Standing Rules
- **4-Tier Base Pro Routing:** Tier 1 `3.8 Flash (High)` (80% workhorse) · Tier 2 `Argon (Low)` (15% lead/reviewer/packaging) · Tier 3 `Argon (Medium)` (4% engine/escalation) · Tier 4 `Opus 5.5 (High)` (1% save/redaction/RC verifier).
- **Owner Rules:** `BUG-061` is P0 only if memory grows > ~1 GB in a normal 2h session, else P1 + `KI-009`. On 8GB laptop, use `lto = "thin"` + `codegen-units = 1` for release builds. Never push (`owner pushes`).
