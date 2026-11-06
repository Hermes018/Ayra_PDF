#!/usr/bin/env bash
# SessionStart hook (compact|resume|startup): print the living handoff so it lands in context.
# Total output is capped at ~9,000 chars: above ~10k Claude Code stores hook output in a file and
# injects only a 2 KB preview (observed 2026-11-06 07:00, 14 KB output → preview only; ERROR_LOG).
# Always exits 0.
set +e
ROOT="${CLAUDE_PROJECT_DIR:-/home/rakib/Desktop/Ayra_PDF}"
L="$ROOT/master/logs"
CK_MAX=5600   # CHECKPOINT.md budget (chars)
SA_MAX=2600   # STATE_AUTO.md budget (chars)

cap() { # cap <file> <max-chars> [label]: print up to max chars, cut at a line end, note truncation
  local f="$1" max="$2" label="${3:-$1}" size
  [ -f "$f" ] || { echo "(missing: $f — run master/tools/snapshot-state.sh)"; return; }
  size=$(wc -c < "$f")
  if [ "$size" -le "$max" ]; then cat "$f"; else
    head -c "$max" "$f" | sed '$d'
    echo "… [truncated: $size chars total — Read $label for the rest]"
  fi
}

echo "=== Context was compacted (or session resumed). Restore from these files before doing anything else. ==="
echo "----- master/logs/CHECKPOINT.md -----"
cap "$L/CHECKPOINT.md" "$CK_MAX"
echo "----- master/logs/STATE_AUTO.md (facts at last compact; trust over CHECKPOINT) -----"
# Facts that matter most first: worktrees, processes, disk/RAM, locks. Skip the activity tail.
if [ -f "$L/STATE_AUTO.md" ]; then
  awk '/^## Last 10 activity/{exit} {print}' "$L/STATE_AUTO.md" > "${TMPDIR:-/tmp}/lpdf-sa.$$"
  cap "${TMPDIR:-/tmp}/lpdf-sa.$$" "$SA_MAX" "$L/STATE_AUTO.md"; rm -f "${TMPDIR:-/tmp}/lpdf-sa.$$"
else echo "(missing — run master/tools/snapshot-state.sh)"; fi
echo "INSTRUCTION: Re-read master/logs/BUGS.md and the checkpoints/<worktree>.md of any worktree you touch. If CHECKPOINT and STATE_AUTO disagree, trust STATE_AUTO and fix CHECKPOINT. Then continue the next action without waiting for the owner."
exit 0
