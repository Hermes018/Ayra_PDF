//! ERR-007 / BUG-015: the default hatch (the one Studio save uses) must stay inside the polygon,
//! respect holes, and give every pattern its own look. Written first; red while the default
//! engine is `Legacy` (bounding-box hatching), green with the clipped `Studio` engine.
//! Only the public default entry point `hatch_segments` is used here, so these tests judge
//! whatever Studio save would write.

use lpdf_geom::{hatch_segments, GeomError, HatchPatternId, Loop, PdfPt, Segment};

type Pt = (f64, f64);

const RECT: [Pt; 4] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 200.0),
    (100.0, 200.0),
];
const L_POLY: [Pt; 6] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 140.0),
    (150.0, 140.0),
    (150.0, 200.0),
    (100.0, 200.0),
];

/// Ten-vertex five-pointed star (concave), centre (300, 300).
fn star() -> Vec<Pt> {
    (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { 90.0 } else { 38.0 };
            let a = std::f64::consts::FRAC_PI_2 + f64::from(i) * std::f64::consts::PI / 5.0;
            (300.0 + r * a.cos(), 300.0 + r * a.sin())
        })
        .collect()
}

const OUTER: [Pt; 4] = [
    (100.0, 100.0),
    (300.0, 100.0),
    (300.0, 300.0),
    (100.0, 300.0),
];
const HOLE: [Pt; 4] = [
    (160.0, 160.0),
    (160.0, 240.0),
    (240.0, 240.0),
    (240.0, 160.0),
];

fn to_loop(pts: &[Pt]) -> Loop {
    Loop::new(pts.iter().map(|&(x, y)| PdfPt::new(x, y)).collect())
}

fn rings_of(shape: &[&[Pt]]) -> Vec<Loop> {
    shape.iter().map(|r| to_loop(r)).collect()
}

fn dist_to_segment(p: Pt, a: Pt, b: Pt) -> f64 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let len2 = vx * vx + vy * vy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p.0 - (a.0 + t * vx)).hypot(p.1 - (a.1 + t * vy))
}

fn edges(shape: &[&[Pt]]) -> Vec<(Pt, Pt)> {
    let mut out = Vec::new();
    for r in shape {
        for i in 0..r.len() {
            out.push((r[i], r[(i + 1) % r.len()]));
        }
    }
    out
}

/// Even-odd over all rings, or within `tol` of any edge.
fn inside_or_on(p: Pt, shape: &[&[Pt]], tol: f64) -> bool {
    let mut c = false;
    for (a, b) in edges(shape) {
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
    }
    c || edges(shape)
        .iter()
        .any(|&(a, b)| dist_to_segment(p, a, b) <= tol)
}

fn orient(a: Pt, b: Pt, c: Pt) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// Segment p-q crosses edge a-b at a point interior to both (touching/running along is allowed).
fn properly_crosses(p: Pt, q: Pt, a: Pt, b: Pt) -> bool {
    const EPS: f64 = 1e-7;
    let (d1, d2) = (orient(a, b, p), orient(a, b, q));
    let (d3, d4) = (orient(p, q, a), orient(p, q, b));
    ((d1 > EPS && d2 < -EPS) || (d1 < -EPS && d2 > EPS))
        && ((d3 > EPS && d4 < -EPS) || (d3 < -EPS && d4 > EPS))
}

const TOL: f64 = 1e-6;

fn assert_segment_inside(name: &str, s: &Segment, shape: &[&[Pt]]) {
    let (p, q) = ((s.a.x, s.a.y), (s.b.x, s.b.y));
    let mid = ((p.0 + q.0) / 2.0, (p.1 + q.1) / 2.0);
    for (what, pt) in [("start", p), ("midpoint", mid), ("end", q)] {
        assert!(
            inside_or_on(pt, shape, TOL),
            "{name}: {what} {pt:?} of {s:?} is outside the polygon"
        );
    }
    for (a, b) in edges(shape) {
        assert!(
            !properly_crosses(p, q, a, b),
            "{name}: {s:?} crosses polygon edge {a:?}-{b:?}"
        );
    }
    // Between the samples as well: 17 points along the segment.
    for k in 0..=16 {
        let t = f64::from(k) / 16.0;
        let pt = (p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t);
        assert!(
            inside_or_on(pt, shape, TOL),
            "{name}: sample {pt:?} of {s:?} is outside the polygon"
        );
    }
}

fn run(p: HatchPatternId, shape: &[&[Pt]], scale: f64) -> Vec<Segment> {
    match hatch_segments(p, &rings_of(shape), scale) {
        Ok(s) => s,
        Err(e) => panic!("{}: {e}", p.as_id()),
    }
}

fn shapes() -> Vec<(&'static str, Vec<Vec<Pt>>)> {
    vec![
        ("rectangle", vec![RECT.to_vec()]),
        ("concave L", vec![L_POLY.to_vec()]),
        ("star", vec![star()]),
        ("square with a hole", vec![OUTER.to_vec(), HOLE.to_vec()]),
        (
            "hole wound the same way",
            vec![OUTER.to_vec(), HOLE.iter().rev().copied().collect()],
        ),
    ]
}

fn as_refs(rings: &[Vec<Pt>]) -> Vec<&[Pt]> {
    rings.iter().map(|r| r.as_slice()).collect()
}

#[test]
fn every_saved_segment_lies_inside_the_polygon() {
    for (shape_name, rings) in shapes() {
        let shape = as_refs(&rings);
        for p in HatchPatternId::ALL {
            for scale in [1.0, 0.5, 2.0] {
                let segs = run(p, &shape, scale);
                assert!(
                    !segs.is_empty(),
                    "{shape_name}/{}: no hatch at scale {scale}",
                    p.as_id()
                );
                let name = format!("{shape_name}/{}/scale-{scale}", p.as_id());
                for s in &segs {
                    assert_segment_inside(&name, s, &shape);
                }
            }
        }
    }
}

#[test]
fn hatch_reaches_the_whole_region_not_just_part_of_it() {
    // The L's long arm and the square's frame (around the hole) must both receive hatch.
    let l = vec![L_POLY.to_vec()];
    let framed = vec![OUTER.to_vec(), HOLE.to_vec()];
    for p in HatchPatternId::ALL {
        let segs = run(p, &as_refs(&l), 1.0);
        assert!(
            segs.iter().any(|s| s.a.y > 150.0 || s.b.y > 150.0),
            "{}: nothing in the upper arm of the L",
            p.as_id()
        );
        assert!(
            segs.iter().any(|s| s.a.x > 200.0 || s.b.x > 200.0),
            "{}: nothing in the lower arm of the L",
            p.as_id()
        );
        let segs = run(p, &as_refs(&framed), 1.0);
        assert!(
            segs.iter().any(|s| s.a.x < 150.0 && s.b.x < 150.0)
                && segs.iter().any(|s| s.a.x > 250.0 && s.b.x > 250.0),
            "{}: frame not hatched on both sides of the hole",
            p.as_id()
        );
    }
}

#[test]
fn nothing_is_drawn_inside_a_hole() {
    let rings = vec![OUTER.to_vec(), HOLE.to_vec()];
    let shape = as_refs(&rings);
    for p in HatchPatternId::ALL {
        for s in run(p, &shape, 1.0) {
            let mid = ((s.a.x + s.b.x) / 2.0, (s.a.y + s.b.y) / 2.0);
            let strictly_in_hole = mid.0 > 160.0 + TOL
                && mid.0 < 240.0 - TOL
                && mid.1 > 160.0 + TOL
                && mid.1 < 240.0 - TOL;
            assert!(!strictly_in_hole, "{}: {s:?} is inside the hole", p.as_id());
        }
    }
}

#[test]
fn every_pattern_gives_distinct_output() {
    for (shape_name, rings) in shapes() {
        let shape = as_refs(&rings);
        let outputs: Vec<(HatchPatternId, Vec<Segment>)> = HatchPatternId::ALL
            .iter()
            .map(|&p| (p, run(p, &shape, 1.0)))
            .collect();
        for (i, (pa, a)) in outputs.iter().enumerate() {
            for (pb, b) in outputs.iter().skip(i + 1) {
                assert_ne!(
                    a,
                    b,
                    "{shape_name}: {} and {} produce the same hatch",
                    pa.as_id(),
                    pb.as_id()
                );
            }
        }
    }
}

#[test]
fn scale_changes_spacing_and_output_is_deterministic() {
    let rings = vec![RECT.to_vec()];
    let shape = as_refs(&rings);
    let fine = run(HatchPatternId::DiagonalSteel, &shape, 0.5);
    let coarse = run(HatchPatternId::DiagonalSteel, &shape, 2.0);
    assert!(
        fine.len() > coarse.len(),
        "{} !> {}",
        fine.len(),
        coarse.len()
    );
    assert_eq!(fine, run(HatchPatternId::DiagonalSteel, &shape, 0.5));
}

// ---- degenerate cases: never panic, never leave the polygon ------------------------------------

/// A diamond whose left and right vertices sit exactly on hatch lines (y = 100).
#[test]
fn vertex_on_a_scanline_is_handled() {
    let diamond: Vec<Pt> = vec![(100.0, 80.0), (120.0, 100.0), (100.0, 120.0), (80.0, 100.0)];
    let shape: Vec<&[Pt]> = vec![&diamond];
    for p in HatchPatternId::ALL {
        for scale in [0.5, 1.0, 2.5] {
            let segs = run(p, &shape, scale);
            let name = format!("diamond/{}/{scale}", p.as_id());
            for s in &segs {
                assert_segment_inside(&name, s, &shape);
            }
        }
    }
    // Timber is horizontal: the line through both side vertices is a full-width span.
    let timber = run(HatchPatternId::Timber, &shape, 1.0);
    assert!(
        timber
            .iter()
            .any(|s| (s.a.y - 100.0).abs() < 1e-9 && (s.b.x - s.a.x).abs() > 10.0),
        "no hatch line through the side vertices: {timber:?}"
    );
}

/// Extra vertices in the middle of straight edges change nothing.
#[test]
fn collinear_edges_give_the_same_hatch() {
    let plain: Vec<Pt> = RECT.to_vec();
    let split: Vec<Pt> = vec![
        (100.0, 100.0),
        (170.0, 100.0),
        (240.0, 100.0),
        (240.0, 150.0),
        (240.0, 200.0),
        (170.0, 200.0),
        (100.0, 200.0),
        (100.0, 150.0),
    ];
    for p in HatchPatternId::ALL {
        let a = run(p, &[&plain], 1.0);
        let b = run(p, &[&split], 1.0);
        assert_eq!(a.len(), b.len(), "{}: segment count", p.as_id());
        let key = |s: &Segment| (s.a.x, s.a.y, s.b.x, s.b.y);
        let mut ka: Vec<_> = a.iter().map(key).collect();
        let mut kb: Vec<_> = b.iter().map(key).collect();
        ka.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        kb.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        for (x, y) in ka.iter().zip(kb.iter()) {
            assert!(
                (x.0 - y.0).abs() < 1e-9
                    && (x.1 - y.1).abs() < 1e-9
                    && (x.2 - y.2).abs() < 1e-9
                    && (x.3 - y.3).abs() < 1e-9,
                "{}: {x:?} vs {y:?}",
                p.as_id()
            );
        }
    }
}

#[test]
fn zero_area_regions_hatch_nothing_and_do_not_panic() {
    let line: Vec<Pt> = vec![(10.0, 10.0), (50.0, 10.0), (90.0, 10.0)];
    let dot: Vec<Pt> = vec![(10.0, 10.0), (10.0, 10.0), (10.0, 10.0)];
    let diag: Vec<Pt> = vec![(0.0, 0.0), (40.0, 40.0), (80.0, 80.0)];
    let back_and_forth: Vec<Pt> = vec![(0.0, 0.0), (50.0, 0.0), (0.0, 0.0), (50.0, 0.0)];
    for ring in [line, dot, diag, back_and_forth] {
        for p in HatchPatternId::ALL {
            let got = hatch_segments(p, &[to_loop(&ring)], 1.0);
            assert_eq!(got, Ok(Vec::new()), "{}: {ring:?}", p.as_id());
        }
    }
}

#[test]
fn non_finite_points_are_ignored_and_empty_input_is_typed() {
    let mut ring = RECT.to_vec();
    ring.push((f64::NAN, 10.0));
    ring.push((f64::INFINITY, f64::NEG_INFINITY));
    let clean = run(HatchPatternId::Crosshatch, &[&RECT], 1.0);
    let with_junk = run(HatchPatternId::Crosshatch, &[&ring], 1.0);
    assert_eq!(clean, with_junk);
    assert_eq!(
        hatch_segments(HatchPatternId::Crosshatch, &[], 1.0),
        Err(GeomError::NoFiniteGeometry)
    );
    let nan = Loop::new(vec![PdfPt::new(f64::NAN, 1.0), PdfPt::new(2.0, f64::NAN)]);
    assert_eq!(
        hatch_segments(HatchPatternId::Crosshatch, &[nan], 1.0),
        Err(GeomError::NoFiniteGeometry)
    );
    // Non-finite scale falls back to the default instead of hanging.
    assert_eq!(
        run(HatchPatternId::Timber, &[&RECT], f64::NAN),
        run(HatchPatternId::Timber, &[&RECT], 1.0)
    );
}

#[test]
fn work_cap_is_kept_and_far_coordinates_do_not_hang() {
    let huge = Loop::from_rect([0.0, 0.0, 1.0e9, 1.0e9]);
    for p in HatchPatternId::ALL {
        let got = hatch_segments(p, std::slice::from_ref(&huge), 1.0);
        assert!(
            matches!(got, Err(GeomError::WorkLimitExceeded { .. })),
            "{}: {got:?}",
            p.as_id()
        );
    }
    let err = hatch_segments(HatchPatternId::Timber, &[huge], 1.0).err();
    assert_eq!(err.map(|e| e.code()), Some("GEOM_WORK_LIMIT"));
    // 1e17 + a few points: finite answer (a typed error or segments), never a hang or panic.
    let far = Loop::from_rect([1.0e17, 1.0e17, 1.0e17 + 32.0, 1.0e17 + 32.0]);
    for p in HatchPatternId::ALL {
        let _ = hatch_segments(p, std::slice::from_ref(&far), 1.0);
    }
}
