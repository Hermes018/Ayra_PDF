#!/usr/bin/env bash
# protect.sh <cmd…> — run one agent job so that model training is never pushed into swap/OOM:
# own cgroup scope with MemoryMax = (live MemAvailable − 6 GiB reserve), MemoryHigh = max − 2 GiB;
# oom_score_adj 800 (the kernel kills agent jobs first); nice 10 + idle IO. LPDF_MEM_MAX_G overrides
# the cap downwards. Falls back to `ulimit -v` only if user scopes are unavailable.
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[ -f "$here/ram-budget.env" ] && . "$here/ram-budget.env"
avail_mb=$(awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo)
min_guard=${MIN_AVAIL_MIB:-2200}
if [ "$avail_mb" -lt "$min_guard" ]; then
  echo "[protect] REFUSED: MemAvailable ${avail_mb} MiB leaves less than the ${min_guard} MiB guard" >&2
  exit 75
fi
RESERVE_G=${LPDF_RESERVE_G:-1}
max_g=$(( (avail_mb - (RESERVE_G * 1024)) / 1024 ))
[ "$max_g" -lt 2 ] && max_g=2
[ -n "${LPDF_MEM_MAX_G:-}" ] && [ "$LPDF_MEM_MAX_G" -lt "$max_g" ] && max_g=$LPDF_MEM_MAX_G
high_g=$(( max_g > 3 ? max_g - 2 : max_g ))
export NEXTEST_TEST_THREADS=${NEXTEST_TEST_THREADS:-2} NODE_OPTIONS="${NODE_OPTIONS:---max-old-space-size=2048}"
if systemd-run --user --scope -q true 2>/dev/null; then
  exec choom -n 800 -- systemd-run --user --scope -q --collect \
    -p MemoryHigh="${high_g}G" -p MemoryMax="${max_g}G" -p MemorySwapMax=0 \
    -- nice -n 10 ionice -c3 "$@"
else
  echo "[protect] WARNING: no user scopes; falling back to ulimit -v" >&2
  ulimit -v $(( (max_g + 8) * 1048576 ))   # virtual size; generous because node/rustc reserve address space
  exec choom -n 800 -- nice -n 10 ionice -c3 "$@"
fi
