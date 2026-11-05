#!/usr/bin/env bash
# tools/verify.sh — THE definition of green (ADR-020 / PROMPT_LOG #18). Lead, builders and
# verifiers all run exactly this, from inside the worktree being verified:
#   master/tools/verify.sh            # full: Rust gate + frontend + project checks
#   master/tools/verify.sh --rust     # Rust gate only      --web  frontend only
# Every step goes through tools/run.sh (build lock, logs in master/logs/runs/). All steps run
# even after a failure, so one run reports everything; exit 1 if any step failed.
set -uo pipefail
mode="${1:-all}"
top="$(git rev-parse --show-toplevel)"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
m="$top/master"; run="$here/run.sh"   # scripts from this copy; checks run on the worktree in $m
cd "$m" || exit 2
results=(); failed=0
step() { local n="$1"; shift; if "$run" "verify-$n" "$@"; then results+=("PASS $n"); else results+=("FAIL $n"); failed=1; fi; }
if [ "$mode" = all ] || [ "$mode" = --rust ]; then
  step gate cargo xtask gate                       # fmt, clippy, nextest, deny, audit, protocol-lint, android check
  step feature-map cargo xtask feature-map-check
  step studio-audit cargo xtask studio-audit --strict
fi
if [ "$mode" = all ] || [ "$mode" = --web ]; then
  step svelte-check pnpm --dir app-tauri check
  step vitest pnpm --dir app-tauri exec vitest run --maxWorkers=2
fi
step tree-map python3 tools/tree/render_tree.py --check
git -C "$top" checkout -- master/logs/sbom.cdx.json 2>/dev/null || true   # gate regenerates it
echo "== verify.sh ($mode) @ $(git -C "$top" rev-parse --short HEAD) $(git -C "$top" branch --show-current)"
printf '  %s\n' "${results[@]}"
[ "$failed" -eq 0 ] && echo "VERIFY: GREEN" || echo "VERIFY: RED"
exit "$failed"
