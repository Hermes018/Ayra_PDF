#!/usr/bin/env bash
# tools/run.sh <name> <cmd...> — run one heavy command through the shared build lock, keep the
# full output in master/logs/runs/<name>.log (git-ignored), and print only a short summary:
# exit status, duration, failing test names, and the first 30 error lines. Open the log only
# when the summary isn't enough. Works from any worktree (logs go to that worktree).
set -uo pipefail
if [ $# -lt 2 ]; then echo "usage: tools/run.sh <name> <cmd...>" >&2; exit 64; fi
name="$1"; shift
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
top="$(git rev-parse --show-toplevel 2>/dev/null || (cd "$here/../.." && pwd))"
logdir="$top/master/logs/runs"; mkdir -p "$logdir"
log="$logdir/${name//[^A-Za-z0-9._-]/_}.log"
start=$(date +%s)
{ echo "# cmd: $*"; echo "# cwd: $(pwd)"; echo "# started: $(date -Is)"; } > "$log"
"$here/build-lock.sh" "$@" >> "$log" 2>&1
rc=$?
dur=$(( $(date +%s) - start ))
echo "# exit: $rc (${dur}s)" >> "$log"
if [ "$rc" -eq 0 ]; then
  echo "[run] $name: PASS (${dur}s) log=$log"
else
  echo "[run] $name: FAIL exit=$rc (${dur}s) log=$log"
  fails=$(grep -aE '^\s+FAIL \[|^test .* \.\.\. FAILED$|^ (FAIL|×) |✗ |^---- .* stdout ----$' "$log" | sed -E 's/^\s+//' | sort -u | head -20)
  [ -n "$fails" ] && { echo "[run] failing tests:"; echo "$fails" | sed 's/^/  /'; }
  echo "[run] first error lines:"
  grep -aE '^error(\[|:)|^Error|ERROR|panicked at|^\s*(FAIL|×)|Gate failed|error TS|^\s+Error:' "$log" | head -30 | sed 's/^/  /'
fi
exit "$rc"
