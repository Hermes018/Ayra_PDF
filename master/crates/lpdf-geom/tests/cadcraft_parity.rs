//! ADR-018 R2 step 4: parity of the cadcraft hatch engine against the legacy goldens.
//! Expected differences (ERR-007 / BUG-015): on the concave L every segment must lie inside the
//! polygon, and every pattern must produce distinct output. On the rectangle the region is the
//! same as the legacy bbox, so segments stay inside it too.
#![cfg(feature = "cadcraft")]

use lpdf_geom::{
    hatch_segments_with, GeomError, HatchEngine, HatchPatternId, Loop, PdfPt, Segment,
};

const RECT: [(f64, f64); 4] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 200.0),
    (100.0, 200.0),
];
const L_POLY: [(f64, f64); 6] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 140.0),
    (150.0, 140.0),
    (150.0, 200.0),
    (100.0, 200.0),
];

fn to_loop(pts: &[(f64, f64)]) -> Loop {
    Loop::new(pts.iter().map(|&(x, y)| PdfPt::new(x, y)).collect())
}

fn dist_to_boundary(p: (f64, f64), poly: &[(f64, f64)]) -> f64 {
    let n = poly.len();
    (0..n)
        .map(|i| {
            let (ax, ay) = poly[i];
            let (bx, by) = poly[(i + 1) % n];
            let (vx, vy) = (bx - ax, by - ay);
            let t = (((p.0 - ax) * vx + (p.1 - ay) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
            (p.0 - (ax + t * vx)).hypot(p.1 - (ay + t * vy))
        })
        .fold(f64::INFINITY, f64::min)
}

fn inside_or_on(p: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[(i + n - 1) % n];
        if (yi > p.1) != (yj > p.1) && p.0 < (xj - xi) * (p.1 - yi) / (yj - yi) + xi {
            c = !c;
        }
    }
    c || dist_to_boundary(p, poly) <= 1e-6
}

fn run(p: HatchPatternId, poly: &[(f64, f64)]) -> Vec<Segment> {
    match hatch_segments_with(HatchEngine::Cadcraft, p, &[to_loop(poly)], 1.0) {
        Ok(s) => s,
        Err(e) => panic!("{}: {e}", p.as_id()),
    }
}

#[test]
fn every_segment_is_inside_the_concave_polygon() {
    for poly in [&L_POLY[..], &RECT[..]] {
        for p in HatchPatternId::ALL {
            let segs = run(p, poly);
            assert!(!segs.is_empty(), "{}: no hatch", p.as_id());
            for s in &segs {
                let mid = ((s.a.x + s.b.x) / 2.0, (s.a.y + s.b.y) / 2.0);
                for q in [(s.a.x, s.a.y), (s.b.x, s.b.y), mid] {
                    assert!(
                        inside_or_on(q, poly),
                        "{}: point {q:?} of {s:?} is outside",
                        p.as_id()
                    );
                }
            }
        }
    }
}

#[test]
fn every_pattern_is_distinct() {
    let outputs: Vec<(HatchPatternId, Vec<Segment>)> = HatchPatternId::ALL
        .iter()
        .map(|&p| (p, run(p, &L_POLY)))
        .collect();
    for (i, (pa, a)) in outputs.iter().enumerate() {
        for (pb, b) in outputs.iter().skip(i + 1) {
            assert_ne!(
                a,
                b,
                "{} and {} produce the same hatch",
                pa.as_id(),
                pb.as_id()
            );
        }
    }
}

#[test]
fn guards_and_errors() {
    let huge = Loop::from_rect([0.0, 0.0, 1.0e9, 1.0e9]);
    assert_eq!(
        hatch_segments_with(
            HatchEngine::Cadcraft,
            HatchPatternId::Crosshatch,
            &[huge],
            1.0
        ),
        Err(GeomError::WorkLimitExceeded {
            limit: lpdf_geom::MAX_CADCRAFT_RUNS
        })
    );
    assert_eq!(
        hatch_segments_with(HatchEngine::Cadcraft, HatchPatternId::Crosshatch, &[], 1.0),
        Err(GeomError::NoFiniteGeometry)
    );
}
