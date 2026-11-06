//! cadcraft-backed hatching (ADR-018 R2 step 3; fixes ERR-007).
//!
//! Pattern data comes from `cadcraft-doc` (`library::pattern`, cadcraft@df65974
//! `crates/doc/src/library.rs:240`), and the lines are swept across the real boundary loops with
//! even-odd clipping and dashes by `cadcraft-render::pattern_lines` (`crates/render/src/hatch.rs:370`),
//! which our local patch re-exports (`vendor/patches/cadcraft-render/PATCH_NOTES.md`). No cadcraft
//! type leaves this module.

use crate::{GeomError, HatchPatternId, Loop, Segment};
use cadcraft_doc::library::{self, PatternLine};
use cadcraft_doc::{Drawing, Hatch};
use cadcraft_geom::Vec2;

/// Pattern units to PDF points. The library patterns repeat every 0.125 unit (ANSI31 & co.), so
/// 64 gives the legacy default spacing of 8 pt at scale 1.
pub const PATTERN_UNIT_PT: f64 = 64.0;

/// Most runs one call may produce. Checked from an estimate BEFORE sweeping, so cadcraft's own
/// silent caps (1,000,000 runs, 50M work units) are never reached and nothing is truncated
/// without telling the caller (DON'T-7).
pub const MAX_CADCRAFT_RUNS: usize = 100_000;

/// Where a pattern's line families come from.
enum Families {
    /// A `cadcraft-doc` standard library pattern, by name.
    Library(&'static str),
    /// A family set defined here because cadcraft has no close match (documented per pattern).
    Custom(Vec<PatternLine>),
}

fn pl(angle: f64, origin: (f64, f64), delta: (f64, f64), dashes: &[f64]) -> PatternLine {
    PatternLine {
        angle,
        origin,
        delta,
        dashes: dashes.to_vec(),
    }
}

/// Our pattern ids → cadcraft patterns. Every id maps to a different family set.
fn families(pattern: HatchPatternId) -> Families {
    match pattern {
        HatchPatternId::DiagonalSteel => Families::Library("ANSI31"),
        HatchPatternId::Steel => Families::Library("ANSI32"),
        HatchPatternId::Crosshatch => Families::Library("ANSI37"),
        HatchPatternId::Tiling => Families::Library("NET"),
        HatchPatternId::Masonry => Families::Library("BRICK"),
        HatchPatternId::Earth => Families::Library("EARTH"),
        HatchPatternId::Concrete => Families::Library("AR-CONC"),
        HatchPatternId::Insulation => Families::Library("ZIGZAG"),
        // Timber: no wood-grain pattern in cadcraft's library. Staggered long dashes (grain) on two
        // offset horizontal families.
        HatchPatternId::Timber => Families::Custom(vec![
            pl(0.0, (0.0, 0.0), (0.0625, 0.125), &[0.25, -0.0625]),
            pl(0.0, (0.0, 0.0625), (0.1875, 0.125), &[0.125, -0.125]),
        ]),
    }
}

/// Per-pattern correction of [`PATTERN_UNIT_PT`]: `AR-CONC` is defined 16x coarser than the ANSI
/// patterns upstream (`scaled_pattern(.., 16.0)`, cadcraft@df65974 `crates/doc/src/library.rs`), so
/// it gets 1/16 to show its aggregate at the same plan density as the others.
fn unit_factor(pattern: HatchPatternId) -> f64 {
    match pattern {
        HatchPatternId::Concrete => 1.0 / 16.0,
        _ => 1.0,
    }
}

fn line_families(pattern: HatchPatternId) -> Vec<PatternLine> {
    match families(pattern) {
        Families::Library(name) => library::pattern(name).map(|p| p.lines).unwrap_or_default(),
        Families::Custom(lines) => lines,
    }
}

/// Upper estimate of the runs a sweep produces: per family, lines across the bounding-box
/// diagonal times dash repeats along it.
fn estimated_runs(lines: &[PatternLine], scale: f64, diag: f64) -> f64 {
    lines
        .iter()
        .map(|l| {
            let spacing = (l.delta.1 * scale).abs();
            if !spacing.is_finite() || spacing <= 1e-9 {
                return 0.0;
            }
            let rows = diag / spacing + 1.0;
            let repeat: f64 = l.dashes.iter().map(|d| d.abs()).sum::<f64>() * scale;
            let per_row = if l.dashes.is_empty() || !repeat.is_finite() || repeat <= 1e-9 {
                1.0
            } else {
                (diag / repeat + 1.0) * l.dashes.len() as f64
            };
            rows * per_row
        })
        .sum()
}

pub(crate) fn hatch_segments(
    pattern: HatchPatternId,
    loops: &[Loop],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    let rings: Vec<Vec<Vec2>> = loops
        .iter()
        .map(|l| {
            l.points
                .iter()
                .filter(|p| p.x.is_finite() && p.y.is_finite())
                .map(|p| Vec2::new(p.x, p.y))
                .collect::<Vec<_>>()
        })
        .filter(|r| r.len() >= 3)
        .collect();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for p in rings.iter().flatten() {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    if rings.is_empty() || !min_x.is_finite() || !min_y.is_finite() {
        return Err(GeomError::NoFiniteGeometry);
    }
    // Same user-facing scale range as the legacy generator (0.25..4 of the default spacing).
    let user_scale = if scale.is_finite() {
        scale.clamp(0.25, 4.0)
    } else {
        1.0
    };
    let hatch_scale = user_scale * PATTERN_UNIT_PT * unit_factor(pattern);
    let lines = line_families(pattern);
    let diag = (max_x - min_x).hypot(max_y - min_y);
    let estimate = estimated_runs(&lines, hatch_scale, diag);
    if !estimate.is_finite() || estimate > MAX_CADCRAFT_RUNS as f64 {
        return Err(GeomError::WorkLimitExceeded {
            limit: MAX_CADCRAFT_RUNS,
        });
    }

    let hatch = Hatch {
        pattern: String::new(),
        solid: false,
        loops: Vec::new(),
        scale: hatch_scale,
        angle: 0.0,
        associative: false,
        style: 0, // odd parity (even-odd): holes and concave notches stay empty
        elevation: 0.0,
        gradient: None,
        origin: Vec2::ZERO,
        background: None,
        pattern_lines: lines,
    };
    // px = 0: full detail (no level-of-detail merging), view = None: the whole region.
    let runs = cadcraft_render::pattern_lines(&Drawing::default(), &hatch, &rings, 0.0, None);
    if runs.len() >= MAX_CADCRAFT_RUNS {
        return Err(GeomError::WorkLimitExceeded {
            limit: MAX_CADCRAFT_RUNS,
        });
    }
    Ok(runs_to_segments(runs))
}

/// Square preview of a pattern for the on-screen `<pattern>` tiles (`HatchDefs.svelte`), from
/// `cadcraft-render::pattern_swatch` (`crates/render/src/hatch.rs:423`): the pattern's runs inside
/// the `side` x `side` square at the origin, at a scale showing about six repeats. Same families
/// as [`hatch_segments`], so screen and PDF come from one source.
pub(crate) fn hatch_swatch(pattern: HatchPatternId, side: f64) -> Result<Vec<Segment>, GeomError> {
    if !side.is_finite() || side <= 0.0 || side > 10_000.0 {
        return Err(GeomError::NoFiniteGeometry);
    }
    let runs = cadcraft_render::pattern_swatch(&line_families(pattern), side);
    Ok(runs_to_segments(runs))
}

fn runs_to_segments(runs: Vec<Vec<Vec2>>) -> Vec<Segment> {
    let mut out = Vec::with_capacity(runs.len());
    for run in runs {
        match run.as_slice() {
            [] => {}
            // A single-point run is a dot: a zero-length segment (drawn with a round cap).
            [p] => out.push(Segment {
                a: crate::PdfPt::new(p.x, p.y),
                b: crate::PdfPt::new(p.x, p.y),
            }),
            pts => {
                for w in pts.windows(2) {
                    if let [a, b] = w {
                        out.push(Segment {
                            a: crate::PdfPt::new(a.x, a.y),
                            b: crate::PdfPt::new(b.x, b.y),
                        });
                    }
                }
            }
        }
    }
    out
}
