#!/usr/bin/env bash
# protect.sh <cmd…> — run one agent job so that model training is never pushed into swap/OOM:
# own cgroup scope with MemoryMax = (live MemAvailable − 6 GiB reserve), MemoryHigh = max − 2 GiB;
# oom_score_adj 800 (the kernel kills agent jobs first); nice 10 + idle IO. LPDF_MEM_MAX_G overrides
# the cap downwards. Falls back to `ulimit -v` only if user scopes are unavailable.
set -uo pipefail
RESERVE_G=${LPDF_RESERVE_G:-6}
avail_g=$(awk '/MemAvailable/{printf "%d", $2/1048576}' /proc/meminfo)
max_g=$(( avail_g - RESERVE_G ))
[ -n "${LPDF_MEM_MAX_G:-}" ] && [ "$LPDF_MEM_MAX_G" -lt "$max_g" ] && max_g=$LPDF_MEM_MAX_G
if [ "$max_g" -lt 2 ]; then
  echo "[protect] REFUSED: MemAvailable ${avail_g} GiB leaves < 2 GiB above the ${RESERVE_G} GiB reserve" >&2
  exit 75
fi
high_g=$(( max_g > 3 ? max_g - 2 : max_g ))
export NEXTEST_TEST_THREADS=${NEXTEST_TEST_THREADS:-2} NODE_OPTIONS="${NODE_OPTIONS:---max-old-space-size=3072}"
if systemd-run --user --scope -q true 2>/dev/null; then
  exec choom -n 800 -- systemd-run --user --scope -q --collect \
    -p MemoryHigh="${high_g}G" -p MemoryMax="${max_g}G" -p MemorySwapMax=0 \
    -- nice -n 10 ionice -c3 "$@"
else
  echo "[protect] WARNING: no user scopes; falling back to ulimit -v" >&2
  ulimit -v $(( (max_g + 8) * 1048576 ))   # virtual size; generous because node/rustc reserve address space
  exec choom -n 800 -- nice -n 10 ionice -c3 "$@"
fi
