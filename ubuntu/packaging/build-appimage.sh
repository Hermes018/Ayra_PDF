#!/usr/bin/env bash
# Builds the LocalPDF Studio AppImage (portable; built on the Ubuntu 24.04 base) from the shared
# master/ workspace. The base config is merged with ubuntu/packaging/tauri.appimage.conf.json via
# the Tauri CLI --config overlay (JSON merge patch).
#
#   ubuntu/packaging/build-appimage.sh --prefetch   download the pinned AppImage tools once (network)
#   ubuntu/packaging/build-appimage.sh --check-tools verify + install the cached tools, no build
#   ubuntu/packaging/build-appimage.sh              build; works offline once the tools are cached
#
# Run this script directly, NOT wrapped in tools/run.sh: each heavy step below goes through
# master/tools/run.sh itself (RAM scheduler + build lock), and nesting run.sh would deadlock.
#
# Tool pinning: the Tauri bundler would otherwise download linuxdeploy, its AppImage plugin and
# AppRun from GitHub on first use, including the moving "continuous" plugin tag. Here every tool
# comes from an immutable release URL, is checked against a SHA-256 before use, and is placed in
# Tauri's tool cache (~/.cache/tauri), where the bundler finds it and skips its own download. The
# GTK/GStreamer plugin scripts are embedded in the Tauri CLI, so nothing is downloaded for them.
# To update a pin: change URL + SHA-256 together, after reviewing the upstream release.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MASTER="$ROOT/master"
OVERLAY="$ROOT/ubuntu/packaging/tauri.appimage.conf.json"
RUN="$MASTER/tools/run.sh"
CACHE="${LPDF_APPIMAGE_TOOLS:-${XDG_CACHE_HOME:-$HOME/.cache}/lpdf/appimage-tools}"
TAURI_TOOLS="${XDG_CACHE_HOME:-$HOME/.cache}/tauri"

# name-in-tauri-cache | immutable URL | sha256 of the upstream download | sha256 as Tauri expects it
# The Tauri bundler zeroes the AppImage magic bytes (offsets 8-10, "AI\x02") of linuxdeploy after
# downloading it (so it runs without binfmt AppImage detection); we apply the same patch, so the
# file in Tauri's cache is byte-identical to what the bundler would have produced.
TOOLS=(
  "linuxdeploy-07333c6-x86_64.AppImage|https://github.com/tauri-apps/binary-releases/releases/download/linuxdeploy-07333c6/linuxdeploy-x86_64.AppImage|36a2d7e274d12e1050d0e9ecfe11d339ed54720b2bec464c286d53f8b07f5c62|bd9521cd5ff3ca351fecb78cd9b236a4e37b3621014ac05cac0c2d767ffa0474"
  "linuxdeploy-plugin-appimage.AppImage|https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/1-alpha-20250213-1/linuxdeploy-plugin-appimage-x86_64.AppImage|992d502a248e14ab185448ddf6f6e7d25558cb84d4623c354c3af350c25fccb3|992d502a248e14ab185448ddf6f6e7d25558cb84d4623c354c3af350c25fccb3"
  "AppRun-x86_64|https://github.com/tauri-apps/binary-releases/releases/download/apprun-old/AppRun-x86_64|f30140a43a0a59e46db21bdefdf749b9e9f2c6946e92afabbacf98b8ae73fb4f|f30140a43a0a59e46db21bdefdf749b9e9f2c6946e92afabbacf98b8ae73fb4f"
)

sha_ok() { [ -f "$1" ] && [ "$(sha256sum "$1" | cut -d' ' -f1)" = "$2" ]; }

prefetch() {
  mkdir -p "$CACHE"
  local entry name url sum _installed
  for entry in "${TOOLS[@]}"; do
    IFS='|' read -r name url sum _installed <<<"$entry"
    if sha_ok "$CACHE/$name" "$sum"; then echo "  $name: cached"; continue; fi
    echo "  $name: downloading $url"
    curl -fsSL --retry 2 -o "$CACHE/$name.part" "$url"
    if ! sha_ok "$CACHE/$name.part" "$sum"; then
      echo "SHA-256 mismatch for $name ($url): got $(sha256sum "$CACHE/$name.part" | cut -d' ' -f1), want $sum" >&2
      rm -f "$CACHE/$name.part"
      exit 1
    fi
    chmod +x "$CACHE/$name.part"
    mv -f "$CACHE/$name.part" "$CACHE/$name"
  done
}

install_tools() {
  # Verify the cache, then place verified copies where the Tauri bundler looks (replacing anything
  # it may have downloaded earlier), and re-verify the placed copies.
  mkdir -p "$TAURI_TOOLS"
  local entry name url sum installed
  for entry in "${TOOLS[@]}"; do
    IFS='|' read -r name url sum installed <<<"$entry"
    if ! sha_ok "$CACHE/$name" "$sum"; then
      echo "$name missing or wrong hash in $CACHE: run '$0 --prefetch' first (needs network once)." >&2
      exit 1
    fi
    install -m 0755 "$CACHE/$name" "$TAURI_TOOLS/$name.part"
    if [ "$sum" != "$installed" ]; then
      printf '\000\000\000' | dd of="$TAURI_TOOLS/$name.part" bs=1 seek=8 count=3 conv=notrunc status=none
    fi
    sha_ok "$TAURI_TOOLS/$name.part" "$installed" || { echo "$name failed verification after install" >&2; exit 1; }
    mv -f "$TAURI_TOOLS/$name.part" "$TAURI_TOOLS/$name"
  done
}

if [ "${1:-}" = "--prefetch" ]; then
  echo "Prefetching pinned AppImage tools into $CACHE ..."
  prefetch
  echo "Done. Later builds need no network."
  exit 0
fi

if [ "${1:-}" = "--check-tools" ]; then
  install_tools
  echo "Pinned AppImage tools verified and installed in $TAURI_TOOLS."
  exit 0
fi

command -v pnpm >/dev/null || { echo "pnpm not on PATH: run 'corepack enable' once, then retry." >&2; exit 1; }
install_tools

cd "$MASTER"
"$RUN" appimage-cargo cargo build --locked --offline --release -p lpdf-cli -p lpdf-worker
# Offline install from the existing pnpm store, exactly the locked versions.
"$RUN" appimage-pnpm-install corepack pnpm --dir app-tauri install --offline --frozen-lockfile
# NO_STRIP: linuxdeploy's bundled (older) strip is known to fail on newer distro libraries
# (.relr.dyn); the release binary is already stripped by the cargo profile.
NO_STRIP=true "$RUN" appimage-bundle corepack pnpm --dir app-tauri tauri build --bundles appimage --config "$OVERLAY"
echo "Package written to: $MASTER/target/release/bundle/appimage/"
