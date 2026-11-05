# sched.sh — RAM-aware admission control for agent jobs (sourced by run.sh and build-lock.sh).
# Training on this machine has absolute priority (PROMPT_LOG #20); see docs/RAM_BUDGET.md.
#   LIGHT jobs: run in parallel while MemAvailable > reserve + 2 GiB.
#   HEAVY jobs: FIFO queue ordered verifier > p0 > e2e > other; admitted to a slot only if
#               MemAvailable >= class peak + reserve (slot 2 only if >= 2 x peak + reserve).
# Every admit/wait decision is logged with MemAvailable to <main>/master/logs/runs/scheduler.log.
# Env: LPDF_PRIO=verifier|p0|e2e|other   LPDF_CLASS=LIGHT|HEAVY (override)   LPDF_EXCLUSIVE=1 (both slots)

SCHED_DIR=/tmp/lpdf-sched
mkdir -p "$SCHED_DIR/queue" 2>/dev/null
SCHED_HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCHED_MAIN="$(git -C "$SCHED_HERE" rev-parse --path-format=absolute --git-common-dir 2>/dev/null | sed 's#/\.git$##')"
SCHED_LOG="${LPDF_SCHED_LOG:-$SCHED_MAIN/master/logs/runs/scheduler.log}"
mkdir -p "$(dirname "$SCHED_LOG")" 2>/dev/null
# shellcheck disable=SC1091
[ -f "$SCHED_HERE/ram-budget.env" ] && . "$SCHED_HERE/ram-budget.env"
RESERVE_MIB=$(( ${LPDF_RESERVE_G:-6} * 1024 ))
PEAK_LIGHT_MIB=${PEAK_LIGHT_MIB:-1536}
PEAK_HEAVY_MIB=${PEAK_HEAVY_MIB:-6144}
SLOT_FILES=(/tmp/lpdf-build.lock /tmp/lpdf-build.lock.2)

sched_mem_avail() { awk '/MemAvailable/{print int($2/1024)}' /proc/meminfo; }
sched_log() { printf '%s %s\n' "$(date -Is)" "$*" >> "$SCHED_LOG" 2>/dev/null; }

# sched_classify <cmd…>: prints LIGHT or HEAVY (unknown → HEAVY, the safe side).
sched_classify() {
  [ -n "${LPDF_CLASS:-}" ] && { echo "$LPDF_CLASS"; return; }
  local c=" $* "
  case "$c" in
    *" cargo fmt "*|*" cargo tree "*|*" cargo metadata "*|*" git "*|*render_tree.py*|*" true "*) echo LIGHT; return;;
    *" cargo check -p "*|*" cargo clippy -p "*) echo LIGHT; return;;
    *" xtask feature-map-check "*|*" xtask studio-audit "*|*" xtask protocol-lint "*) echo LIGHT; return;;
    *" vitest run "*.test.ts*) echo LIGHT; return;;     # vitest on named file(s)
  esac
  echo HEAVY
}

sched_prio_num() { case "${LPDF_PRIO:-other}" in verifier) echo 0;; p0) echo 1;; e2e) echo 2;; *) echo 3;; esac; }

# sched_acquire <name> <class>: blocks until admitted. Sets SCHED_FDS (fds to close for the child).
sched_acquire() {
  local name="$1" class="$2" t0 avail logged="" want slots
  t0=$(date +%s); SCHED_FDS=()
  if [ "$class" = LIGHT ]; then
    while :; do
      avail=$(sched_mem_avail)
      if [ "$avail" -gt $(( RESERVE_MIB + 2048 )) ]; then
        sched_log "ADMIT LIGHT name=$name avail_MiB=$avail waited_s=$(( $(date +%s) - t0 ))"; return 0
      fi
      [ -z "$logged" ] && { sched_log "WAIT LIGHT name=$name avail_MiB=$avail (< reserve+2GiB)"; logged=1; }
      sleep 2
    done
  fi
  local ticket="$SCHED_DIR/queue/$(sched_prio_num)-$(date +%s%N)-$$"
  : > "$ticket"
  want=1; [ "${LPDF_EXCLUSIVE:-0}" = 1 ] && want=2
  while :; do
    for t in "$SCHED_DIR"/queue/*; do            # drop tickets of dead waiters
      [ -e "$t" ] || continue; kill -0 "${t##*-}" 2>/dev/null || rm -f "$t"
    done
    local head; head=$(ls "$SCHED_DIR/queue" 2>/dev/null | sort | head -1)
    avail=$(sched_mem_avail)
    if [ "$SCHED_DIR/queue/$head" = "$ticket" ]; then
      SCHED_FDS=(); local i got=0
      for i in 0 1; do
        [ "$got" -ge "$want" ] && break
        if [ "$i" = 1 ] && [ "$want" = 1 ] && [ "$avail" -lt $(( 2 * PEAK_HEAVY_MIB + RESERVE_MIB )) ]; then break; fi
        local fd; exec {fd}>"${SLOT_FILES[$i]}"
        if flock -n "$fd"; then SCHED_FDS+=("$fd"); got=$((got+1)); else exec {fd}>&-; fi
      done
      if [ "$got" -ge "$want" ] && [ "$avail" -ge $(( PEAK_HEAVY_MIB + RESERVE_MIB )) ]; then
        rm -f "$ticket"
        slots=$(for f in "${SLOT_FILES[@]}"; do flock -n "$f" true 2>/dev/null || echo x; done | wc -l)
        sched_log "ADMIT HEAVY name=$name prio=${LPDF_PRIO:-other} avail_MiB=$avail need_MiB=$(( PEAK_HEAVY_MIB + RESERVE_MIB )) slots_held=$got waited_s=$(( $(date +%s) - t0 ))"
        return 0
      fi
      for fd in "${SCHED_FDS[@]}"; do exec {fd}>&-; done; SCHED_FDS=()
      [ "$logged" != head ] && { sched_log "WAIT HEAVY name=$name prio=${LPDF_PRIO:-other} avail_MiB=$avail need_MiB=$(( PEAK_HEAVY_MIB + RESERVE_MIB )) reason=$([ "$avail" -lt $(( PEAK_HEAVY_MIB + RESERVE_MIB )) ] && echo memory || echo slots-busy)"; logged=head; }
    else
      [ "$logged" != queued ] && [ "$logged" != head ] && { sched_log "WAIT HEAVY name=$name prio=${LPDF_PRIO:-other} avail_MiB=$avail reason=queued-behind=$head"; logged=queued; }
    fi
    sleep 2
  done
}

sched_release() { local fd; for fd in "${SCHED_FDS[@]}"; do exec {fd}>&-; done; SCHED_FDS=(); }

# sched_exec <name> <cmd…>: classify, acquire, run under protect.sh (child never inherits slot fds),
# release. Re-queues once with both slots if the job is killed (exit 137), then reports.
sched_exec() {
  local name="$1"; shift
  local class rc closes=""
  class=$(sched_classify "$@")
  sched_acquire "$name" "$class"
  # Tight budget: compile with 1 job instead of 2.
  if [ "$class" = HEAVY ] && [ "$(sched_mem_avail)" -lt $(( PEAK_HEAVY_MIB + RESERVE_MIB + 2048 )) ]; then
    export CARGO_BUILD_JOBS=1; sched_log "TIGHT name=$name → CARGO_BUILD_JOBS=1"
  fi
  for fd in "${SCHED_FDS[@]}"; do closes+=" $fd>&-"; done
  eval "\"$SCHED_HERE/protect.sh\" \"\$@\" $closes"; rc=$?
  sched_release
  if [ "$rc" = 137 ] && [ "${LPDF_REQUEUED:-0}" != 1 ]; then
    sched_log "KILLED name=$name class=$class rc=137 (likely memory) → re-queue once exclusive"
    LPDF_REQUEUED=1 LPDF_EXCLUSIVE=1 LPDF_CLASS=HEAVY sched_exec "$name" "$@"; rc=$?
    [ "$rc" = 137 ] && sched_log "KILLED-AGAIN name=$name rc=137 → giving up, report to lead"
  fi
  sched_log "DONE name=$name class=$class rc=$rc"
  return "$rc"
}
