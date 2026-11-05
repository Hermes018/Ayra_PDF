#!/usr/bin/env bash
# Samples the training job's total RSS (Jupyter kernel running swin_v2 + all descendants, e.g. dataloader
# workers) every 30 s into logs/runs/training-mem.log, so the RAM budget uses a measured peak. Read-only.
LOG="$(cd "$(dirname "$0")/.." && pwd)/logs/runs/training-mem.log"
while true; do
  root=$(pgrep -f 'swin_v2_training.*ipykernel_launcher' | head -1)
  if [ -n "$root" ]; then
    pids="$root $(pgrep -d' ' -P "$root") $(for c in $(pgrep -P "$root"); do pgrep -d' ' -P "$c"; done)"
    kb=0; n=0; for p in $pids; do r=$(awk '/VmRSS/{print $2}' /proc/$p/status 2>/dev/null); [ -n "$r" ] && kb=$((kb+r)) && n=$((n+1)); done
    hwm=$(awk '/VmHWM/{print $2}' /proc/$root/status 2>/dev/null)
    echo "$(date -Is) procs=$n total_rss_MiB=$((kb/1024)) kernel_hwm_MiB=$((hwm/1024)) memavail_MiB=$(( $(awk '/MemAvailable/{print $2}' /proc/meminfo)/1024 ))" >> "$LOG"
  fi
  sleep 30
done
