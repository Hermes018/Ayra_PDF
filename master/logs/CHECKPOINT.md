# CHECKPOINT — living handoff (< 150 lines, < 5,000 chars; history in CHECKPOINT_HISTORY.md)
Updated 2026-11-07 ~01:57 Asia/Dhaka by Fresh-Context RC Verifier (Gemini 4 Argon Low). Trust `STATE_AUTO.md` for git/process facts.

## STATE: v0.2.0-rc1 VERIFIED & TAGGED (SPRINT COMPLETE)
- **Current Host:** Fedora 44 KDE Plasma (Wayland, `rpm`/`dnf` native, WebKitGTK) · Intel Core i5-1135G7 · ~8 GiB RAM (7.65 GiB total, ~3.6 GiB available, 32 GiB disk free).
- **8GB RAM Guard Active:** `1` HEAVY slot max (`SLOT_FILES=(/tmp/lpdf-build.lock)`), `MemAvailable >= 2200 MB` guard, `CARGO_BUILD_JOBS=2`, `NEXTEST_TEST_THREADS=2`, `VITEST_MAX_WORKERS=2`, `NODE_OPTIONS="--max-old-space-size=2048"`.
- **Phase F4 RC Verification & Sign-Off (COMPLETE):**
  - **Master Gate (`tools/verify.sh`):** ALL PASS (Exit Code 0; fmt, clippy, nextest 260 tests, vitest 56 tests, svelte-check, cargo deny, cargo audit, protocol-lint, studio-audit, feature-map, tree-map).
  - **Release Bundles & Checksums:** `.rpm` (17.08 MiB), `.deb` (17.08 MiB), and `.AppImage` (122.08 MiB) verified against `docs/SHA256SUMS.txt`.
  - **Supply Chain SBOM:** `docs/SBOM-v0.2.0.json` verified (1002 components: 829 Cargo, 173 npm; version `0.2.0`).
  - **Network Isolation (`unshare -rn`):** PASS (7/7 `lpdf-fs-safety` tests passed with 0 interfaces).
  - **Cryptographic Redaction:** PASS (physical stream destruction verified).
- **Goal:** `v0.2.0-rc1` **VERIFIED & TAGGED** on `main`.

## Branches (all local / remote tracking)
| Branch | Head | State |
|---|---|---|
| `main` | `v0.2.0-rc1` | **VERIFIED & TAGGED:** All gates green, packages built, SBOM & docs complete, `RC_RECEIPT.txt` verified. |
| `wt/fixes` | `8e1386e` | **MERGED into main.** |
| `wt/release` | `67aaf5c` | **MERGED into main.** |
| `origin/wt/engines` | `13b9bb3` | **WIP, unverified:** CP3 (deferred post-rc1). |
| `origin/track/android` | `644caf3` | Paused (ADR-019 draft). |

## Next Immediate Action
1. **Owner Dogfooding & Push:** Owner hands-on dogfooding (`master/logs/DOGFOOD.md`) + `git push origin main --tags` (`owner pushes`).

## Standing Rules
- **4-Tier Base Pro Routing:** Tier 1 `3.8 Flash (High)` (80% workhorse) · Tier 2 `Argon (Low)` (15% lead/reviewer/packaging) · Tier 3 `Argon (Medium)` (4% engine/escalation) · Tier 4 `Opus 5.5 (High)` (1% save/redaction/RC verifier).
- **Owner Rules:** `BUG-061` is P1 + `KI-009`. On 8GB laptop, use `lto = "thin"` + `codegen-units = 1` for release builds. Never push (`owner pushes`).
