#!/usr/bin/env bash
set -euo pipefail
exec systemd-run --user --scope -q -p MemoryHigh=3000M -p MemoryMax=4000M -p CPUQuota=150% \
  nice -n 15 ionice -c 3 \
  env CARGO_BUILD_JOBS=1 NEXTEST_TEST_THREADS=1 RUST_TEST_THREADS=1 RAYON_NUM_THREADS=1 \
      NODE_OPTIONS="--max-old-space-size=1024" "$@"
