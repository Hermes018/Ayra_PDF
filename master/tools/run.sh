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
# Disk guard (2026-11-06): refuse to start below 15 GB free; report instead of filling the disk.
MIN_FREE_GB="${LPDF_MIN_FREE_GB:-15}"
free_gb=$(df -BG --output=avail "$top" | tail -1 | tr -dc '0-9')
if [ "${free_gb:-0}" -lt "$MIN_FREE_GB" ]; then
  echo "[run] $name: REFUSED — only ${free_gb} GB free on $(df --output=target "$top" | tail -1) (< ${MIN_FREE_GB} GB). Free disk or drop to 2 builders; do not delete caches mid-sprint." >&2
  exit 75
fi
# Start sccache OUTSIDE any lock, so its daemon never inherits a lock fd (deadlock 2026-11-06).
command -v sccache >/dev/null 2>&1 && sccache --start-server >/dev/null 2>&1 || true
start=$(date +%s)
{ echo "# cmd: $*"; echo "# cwd: $(pwd)"; echo "# started: $(date -Is)"; } > "$log"
( TOP="$top"; export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" CARGO_TARGET_DIR="$top/master/target"
  [ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
  . "$here/sched.sh"; sched_exec "$name" "$@" ) >> "$log" 2>&1
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
