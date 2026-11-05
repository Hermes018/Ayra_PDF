# winprint — LocalPDF Studio patch (ADR-017)

Upstream: https://github.com/storytold/winprint.rs @ `5f80b9baf6b58937c401656634e42d33f00ddf49` (BSD-3-Clause, see LICENSE.md).
Pulled in by pdfcraft → `craft-print` (`[target.'cfg(windows)'.dependencies] winprint = { …, default-features = false }`).

## Why
Upstream `build.rs` downloads PDFium binaries over HTTPS (reqwest + sigstore) when the `pdfium` feature is on.
`craft-print` disables that feature, so the download never runs, but Cargo still compiles every build-dependency,
which put an HTTP client (`reqwest`, `hyper`, `rustls-platform-verifier`, `webpki-root-certs`) into our Windows
dependency tree (ERR-006, protocol F9 / DON'T-1).

## What changed (and nothing else)
- `src/`, `LICENSE.md`, `README.md`: byte-identical copies of upstream.
- `Cargo.toml`: removed `[build-dependencies]` (reqwest, flate2, tar, sigstore-*) and `[dev-dependencies]`;
  `default` features is now empty; `pdfium` is still declared so the upstream `cfg(feature = "pdfium")` code
  compiles, but it is unsupported here (no build script to link PDFium).
- `build.rs`: removed (with `pdfium` off it only printed a `rerun-if-env-changed` line).
- Not copied: `examples/`, `tests/`, `test_data/`, `utils/`.

## Re-sync
When pdfcraft bumps its `craft-print` / winprint revision, re-copy `src/` from the new revision, re-apply the
Cargo.toml edits above, and diff `src/` against upstream (`diff -r`) to confirm it is still unmodified.
