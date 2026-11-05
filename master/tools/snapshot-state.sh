#!/usr/bin/env bash
# PreCompact hook (auto|manual): write master/logs/STATE_AUTO.md from FACTS (git, processes, disk),
# not memory. Read-only on the repo except this one generated (git-ignored) file. Every probe has
# a timeout so the whole script stays under ~5 s, and it always exits 0: it must never fail a compact.
set +e
ROOT="${CLAUDE_PROJECT_DIR:-/home/rakib/Desktop/Ayra_PDF}"
OUT="$ROOT/master/logs/STATE_AUTO.md"
T() { timeout "${1}" "${@:2}" 2>/dev/null; }
{
  echo "# STATE_AUTO — generated $(date -Is) by tools/snapshot-state.sh (facts; trust this over CHECKPOINT.md)"
  echo
  echo "## Worktrees"
  echo '```'
  T 2 git -C "$ROOT" worktree list
  echo '```'
  T 2 git -C "$ROOT" worktree list --porcelain | awk '/^worktree /{print $2}' | while read -r wt; do
    echo "### $(basename "$wt") — $(T 1 git -C "$wt" branch --show-current)"
    echo '```'
    T 1 git -C "$wt" log -3 --oneline
    echo "-- status (first 20):"
    T 1 git -C "$wt" status --short | head -20
    echo '```'
  done
  echo
  echo "## Open P0/P1 (from logs/BUGS.md)"
  grep -E '^## BUG-[0-9]+ \[P[01]\].*status: (open|fixing)' "$ROOT/master/logs/BUGS.md" 2>/dev/null | head -30
  echo
  echo "## Build/test processes running"
  echo '```'
  ps -eo pid,etimes,args 2>/dev/null | grep -E '[c]argo |[r]ustc |[n]extest|[n]ode .*(vite|vitest|svelte)|[t]auri|[a]pp-tauri|[t]auri-driver|WebKitWebDriver' | cut -c1-160 | head -15 || true
  echo '```'
  echo
  echo "## Resources"
  echo "- disk: $(df -h --output=avail,pcent "$ROOT" 2>/dev/null | tail -1 | xargs) free/used"
  echo "- ram: $(free -g 2>/dev/null | awk '/^Mem:/{print $7" GB available of "$2" GB"}')"
  echo "- sccache: $(T 2 sccache --show-stats | grep -E 'Cache size|Cache hits \(Rust\)' | tr -s ' ' | paste -sd ';')"
  echo "- build lock held: $( (T 1 flock -n /tmp/lpdf-build.lock true && echo no) || echo yes)"
  echo
  echo "## Last 10 agent activity lines"
  tail -10 "$ROOT/master/logs/AGENT_ACTIVITY.md" 2>/dev/null
} > "$OUT.tmp" 2>/dev/null && mv -f "$OUT.tmp" "$OUT"
exit 0
