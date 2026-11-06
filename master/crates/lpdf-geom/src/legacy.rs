//! The pre-ADR-018 hatch generator, moved verbatim from
//! `app-tauri/src-tauri/src/commands/studio.rs` @ `562e4f9` (`normalize_rect`,
//! `bbox_from_points_or_rect`, `generate_hatch_strokes`). Behaviour is pinned byte-for-byte by
//! `tests/hatch_golden.rs`; do not "fix" it here. ERR-007 is fixed by the cadcraft
//! implementation (ADR-018 checkpoint 2), and this module is removed one release after that.
//!
//! Known defects kept on purpose (characterised, not fixed):
//! - ERR-007: it hatches the bounding box of the loops, not the loops.
//! - `crosshatch`, `tiling` and `masonry` draw the same grid; `timber`, `earth`, `insulation`
//!   and `concrete` draw the same horizontal lines; `steel` equals `diagonal-steel`.
//! - Unbounded work in the original: a huge extent gives an unbounded segment count, and far from
//!   the origin `d + step == d` (or `x + step == x`) never advances, so the original loops forever.
//!   The only change made here is a work guard, [`MAX_LEGACY_SEGMENTS`], that turns both into
//!   [`GeomError::WorkLimitExceeded`] instead of a hang or unbounded memory. Every input that
//!   terminated before produces exactly the same output (golden + parity tests), because the
//!   guard is far above any real page (an A0 sheet at step 3 pt needs ~2,000 segments).

use crate::{GeomError, HatchPatternId, Loop, PdfPt, Segment};

/// Work guard for the legacy generator: the most segments one call may produce.
pub const MAX_LEGACY_SEGMENTS: usize = 100_000;

/// `studio.rs::normalize_rect` @ 562e4f9: accepts `[x0, y0, x1, y1]` (any corner order) or, when
/// the second pair is smaller than the first, `[x, y, w, h]`; always at least 1 pt wide and high.
fn normalize_rect(r: [f64; 4]) -> [f64; 4] {
    let (x0, y0, x1, y1) = if r[2] > 0.0 && r[3] > 0.0 && (r[2] < r[0] || r[3] < r[1]) {
        (r[0], r[1], r[0] + r[2], r[1] + r[3])
    } else {
        (
            r[0].min(r[2]),
            r[1].min(r[3]),
            r[0].max(r[2]),
            r[1].max(r[3]),
        )
    };
    [x0, y0, x1.max(x0 + 1.0), y1.max(y0 + 1.0)]
}

/// The bounding box of every finite point of every loop, normalised like
/// `studio.rs::bbox_from_points_or_rect` @ 562e4f9.
fn bbox_of_loops(loops: &[Loop]) -> Result<[f64; 4], GeomError> {
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for pt in loops.iter().flat_map(|l| l.points.iter().copied()) {
        if pt.is_finite() {
            min_x = min_x.min(pt.x);
            min_y = min_y.min(pt.y);
            max_x = max_x.max(pt.x);
            max_y = max_y.max(pt.y);
        }
    }
    if !min_x.is_finite() || !min_y.is_finite() {
        return Err(GeomError::NoFiniteGeometry);
    }
    Ok(normalize_rect([min_x, min_y, max_x, max_y]))
}

pub(crate) fn hatch_segments(
    pattern: HatchPatternId,
    loops: &[Loop],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    let rect = bbox_of_loops(loops)?;
    Ok(generate_hatch_strokes(rect, pattern, scale)?
        .into_iter()
        .map(|[a, b]| Segment {
            a: PdfPt::new(a[0], a[1]),
            b: PdfPt::new(b[0], b[1]),
        })
        .collect())
}

/// `studio.rs::generate_hatch_strokes` @ 562e4f9, with the pattern string match replaced by
/// [`HatchPatternId`] (same groups), two-point strokes returned as pairs, and the
/// [`MAX_LEGACY_SEGMENTS`] guard (the only behavioural difference: an error where the original hung).
fn generate_hatch_strokes(
    rect: [f64; 4],
    pattern: HatchPatternId,
    scale: f64,
) -> Result<Vec<[[f64; 2]; 2]>, GeomError> {
    let [x0, y0, x1, y1] = normalize_rect(rect);
    let step = (8.0 * scale.clamp(0.25, 4.0)).max(3.0);
    let mut strokes = Vec::new();
    let guard = |n: usize| {
        if n >= MAX_LEGACY_SEGMENTS {
            Err(GeomError::WorkLimitExceeded {
                limit: MAX_LEGACY_SEGMENTS,
            })
        } else {
            Ok(())
        }
    };

    match pattern {
        HatchPatternId::DiagonalSteel | HatchPatternId::Steel => {
            let w = x1 - x0;
            let h = y1 - y0;
            let span = w + h;
            let mut d = 0.0;
            while d <= span {
                let sx = (x0 + d - h).clamp(x0, x1);
                let sy = (y0 + (d - (sx - x0))).clamp(y0, y1);
                let ex = (x0 + d).clamp(x0, x1);
                let ey = (y0 + (d - (ex - x0))).clamp(y0, y1);
                guard(strokes.len())?;
                strokes.push([[sx, sy], [ex, ey]]);
                d += step;
            }
        }
        HatchPatternId::Crosshatch | HatchPatternId::Tiling | HatchPatternId::Masonry => {
            let mut x = x0;
            while x <= x1 {
                guard(strokes.len())?;
                strokes.push([[x, y0], [x, y1]]);
                x += step;
            }
            let mut y = y0;
            while y <= y1 {
                guard(strokes.len())?;
                strokes.push([[x0, y], [x1, y]]);
                y += step;
            }
        }
        HatchPatternId::Timber
        | HatchPatternId::Earth
        | HatchPatternId::Insulation
        | HatchPatternId::Concrete => {
            let mut y = y0;
            while y <= y1 {
                guard(strokes.len())?;
                strokes.push([[x0, y], [x1, y]]);
                y += step;
            }
        }
    }

    Ok(strokes)
}
