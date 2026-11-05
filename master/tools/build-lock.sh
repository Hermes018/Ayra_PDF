#!/usr/bin/env bash
# Runs one agent job through the RAM-aware scheduler (tools/sched.sh) with raw output.
# Prefer tools/run.sh <name> <cmd…> (adds logs/runs/<name>.log + short summary).
# Each worktree builds in its OWN target dir (ERR-008). Never run `cargo clean`.
set -uo pipefail
TOP="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
export CARGO_TARGET_DIR="$TOP/master/target"
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
# shellcheck disable=SC1091
. "$(dirname "${BASH_SOURCE[0]}")/sched.sh"
sched_exec "build-lock:$(basename "${1:-job}")" "$@"
