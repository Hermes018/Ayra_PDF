# cadcraft-render — LocalPDF Studio patch (ADR-018 rev 3, checkpoint 2, ERR-007)

Upstream: https://github.com/storytold/cadcraft @ `df6597445803c3db1274412f403e2aba9d76626a`, crate `crates/render`
(v0.17.0, MIT OR Apache-2.0; licence texts and NOTICE in `vendor/craft/cadcraft/` and THIRD_PARTY_NOTICES.md).
Used by `crates/lpdf-geom` (`HatchEngine::Cadcraft`) through the root `[patch."https://github.com/storytold/cadcraft"]`.

## Why
`pattern_lines(&Drawing, &Hatch, loops, px, view)` (`src/hatch.rs:370`) sweeps hatch pattern families across real
boundary loops (even-odd, dashes, work budgets). It is `pub` but lives in the private `mod hatch` (`src/lib.rs:13`),
and only `fill_loops` and `pattern_swatch` are re-exported (`src/lib.rs:35`), so no other crate can call it.

## What changed (and nothing else)
1. `src/lib.rs:35`: `pub use hatch::{fill_loops as hatch_fill_loops, pattern_swatch};`
   → `pub use hatch::{fill_loops as hatch_fill_loops, pattern_lines, pattern_swatch};`
   Every other file under `src/` is byte-identical to upstream (`diff -r` against the pinned rev).
2. `Cargo.toml`: the `cargo vendor`-normalised manifest (workspace-inherited `version`, `edition`, `license`
   written out), with the sibling dependencies `cadcraft-color`, `cadcraft-doc`, `cadcraft-fonts`, `cadcraft-geom`
   changed from `path = "../<name>"` to `version = "0.17.0", git = "https://github.com/storytold/cadcraft",
   rev = "df65974…"` (resolved offline from `vendor/craft/cadcraft` by source replacement), and the `[[test]]`
   targets and the `serde_json` dev-dependency removed.
3. Not copied: `tests/`, `.cargo-checksum.json`.

## Re-sync / removal
- On a cadcraft rev bump (docs/UPGRADING.md): re-vendor, copy the new `crates/render` from the vendor output,
  re-apply 1-3, then `diff -r <upstream>/crates/render/src src` must show only the line in 1.
- Drop this patch (and the root `[patch]`) when upstream re-exports `pattern_lines`; the upstream request is to be
  drafted under `docs/upstream/`.
