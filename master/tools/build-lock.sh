#!/usr/bin/env bash
# Runs one heavy build command (cargo / pnpm / gradle) under the shared build lock, so only one
# heavy build runs at a time across all worktrees and agents (ADR-020 agent workflow).
# Usage: master/tools/build-lock.sh cargo xtask gate
#        master/tools/build-lock.sh pnpm --dir app-tauri exec vitest run
# Always CARGO_BUILD_JOBS=2; never run `cargo clean`.
set -euo pipefail
LOCK=/tmp/lpdf-build.lock
# Each worktree uses ITS OWN target dir (2026-11-06): Cargo names workspace crates by their path
# relative to the workspace root, so a shared target dir lets one worktree's artifacts pass as
# up to date in another (wrong code under test). sccache still shares dependency builds.
TOP="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$TOP/master/target"
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
echo "[build-lock] waiting for $LOCK (target dir: $CARGO_TARGET_DIR)" >&2
exec flock "$LOCK" "$@"
