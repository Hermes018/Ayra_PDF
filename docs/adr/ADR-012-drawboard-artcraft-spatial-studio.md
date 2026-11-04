# ADR-012 — Drawboard-Class Spatial Studio & ArtCraft Engine Federation

- **Status:** PROPOSED (2026-11-04). Binding once the user approves; implementation (Phases D1–D6) is blocked until then (protocol DON'T #13: scope outside `07_FEATURE_MAP.md` needs PLAN_LOG + ADR + approval).
- **Supersedes:** none. **Pauses:** ADR-011 execution track (Windows W2–W4) at W1 (`ee1d552`).
- **Related:** ADR-002 (memory tiers), ADR-003 (Svelte 5 runes UI), ADR-004 (local LLM / Candle), ADR-005 (native feature expansion), ADR-009 (vendored PDF.js).
- **Companion blueprint:** `docs/DRAWBOARD_STUDIO_MASTER_PLAN.md`.

## Context

LocalPDF Studio today is a tool-grid PDF processor (organize / convert / protect / OCR) powered by the pinned `vendor/pdfcraft` engine (`75360d1`). Users want a Drawboard PDF-class *spatial* workspace: an infinite pen-first canvas, a radial menu, pressure ink, CAD-grade measurement and calibrated takeoff. Drawboard itself only layers annotations over flattened PDFs. The `storytold` organisation publishes sibling "ArtCraft" engines (`pdfcraft`, `cadcraft`, `designcraft`, `figcraft`, `materialcraft`, `texturecraft`, `craft-anything`) whose patterns cover most of this surface.

Hard constraints from the protocol: offline only, no localhost servers, no GPL/AGPL linked into the app, no `unwrap`/`unsafe`/`todo!` in non-test code, atomic new-file output, byte-level destructive redaction, verified APIs only.

## Decision 1 — Open-Source ArtCraft Crate Federation (MIT / Apache-2.0 only)

We federate `pdfcraft` (already vendored) with surgical modules/patterns from:

| Source | What we take | Lands in |
|---|---|---|
| `storytold/cadcraft` | OSNAP vector snapping, pick-point boundary hatching w/ island detection, associative linear/area/angular dimensions, two-point scale calibration, multileader callouts, DXF import | `crates/lpdf-cad-measure` |
| `storytold/designcraft` | Knuth-Plass paragraph composer, threaded text frames, baseline grids, master-page overlay layers, copy-on-write undo snapshots, headless verification pattern | `crates/lpdf-layout-edit`, `app-tauri/src/lib/core/commandEngine.svelte.ts` |
| `storytold/figcraft` | Variable-width vector networks, boolean partial-erase path splitting, connectors, align/distribute/group, 1°/15° rotation convention | `crates/lpdf-vector-ink` |
| `storytold/materialcraft` + `texturecraft` | 2D procedural hatch pattern generators, stylus pressure/tilt stamp curves | `crates/lpdf-vector-ink::pattern`, `toolEngine.svelte.ts` |
| `storytold/craft-anything` | `control-protocol` message shape (`execute`, `pointer`, `ui.key`, `render_page`) and JSON replay verification | `crates/lpdf-control-bus` |

**Binding licence rules**

1. **Audit before import (hard gate, D1 pre-phase).** Each source repo is vendored as a git submodule at a pinned commit under `vendor/<name>`; its `LICENSE*` files and every transitive dependency are checked with `cargo deny check licenses`. Only `MIT`, `Apache-2.0`, `MIT OR Apache-2.0`, `BSD-2/3-Clause`, `ISC`, `Zlib`, `Unicode-3.0` are accepted. Note: the user asked for MIT/Apache-2.0 only. Any other permissive licence still needs the user to approve it for each crate, and the approval gets recorded in `ENGINE_NOTES.md`. If a module turns out to be GPL/AGPL/MPL/unlicensed, we don't take it. Instead we write our own version from the published algorithm (not from that code).
2. **Attribution is kept, branding is removed.** MIT and Apache-2.0 *require* keeping copyright and licence notices (and Apache `NOTICE` files). We keep them in `THIRD_PARTY_LICENSES.md` and in the source headers. We remove ArtCraft names, logos, icons, and marketing copy from all UI, assets, and crate names that users see. Apache-2.0 §6 does not grant any trademark rights.
3. **Prefer copying patterns over taking dependencies** (DO #11). Small algorithms (Knuth-Plass, RDP, shoelace, OSNAP ranking) are re-implemented in our crates. We keep the attribution comment where we follow the upstream structure.
4. **Excluded:** DWG reading. We know of no permissive Rust DWG reader, and LibreDWG is GPL-3. DXF is native through our own parser or the `dxf` crate (MIT, vetted in D5). We revisit DWG only as an isolated sidecar under a separate ADR. Persistent-collection crates `im`/`imbl`/`rpds` are **MPL-2.0**, so they are rejected. Copy-on-write uses `std::sync::Arc` plus our own structural sharing.

**Proposed new third-party deps (all vetted per file 08 in the phase that adds them):** `rstar` (MIT/Apache, R-tree), `kurbo` (MIT/Apache, Bézier math), `i_overlay` (MIT, polygon boolean ops), `rust_xlsxwriter` (MIT/Apache, XLSX takeoff export), `ts-rs` (MIT, type export), `notify` (licence varies by version: pin a version whose licence is `MIT OR Apache-2.0` or drop it), MathJax 3 `es5/tex-svg-full` vendored offline (Apache-2.0).

## Decision 2 — Universal Command Bus & Anti-Hardcoding Law

1. **Every user-visible state change goes through one typed message bus.** It uses four message kinds: `execute { command, args }`, `pointer { phase, x, y, pressure, tiltX, tiltY, twist, pointerType, buttons, viewId }`, `ui.key { chord, phase }`, and `render_page { docId, page, scale, viewId }`. The TS side is `commandEngine.svelte.ts`. The Rust side is `crates/lpdf-control-bus`, and both share one schema through `ts-rs`.
2. **Four centralized Svelte 5 Rune engines are the only owners of their domains:**
   - `GlobalThemeContract` (`core/themeContract.svelte.ts`): all colours, glass, canvas, and snap-indicator tokens.
   - `ToolStateEngine` (`core/toolEngine.svelte.ts`): active tool, presets, pressure curves, stroke/fill/hatch/font state.
   - `ViewportCoordinateEngine` (`core/viewportEngine.svelte.ts`): all screen ↔ PDF pt ↔ calibrated unit transforms and split/multi-window sync.
   - `CommandBusAndShortcutEngine` (`core/commandEngine.svelte.ts`): transactions, undo/redo, shortcut registry, and the headless control channel.
3. **Anti-hardcoding law (enforced by a new `cargo xtask protocol-lint` rule in D1).** Under `app-tauri/src/lib/{views,components,tools}`, these are lint failures: hex/rgb/hsl colour literals in `.svelte`/`.ts`/`.css`; inline `scale(`/`matrix(`/`translate(` transforms built outside `viewportEngine`; any `undoStack`/`history` array outside `commandEngine`; and a `keydown` listener outside `commandEngine`. Colour literals may appear only in `themeContract.svelte.ts`.
4. **No network or server.** The "MCP/headless" channel is a stdio JSON-lines replay harness (`cargo xtask control-check <script.jsonl>`) plus in-process vitest drivers. No socket and no localhost listener (DON'T #2). It is compiled only under the `control-headless` cargo feature, which is off in release bundles.

## Decision 3 — Dual-Mode PDF Architecture

- **Mode A: Markup.** Annotations live in a non-destructive overlay model and render at 60–120 fps (vector ink on a `<canvas>` per tile, with a dirty-rect repaint). They are written as standard PDF annotations (`/Ink`, `/Line`, `/PolyLine`, `/Polygon`, `/Square`, `/Circle`, `/FreeText`, `/Highlight`, `/Sound`, `/Stamp`, `/Link`) through `pdfcraft`. Measurements carry `/Measure` (RL / `/NumberFormat`) dictionaries per ISO 32000-1 §12.9.
- **Mode B: Structural editing.** Inline text editing reflows through the Knuth-Plass composer and is written back as a replaced content stream with font subsetting. The other operations are object move/resize/rotate/flip (matrix rewrite on the `cm`/XObject), destructive redaction (glyph/image byte removal plus the F25 verify pass), and page split/merge/reorder/crop/rotate. All of these run as worker jobs (DO #8) with atomic new-file output (DO #5, F1, F6).
- The user always picks the mode explicitly per session. Saving Mode B output never overwrites the original unless the user chooses "overwrite", in which case a backup is made first (F8).

## Consequences

- **Positive:** One deterministic, testable interaction substrate. Every gesture can be replayed headless. Theme, units, and undo are consistent across all tools. CAD-grade takeoff works fully offline.
- **Negative / cost:** We take on four new crates and a large UI surface (Phases D1–D6). The licence audit can remove upstream modules and force us to write our own. The Windows track stays paused until the user resumes it.
- **Risks:** Upstream APIs are unverified until D1 vendoring (DO #2). Structural text editing of PDFs with Type3 or CID fonts that lack a `ToUnicode` map degrades to "replace-as-new-text-box" with a visible notice (no silent fallback, DON'T #5).
