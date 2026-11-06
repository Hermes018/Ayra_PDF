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
| 2026-11-06 09:28 | explore | UI screen inventory | haiku/low (agent file) | running | | |
| 2026-11-06 09:28 | fixes-build | studio-audit over all .svelte (A2) | sonnet/high (agent file) | running | | |
