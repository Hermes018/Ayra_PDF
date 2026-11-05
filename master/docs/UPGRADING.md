# UPGRADING — bumping a pinned upstream engine

> **Status:** DRAFT, proposed with ADR-018 rev 2. It takes effect when ADR-018 rev 2 is approved. Until then, pdfcraft bumps follow DON'T-15 and the winprint patch follows ADR-017.

This applies to every entry in `master/upstream/UPSTREAM.toml`: `dependency` (git rev or version pin), `patch` (local copy with a documented diff) and `port` (hand-ported code with its upstream origin recorded).

## 1. Decide to bump
1. Run `cargo xtask upstream-check`. It lists, per entry, the commits upstream since our pinned rev, and the commits that touch the paths we rely on.
2. Read upstream's changelog / release notes and the commits that touch our paths. Note breaking changes, new dependencies, new features (look for network ones), licence changes and security fixes.
3. One bump = one engine (one manifest entry, or a group that must move together, e.g. `cadcraft-doc` + `cadcraft-render`).

## 2. Branch
- Run `git worktree add -b upgrade/<engine>-<short-new-rev> ../Ayra_PDF-wt/upgrade-<engine> main`, or reuse `track/<platform>` if the bump is part of a track.
- Never bump on `main` directly. Never bump two unrelated engines in one branch.

## 3. Change
1. Update the pin in the relevant `Cargo.toml` (`rev = "…"` / `version = "=…"`), keeping `default-features = false` and the explicit feature list. Update `rev` in `UPSTREAM.toml`.
2. **For `kind = "patch"`:**
   - Re-copy the upstream files at the new rev.
   - Re-apply only the changes documented in the patch's `PATCH_NOTES.md`.
   - Run `cargo xtask upstream-check --strict`, which must show no undocumented diff.
   - If upstream has merged our request, delete the patch, switch the entry to `dependency`, and remove the `[patch]` section.
3. **For `kind = "port"`:** re-port the upstream changes to the listed local file. Better: check whether the code can now be consumed as a dependency or via WASM (ADR-018 §4).
4. **Fix compile errors in the adapter crate only** (`crates/lpdf-adapter-*`). If code outside the adapter needs changing, the adapter's API is leaking; fix the adapter instead.
5. `cargo update -p <crate>` only for the crates being bumped. Review the `Cargo.lock` diff for new transitive crates.

## 4. Gate (all through `master/tools/build-lock.sh`, one at a time)
1. `cargo xtask gate`: fmt, clippy, nextest, deny (including the network-crate bans), audit, protocol-lint (including the adapter boundary), Android core check.
2. Adapter golden tests. Any change in output is reviewed and the goldens updated deliberately, with the reason noted in the log.
3. Frontend: `pnpm --dir app-tauri check`, then `pnpm --dir app-tauri exec vitest run`, then `cargo xtask studio-audit --strict`.
4. For pdfcraft and other PDF engines: corpus diff (DON'T-15): `bash corpus/fetch.sh`, then the engine tests on the corpus. Compare page counts and `qpdf --check` results with the previous pin.
5. Platform checks for every platform the engine's features declare in the checklist (`[platforms: …]`).

## 5. Verify
- Spawn a **fresh verifier agent**. It gets only ADR-018, this file, the branch diff (`git diff main...upgrade/<engine>-<rev>`) and the gate commands.
- The verifier runs the gates itself and reports failures and any undocumented behaviour change.
- No merge until the verifier passes and the gate is green.

## 6. Log and merge
1. `PROGRESS_LOG.md`: old → new rev, `upstream-check` summary, the gate output tails, the verifier's verdict.
2. `DECISIONS_ADR.md`: one line per bump (`ADR-018 bump: <engine> <old> → <new>`). A full ADR is needed if the bump changes behaviour, licences or the dependency footprint materially.
3. Update `CHECKPOINT.md` if the bump was the current task.
4. Commit on the branch, fast-forward `main`, tag `upgrade-<engine>-<short-rev>`, and remove the worktree.
5. Never push unless the project owner asks.

## 7. Roll back
`git revert` the bump commit on `main` (protocol F15). Manifest, `Cargo.toml` and `Cargo.lock` move back together because they were changed in one commit.
