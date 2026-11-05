# ADR-018 (rev 3) — Reuse storytold engines as pinned dependencies behind adapters

- **Status:** **APPROVED** (rev 3, 2026-11-05 22:30 Asia/Dhaka, PROMPT_LOG #17).
- Rev 3 = the user's binding rules below, plus the rev 2 analysis (§1–§9) as supporting evidence. **Where they differ, rev 3 wins.**
- **Goal:** future upstream craft releases are pulled in by changing a pinned revision, not by rewriting code. Reuse with intent, not copy-paste.

## Rev 3 binding rules

### R1. Decide per component, with a written reason
Every engine function we hand-wrote gets a row in `docs/ENGINE_REUSE_MATRIX.md`: our module → candidate craft crate/module → decision → reason → risk. The decision is exactly one of:

| Decision | Meaning |
|---|---|
| **ADOPT** | Pinned craft crate behind our adapter. The default for Rust-side, per-edit work: geometry, hatching, path booleans, measurement math, text reflow, PDF ops. |
| **ADOPT-WASM** | Craft crate compiled to `wasm32` (wasm-bindgen) and called in the WebView. Only for pointer-speed paths (snapping, ink smoothing), and only if a benchmark on this machine shows **< 2 ms per call at 120 Hz**. Tauri IPC is never used per pointer event. |
| **KEEP-PORT** | Our TS code stays, with provenance (upstream repo + commit + file) in the manifest. Only when ADOPT/ADOPT-WASM is measured or shown to be worse. |
| **REJECT** | Irrelevant, network dependencies, or a different document model with no clean seam. One line saying why. |

**Starting candidates:**
- **pdfcraft:** prefer pdfcraft's own Organize/Forms/Security/OCR/Redaction over ours.
- **cadcraft:** OSNAP and spatial index, geometry/intersection, units and scale calibration, dimension/measure math, hatch generation (ERR-007).
- **vectorcraft:** pathops boolean erase, curve fitting, stroke outline.
- **designcraft:** compose text reflow.
- **materialcraft / texturecraft:** only for pattern definitions.
- **photocraft:** only if it cleanly replaces our signature/image-stamp code.
- **figcraft:** engine REJECT (ureq).
- **lightcraft / craft-anything:** REJECT unless there's a clear need.
- **Shared crates and conventions** the craft apps use (craft-libs, "craftrules"): adopt where they match ours.

### R2. Integration: strangler pattern, one engine at a time
1. **Characterization tests first:** golden fixtures of the current outputs on the corpus (snaps, measurements, hatch paths, eraser output, reflowed text).
2. **Adapter crate/trait** (e.g. `lpdf-geom`, `lpdf-ink`, `lpdf-text`) with the CURRENT implementation inside. App code calls only the adapter. Merge when green.
3. **Craft-backed implementation** behind the same adapter, selected by a build/runtime switch.
4. **Parity run** against the goldens. Every difference is explained; where upstream is better, the fixture is updated with a note. ERR-007 is expected to differ: verify the hatch is clipped to the polygon and every pattern is distinct.
5. **Default switches to craft** only when parity passes and the gate is green. The old implementation stays one release as a fallback, then is removed.

**Order:**
1. cadcraft hatch (ERR-007)
2. cadcraft geometry/measure
3. vectorcraft pathops eraser
4. cadcraft snapping (ADOPT-WASM benchmark)
5. designcraft text reflow
6. the rest

### R3. Dependency hygiene
- **Pins:** every craft dependency is pinned to an exact git `rev` (or version), with `default-features = false` and only the needed features. `cargo deny` must pass: no HTTP/TLS clients; licences MIT/Apache only. Tauri's Android dev-only `reqwest` exception is handled per §5 and ADR-019.
- **Vendoring:** craft dependencies are **vendored** into the repo (`cargo vendor`, or an offline registry mirror) so builds stay offline and reproducible.
- **Manifest:** **`docs/UPSTREAM_MANIFEST.toml`** (supersedes rev 2's `master/upstream/UPSTREAM.toml`). It covers every adopted crate, KEEP-PORT file and patch (including the ADR-017 winprint patch and craft-winprint-gdi): repo, rev, licence, our adapter, last-checked date.
- **`cargo xtask upstream-check`:** **offline** (supersedes rev 2's network cache clone). It compares pinned revs against the `~/craft-ref` clones, after a manual `git fetch` there, and lists changed files that touch what we use. It also checks a patch's drift against the rev its requesting crate asks for (rev 2 §5).
- **`docs/UPGRADING.md`:** one branch per engine bump → bump rev → re-vendor → characterization + parity tests → full gate → verifier → log → merge.
- **`THIRD_PARTY_NOTICES.md`:** every licence text, bundled into the packages.

### R4. Hard timebox (release v0.2.0-prototype, Sat 2026-11-07 23:59 Asia/Dhaka)
- Adapters for all in-scope engines are merged by **Fri 18:00**.
- A craft swap that hasn't passed parity by then stays on the current implementation behind its adapter. It ships that way and swaps after the release. **Never ship a half-swapped engine.**
- **Exception: ERR-007 must ship fixed**, via cadcraft, or via a direct port recorded as KEEP-PORT if adoption is blocked.

---

# Supporting analysis (rev 2, retained as evidence)

## 1. Decision (rev 2 proposal — superseded where rev 3 differs)

1. **Engine logic is consumed as pinned dependencies, not hand-ported, wherever feasible.**
   - Each dependency is pinned to an exact git `rev` or `=version`.
   - Network-pulling features stay disabled (`default-features = false` plus an explicit feature list).
   - Each one passes `cargo deny` / `cargo audit` like any other dependency.
2. **Every engine sits behind our own adapter crate.**
   - Adapters are named `crates/lpdf-adapter-<engine>/`.
   - Only an adapter may name upstream engine types. The rest of our code sees our own types and trait.
   - Upstream API changes touch only the adapter. `protocol-lint` enforces the boundary (§5).
3. **Pointer-speed frontend code: a WASM spike decides between WASM and TS.**
   - The engine crates are compiled to WASM through the same adapter crates, and the TS ports are kept only where the spike numbers say WASM doesn't pay (§4).
   - Any TS port that remains is recorded in the upstream manifest with its upstream repo and commit.
4. **One manifest, `master/upstream/UPSTREAM.toml`, records every upstream relationship:**
   - dependency, local patch (ADR-017 style), or port
   - repo, pinned rev, the paths we rely on, the adapter

   A new `cargo xtask upstream-check` reports upstream changes since each pinned rev.
5. **`master/docs/UPGRADING.md`** defines the bump procedure: one branch per engine bump → full gate → verifier agent → log. It is drafted alongside this ADR.
6. **Pin bumps go through the procedure.** DON'T-15 (no pdfcraft pin bump without changelog, gate, corpus diff and an ADR) is generalised to every pinned engine.

## 2. Feasibility per engine (evidence)

Dependency counts are for normal + build dependencies.
- "Network" = reqwest, ureq, hyper, isahc, surf, curl, tungstenite, rustls, native-tls, openssl, quinn or h2.
- "GUI/GPU" = egui, eframe, winit, wgpu, plus wasmi, tokio and rfd.

Each crate was resolved for `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`, `aarch64-linux-android` and `wasm32-unknown-unknown`.

| Engine need | Upstream crate (repo @ surveyed HEAD) | Deps (linux/win/android/wasm) | Network / GUI | Public API fits? | Verdict |
|---|---|---|---|---|---|
| Hatch pattern data (ANSI31–38, BRICK, EARTH, AR-CONC, …) | `cadcraft-doc` (cadcraft @ `df65974`) | 20/20/20/20 | none | yes: `library::standard_patterns()`, `pattern()`, `PatternLine`, `HatchPattern` | **Dependency** |
| Hatch clipping to polygon loops (ERR-007) | `cadcraft-render` | 106/106/105/98 | none | **no**: `pattern_lines(&Drawing, &Hatch, loops, px, view)` is `pub` inside the private `mod hatch` (`crates/render/src/lib.rs:13`); only `fill_loops` and `pattern_swatch` are re-exported (`:35`). `pattern_swatch` itself calls it with `Drawing::default()` (`hatch.rs:444`), so an adapter can call it the same way once it is re-exported. | **Dependency + small local patch** tracked in the manifest. The source change is one re-export line (`pub use hatch::pattern_lines;`), but the local copy's `Cargo.toml` must also be rewritten, because upstream inherits `version`/`edition`/deps with `workspace = true` from the cadcraft workspace. PATCH_NOTES must list both. Upstream issue drafted. Patch dropped when upstream merges. Cost: ~85 extra crates (tiny-skia, lyon, cadcraft-fonts), all pure Rust, no network. |
| Snap and measurement math (intersections, shoelace, units) | `cadcraft-geom` | 10/10/10/10 | none | yes (geom functions are pub) | **Dependency** (Rust) and **WASM candidate** (pointer speed) |
| CAD object snap (ranking, 14 kinds) | `cadcraft-engine` | heavy (doc, dxf, render, fonts, rstar, libc) | none found in the survey | needs a full cadcraft `Drawing` | **Not as a dependency.** Use `cadcraft-geom` primitives through the adapter. The ranking logic (`snap_score`, 3 lines at `crates/engine/src/snap.rs:549-551`) is a **recorded port**. |
| Shape recognition | `vectorcraft-geom` (vectorcraft @ `f6e3a44`) | 18 everywhere | none | yes: `recognize(&[Point]) -> Option<Recognized>` | **Dependency** and **WASM candidate** |
| Boolean erase, curve fitting, outlines | `vectorcraft-pathops` | 26 everywhere | none | yes: `boolean`/`try_boolean`, `simplify_with`, `outline_stroke`, `try_unite_all` | **Dependency** (Rust, on commit) and **WASM candidate** |
| Variable-width pressure outline | `vectorcraft-effects` (`width_outline`) | pulls wasmi, three-d | wasmi | yes, but the crate is too heavy | **Not as a dependency.** Ask upstream to move `stroke/width.rs` into pathops; until then a **recorded port**. |
| Selection geometry (hit, decompose, constrain) | `designcraft-geom` (designcraft @ `e16785e`) | 34 everywhere | none | yes | **Dependency** and **WASM candidate** |
| Smart guides / distribute | `designcraft-tools` / `-engine` | heavy; the engine reaches reqwest via craft-print (the same winprint chain) | yes, via the engine | tied to their `Document` | **Recorded port** of `snap.rs` passes and `distribute` |
| Knuth-Plass text reflow | `designcraft-compose` | 83/83/83/78 | none | yes: `knuth_plass(...)` on their `Glyph` | **Dependency**, with an adapter mapping pdfcraft glyph metrics to `Glyph` |
| Image resampling (stamps) | `lightcraft-raster` (lightcraft @ `5cd7eb1`) | 20 everywhere | none | yes: `resize`, `fit` | **Dependency** if Rust-side resizing is needed |
| Signature colour-to-alpha + Otsu | `photocraft-algo` (photocraft @ `834e253`) | 40/57/40/30 | none | yes, but it is a large crate (pulls raster, sysinfo, winsafe on Windows) | **Dependency candidate** for Rust; otherwise a **recorded port** |
| Small transform helpers | `figcraft-geom` | 15 everywhere | none | yes | Not needed (kurbo covers it); **out** |

**Not adoptable** (unchanged from the survey):

| Crate | Reason |
|---|---|
| figcraft-engine | `craft-font-store` → `ureq` |
| figcraft-collab-net | tungstenite + rustls |
| designcraft-engine | build-time reqwest |
| lightcraft-segment | SAM licence, candle, HTTPS |
| craft-anything-ai | HTTP AI clients |
| every `ui-egui` / app / web / mcp crate | UI or network |

**Caveats:**
- These trees were resolved, not compiled. Compiling for each target is part of the adoption phase, through the shared build lock.
- All candidates are MIT OR Apache-2.0 (`linesweeper` checked on crates.io). Transitive licences go through `cargo deny` at adoption.

## 3. Adapter design (ADR-020 compatible)

```
crates/lpdf-adapter-cad/        → cadcraft-doc, cadcraft-render (patched), cadcraft-geom
crates/lpdf-adapter-vector/     → vectorcraft-geom, vectorcraft-pathops (+ kurbo)
crates/lpdf-adapter-layout/     → designcraft-geom, designcraft-compose
crates/lpdf-adapter-raster/     → lightcraft-raster (and photocraft-algo if adopted)
crates/lpdf-geom-wasm/          → wasm-bindgen facade over the adapters' pointer-speed functions (only if §4 passes)
```

- Each adapter exposes **our** types: `PdfPt`, `Segment`, `Loop`, `HatchPatternId`, `RecognizedShape`, … It also exposes a narrow trait or a set of free functions returning `Result<_, AdapterError>` (`thiserror`, stable codes).
- No upstream type crosses the adapter boundary.
- Adapter tests pin behaviour (golden outputs), so an upstream bump that changes results shows up in the bump branch's gate, not in production.
- Existing direct engine use is migrated behind adapters when a phase touches it. This covers `app-tauri` using `pdfcraft_engine::Session` directly in `commands/studio.rs`; pdfcraft is otherwise behind `lpdf-engine-port::PdfEngine`.

**ERR-007 under this structure (first Linux phase once this ADR is approved):**
1. `lpdf-adapter-cad::hatch_segments(pattern: HatchPatternId, loops: &[Loop], scale: f64) -> Result<Vec<Segment>, AdapterError>`.
2. It builds a `cadcraft_doc::Hatch` from `library::pattern(...)` and calls the re-exported `cadcraft_render::pattern_lines(&Drawing::default(), …)`.
3. It maps our 8 pattern ids to cadcraft patterns, with documented custom families only for `timber` and `insulation`, if cadcraft has no close match.
4. `studio.rs` passes real polygon loops instead of the bounding box.
5. On screen, `HatchDefs.svelte` gets its pattern data from the adapter. Either the backend sends pattern line data once over IPC at startup, or `pattern_swatch` output is used, so screen and PDF share one source.

## 4. Pointer-speed code: WASM vs TS (spike S1 decides)

**Why WASM is attractive:** the same upstream code runs in the webview at pointer speed with no IPC round-trip, and an upstream fix arrives with a rev bump instead of a re-port.

**What it costs:**
- **CSP:** our policy is `default-src 'self'; connect-src ipc: http://ipc.localhost`. WASM needs two changes:
  - `'wasm-unsafe-eval'` in `script-src`, to compile WebAssembly. That is narrower than `unsafe-eval` and doesn't allow JS `eval`.
  - **No `fetch` and no `connect-src` change.** Loading via `WebAssembly.instantiateStreaming(fetch(...))` would need `connect-src 'self'` (DON'T-1: `connect-src` is IPC-only) and a frontend `fetch(` (DON'T-18). Instead, the `.wasm` bytes come over **binary Tauri IPC** (a raw-bytes command response, no base64; DON'T-6) and are compiled with `WebAssembly.instantiate(bytes)`.

  That leaves `'wasm-unsafe-eval'` as the **only** CSP change. It is a protocol-level security change, so it needs the user's explicit approval and its own DECISIONS line, even if spike S1 passes.
- **Engine support (UNVERIFIED until the spike):** WebKitGTK (Linux), WebView2 (Windows) and Android System WebView must all accept `'wasm-unsafe-eval'`.
- **New dev dependencies:** `wasm-bindgen` (+ `js-sys`), MIT OR Apache-2.0, vetted under protocol 08, plus the `wasm32-unknown-unknown` Rust target. They're used at build time only; the shipped artifact is a static `.wasm` asset served by Tauri's asset protocol. Its MIME type must be verified in the spike.
- **Data marshalling:** point arrays cross as `Float64Array`. That is µs-scale per call, but unmeasured here.

**Spike S1** (one branch, no product UI change). Build `lpdf-geom-wasm` exposing `vectorcraft-geom::recognize` and one `cadcraft-geom` intersection. Load it in the Studio behind a flag and measure:
1. `.wasm` size, gzipped and uncompressed.
2. Cold load time.
3. p50/p99 call latency for a 2,000-point stroke, compared with the current TS.
4. CSP / `'wasm-unsafe-eval'` behaviour in WebKitGTK and, later, the Android WebView.

**Acceptance:** p99 at or below the TS path at 120 Hz; total wasm ≤ 1.5 MB uncompressed for the first module set; no CSP relaxation beyond `'wasm-unsafe-eval'` (and that one only with the user's approval). If it passes, pointer-speed engines move to WASM adapters. If it fails, the TS ports stay, each recorded in the manifest (§5).

## 5. Upstream manifest and `cargo xtask upstream-check`

`master/upstream/UPSTREAM.toml`, one entry per upstream relationship. Initial content once approved:

```toml
[[upstream]]
name = "pdfcraft"
kind = "dependency"            # git dependency + submodule (vendor/pdfcraft)
repo = "https://github.com/storytold/pdfcraft"
rev  = "75360d1d7db06fed974f48c914d4c04132757187"
adapter = "crates/lpdf-engine-pdfcraft"   # behind lpdf-engine-port::PdfEngine

[[upstream]]
name = "winprint"
kind = "patch"                 # ADR-017: src/ byte-identical, Cargo.toml without build-deps
repo = "https://github.com/storytold/winprint.rs"
rev  = "5f80b9baf6b58937c401656634e42d33f00ddf49"
local = "vendor/patches/winprint"
paths = ["src", "Cargo.toml", "build.rs"]
requested_by = "craft-print"   # rev is chosen upstream: craft-libs crates/print/Cargo.toml (via pdfcraft's craft-libs pin)
upstream_request = "docs/upstream/winprint-optional-pdfium-build-deps.md"

[[upstream]]
name = "craft-winprint-gdi"    # same repo, different rev; no network build-deps, used unpatched
kind = "dependency"
repo = "https://github.com/storytold/winprint.rs"
rev  = "417ac3c4d14d3da5485a99c7d5ce8f05664ec312"
requested_by = "craft-print"

# Added by the adoption phases, e.g.:
# [[upstream]] name = "cadcraft-render" kind = "patch" (one-line re-export) …
# [[upstream]] name = "cadcraft-doc"    kind = "dependency" …
# [[upstream]] name = "snap_score"      kind = "port" repo = cadcraft, rev = "df65974…", paths = ["crates/engine/src/snap.rs"], local = "app-tauri/src/lib/core/snapEngine.svelte.ts"
```

**`cargo xtask upstream-check`** (dev tooling only; uses the network at dev time, never at runtime):
1. For each entry, it keeps a blobless cache clone in `~/.cache/lpdf-upstream/<name>` (`git clone --filter=blob:none`, then `git fetch`).
2. It reports `rev..origin/HEAD`: the commit count and the commits touching `paths`.
3. For `kind = "patch"` it also re-diffs our local copy against the pinned upstream rev, to prove only the documented changes exist.
   - Where `requested_by` is set (the rev is chosen by another upstream crate, as with winprint via craft-print), it also compares the manifest rev with the rev that crate's manifest requests in our resolved graph.
   - A `[patch]` with a `path` silently overrides whatever rev upstream asks for, so this comparison is the only way to notice that craft-print moved to a newer winprint.
4. For `kind = "port"` it lists upstream commits to the ported file since `rev`.
5. Output is a table plus an exit code: 0 = informational; `--strict` fails if a patch drifted from its documented diff.
6. It is not part of the per-commit gate, because the gate stays offline. It is run by the bump procedure and on demand.

**Additional gate hardening:**
- `deny.toml` `[bans] deny` gains reqwest, ureq, hyper, isahc, surf, curl, attohttpc, tungstenite, quinn and h2. An upstream bump that pulls in an HTTP client then fails `cargo deny` automatically. Today the ban list is empty, and ERR-006 was caught only by the advisory and licence checks.
  - **Exception required (verified 2026-11-05):** `tauri` 2.12.2 itself depends on `reqwest` 0.13 and `rustls` on Android and iOS (`tauri-2.12.2/Cargo.toml` `[target.'cfg(any(target_os = "android", …))'.dependencies.reqwest]`). Every use is behind `#[cfg(all(dev, mobile))]` in `src/protocol/tauri.rs` (reqwest uses at lines 41, 58, 124, 197; `proxy_dev_request` spans ~196-267; dev-server proxying).
  - `deny.toml` checks all targets (`targets = []`). cargo-deny `wrappers` name a banned crate's **direct** dependents, so the exception is a chain:
    - `{ crate = "reqwest", wrappers = ["tauri"] }`
    - `{ crate = "hyper", wrappers = ["reqwest", "hyper-util"] }`, with any further hyper-family crates that `cargo tree -i` shows on Android at implementation time

    Each entry carries a reason pointing here and at ADR-019. `h2` is not in our graph today, so it stays banned with no exception. Desktop trees stay clean, and the exception is verified per target when implemented.
  - An upstream request to make that dependency dev-only is to be drafted under `docs/upstream/`.
- `protocol-lint` gains an adapter-boundary rule: only `crates/lpdf-adapter-*` (and the existing `lpdf-engine-pdfcraft`, plus `app-tauri` until its migration) may list a storytold crate in `[dependencies]`.

## 6. ADR-017 under this structure
winprint becomes a manifest entry of `kind = "patch"` (above). `upstream-check` reports new winprint commits and re-verifies that `src/` is still byte-identical to the pinned rev. `UPGRADING.md` covers re-syncing it when `craft-print` bumps its winprint rev. The patch is deleted, and the manifest entry changed to `dependency`, when upstream accepts the drafted issue.

## 7. Consequences
- **Upstream fixes:** they arrive as rev bumps through one procedure. Hand-ports shrink to a short, tracked list: `snap_score`, smart-guide passes, `width_outline` until upstream moves it, and TS ports if the WASM spike fails.
- **Build cost:**
  - Bigger graph: `cadcraft-render` alone adds ~85 pure-Rust crates.
  - Longer clean builds. Disk is tight (36 GB free), so this is monitored.
  - The Android/WASM checks add targets to compile in adoption phases.
- **Two local patches to maintain until upstream merges:** winprint and the cadcraft-render re-export.
- **CSP change:** `'wasm-unsafe-eval'` only (WASM bytes come over binary IPC, so there is no `fetch` and no `connect-src` change), if, and only if, spike S1 passes and you approve.

## 8. Proposed phase order (after approval)

| Phase | Work |
|---|---|
| U0 | `UPSTREAM.toml` + `xtask upstream-check` + deny bans + the adapter-boundary lint. Manifest seeded with pdfcraft, winprint and craft-winprint-gdi. |
| L1 (ERR-007) | `lpdf-adapter-cad` with `cadcraft-doc` + patched `cadcraft-render`; `studio.rs` passes real loops; screen and PDF share pattern data. Includes the upstream issue draft for the re-export. |
| S1 | WASM spike (§4), then a decision. |
| L2 | UI wording fix. It doesn't depend on this ADR and can run in parallel with U0. |
| L3 | Ink quality via `lpdf-adapter-vector` (`vectorcraft-geom` + `-pathops`); WASM or TS per S1. |
| Later | Selection (`lpdf-adapter-layout`), measurement, text reflow, raster (survey §6 order). |

## 9. Open questions for the user
1. Accept `cadcraft-render`'s ~85 extra crates for ERR-007, or wait for upstream to split hatching into a lighter crate? (Recommended: accept now, drop the patch later.)
2. May spike S1 use `'wasm-unsafe-eval'` in a branch to measure it? It is the only CSP change; `fetch` and `connect-src` stay untouched via binary IPC loading. No CSP change on `main` without a separate approval.
3. Pin format for craft repos without crates.io releases: exact git `rev` (proposed), or wait for tagged releases?
