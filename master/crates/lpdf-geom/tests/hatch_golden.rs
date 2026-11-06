//! Characterization goldens for the legacy hatch generator (ADR-018 R2 step 1).
//!
//! `tests/goldens/hatch_legacy.golden.json` records the output of the ORIGINAL
//! `studio.rs::generate_hatch_strokes` @ 562e4f9 (copied verbatim below as `original`), for every
//! pattern on a rectangle and on a concave L-polygon. The adapter must reproduce it exactly:
//! numbers are compared through their shortest round-trip text (serde_json / ryu), so equal text
//! means bit-equal `f64`s.
//!
//! The L-polygon cases are **known-wrong, ERR-007**: the legacy generator hatches the bounding
//! box, so segments leave the polygon. Checkpoint 2 (cadcraft) is expected to differ there.
//!
//! ERR-007 intentional change, approved by owner 2026-11-06: the DEFAULT engine is now
//! `HatchEngine::Studio` (clipped hatch, own patterns). These goldens keep pinning the ORIGINAL
//! generator, so every call here selects `HatchEngine::Legacy` explicitly. The Studio engine's own
//! golden is `tests/goldens/hatch_studio.golden.json` (tests/studio_golden.rs).
//!
//! Regenerate (only when a change is intended and explained): `LPDF_UPDATE_GOLDENS=1 cargo nextest run -p lpdf-geom`.

use lpdf_geom::{
    hatch_segments_with, GeomError, HatchEngine, HatchPatternId, Loop, PdfPt, Segment,
};
use std::path::PathBuf;

/// Verbatim copy of the pre-adapter code path in `studio.rs` @ 562e4f9 (lines 209-298), used as
/// the oracle that produced the golden file. Test-only; removed together with the legacy engine.
mod original {
    #[derive(Clone, Copy)]
    pub struct StudioPointDto {
        pub x: f64,
        pub y: f64,
    }

    pub fn normalize_rect(r: [f64; 4]) -> [f64; 4] {
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

    pub fn bbox_from_points_or_rect(points: &[StudioPointDto], fallback: [f64; 4]) -> [f64; 4] {
        if points.is_empty() {
            return normalize_rect(fallback);
        }
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for pt in points {
            if pt.x.is_finite() && pt.y.is_finite() {
                min_x = min_x.min(pt.x);
                min_y = min_y.min(pt.y);
                max_x = max_x.max(pt.x);
                max_y = max_y.max(pt.y);
            }
        }
        if !min_x.is_finite() || !min_y.is_finite() {
            return normalize_rect(fallback);
        }
        normalize_rect([min_x, min_y, max_x, max_y])
    }

    pub fn generate_hatch_strokes(rect: [f64; 4], pattern: &str, scale: f64) -> Vec<Vec<[f64; 2]>> {
        let [x0, y0, x1, y1] = normalize_rect(rect);
        let step = (8.0 * scale.clamp(0.25, 4.0)).max(3.0);
        let mut strokes = Vec::new();

        match pattern {
            "diagonal-steel" | "steel" => {
                let w = x1 - x0;
                let h = y1 - y0;
                let span = w + h;
                let mut d = 0.0;
                while d <= span {
                    let sx = (x0 + d - h).clamp(x0, x1);
                    let sy = (y0 + (d - (sx - x0))).clamp(y0, y1);
                    let ex = (x0 + d).clamp(x0, x1);
                    let ey = (y0 + (d - (ex - x0))).clamp(y0, y1);
                    strokes.push(vec![[sx, sy], [ex, ey]]);
                    d += step;
                }
            }
            "crosshatch" | "tiling" | "masonry" => {
                let mut x = x0;
                while x <= x1 {
                    strokes.push(vec![[x, y0], [x, y1]]);
                    x += step;
                }
                let mut y = y0;
                while y <= y1 {
                    strokes.push(vec![[x0, y], [x1, y]]);
                    y += step;
                }
            }
            "timber" | "earth" | "insulation" | "concrete" => {
                let mut y = y0;
                while y <= y1 {
                    strokes.push(vec![[x0, y], [x1, y]]);
                    y += step;
                }
            }
            _ => {}
        }

        strokes
    }

    /// The whole old path in `studio_save_annotations`: bbox of the annotation points, then hatch.
    pub fn hatch(points: &[(f64, f64)], pattern: &str, scale: f64) -> Vec<Vec<[f64; 2]>> {
        let pts: Vec<StudioPointDto> = points
            .iter()
            .map(|&(x, y)| StudioPointDto { x, y })
            .collect();
        let rect = bbox_from_points_or_rect(&pts, [0.0; 4]);
        generate_hatch_strokes(rect, pattern, scale)
    }
}

fn hatch_segments(
    pattern: HatchPatternId,
    loops: &[Loop],
    scale: f64,
) -> Result<Vec<Segment>, GeomError> {
    hatch_segments_with(HatchEngine::Legacy, pattern, loops, scale)
}

const RECT: [(f64, f64); 4] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 200.0),
    (100.0, 200.0),
];
/// Concave L: same bounding box as RECT, notch removed from the top-right.
const L_POLY: [(f64, f64); 6] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 140.0),
    (150.0, 140.0),
    (150.0, 200.0),
    (100.0, 200.0),
];

fn golden_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/hatch_legacy.golden.json")
}

fn num(v: f64) -> String {
    match serde_json::to_string(&v) {
        Ok(s) => s,
        Err(e) => panic!("non-finite number in golden: {v} ({e})"),
    }
}

fn to_loop(points: &[(f64, f64)]) -> Loop {
    Loop::new(points.iter().map(|&(x, y)| PdfPt::new(x, y)).collect())
}

fn original_as_segments(strokes: &[Vec<[f64; 2]>]) -> Vec<Segment> {
    strokes
        .iter()
        .map(|s| {
            assert_eq!(s.len(), 2, "legacy strokes are two-point lines");
            Segment {
                a: PdfPt::new(s[0][0], s[0][1]),
                b: PdfPt::new(s[1][0], s[1][1]),
            }
        })
        .collect()
}

/// Even-odd point-in-polygon (test helper for the ERR-007 evidence column).
fn inside(p: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (xi, yi) = poly[i];
        let (xj, yj) = poly[(i + n - 1) % n];
        if (yi > p.1) != (yj > p.1) && p.0 < (xj - xi) * (p.1 - yi) / (yj - yi) + xi {
            c = !c;
        }
    }
    c
}

fn status(shape: &str, pattern: HatchPatternId) -> &'static str {
    match (shape, pattern) {
        ("L", _) => "known-wrong, ERR-007: bounding-box hatching, segments leave the concave polygon; patterns not distinct",
        (_, HatchPatternId::DiagonalSteel) => "current behaviour (rectangle = its own bbox, so clipping is correct here)",
        (_, HatchPatternId::Steel) => "known-wrong: identical to diagonal-steel (patterns not distinct, PROTOTYPE_SCOPE flow 5)",
        (_, HatchPatternId::Crosshatch | HatchPatternId::Tiling | HatchPatternId::Masonry) => {
            "known-wrong: crosshatch = tiling = masonry (patterns not distinct, PROTOTYPE_SCOPE flow 5)"
        }
        _ => "known-wrong: timber = earth = insulation = concrete (patterns not distinct, PROTOTYPE_SCOPE flow 5)",
    }
}

/// (name, shape, loop, pattern, scale)
type GoldenCase = (
    String,
    &'static str,
    &'static [(f64, f64)],
    HatchPatternId,
    f64,
);

fn render_golden() -> String {
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
    // Scale clamping: 0.1 -> 0.25 (step 2 -> max 3), 10 -> 4 (step 32).
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

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"provenance\": \"app-tauri/src-tauri/src/commands/studio.rs generate_hatch_strokes @ 562e4f9 (legacy, bbox hatching)\",\n");
    out.push_str("  \"segmentFormat\": \"[x1, y1, x2, y2] in PDF pt\",\n");
    out.push_str("  \"notes\": \"segmentsWithMidpointNotStrictlyInsideLoop counts midpoints outside OR on the boundary (even-odd test): on rectangles the non-zero values are boundary lines and the zero-length corner segment at d = 0 (a legacy quirk); on the L-polygon they are ERR-007.\",\n");
    out.push_str("  \"cases\": [\n");
    for (ci, (name, shape, pts, pattern, scale)) in cases.iter().enumerate() {
        let strokes = original::hatch(pts, pattern.as_id(), *scale);
        let outside = strokes
            .iter()
            .filter(|s| !inside(((s[0][0] + s[1][0]) / 2.0, (s[0][1] + s[1][1]) / 2.0), pts))
            .count();
        out.push_str("    {\n");
        out.push_str(&format!("      \"name\": \"{}\",\n", name));
        out.push_str(&format!("      \"pattern\": \"{}\",\n", pattern.as_id()));
        out.push_str(&format!("      \"scale\": {},\n", num(*scale)));
        let lp: Vec<String> = pts
            .iter()
            .map(|&(x, y)| format!("[{}, {}]", num(x), num(y)))
            .collect();
        out.push_str(&format!("      \"loop\": [{}],\n", lp.join(", ")));
        out.push_str(&format!(
            "      \"status\": \"{}\",\n",
            status(shape, *pattern)
        ));
        out.push_str(&format!("      \"segmentCount\": {},\n", strokes.len()));
        out.push_str(&format!(
            "      \"segmentsWithMidpointNotStrictlyInsideLoop\": {},\n",
            outside
        ));
        out.push_str("      \"segments\": [\n");
        for (si, s) in strokes.iter().enumerate() {
            let comma = if si + 1 < strokes.len() { "," } else { "" };
            out.push_str(&format!(
                "        [{}, {}, {}, {}]{}\n",
                num(s[0][0]),
                num(s[0][1]),
                num(s[1][0]),
                num(s[1][1]),
                comma
            ));
        }
        out.push_str("      ]\n");
        let comma = if ci + 1 < cases.len() { "," } else { "" };
        out.push_str(&format!("    }}{}\n", comma));
    }
    out.push_str("  ]\n}\n");
    out
}

/// The golden file is exactly what the original generator produces (oracle == golden).
#[test]
fn golden_file_matches_original_generator() {
    let expected = render_golden();
    let path = golden_path();
    if std::env::var_os("LPDF_UPDATE_GOLDENS").is_some() {
        if let Err(e) = std::fs::write(&path, &expected) {
            panic!("cannot write {}: {e}", path.display());
        }
    }
    let actual = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => panic!(
            "missing golden {} ({e}); run with LPDF_UPDATE_GOLDENS=1",
            path.display()
        ),
    };
    assert!(
        actual == expected,
        "hatch golden drifted from the original generator: {}",
        path.display()
    );
}

/// The adapter (legacy engine) reproduces every golden case bit-for-bit.
#[test]
fn adapter_matches_golden_cases() {
    let raw = match std::fs::read_to_string(golden_path()) {
        Ok(s) => s,
        Err(e) => panic!("missing golden ({e}); run with LPDF_UPDATE_GOLDENS=1"),
    };
    let golden: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => panic!("golden is not JSON: {e}"),
    };
    let cases = golden["cases"].as_array().cloned().unwrap_or_default();
    assert_eq!(cases.len(), 21, "9 patterns x {{rect, L}} + 3 scale cases");
    for case in cases {
        let name = case["name"].as_str().unwrap_or_default().to_string();
        let pattern = HatchPatternId::from_id(case["pattern"].as_str().unwrap_or_default());
        let pattern = match pattern {
            Some(p) => p,
            None => panic!("{name}: unknown pattern"),
        };
        let scale = case["scale"].as_f64().unwrap_or(f64::NAN);
        let pts: Vec<(f64, f64)> = case["loop"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|p| {
                (
                    p[0].as_f64().unwrap_or(f64::NAN),
                    p[1].as_f64().unwrap_or(f64::NAN),
                )
            })
            .collect();
        let segs = match hatch_segments(pattern, &[to_loop(&pts)], scale) {
            Ok(s) => s,
            Err(e) => panic!("{name}: {e}"),
        };
        let got: Vec<String> = segs
            .iter()
            .map(|s| {
                format!(
                    "[{}, {}, {}, {}]",
                    num(s.a.x),
                    num(s.a.y),
                    num(s.b.x),
                    num(s.b.y)
                )
            })
            .collect();
        let want: Vec<String> = case["segments"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .map(|s| {
                format!(
                    "[{}, {}, {}, {}]",
                    num(s[0].as_f64().unwrap_or(f64::NAN)),
                    num(s[1].as_f64().unwrap_or(f64::NAN)),
                    num(s[2].as_f64().unwrap_or(f64::NAN)),
                    num(s[3].as_f64().unwrap_or(f64::NAN))
                )
            })
            .collect();
        assert_eq!(got, want, "{name}: adapter output differs from golden");
    }
}

/// Parity with the original on many deterministic pseudo-random regions, scales and patterns
/// (including far-from-origin, degenerate and non-finite vertices), compared bit-for-bit.
#[test]
fn adapter_matches_original_on_random_regions() {
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    for case in 0..300 {
        let n = 1 + (next() * 9.0) as usize;
        let base = if case % 7 == 0 { 1.0e6 } else { 0.0 };
        let mut pts: Vec<(f64, f64)> = (0..n)
            .map(|_| (base + next() * 600.0, base + next() * 800.0))
            .collect();
        if case % 11 == 0 {
            pts.push((f64::NAN, 10.0));
        }
        if case % 13 == 0 {
            pts.push((f64::INFINITY, f64::NEG_INFINITY));
        }
        let scale = match case % 5 {
            0 => 1.0,
            1 => next() * 6.0,
            2 => 0.0,
            3 => -2.0,
            _ => 0.25 + next(),
        };
        for p in HatchPatternId::ALL {
            let want = original_as_segments(&original::hatch(&pts, p.as_id(), scale));
            let got = match hatch_segments_with(HatchEngine::Legacy, p, &[to_loop(&pts)], scale) {
                Ok(s) => s,
                Err(e) => panic!("case {case}: {e}"),
            };
            assert_eq!(
                got.len(),
                want.len(),
                "case {case} {}: segment count",
                p.as_id()
            );
            for (g, w) in got.iter().zip(want.iter()) {
                for (a, b) in [
                    (g.a.x, w.a.x),
                    (g.a.y, w.a.y),
                    (g.b.x, w.b.x),
                    (g.b.y, w.b.y),
                ] {
                    assert_eq!(
                        a.to_bits(),
                        b.to_bits(),
                        "case {case} {}: {a} vs {b}",
                        p.as_id()
                    );
                }
            }
        }
    }
}

#[test]
fn ids_round_trip_and_fills_have_no_line_hatch() {
    for p in HatchPatternId::ALL {
        assert_eq!(HatchPatternId::from_id(p.as_id()), Some(p));
    }
    assert_eq!(HatchPatternId::from_id("solid"), None);
    assert_eq!(HatchPatternId::from_id("none"), None);
    assert_eq!(HatchPatternId::from_id("ANSI31"), None);
}

#[test]
fn no_finite_geometry_is_a_typed_error() {
    let err = hatch_segments(HatchPatternId::Crosshatch, &[], 1.0);
    assert_eq!(err, Err(lpdf_geom::GeomError::NoFiniteGeometry));
    let nan_loop = Loop::new(vec![
        PdfPt::new(f64::NAN, 1.0),
        PdfPt::new(2.0, f64::INFINITY),
    ]);
    let err = hatch_segments(HatchPatternId::Crosshatch, &[nan_loop], 1.0);
    assert!(matches!(err, Err(ref e) if e.code() == "GEOM_NO_FINITE_GEOMETRY"));
}

#[test]
fn loop_from_rect_has_the_rect_bbox() {
    let segs = hatch_segments(
        HatchPatternId::Timber,
        &[Loop::from_rect([100.0, 100.0, 240.0, 200.0])],
        1.0,
    );
    let want = original_as_segments(&original::hatch(&RECT, "timber", 1.0));
    assert_eq!(segs, Ok(want));
}

/// Kept defect, now guarded: a huge extent used to give an unbounded segment count.
#[test]
fn huge_extent_reports_the_work_limit() {
    let huge = Loop::from_rect([0.0, 0.0, 1.0e9, 1.0e9]);
    for p in HatchPatternId::ALL {
        let got = hatch_segments(p, std::slice::from_ref(&huge), 1.0);
        assert_eq!(
            got,
            Err(lpdf_geom::GeomError::WorkLimitExceeded {
                limit: lpdf_geom::MAX_LEGACY_SEGMENTS
            }),
            "{}",
            p.as_id()
        );
    }
    let err = hatch_segments(HatchPatternId::Timber, &[huge], 1.0).err();
    assert_eq!(err.map(|e| e.code()), Some("GEOM_WORK_LIMIT"));
}

/// Kept defect, now guarded: far from the origin `y + step == y`, so the original never ended.
#[test]
fn non_advancing_step_reports_the_work_limit_instead_of_hanging() {
    let far = Loop::from_rect([1.0e17, 1.0e17, 1.0e17 + 32.0, 1.0e17 + 32.0]);
    // The diagonal families step `d` from 0, so they advance; the grid and horizontal families
    // step from x0/y0 = 1e17, where 3 pt is below half an ulp (8).
    for p in HatchPatternId::ALL
        .into_iter()
        .filter(|p| !matches!(p, HatchPatternId::DiagonalSteel | HatchPatternId::Steel))
    {
        assert!(
            matches!(
                hatch_segments(p, std::slice::from_ref(&far), 1.0),
                Err(lpdf_geom::GeomError::WorkLimitExceeded { .. })
            ),
            "{}",
            p.as_id()
        );
    }
}

/// The guard does not change output below it: a region needing close to the limit still matches
/// the original exactly.
#[test]
fn output_just_below_the_limit_is_unchanged() {
    // timber, step 3 (scale 0.25): ceil(h / 3) lines; h chosen to give 99_999 lines.
    let h = 3.0 * 99_998.0;
    let pts = [(0.0, 0.0), (10.0, 0.0), (10.0, h), (0.0, h)];
    let want = original_as_segments(&original::hatch(&pts, "timber", 0.25));
    assert_eq!(want.len(), 99_999);
    assert_eq!(
        hatch_segments(HatchPatternId::Timber, &[to_loop(&pts)], 0.25),
        Ok(want)
    );
}
