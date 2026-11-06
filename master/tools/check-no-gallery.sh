#!/usr/bin/env bash
# tools/check-no-gallery.sh [dir] — fail if the frontend bundle contains the component-gallery marker.
# Build WITHOUT VITE_LPDF_GALLERY first (run.sh <name> bash -c 'cd app-tauri && pnpm build'), then run this.
# With --expect-present the check is inverted (positive control for a VITE_LPDF_GALLERY=1 build).
set -uo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mode=absent; [ "${1:-}" = "--expect-present" ] && { mode=present; shift; }
dir="${1:-$here/../app-tauri/build}"
marker=$(sed -n "s/.*GALLERY_MARKER = '\([^']*\)'.*/\1/p" "$here/../app-tauri/src/lib/gallery/marker.ts")
[ -n "$marker" ] || { echo "[no-gallery] FAIL: cannot read the marker from app-tauri/src/lib/gallery/marker.ts" >&2; exit 2; }
[ -d "$dir" ] || { echo "[no-gallery] FAIL: bundle dir not found: $dir (build the frontend first)" >&2; exit 2; }
n_files=$(find "$dir" -type f | wc -l)
hits=$(grep -rl --binary-files=text "$marker" "$dir" 2>/dev/null || true)
if [ "$mode" = absent ]; then
  if [ -n "$hits" ]; then echo "[no-gallery] FAIL: gallery marker found in:"; echo "$hits"; exit 1; fi
  echo "[no-gallery] PASS: marker absent from $n_files bundle files in $dir"
else
  if [ -z "$hits" ]; then echo "[no-gallery] FAIL: marker not found in a gallery-enabled bundle ($dir)"; exit 1; fi
  echo "[no-gallery] PASS (control): marker present in gallery-enabled bundle"
fi
