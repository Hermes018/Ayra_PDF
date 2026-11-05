# DRAFT upstream issue (not posted) — storytold/winprint.rs

> Drafted 2026-11-05 for LocalPDF Studio (ADR-017). Post it to https://github.com/storytold/winprint.rs/issues only after the project owner approves; we must not post anything on their behalf.

**Title:** Make the PDFium download build-dependencies optional (`reqwest`, `sigstore-*` are compiled even with `default-features = false`)

**Body:**

Hi, and thanks for maintaining this fork.

`craft-print` (storytold/craft-libs) depends on winprint with `default-features = false`, which turns off the `pdfium` feature. With the feature off, `build.rs` never downloads anything (`try_link_pdfium` is behind `#[cfg(feature = "pdfium")]`).

Cargo still compiles every `[build-dependencies]` entry, though, because build-dependencies cannot be gated by a feature unless they are declared `optional`. So every Windows build of an app that uses craft-print compiles an HTTP and TLS stack it never runs:

```toml
[build-dependencies]
reqwest = { version = "0.13", features = ["blocking"] }
flate2 = "1.1"
tar = "0.4"
sigstore-trust-root = { version = "0.6.5", default-features = false }
sigstore-types = "0.6.5"
sigstore-verify = { version = "0.6.5", default-features = false }
```

(rev `5f80b9baf6b58937c401656634e42d33f00ddf49`, `Cargo.toml` lines 56–62; `build.rs` `main` at line 156.)

For offline or air-gapped apps this matters. It adds `reqwest`, `hyper`, `rustls-platform-verifier` and `webpki-root-certs` (CDLA-Permissive-2.0) to the dependency graph, which fails `cargo deny` network bans and licence allow-lists. It also adds build time.

**Proposed change (backwards compatible):**

```toml
[build-dependencies]
reqwest = { version = "0.13", features = ["blocking"], optional = true }
flate2 = { version = "1.1", optional = true }
tar = { version = "0.4", optional = true }
sigstore-trust-root = { version = "0.6.5", default-features = false, optional = true }
sigstore-types = { version = "0.6.5", optional = true }
sigstore-verify = { version = "0.6.5", default-features = false, optional = true }

[features]
default = ["pdfium"]
pdfium = ["dep:reqwest", "dep:flate2", "dep:tar", "dep:sigstore-trust-root", "dep:sigstore-types", "dep:sigstore-verify"]
```

Cargo resolves optional build-dependencies through features exactly like normal ones. The existing `#[cfg(feature = "pdfium")]` in `build.rs` already guards every use. `default` keeps today's behaviour, and `default-features = false` users stop compiling the download stack.

Happy to open a PR if this direction works for you.
