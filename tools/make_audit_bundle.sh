#!/usr/bin/env bash
# tools/make_audit_bundle.sh — builds audit_bundle.tar.gz for an external reviewer (Claude).
# Run from anywhere inside the repo. Read-only except for writing ./audit_bundle*.
# Heavy cargo commands run through the cgroup wrapper (8 GB host). Failures of individual
# evidence steps are recorded, not fatal.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"

OUT=audit_bundle
rm -rf "$OUT" "$OUT.tar.gz"
mkdir -p "$OUT/evidence" "$OUT/meta"

SB() {
  if [ -x tools/sb.sh ]; then tools/sb.sh "$@"; else
    systemd-run --user --scope -q -p MemoryHigh=3000M -p MemoryMax=4000M -p CPUQuota=150% \
      nice -n 15 ionice -c 3 env CARGO_BUILD_JOBS=1 NEXTEST_TEST_THREADS=1 RUST_TEST_THREADS=1 \
      RAYON_NUM_THREADS=1 NODE_OPTIONS="--max-old-space-size=1024" "$@"
  fi
}
run() { # run <name> <cmd...>   (captures stdout+stderr, never aborts the script)
  local name="$1"; shift
  echo "== $name: $*" >&2
  { echo "# cmd: $*"; "$@"; echo "# exit: $?"; } >"$OUT/evidence/$name.txt" 2>&1 || true
}

# ---- 1. git state ----------------------------------------------------------
run git_status      git status --short --branch
run git_log         git log --oneline --decorate -n 300
run git_tags        git tag -n1 --sort=creatordate
run git_shortstat   git log --shortstat --pretty=format:'%h %ad %s' --date=short -n 80
run git_deleted     bash -c "git log --diff-filter=D --name-only --pretty=format:'--- %h %s' -n 200"
run git_submodules  git submodule status

# ---- 2. source archive (no build output, deps, corpus, vendor) -------------------
git archive --format=tar.gz -o "$OUT/source.tar.gz" HEAD -- . \
  ':(exclude)corpus' ':(exclude)vendor' ':(exclude)target' ':(exclude)node_modules' \
  ':(exclude)*.gguf' ':(exclude)*.safetensors' ':(exclude)*.rpm' 2>"$OUT/evidence/git_archive_err.txt" || true
git rev-parse HEAD > "$OUT/meta/HEAD_SHA.txt"
[ -d vendor/pdfcraft ] && git -C vendor/pdfcraft rev-parse HEAD > "$OUT/meta/pdfcraft_pin.txt" 2>/dev/null || true

# ---- 3. structure and size -------------------------------------------------
run file_tree  bash -c "git ls-files | grep -v -E '^(corpus|vendor)/' | head -n 5000"
if command -v tokei >/dev/null 2>&1; then run loc tokei --sort lines .
else run loc bash -c "for d in crates/*/ app-tauri/ xtask/; do printf '%s ' \"\$d\"; find \"\$d\" -name '*.rs' -o -name '*.ts' -o -name '*.svelte' | grep -v node_modules | xargs cat 2>/dev/null | wc -l; done"; fi

# ---- 4. static scans (leads for the auditor; noisy in #[cfg(test)] blocks) ---------
EXCL=(--exclude-dir=target --exclude-dir=node_modules --exclude-dir=vendor --exclude-dir=corpus --exclude-dir=.git)
scan() { local name="$1" pat="$2"; shift 2; run "scan_$name" grep -rnE "${EXCL[@]}" "$@" -e "$pat" .; }
scan unwrap_expect_todo  '\.unwrap\(\)|\.expect\(|todo!\(|unimplemented!\(|panic!\(' --include='*.rs'
scan unsafe_code         '\bunsafe\b|forbid\(unsafe_code\)' --include='*.rs'
scan ignored_tests       '#\[ignore' --include='*.rs'
scan weak_assertions     'assert!\(true\)|assert_eq!\(1, ?1\)|assert!\(.*is_ok\(\)\);?$' --include='*.rs'
scan stub_words          'not implemented|unimplemented|stub|placeholder|hardcoded|hard-coded|fake|dummy|for now|temporary|FIXME|HACK|XXX' --include='*.rs' --include='*.ts' --include='*.svelte'
scan ui_network_html     'fetch\(|XMLHttpRequest|WebSocket|EventSource|\{@html|eval\(|new Function\(' --include='*.ts' --include='*.svelte' --include='*.js'
scan ui_external_urls    'https?://' --include='*.ts' --include='*.svelte' --include='*.css' --include='*.html'
scan process_spawn       'Command::new|std::process|tokio::process' --include='*.rs'
scan password_logging    'password|passphrase|secret' --include='*.rs'
scan network_crates_lock '^name = "(reqwest|hyper|ureq|isahc|surf|curl|attohttpc|awc|tungstenite|tokio-tungstenite|openssl-sys|native-tls)"' --include='Cargo.lock'
scan base64_usage        'base64' --include='*.rs' --include='*.ts'
scan sidecar_refs        'pandoc|soffice|libreoffice|tesseract|scanimage' --include='*.rs' --include='*.json' --include='*.toml' --include='*.md'

# ---- 5. tests inventory ------------------------------------------------------
run rust_test_list  SB cargo nextest list --workspace
run rust_test_attr_count bash -c "grep -rE '#\[(test|tokio::test|proptest)' --include='*.rs' ${EXCL[*]} . | wc -l"
run ui_test_files   bash -c "find app-tauri -path '*/node_modules' -prune -o \( -name '*.test.ts' -o -name '*.spec.ts' \) -print"
run ui_test_count   bash -c "grep -rcE '\b(it|test)\(' --include='*.test.ts' --include='*.spec.ts' app-tauri --exclude-dir=node_modules | awk -F: '{s+=\$2} END {print s}'"
run fuzz_targets    bash -c "ls -R fuzz 2>/dev/null | head -n 200"
run e2e_files       bash -c "find . -path ./node_modules -prune -o \( -iname '*e2e*' -o -iname '*playwright*' -o -iname '*tauri-driver*' \) -print | grep -v -E 'node_modules|target' | head -n 100"

# ---- 6. project gates and supply chain --------------------------------------
run protocol_lint     SB cargo xtask protocol-lint
run feature_map_check SB cargo xtask feature-map-check
run cargo_deny        SB cargo deny check
run cargo_tree_depth1 SB cargo tree --workspace -e normal --depth 1
run cargo_tree_dupes  SB cargo tree --workspace -d
run cargo_audit       SB cargo audit
run pnpm_ls           SB pnpm --dir app-tauri ls --depth 0
run pnpm_audit        SB pnpm --dir app-tauri audit --prod

# ---- 7. docs, config, logs ---------------------------------------------------
for p in AGENTS.md MASTER_PROMPT.md protocol logs docs Cargo.toml deny.toml rust-toolchain.toml \
         app-tauri/src-tauri/tauri.conf.json app-tauri/src-tauri/capabilities app-tauri/package.json \
         packaging .github; do
  [ -e "$p" ] && mkdir -p "$OUT/copy/$(dirname "$p")" && cp -r "$p" "$OUT/copy/$(dirname "$p")/"
done

# ---- 8. pack -----------------------------------------------------------------
tar -czf "$OUT.tar.gz" "$OUT"
ls -lh "$OUT.tar.gz" "$OUT/source.tar.gz"
echo
echo "Done. Attach $OUT.tar.gz to the reviewer. If it exceeds ~25 MB, attach $OUT/evidence + $OUT/copy first,"
echo "then source per crate:  git archive --format=tar.gz -o <crate>.tar.gz HEAD crates/<crate>"
