#!/usr/bin/env bash
# Runs one heavy build command (cargo / pnpm / gradle) under the shared build lock, so only one
# heavy build runs at a time across all worktrees and agents (ADR-020 agent workflow).
# Usage: master/tools/build-lock.sh cargo xtask gate
#        master/tools/build-lock.sh pnpm --dir app-tauri exec vitest run
# Always CARGO_BUILD_JOBS=2. Worktrees share master/target via CARGO_TARGET_DIR (disk is tight);
# never run `cargo clean`.
set -euo pipefail
LOCK=/tmp/lpdf-build.lock
MAIN_TARGET="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && git rev-parse --path-format=absolute --git-common-dir 2>/dev/null | sed 's#/\.git$##')/master/target"
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$MAIN_TARGET}"
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
echo "[build-lock] waiting for $LOCK (target dir: $CARGO_TARGET_DIR)" >&2
exec flock "$LOCK" "$@"
