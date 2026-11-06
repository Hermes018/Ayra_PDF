# ROUTING — model routing log (append-only rows; lead writes)
Routing table: CLAUDE.md "Model routing" + `.claude/agents/*.md`. Never Fable.
Columns: `time | task_type | agent | model/effort | outcome | from → to | reason`.
Outcome = `ok` | `fail` | `escalated`. Every task gets a row (needed for escalation rates); escalations add `from → to`.
Per-milestone rates: `python3 master/tools/routing-stats.py`.

| time | task_type | agent | model/effort | outcome | from → to | reason |
|---|---|---|---|---|---|---|
| 2026-11-06 07:31 | docs-lookup | claude-code-guide | haiku | ok | | subagent frontmatter fields |
| 2026-11-06 07:32 | reviewer | release R3 reviewer | sonnet/medium | ok | | PASS, minors |
| 2026-11-06 07:39 | checkpoint-update | logs+tree+verify main | sonnet/low | ok | | verify GREEN; 1 description guessed wrong, lead corrected |
| 2026-11-06 07:39 | release-build | release follow-ups (6 minors) | sonnet/medium | running | | first Sonnet release spawn |
| 2026-11-06 07:43 | fixes-build | fixes R5 (tabs final) + engines contract | opus/high (legacy spawn) | ok | | vitest 253/254: stale BUG-022 golden → regen approved |
| 2026-11-06 07:50 | release-build | release follow-ups (6 minors) | sonnet/medium | ok | | 5/6 done; item 4 (AppImage offline proof) stopped by lead for tier/slot; KI-006 unproven |
| 2026-11-06 07:54 | fixes-build | fixes R5 golden+polyline test | opus/high (legacy) | ok | | |
| 2026-11-06 07:54 | reviewer | fixes R5 | sonnet/medium | ok | | |
| 2026-11-06 07:54 | merge-verify | fixes R5 (save path) | opus/high | ok | | |
| 2026-11-06 08:00 | status-snapshot | routing proof after restart | haiku (agent file) | ok | | requestedModel=claude-haiku-5-5 ×4; effort=medium (inherited from ~/.claude/settings.json effortLevel; no effort field in file) |
| 2026-11-06 08:02 | fixes-build | ERR-007 / BUG-015 clipping + own patterns | sonnet/high (agent file) | ok | | verify GREEN; own code |
| 2026-11-06 08:15 | reviewer | ERR-007 | sonnet/medium (agent file) | ok | | |
| 2026-11-06 08:15 | merge-verify | ERR-007 (engine + saved geometry) | opus/high (agent file) | ok | | |
| 2026-11-06 08:18 | e2e-run | flows 1,2,4,6,11 on main | haiku/low (agent file) | ok | | 4 PASS, flow 1 BLOCKED (no bundle); build 18 s |
| 2026-11-06 08:20 | dogfood-build | AppImage + pkg smoke (flows 1-2) | haiku/low (agent file) | fail | | artifacts OK (build + smoke PASS) but final report never delivered; lead read summary.json |
| 2026-11-06 09:27 | dogfood-build | (escalation rule) | haiku/low | escalated | haiku → sonnet/low | next long-running script task of this type goes to Sonnet low (owner) |
| 2026-11-06 09:28 | release-build | BUG-031 + real-world PDF dogfood run | sonnet/medium (agent file) | running | | |
| 2026-11-06 09:28 | explore | UI screen inventory | haiku/low (agent file) | ok | | 12 screens, 1 dialog, 20 components, E2E: Organize only |
| 2026-11-06 09:28 | fixes-build | studio-audit over all .svelte (A2) | sonnet/high (agent file) | ok | | 385 violations in 30 files; verify GREEN |
| 2026-11-06 09:29 | design-spec | DESIGN_LANGUAGE.md one pass | opus/medium | ok | | 10 CREATE, 2 reuse, 11 NEW tokens |
| 2026-11-06 09:35 | review+verify | UI audit A2 (tooling) | sonnet/medium (agent file) | stopped | | lead stopped: owner triage changes the diff |
| 2026-11-06 09:37 | fixes-build | audit triage: px allowlist, 7 key listeners→commandEngine, 6 icon labels, overclaim | sonnet/low (owner) | ok | | 1/7 moved, 6 deferred (role=button Enter/Space → fixed by shared Button in sweep); raw-px real 283 |
| 2026-11-06 09:51 | review+verify | A2 audit + triage | sonnet/medium (agent file) | ok | | PASS, minors → BUG-042 |
| 2026-11-06 09:51 | review+verify | release WIP + BUG-031 | sonnet/medium (agent file) | ok | | PASS, minors → BUG-041 |
| 2026-11-06 09:53 | fixes-build | C0 shared component library + gallery | sonnet/medium (owner) | ok | | from scratch (Bits UI/Melt REJECT), vitest 301 |
| 2026-11-06 10:04 | review+verify+visual | C0 library | sonnet/medium (agent file) | ok | | PASS w/ 2 visual majors (dark dialog contrast, tooltip clip) → fix round |
| 2026-11-06 10:05 | fixes-build | C0 fix round (dialog contrast, tooltip clamp, gallery check control, env -u, BUG-043) | sonnet/medium | ok | | |
| 2026-11-06 10:11 | review+verify | C0 delta recheck + fresh gate | sonnet/medium (agent file) | ok | | 3/4 PASS, tooltip overlap minor; gate GREEN |
| 2026-11-06 10:11 | fixes-build | sweep group 1 (main-flow screens) | sonnet/medium (owner) | ok | | Organize 25→0; missed one behaviour change in report |
| 2026-11-06 10:24 | reviewer | sweep g1 | sonnet/medium | ok | | PASS, found unlisted behaviour change |
| 2026-11-06 10:24 | merge-verify | sweep g1 (save path) | opus/high | ok | | GREEN, flow10 18/18 |
| 2026-11-06 10:24 | fixes-build | sweep group 2 (Sidebar, StatusBar, MainLayout) | sonnet/medium | ok | | all behaviour changes listed |
| 2026-11-06 10:31 | review+verify+visual | sweep g2 | sonnet/medium | ok | | PASS, gate GREEN |
| 2026-11-06 10:32 | fixes-build | sweep group 3 (Toast, FAQ) | sonnet/medium | ok | | |
| 2026-11-06 10:42 | review+verify+visual | sweep g3 | sonnet/medium | ok | | PASS; audit overclaim gap found |
| 2026-11-06 10:44 | fixes-build | sweep group 4 (+ footer wording, overclaim audit gap) | sonnet/medium | ok | | close dialog flagged (save path) |
| 2026-11-06 10:54 | reviewer | sweep g4 | sonnet/medium | ok | | PASS, a11y minors |
| 2026-11-06 10:54 | merge-verify | sweep g4 (close/save dialog) | opus/high | ok | | GREEN; found untested guard (probe proved) |
| 2026-11-06 10:58 | fixes-build | sweep group 5 (+ Escape-during-save test, alert Dialog, Button attrs) | sonnet/medium | ok | | 2/3 screens, PropertyPill → backlog |
| 2026-11-06 11:04 | review+verify+visual | sweep g5 | sonnet/medium | ok | | review PASS; gate RED invalid (lead ran a mutating check concurrently, ERR-010) |
| 2026-11-06 11:04 | merge-verify (narrow) | g5 DocumentTabs alert + data-safety test | opus/high | ok | | red/green proven; tree restored |
| 2026-11-06 11:05 | gate-rerun | g5 verify.sh + flow18 reshoot | sonnet/low (escalation rule) | ok | | GREEN vitest 330; reshoot PASS |
| 2026-11-06 11:07 | fixes-build | sweep group 6 (+ capture-Escape fix, calibration discard confirm) | sonnet/medium | ok | | found flow 7 signature failure on base |
| 2026-11-06 11:15 | review+verify+visual | sweep g6 | sonnet/medium | ok | | PASS, gate GREEN |
| 2026-11-06 11:15 | log-read | flow 7 signature failure | haiku/low (agent file) | ok | | verbatim; lead triaged harness false-fail |
| 2026-11-06 11:16 | release-build | BUG-046 flow 7 matcher | sonnet/medium (agent file) | ok | | flow 7 PASS |
| 2026-11-06 11:16 | fixes-build | sweep group 7 (AnnotationLedger + password dialog top layer) | sonnet/medium | ok | | 104→0; flow 2 blocked (corpus) |
| 2026-11-06 11:20 | review+verify | BUG-046 | sonnet/medium | ok | | PASS, minor → BUG-047 |
| 2026-11-06 11:24 | review+verify+visual | sweep g7 | sonnet/medium | ok | | PASS code; visual not done (screens wiped) |
| 2026-11-06 11:24 | merge-verify | g7 password dialog (security) + flow 2 | opus/high | fail | | autocomplete=off dropped (security regression); flow 2 PASS |
| 2026-11-06 11:26 | fixes-build | g7 fix round (autocomplete, audio close, reshoot) | sonnet/medium | ok | | |
| 2026-11-06 11:30 | merge-verify | g7 password recheck (read-only) | opus/high | ok | | PASS |
| 2026-11-06 11:30 | review+gate+visual | g7 fix round | sonnet/medium | ok | | GREEN; ledger visual minors → g8 |
| 2026-11-06 11:30 | fixes-build | sweep group 8 (SpatialCanvas chrome + ledger visual + signature labels) | sonnet/medium | ok | | 124→0; save/redaction flagged |
| 2026-11-06 11:37 | review+gate+visual | sweep g8 | sonnet/medium | fail | | gate GREEN; 2 majors (undo label, dark bar contrast) |
| 2026-11-06 11:37 | merge-verify | g8 save/flatten/redaction + flows 8,11 | opus/high | ok | | handlers unchanged; flows 8,11 PASS |
| 2026-11-06 11:45 | fixes-build | g8 fix round (undo tooltip, dark bar contrast, stale tooltip) | sonnet/medium | ok | | |
| 2026-11-06 11:49 | review+gate | g8 fix delta | sonnet/medium | ok | | PASS, GREEN |
| 2026-11-06 11:49 | fixes-build | sweep group 9 PropertyPill (+ undo wiring test) | sonnet/medium | ok | | last main-flow screen; app total 808 in 14 non-main files |
| 2026-11-06 11:54 | review+gate+visual | sweep g9 | sonnet/medium | ok | | PASS, GREEN |
| 2026-11-06 11:54 | release-build | BUG-048 per-run screenshots | sonnet/medium (agent file) | ok | | |
| 2026-11-06 11:55 | fixes-build | sweep group 10 (Convert tabs, Optimize, Edit + BUG-045) | sonnet/medium | ok | | output paths not flagged by builder; lead escalated |
| 2026-11-06 11:57 | review+verify | BUG-048 | sonnet/medium | ok | | PASS, minors → BUG-049 |
| 2026-11-06 12:05 | review+gate+visual | sweep g10 | sonnet/medium | fail | | fake demo highlight (major), tabs a11y, opacity 0 |
| 2026-11-06 12:05 | merge-verify (read-only) | g10 Optimize/Edit output paths | opus/high | ok | | runner OK; found pre-existing P0 output==input |
| 2026-11-06 12:07 | fixes-build | g10 fix round (P0 BUG-050 first) | sonnet/medium | ok | | backend guard + collision-safe names app-wide |
| 2026-11-06 12:21 | merge-verify | BUG-050 + g10 (backend guard, security naming, flows 2,8,10,11) | opus/high | ok | | GREEN 233/233; flows PASS; found unguarded writers + 2nd-run regression |
| 2026-11-06 12:21 | reviewer (read-only) | g10 fix round frontend + visual | sonnet/medium | fail | | naming test covers 2 of 9 views |
| 2026-11-06 12:27 | fixes-build | BUG-051 (2nd-run naming, all-view tests, remaining writers, weak test) | sonnet/medium | ok | | |
| 2026-11-06 12:37 | merge-verify | BUG-051 (writers, naming, flows 2,6,8,10,11) | opus/high | ok | | GREEN; diagnostics can replace a non-open .pdf |
| 2026-11-06 12:37 | reviewer (read-only) | BUG-051 frontend naming | sonnet/medium | fail | | UX: refusal not shown, submit races |
| 2026-11-06 12:41 | fixes-build | BUG-052 robustness (refusal ErrorState, submit races, -N naming, focus, diagnostics .pdf) | sonnet/medium | ok | | |
| 2026-11-06 12:49 | merge-verify | BUG-052 backend + error path + flows 10,11 | opus/high | ok | | GREEN; symlink-to-pdf note |
| 2026-11-06 12:49 | reviewer (read-only) | BUG-052 frontend | sonnet/medium | fail | | BLOCKER Convert status banner deleted (silent errors) |
| 2026-11-06 12:53 | fixes-build | BUG-052 fix round (banner, toast, pipelines banner, diag canonical) | sonnet/medium | ok | | |
| 2026-11-06 12:59 | review+verify (combined) | BUG-052 fix delta | opus/high | ok | | GREEN, flow 10 PASS |
| 2026-11-06 13:03 | fixes-build | sweep group 11 (Forms, Security, PdfViewer) | sonnet/medium | ok | | password kept after failure (flagged) |
| 2026-11-06 13:09 | merge-verify | g11 security/redaction/password + flows 2,8 | opus/high | fail | | password not cleared on file change; blank regions default |
| 2026-11-06 13:09 | reviewer (read-only) | g11 frontend + visual | sonnet/medium | fail | | P0 redaction page off-by-one |
| 2026-11-06 13:13 | fixes-build | g11 fix round (BUG-053 P0 page, blanks, password on file change, tests, layout) | sonnet/medium | ok | | |
| 2026-11-06 13:20 | review+verify (combined) | BUG-053 + g11 (callers page base, flows 2,8, CLI redact check) | opus/high | ok | | GREEN 243; CLI page-2 check exact |
| 2026-11-06 13:25 | fixes-build | BUG-054 + BUG-055 + sweep g12 (Convert from/to) | sonnet/medium | ok | | inventory 446 in 5 files |
| 2026-11-06 13:33 | review+verify (combined) | BUG-054/055 + g12 | opus/high | ok | | GREEN 245; flows 8,11 PASS |
| 2026-11-06 13:36 | fixes-build | sweep g13 (Markdown, Pipelines) + mixed-batch test; found BUG-057 | sonnet/medium | ok | | |
| 2026-11-06 13:42 | review+verify (combined) | g13 + BUG-057 (secrets sweep) | opus/high | ok | | GREEN; found BUG-058 password in debug log |
| 2026-11-06 13:46 | fixes-build | BUG-058 secrets redaction (+ wording, opacity limits, temp test) | sonnet/medium | ok | | encrypted-open password not listed → opus checks |
| 2026-11-06 13:52 | review+verify (combined) | BUG-058 secrets (all password holders, log path, flow 2 grep) | opus/high | ok | | no leaks; owner logs clean |
| 2026-11-06 13:57 | fixes-build | sweep group 14 (AiSummarize, Batch, Settings) — last | sonnet/medium | ok | | inventory 0 in 52 files |
| 2026-11-06 14:06 | review+verify (combined) | g14 (Settings persistence, Batch/AI honesty, audit 0) | opus/high | fail | | settings save after load error writes defaults; i18n overclaims |
| 2026-11-06 14:06 | visual review | final sweep screenshots | sonnet/medium | fail | | Batch header/emoji, settings dark renders light, stuck tooltip |
| 2026-11-06 14:11 | fixes-build | g14 fix round (BUG-059 settings fail-closed, i18n overclaims, visual) | sonnet/medium | ok | | |
| 2026-11-06 14:22 | review+verify (combined) | g14 fix round (settings writers, theme, i18n keys) | opus/high | fail | | model import fails open; military-grade |
| 2026-11-06 14:22 | visual recheck | g14 screenshots | sonnet/medium | ok | | 4/4 fixed |
| 2026-11-06 14:28 | fixes-build | g14 last items (model import fail-closed, handler test, military-grade) | sonnet/medium | ok | | |
| 2026-11-06 14:33 | review+verify (combined) | g14 last items + gate + flows 10,11,23 | opus/high | ok | | functional PASS; bn.json + FAQ permissions claim → BUG-060 |
| 2026-11-06 14:40 | fixes-build | BUG-060 FAQ text (bn military-grade, permissions claim) + audit all locales | sonnet/medium | ok | | |
| 2026-11-06 14:44 | review+verify (combined) | BUG-060 text | sonnet/low | fail | | skipped 2 of 4 checks; lead covered with 2 greps (1st Sonnet miss on this task) |
| 2026-11-06 14:44 | e2e-run | full E2E on main: flows 1-12 (1 h soak) + sweep 13-23 | sonnet/low (escalation rule) | ok | | ran all; built flow 13 binary without VITE_LPDF_GALLERY (procedure miss, flag is in the flow header) |
| 2026-11-06 15:14 | docs | ARGON_TASKS.md (step 4, doc-only) | sonnet/medium | ok | | lead removed AT-08/16/17, restricted AT-15 (SpatialCanvas holds save code) |
| 2026-11-06 15:54 | log-read | flow 12 soak data + flow 13 reason | haiku/low | ok | | RSS series + code lines delivered verbatim |
| 2026-11-06 15:54 | release-build | flow 13 rerun (flagged build) + real-PDF rerun | sonnet/medium | ok | | flow 13 PASS; real-PDF steps 3,5,6 regressed → triage |
| 2026-11-06 15:55 | fixes-build | packaging prep P-a: version 0.2.0, About screen, reset preferences, build hash | sonnet/medium | ok | | Cargo.lock hand-edited (21 crates) → verifier checks |
| 2026-11-06 16:04 | release-build | triage: real-PDF regressions (driver drift vs app regression) | sonnet/medium | ok | | verdict DRIFT with evidence; driver patched outside the repo |
| 2026-11-06 16:09 | review+verify (combined) | packaging prep A (reset_preferences, Cargo.lock, build hash, About) | opus/high | ok | | GREEN 260/543; minors → BUG-064 |
| 2026-11-06 16:12 | release-build | packaging B1: measure LTO, build rpm/deb/AppImage, SHA256SUMS, SBOM, KI-006 offline proof | sonnet/medium | running | | no app instance; wt/release cold target |
| 2026-11-06 16:14 | release-build | BUG-062 counted retry + BUG-061 close-tabs soak diagnostic (in wt/fixes) | sonnet/medium | running | | harness; shares the app instance with nobody (B1 builds only) |
| 2026-11-06 16:24 | stop | owner budget stop: agents cancelled (B1 LTO measure, soak diagnostic), WIP committed on 3 branches, ARGON_TASKS section 0 written | sonnet/low | ok | | no verify run (owner) |
| 2026-11-06 17:50 | governance | lock Fedora 44 8GB RAM caps, Base Pro 4-tier hybrid routing, and Flash/Argon execution plan | argon/low | ok | | 13 roles mapped; Phase F1-F4 execution plan locked |

---

## 2026-11-06 Fedora 44 KDE Plasma (8GB RAM) — Base Pro Quota Policy & Hybrid Model Routing

### 1. 4-Tier Model Assignment for All 13 `.claude/agents/*.md` Roles
| Tier | Model & Effort | Quota Share | Assigned Agent Roles (`.claude/agents/*.md`) | Operational Rules & Context Limits |
| :--- | :--- | :--- | :--- | :--- |
| **Tier 1: Workhorse Builders & Read-Only Scouts** | **Gemini 3.8 Flash (`High`)** | **80%** | 1. `explorer`<br>2. `log-reader`<br>3. `status-snapshot`<br>4. `checkpoint-writer`<br>5. `fixes-builder` (incl. `ui-sweeper`, `doc-writer`, `soak-runner`, AT-01..15) | • Exact file paths only (zero repo-wide grep).<br>• Replies strictly `<= 150 words` (`PASS/FAIL · findings · files`).<br>• Heavy builds only via `tools/run.sh <name> <cmd>`. |
| **Tier 2: Surgical Reviewer & Packaging Architect** | **Gemini 4 Argon (`Low`)** | **15%** | 6. `reviewer` (diffs <= 200 lines)<br>7. `provenance-auditor` (`tools/sbom_npm.py`)<br>8. `release-builder` (Packaging B1/B2 & Fedora 44 `.rpm`/AppImage/`.deb`)<br>*(+ Lead Orchestrator editorial/planning)* | • Never reads raw build logs or explores directories.<br>• Inspects only pre-extracted `git diff` (`<= 200 lines`) and `<= 150-word` reports.<br>• Delegates file edits and test runs to Tier 1 (`3.8 Flash High`). |
| **Tier 3: Complex Engine & P0 Escalation** | **Gemini 4 Argon (`Medium`)** | **4%** | 9. `engines-builder` (ADR-018 craft adapters)<br>10. `bug-triager` (`BUG-061`/`BUG-062` & P0/P1/P2)<br>11. `escalation` (Tier 1 twice-failed tasks) | • Invoked ONLY when Tier 1 (`3.8 Flash High`) fails twice on the same task or for surgical Rust concurrency/geometry edits.<br>• Drop back to Tier 1/2 immediately once surgical fix compiles. |
| **Tier 4: Supreme Safety & RC Verifier** | **Claude Opus 5.5 (`High`)** *(or Argon `Medium` Fresh Session if Opus = 0%)* | **1%** | 12. `merge-verifier` (`lpdf-fs-safety`, atomic save-path, cryptographic redaction)<br>13. `rc-verifier` (`v0.2.0-rc1` final sign-off) | • Fresh context given ONLY spec/ADR, `git diff`, and `tools/verify.sh` output.<br>• Never sees builder reasoning. Never used for coding, exploration, or UI tasks. |

### 2. Base Pro Quota Throttle Tiers
- **Green Tier (Active Workhorse Execution):**
  - Execute `docs/ARGON_TASKS.md` outside §0 (`AT-01..07`, `AT-10..15`) on **Gemini 3.8 Flash (`High`)** in batched, single-gate micro-phases (`Phase F1` and `Phase F2` in `docs/FLASH_AND_ARGON_EXECUTION_PLAN.md`).
  - Execute §0 preparation (`wt/fixes` inspection, `tools/sbom_npm.py` audit prep, Packaging B1/B2 build execution) on **Gemini 3.8 Flash (`High`)**.
- **Red / Critical Tier (Protected High-Reasoning Reserve):**
  - Reserve **Gemini 4 Argon (`Low`/`Medium`)** and remaining **Claude Opus 5.5 (`High`)** strictly for §0 merge verification (`wt/fixes`, `wt/release`, `wt/engines`), `tools/sbom_npm.py` security sign-off, LTO/package verification, and `v0.2.0-rc1` final tagging.
- **Escalation Protocol:**
  - 1st failure on `3.8 Flash (High)` -> retry with exact line numbers on `3.8 Flash (High)`.
  - 2nd failure on `3.8 Flash (High)` -> escalate to `Argon (Low/Med)` (`escalation` role) and log in this table.

