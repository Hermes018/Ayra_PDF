# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-06 ~23:53 Asia/Dhaka by Tier 2 Reviewer & Packaging Architect (Gemini 4 Argon Low). Trust `STATE_AUTO.md` for git/process facts.

## STATE: PHASE F3.2 (BRANCH MERGES, SBOM SECURITY SIGN-OFF & LTO PATCH) COMPLETE
- **Current Host:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK) · Intel Core i5-1135G7 · ~8 GiB RAM (7.65 GiB total, ~3.6 GiB available, 33 GiB disk free).
- **8GB RAM Guard Active:** `1` HEAVY slot max (`SLOT_FILES=(/tmp/lpdf-build.lock)`), `MemAvailable >= 2200 MB` guard, `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
- **Phase F3.2 Sign-off & Merges:**
  - `origin/wt/fixes` **MERGED**: BUG-062 counted retry (`e2e/soak_retry.py`, `test_soak_retry.py`, `flow12_soak.py`, `run.py`) and BUG-061 changes.
  - `origin/wt/release` **MERGED**: `master/tools/sbom_npm.py` audited and given official Security Sign-Off (173 transitive packages from `pnpm-lock.yaml`, zero network calls, version 0.2.0 stamp).
  - `[profile.release]` patched in `Cargo.toml` and verified: `lto = "thin"`, `codegen-units = 1` enforced for 8GB laptop safety.
- **Goal:** `v0.2.0-prototype` (`v0.2.0-rc1` tag target). **v0.2.0-rc1 is NOT built or tagged yet.**

## Branches (all local / remote tracking)
| Branch | Head | State |
|---|---|---|
| `main` | `b5d6682` | **VERIFIED & MERGED:** Phase F1 + F2 + F3.2 (`wt/fixes` and `wt/release` merged, LTO thin configured, SBOM audited). |
| `wt/fixes` | `8e1386e` | **MERGED into main.** |
| `wt/release` | `67aaf5c` | **MERGED into main.** |
| `origin/wt/engines` | `13b9bb3` | **WIP, unverified:** CP3 (BUG-029/030 streaming guard moved to `lpdf-engine-pdfcraft/src/guard.rs`, `path_exists` returns `Result`). |
| `origin/track/android` | `644caf3` | Paused (ADR-019 draft). |

## Next 3 Immediate Actions (Follow `master/docs/FLASH_AND_ARGON_EXECUTION_PLAN.md`)
1. **Action 1 (Phase F3.3 — Packaging B1/B2):** Package `.rpm`, `.deb`, and `.AppImage` with `lto = "thin"`, `codegen-units = 1`, and network-isolated smoke test on **Gemini 3.8 Flash (`High`)**.
2. **Action 2 (Phase F4.1 — Final Verification & Golden E2E):** Full `master/tools/verify.sh` + 12 Golden E2E flows + pdftotext redaction proof.
3. **Action 3 (Phase F4.2 — RC Sign-Off & Tagging):** Fresh `rc-verifier` sign-off -> tag `v0.2.0-rc1` locally on `main`.

## Standing Rules
- **4-Tier Base Pro Routing:** Tier 1 `3.8 Flash (High)` (80% workhorse) · Tier 2 `Argon (Low)` (15% lead/reviewer/packaging) · Tier 3 `Argon (Medium)` (4% engine/escalation) · Tier 4 `Opus 5.5 (High)` (1% save/redaction/RC verifier).
- **Owner Rules:** `BUG-061` is P0 only if memory grows > ~1 GB in a normal 2h session, else P1 + `KI-009`. On 8GB laptop, use `lto = "thin"` + `codegen-units = 1` for release builds. Never push (`owner pushes`).
