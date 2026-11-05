#!/usr/bin/env bash
# SessionStart hook (compact|resume|startup): print the living handoff so it lands in context.
# Output is capped (~300 lines). Always exits 0.
set +e
ROOT="${CLAUDE_PROJECT_DIR:-/home/rakib/Desktop/Ayra_PDF}"
L="$ROOT/master/logs"
echo "=== Context was compacted (or session resumed). Restore from these files before doing anything else. ==="
echo
echo "----- master/logs/CHECKPOINT.md -----"
head -n 160 "$L/CHECKPOINT.md" 2>/dev/null
echo
echo "----- master/logs/STATE_AUTO.md (facts at last compact) -----"
if [ -f "$L/STATE_AUTO.md" ]; then head -n 120 "$L/STATE_AUTO.md"; else echo "(missing — run master/tools/snapshot-state.sh)"; fi
echo
echo "INSTRUCTION: Re-read master/logs/BUGS.md and the master/logs/checkpoints/<worktree>.md of any worktree you're about to touch. If CHECKPOINT.md and STATE_AUTO.md disagree, trust STATE_AUTO.md (git/process facts) and fix CHECKPOINT.md. Then continue the next action without waiting for the owner."
exit 0
