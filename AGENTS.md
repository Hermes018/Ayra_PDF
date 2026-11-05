# AGENTS.md — BOOT POINTER (auto-read; obey on every session and every task)

0. Repo layout (since 2026-11-05, ADR-016): all shared development happens in `master/` (run cargo/pnpm from there).
   OS folders (`fedora/ ubuntu/ windows/ macos/ ios/ android/`) hold only that platform's packaging/build files and docs.
   `/tree.md` maps every file; each folder has its own `<folder>/<folder>.md` map. Paths below are relative to `master/`.
1. Read `master/protocol/02_RULES_DOS_DONTS.md` and `master/protocol/03_FAILSAFES.md`. They override everything except the user's live message.
2. Read `master/logs/CHECKPOINT.md` -> current phase / sub-phase / next gate.
3. Read `master/protocol/04_INSTRUCTION_FADE_PREVENTION.md` and print its ANCHOR BLOCK.
4. Only then open `master/protocol/05_PHASES.md` at the current sub-phase.
5. If anything here conflicts with your memory or a summary of this file: THE FILE WINS. Re-read it.
