#!/usr/bin/env bash
# Builds the LocalPDF Studio .rpm package from the shared master/ workspace.
# The base config (master/app-tauri/src-tauri/tauri.conf.json) is merged with
# fedora/packaging/tauri.fedora.conf.json via the Tauri CLI --config overlay (JSON merge patch).
# Set SB_RELEASE=1 to run under master/tools/sb.sh (cgroup-limited, no LTO) on low-RAM hosts.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MASTER="$ROOT/master"
OVERLAY="$ROOT/fedora/packaging/tauri.fedora.conf.json"
SB=()
if [ "${SB_RELEASE:-0}" = "1" ]; then SB=("$MASTER/tools/sb.sh"); fi

# tauri.conf.json's beforeBuildCommand calls a bare `pnpm`; corepack provides it.
command -v pnpm >/dev/null || { echo "pnpm not on PATH: run 'corepack enable' once, then retry." >&2; exit 1; }

cd "$MASTER"
# The package ships the CLI and the sandboxed worker next to the GUI binary (see overlay "files").
"${SB[@]}" cargo build --release -p lpdf-cli -p lpdf-worker
"${SB[@]}" corepack pnpm --dir app-tauri install --frozen-lockfile
"${SB[@]}" corepack pnpm --dir app-tauri tauri build --bundles rpm --config "$OVERLAY"
echo "Package written to: $MASTER/target/release/bundle/rpm/"
