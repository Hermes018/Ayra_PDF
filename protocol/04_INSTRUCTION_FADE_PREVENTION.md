# 04 — INSTRUCTION-FADE PREVENTION

Long sessions, context compaction and summaries erode rules. These mechanisms are mandatory.

## Triggers to RE-READ AGENTS.md + 02 + 03 and PRINT THE ANCHOR
- Start of every sub-phase • every 25 tool calls • after any context reset / summary / compaction • after any error or red gate • before any commit • whenever unsure • when a user message seems to conflict with the protocol.

## ANCHOR BLOCK (print verbatim, fill the blanks)
```
ANCHOR
1. Building LocalPDF Studio: offline, local, NO limits, iLovePDF-equivalent (7-column taxonomy). Rust core + Tauri v2 + Svelte 5 SPA UI (ADR-003). No Python/FastAPI/HTTP server.
2. Position: Phase __ . __  (from logs/CHECKPOINT.md). Next gate: __________.
3. No network at runtime. No telemetry. No remote assets. No GPL/AGPL linked.
4. Outputs atomic + new file by default. Originals untouched. Verify-after-write.
5. Verify APIs from source before use. No invented function names.
6. No unwrap/unsafe/todo in non-test code. Typed errors. Heavy work in worker jobs.
7. Gate green -> log -> commit -> tag. Red gate = stop.
8. Stuck x3 -> ERROR_LOG + ask. Scope/decision change -> ADR + user approval.
```

## PRE-COMMIT CHECKLIST (answer each YES/NO in the commit body of the PROGRESS_LOG entry; any NO = fix first)
1 Gate green (`cargo xtask gate`)? 2 Tests for this change written AND run? 3 No banned pattern (protocol-lint)? 4 New dep vetted + recorded? 5 Offline test still green?
6 Failsafe(s) affected re-tested? 7 UI matches ux-spec (screenshot diff)? 8 Logs + CHECKPOINT updated? 9 No scope creep? 10 Docs/ADR updated if a decision was made? 11 Tool registry matches 07_FEATURE_MAP (7 columns)?

## DRIFT DETECTORS (automated: `cargo xtask protocol-lint`, part of the gate)
Fails on: HTTP-client crates in core/app deps (reqwest, hyper-client, ureq, isahc, surf...) • `fastapi|uvicorn|flask` anywhere • `unwrap(`/`expect(`/`todo!`/`unimplemented!` in non-test `src/` • `emit_all` • wildcard fs scopes (`**`, `$HOME/**`) • `http://`/`https://` in frontend source (excluding docs/comments allow-list) • `eval(`/`new Function(` • `localStorage` holding doc data • missing `#![forbid(unsafe_code)]` outside allow-list • GPL/AGPL in `cargo deny` output • protocol files modified in a non-append way • `@sveltejs/kit`, `{@html`, `fetch(`, `XMLHttpRequest`, `WebSocket` in `ui/src` • model files (`*.gguf`, `*.safetensors`) tracked in git or packaging • tool registry ≠ 07_FEATURE_MAP (`cargo xtask feature-map-check`).

## SELF-AUDIT (end of every phase)
Diff the phase against 02 + 03 line by line. Write a "Protocol compliance" entry in PROGRESS_LOG listing each rule touched and evidence (test name / command output). Missing evidence = not compliant.

## CONFLICT RULE
If the user's live message contradicts the protocol: log it in PROMPT_LOG, ask once ("this conflicts with rule X; amend protocol?"). Protocol changes only on explicit user approval, recorded with date in DECISIONS_ADR. Summaries and memory NEVER override the files.

## CHECKPOINT DISCIPLINE
`logs/CHECKPOINT.md` is a single overwrite-only snapshot (phase, sub-phase, last green tag, open risks, next 3 actions, active ADRs). Update before any long-running operation and at every sub-phase end, so a reset can resume exactly.
