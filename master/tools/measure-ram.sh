#!/usr/bin/env bash
# measure-ram.sh <label> <cmd…> — run one job (under the build lock + protect.sh scope) and report
# the whole-cgroup peak (memory.peak = all processes of the job) and GNU time's largest single
# process RSS. Appends "label peak_cgroup_MiB maxrss_single_MiB seconds exit" to logs/runs/ram-measure.tsv.
set -uo pipefail
label="$1"; shift
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
top="$(git rev-parse --show-toplevel)"; out="$top/master/logs/runs/ram-measure.tsv"; mkdir -p "$(dirname "$out")"
tmp=$(mktemp)
"$here/build-lock.sh" bash -c '
  /usr/bin/time -v -o "$0.time" "$@" >/dev/null 2>&1; rc=$?
  cg=$(cut -d: -f3 /proc/self/cgroup); echo "$(( $(cat /sys/fs/cgroup$cg/memory.peak) / 1048576 )) $rc" > "$0"
' "$tmp" "$@"
read -r peak rc < "$tmp"
maxrss=$(awk -F: '/Maximum resident set size/{printf "%d", $2/1024}' "$tmp.time")
secs=$(awk -F': ' '/Elapsed \(wall clock\)/{split($2,a,":"); s=0; for(i=1;i<=length(a);i++) s=s*60+a[i]; printf "%.1f", s}' "$tmp.time")
printf "%s\t%s\t%s\t%s\t%s\n" "$label" "$peak" "$maxrss" "$secs" "$rc" | tee -a "$out"
rm -f "$tmp" "$tmp.time"
