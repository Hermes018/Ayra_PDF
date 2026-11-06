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

fn orient(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// True when segment p-q crosses polygon edge a-b at a point interior to both (a proper
/// crossing). Touching at an endpoint or vertex, or running along an edge, is not a crossing.
fn properly_crosses(p: (f64, f64), q: (f64, f64), a: (f64, f64), b: (f64, f64)) -> bool {
    const EPS: f64 = 1e-9;
    let (d1, d2) = (orient(a, b, p), orient(a, b, q));
    let (d3, d4) = (orient(p, q, a), orient(p, q, b));
    ((d1 > EPS && d2 < -EPS) || (d1 < -EPS && d2 > EPS))
        && ((d3 > EPS && d4 < -EPS) || (d3 < -EPS && d4 > EPS))
}

/// A segment stays inside (or on) the polygon when it crosses no edge properly AND 17 evenly
/// spaced points along it are inside or on the boundary. The sampling catches a segment that
/// leaves through a vertex (no proper crossing); the crossing test catches a short excursion
/// between samples, e.g. an off-centre cut across the L's reflex corner.
fn segment_stays_inside(p: (f64, f64), q: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let n = poly.len();
    if (0..n).any(|i| properly_crosses(p, q, poly[i], poly[(i + 1) % n])) {
        return false;
    }
    (0..=16).all(|k| {
        let t = f64::from(k) / 16.0;
        inside_or_on((p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t), poly)
    })
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
                assert!(
                    segment_stays_inside((s.a.x, s.a.y), (s.b.x, s.b.y), poly),
                    "{}: segment {s:?} leaves the polygon",
                    p.as_id()
                );
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

/// The checker itself: a segment whose endpoints and midpoint are all inside-or-on the L but
/// which cuts across the reflex corner (150, 140) off-centre must be rejected; segments along an
/// edge, ending on one, or touching the reflex vertex from inside must be accepted.
#[test]
fn checker_catches_an_off_centre_cut_across_the_reflex_corner() {
    let (a, b) = ((149.0, 145.0), (239.0, 135.0));
    let mid = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
    assert!(inside_or_on(a, &L_POLY) && inside_or_on(b, &L_POLY) && inside_or_on(mid, &L_POLY));
    assert!(
        !segment_stays_inside(a, b, &L_POLY),
        "off-centre reflex cut must be caught"
    );
    // Leaves through the reflex vertex without a proper crossing: caught by sampling.
    assert!(!segment_stays_inside(
        (120.0, 110.0),
        (180.0, 170.0),
        &L_POLY
    ));
    // Legitimate hatch geometry.
    assert!(segment_stays_inside(
        (150.0, 140.0),
        (240.0, 140.0),
        &L_POLY
    )); // along an edge
    assert!(segment_stays_inside(
        (100.0, 120.0),
        (240.0, 120.0),
        &L_POLY
    )); // wall to wall
    assert!(segment_stays_inside(
        (110.0, 100.0),
        (150.0, 140.0),
        &L_POLY
    )); // ends at the reflex vertex
    assert!(segment_stays_inside(
        (150.0, 150.0),
        (150.0, 190.0),
        &L_POLY
    )); // on the notch edge
}

/// (name, shape, loop, pattern, scale)
type GoldenCase = (
    String,
    &'static str,
    &'static [(f64, f64)],
    HatchPatternId,
    f64,
);

/// Parity record (ADR-018 R2 step 4): the cadcraft output for the same 21 cases as the legacy
/// golden, with a note per case on how and why it differs. `tests/goldens/hatch_cadcraft.golden.json`.
/// Regenerate only for an intended, explained change: `LPDF_UPDATE_GOLDENS=1`.
#[test]
fn cadcraft_golden_with_parity_notes() {
    fn num(v: f64) -> String {
        match serde_json::to_string(&v) {
            Ok(s) => s,
            Err(e) => panic!("non-finite number in golden: {v} ({e})"),
        }
    }
    let mut cases: Vec<GoldenCase> = Vec::new();
    for (shape, pts) in [("rect", &RECT[..]), ("L", &L_POLY[..])] {
        for p in HatchPatternId::ALL {
            cases.push((
                format!("{}/{}/scale-1", shape, p.as_id()),
                shape,
                pts,
                p,
                1.0,
            ));
        }
    }
    cases.push((
        "rect/diagonal-steel/scale-0.1".into(),
        "rect",
        &RECT[..],
        HatchPatternId::DiagonalSteel,
        0.1,
    ));
    cases.push((
        "rect/diagonal-steel/scale-10".into(),
        "rect",
        &RECT[..],
        HatchPatternId::DiagonalSteel,
        10.0,
    ));
    cases.push((
        "rect/crosshatch/scale-2.5".into(),
        "rect",
        &RECT[..],
        HatchPatternId::Crosshatch,
        2.5,
    ));
    let mut out = String::from("{\n  \"provenance\": \"lpdf-geom HatchEngine::Cadcraft (cadcraft@df65974 cadcraft-doc library + patched cadcraft-render pattern_lines), same cases as hatch_legacy.golden.json\",\n");
    out.push_str("  \"parity\": \"Every case differs from legacy by design: ERR-007 fixed (segments are clipped to the loop, even-odd) and each pattern id maps to a distinct cadcraft family set (ANSI31, ANSI32, ANSI37, NET, BRICK, custom timber, EARTH, ZIGZAG, AR-CONC). Scale keeps the legacy 0.25..4 clamp; 1 = 64 pt per pattern unit (ANSI 0.125 -> 8 pt; AR-CONC, defined 16x coarser upstream, uses 4).\",\n  \"cases\": [\n");
    for (ci, (name, _shape, pts, p, scale)) in cases.iter().enumerate() {
        let segs = match hatch_segments_with(HatchEngine::Cadcraft, *p, &[to_loop(pts)], *scale) {
            Ok(s) => s,
            Err(e) => panic!("{name}: {e}"),
        };
        let leaving = segs
            .iter()
            .filter(|s| !segment_stays_inside((s.a.x, s.a.y), (s.b.x, s.b.y), pts))
            .count();
        let note = if name.starts_with("L/") {
            "differs from legacy: clipped to the concave L (legacy hatched its bbox, ERR-007)"
        } else {
            "differs from legacy: cadcraft family set for this pattern (legacy drew it like other patterns)"
        };
        out.push_str(&format!(
            "    {{\n      \"name\": \"{name}\",\n      \"pattern\": \"{}\",\n      \"scale\": {},\n      \"loop\": [{}],\n      \"note\": \"{note}\",\n      \"segmentCount\": {},\n      \"segmentsLeavingLoop\": {leaving},\n      \"segments\": [\n",
            p.as_id(),
            num(*scale),
            pts.iter().map(|&(x, y)| format!("[{}, {}]", num(x), num(y))).collect::<Vec<_>>().join(", "),
            segs.len()
        ));
        for (si, s) in segs.iter().enumerate() {
            let comma = if si + 1 < segs.len() { "," } else { "" };
            out.push_str(&format!(
                "        [{}, {}, {}, {}]{comma}\n",
                num(s.a.x),
                num(s.a.y),
                num(s.b.x),
                num(s.b.y)
            ));
        }
        let comma = if ci + 1 < cases.len() { "," } else { "" };
        out.push_str(&format!("      ]\n    }}{comma}\n"));
        assert_eq!(leaving, 0, "{name}");
    }
    out.push_str("  ]\n}\n");
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/goldens/hatch_cadcraft.golden.json");
    if std::env::var_os("LPDF_UPDATE_GOLDENS").is_some() {
        if let Err(e) = std::fs::write(&path, &out) {
            panic!("cannot write {}: {e}", path.display());
        }
    }
    let golden = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => panic!(
            "missing golden {} ({e}); run with LPDF_UPDATE_GOLDENS=1",
            path.display()
        ),
    };
    assert!(
        golden == out,
        "cadcraft hatch drifted from {}",
        path.display()
    );
}

#[test]
fn swatch_is_inside_its_square_and_distinct_per_pattern() {
    let mut seen: Vec<Vec<Segment>> = Vec::new();
    for p in HatchPatternId::ALL {
        let sw = match lpdf_geom::hatch_swatch(p, 24.0) {
            Ok(s) => s,
            Err(e) => panic!("{}: {e}", p.as_id()),
        };
        assert!(!sw.is_empty(), "{}", p.as_id());
        for s in &sw {
            for q in [s.a, s.b] {
                assert!(
                    (-1e-9..=24.0 + 1e-9).contains(&q.x) && (-1e-9..=24.0 + 1e-9).contains(&q.y),
                    "{}: {q:?}",
                    p.as_id()
                );
            }
        }
        assert!(
            !seen.contains(&sw),
            "{} swatch duplicates another pattern",
            p.as_id()
        );
        seen.push(sw);
    }
    assert!(lpdf_geom::hatch_swatch(HatchPatternId::Timber, 0.0).is_err());
    assert!(lpdf_geom::hatch_swatch(HatchPatternId::Timber, f64::NAN).is_err());
}

/// Owner rule (2026-11-06): never edit upstream pattern data; adjust scale only in our adapter.
/// The x1/16 for AR-CONC is applied by the adapter, the vendored definition is still upstream's.
#[test]
fn ar_conc_display_scale_lives_in_the_adapter_only() {
    use cadcraft_doc::library;
    use std::path::PathBuf;

    // 1. The adapter applies 1/16 to AR-CONC (and nothing to the other patterns).
    assert_eq!(lpdf_geom::AR_CONC_DISPLAY_SCALE, 1.0 / 16.0);
    for p in HatchPatternId::ALL {
        let want = if p == HatchPatternId::Concrete {
            1.0 / 16.0
        } else {
            1.0
        };
        assert_eq!(lpdf_geom::cadcraft_display_scale(p), want, "{}", p.as_id());
    }

    // 2. The definition cadcraft-doc hands out is upstream's: the four lines of
    //    `scaled_pattern(AR-CONC, 16.0)` at df65974, spelled out with the same arithmetic.
    let k = 16.0;
    let want = [
        (
            50.0,
            (0.0 * k, 0.0 * k),
            (0.4 * k, 0.25 * k),
            [0.15 * k, -0.6 * k],
        ),
        (
            355.0,
            (0.0 * k, 0.0 * k),
            (0.27 * k, 0.3 * k),
            [0.12 * k, -0.5 * k],
        ),
        (
            100.0,
            (0.12 * k, 0.05 * k),
            (0.35 * k, 0.33 * k),
            [0.1 * k, -0.55 * k],
        ),
        (
            0.0,
            (0.07 * k, 0.02 * k),
            (0.21 * k, 0.17 * k),
            [0.0 * k, -0.4 * k],
        ),
    ];
    let got = library::pattern("AR-CONC")
        .map(|p| p.lines)
        .unwrap_or_default();
    assert_eq!(got.len(), want.len());
    for (g, (angle, origin, delta, dashes)) in got.iter().zip(want) {
        assert_eq!(
            (g.angle, g.origin, g.delta, g.dashes.as_slice()),
            (angle, origin, delta, dashes.as_slice())
        );
    }

    // 3. Nothing in vendor/ was changed to rescale it: our patch of cadcraft-render differs from
    //    the pristine vendored copy only in src/lib.rs (the one re-export), and holds no pattern data.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../vendor");
    let pristine = root.join("craft/cadcraft/cadcraft-render-0.17.0/src");
    let patched = root.join("patches/cadcraft-render/src");
    let mut seen = 0;
    for entry in std::fs::read_dir(&patched)
        .map(|d| d.flatten().collect::<Vec<_>>())
        .unwrap_or_default()
    {
        let name = entry.file_name();
        if !entry.path().is_file() || name == "lib.rs" {
            continue;
        }
        seen += 1;
        let a = std::fs::read(entry.path()).unwrap_or_default();
        let b = std::fs::read(pristine.join(&name)).unwrap_or_else(|_| vec![1]);
        assert!(a == b, "patched {name:?} differs from upstream");
    }
    assert!(seen > 0, "no patched files compared");
    let lib_patched = std::fs::read_to_string(patched.join("lib.rs")).unwrap_or_default();
    let lib_pristine = std::fs::read_to_string(pristine.join("lib.rs")).unwrap_or_default();
    let diff: Vec<_> = lib_patched
        .lines()
        .zip(lib_pristine.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(lib_patched.lines().count(), lib_pristine.lines().count());
    assert_eq!(
        diff.len(),
        1,
        "lib.rs may differ in the one re-export line only: {diff:?}"
    );
    assert!(diff[0].0.contains("pattern_lines"));
}
