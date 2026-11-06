//! Even-odd clipping of hatch line families to polygon rings (original work).
//!
//! Every hatch line is intersected with every ring edge, the crossings are sorted along the line,
//! and consecutive pairs are the inside spans (even-odd rule over all rings, so holes and concave
//! notches stay empty). Vertices that sit exactly on a hatch line are resolved with the half-open
//! rule "an edge crosses the line when one end is on or below it and the other strictly above":
//! each vertex is then counted consistently by both edges that meet there, spans never get an odd
//! crossing count, and edges lying along a line contribute nothing by themselves (a line running
//! along an edge is kept when the region is above it and dropped when it is below, so adjacent
//! tiles draw a shared boundary line once).
//!
//! Lines are handled in a canonical frame where they are closer to horizontal than vertical
//! (`y = slope * x + b`, |slope| <= 1); mostly-vertical families are mirrored across the diagonal
//! (x and y swapped) and the result is swapped back.
//!
//! Work is bounded: the number of lines, the line x edge product, dash steps and output segments are
//! all capped, and exceeding any cap is [`GeomError::WorkLimitExceeded`] (never a hang, never a silent
//! truncation).

use crate::patterns::{unit_pt, HatchFamily};
use crate::{GeomError, PdfPt, Segment};

/// Most segments one call may produce (same limit as the legacy generator).
pub const MAX_STUDIO_SEGMENTS: usize = 100_000;
/// Most (line, edge) pairs one call may test.
const MAX_EDGE_TESTS: f64 = 400_000_000.0;
/// Shortest segment kept, in points.
const MIN_LEN: f64 = 1e-9;

type P = (f64, f64);

fn limit_err() -> GeomError {
    GeomError::WorkLimitExceeded {
        limit: MAX_STUDIO_SEGMENTS,
    }
}

/// Rings of finite points; rings with fewer than three of them cannot bound a region and are dropped.
pub(crate) fn finite_rings(loops: &[crate::Loop]) -> Vec<Vec<P>> {
    loops
        .iter()
        .map(|l| {
            l.points
                .iter()
                .filter(|p| p.x.is_finite() && p.y.is_finite())
                .map(|p| (p.x, p.y))
                .collect::<Vec<_>>()
        })
        .filter(|r| r.len() >= 3)
        .collect()
}

/// Hatch `rings` (even-odd) with the families of a pattern at user `scale`.
pub(crate) fn hatch_rings(
    families: &[HatchFamily],
    rings: &[Vec<P>],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    if rings.is_empty() {
        return Err(GeomError::NoFiniteGeometry);
    }
    fill_rings(families, rings, unit_pt(scale))
}

/// Slope of lines at `angle_deg` in the canonical frame, and whether the frame is mirrored.
fn canonical(angle_deg: f64) -> Option<(f64, bool)> {
    if !angle_deg.is_finite() {
        return None;
    }
    let a = angle_deg.rem_euclid(180.0);
    // Exact slopes for the angles the patterns use; tan() is not exact at 135 degrees.
    if a == 0.0 {
        Some((0.0, false))
    } else if a == 45.0 {
        Some((1.0, false))
    } else if a == 135.0 {
        Some((-1.0, false))
    } else if a == 90.0 {
        Some((0.0, true))
    } else if a < 45.0 || a > 135.0 {
        Some((a.to_radians().tan(), false))
    } else {
        Some(((90.0 - a).to_radians().tan(), true))
    }
}

pub(crate) fn fill_rings(
    families: &[HatchFamily],
    rings: &[Vec<P>],
    unit: f64,
) -> Result<Vec<Segment>, GeomError> {
    if !(unit.is_finite() && unit > 0.0) {
        return Err(GeomError::NoFiniteGeometry);
    }
    let mut out: Vec<Segment> = Vec::new();
    let mut edge_tests = 0.0_f64;
    for fam in families {
        let Some((slope, mirrored)) = canonical(fam.angle_deg) else {
            continue;
        };
        if !(fam.pitch.is_finite() && fam.pitch > 0.0 && fam.offset.is_finite()) {
            continue;
        }
        let pts = |ring: &Vec<P>| -> Vec<P> {
            ring.iter()
                .map(|&(x, y)| if mirrored { (y, x) } else { (x, y) })
                .collect()
        };
        let cring: Vec<Vec<P>> = rings.iter().map(pts).collect();
        // Range of the line parameter b = y - slope * x over the region, in cells of pitch.
        let (mut smin, mut smax) = (f64::INFINITY, f64::NEG_INFINITY);
        for &(x, y) in cring.iter().flatten() {
            let s = y - slope * x;
            smin = smin.min(s);
            smax = smax.max(s);
        }
        if !(smin.is_finite() && smax.is_finite()) {
            return Err(GeomError::NoFiniteGeometry);
        }
        let kmin = ((smin / unit - fam.offset) / fam.pitch).ceil();
        let kmax = ((smax / unit - fam.offset) / fam.pitch).floor();
        if !(kmin.is_finite() && kmax.is_finite()) || kmin.abs() > 4.0e15 || kmax.abs() > 4.0e15 {
            return Err(limit_err());
        }
        if kmax < kmin {
            continue;
        }
        let lines = kmax - kmin + 1.0;
        let n_edges: usize = cring.iter().map(Vec::len).sum();
        edge_tests += lines * n_edges as f64;
        if lines > MAX_STUDIO_SEGMENTS as f64 || edge_tests > MAX_EDGE_TESTS {
            return Err(limit_err());
        }
        let dash = valid_dash(fam.dash);
        let mut xs: Vec<f64> = Vec::new();
        let n_lines = lines as usize;
        for i in 0..n_lines {
            let b = (fam.offset + (kmin + i as f64) * fam.pitch) * unit;
            xs.clear();
            for ring in &cring {
                let n = ring.len();
                for j in 0..n {
                    let (ax, ay) = ring[j];
                    let (bx, by) = ring[(j + 1) % n];
                    let sa = ay - slope * ax - b;
                    let sb = by - slope * bx - b;
                    if (sa <= 0.0 && sb > 0.0) || (sb <= 0.0 && sa > 0.0) {
                        xs.push(ax + (bx - ax) * (-sa) / (sb - sa));
                    }
                }
            }
            xs.sort_by(|p, q| p.total_cmp(q));
            // Crossings come in pairs (half-open rule); an odd tail cannot happen and is ignored.
            for &[x0, x1] in xs.as_chunks::<2>().0 {
                if x1 - x0 > MIN_LEN {
                    emit_span(&mut out, x0, x1, slope, b, mirrored, dash, fam.phase, unit)?;
                }
            }
        }
    }
    Ok(out)
}

/// The dash list when usable (even count, all lengths finite and non-negative, a positive total).
fn valid_dash(dash: &[f64]) -> Option<&[f64]> {
    let ok = !dash.is_empty()
        && dash.len().is_multiple_of(2)
        && dash.iter().all(|d| d.is_finite() && *d >= 0.0)
        && dash.iter().sum::<f64>() > 0.0;
    ok.then_some(dash)
}

#[allow(clippy::too_many_arguments)]
fn emit_span(
    out: &mut Vec<Segment>,
    x0: f64,
    x1: f64,
    slope: f64,
    b: f64,
    mirrored: bool,
    dash: Option<&[f64]>,
    phase: f64,
    unit: f64,
) -> Result<(), GeomError> {
    let mut push = |lo: f64, hi: f64| -> Result<(), GeomError> {
        if out.len() >= MAX_STUDIO_SEGMENTS {
            return Err(limit_err());
        }
        let (ay, by) = (slope * lo + b, slope * hi + b);
        let (a, c) = if mirrored {
            (PdfPt::new(ay, lo), PdfPt::new(by, hi))
        } else {
            (PdfPt::new(lo, ay), PdfPt::new(hi, by))
        };
        out.push(Segment { a, b: c });
        Ok(())
    };
    let Some(dash) = dash else {
        return push(x0, x1);
    };
    let period: f64 = dash.iter().sum::<f64>() * unit;
    let base = phase * unit;
    let first = ((x0 - base) / period).floor();
    if !first.is_finite() {
        return Err(limit_err());
    }
    let mut t = base + first * period;
    let mut steps = 0usize;
    loop {
        for (i, d) in dash.iter().enumerate() {
            steps += 1;
            if steps > MAX_STUDIO_SEGMENTS * 2 {
                return Err(limit_err());
            }
            let len = d * unit;
            if i % 2 == 0 {
                let (lo, hi) = (t.max(x0), (t + len).min(x1));
                if hi - lo > MIN_LEN {
                    push(lo, hi)?;
                }
            }
            t += len;
            if t >= x1 {
                return Ok(());
            }
        }
    }
}
