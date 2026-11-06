//! Goldens and consistency checks for the Studio hatch engine (ERR-007 fix).
//!
//! - `tests/goldens/hatch_studio.golden.json`: output for rectangle / concave L / star, every
//!   pattern. ERR-007 intentional change, approved by owner 2026-11-06 (the legacy golden
//!   `hatch_legacy.golden.json` still pins the old generator; this one pins the replacement).
//! - `app-tauri/src/lib/core/hatchPatterns.json`: the pattern parameters and one clipped tile per
//!   pattern, generated from the Rust definitions. The on-screen preview (`HatchDefs.svelte`) draws
//!   exactly this file, so a mismatch fails here, not on a user's screen.
//!
//! Regenerate both (only for an intended, explained change): `LPDF_UPDATE_GOLDENS=1 cargo nextest run -p lpdf-geom`.

use lpdf_geom::{
    hatch_segments, pattern_families, pattern_tile, HatchPatternId, Loop, PdfPt, Segment,
    SCALE_MAX, SCALE_MIN, TILE_CELLS, UNIT_BASE_PT, UNIT_MIN_PT,
};
use std::path::PathBuf;

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

fn star() -> Vec<Pt> {
    // Whole-number vertices: the golden is parsed back by app-tauri's studio.rs test, and serde_json
    // (no float_roundtrip) can mis-parse long decimals by one ulp.
    (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { 90.0 } else { 38.0 };
            let a = std::f64::consts::FRAC_PI_2 + f64::from(i) * std::f64::consts::PI / 5.0;
            ((300.0 + r * a.cos()).round(), (300.0 + r * a.sin()).round())
        })
        .collect()
}

fn num(v: f64) -> String {
    match serde_json::to_string(&v) {
        Ok(s) => s,
        Err(e) => panic!("non-finite number in golden: {v} ({e})"),
    }
}

fn to_loop(pts: &[Pt]) -> Loop {
    Loop::new(pts.iter().map(|&(x, y)| PdfPt::new(x, y)).collect())
}

fn check_or_update(path: &PathBuf, expected: &str) {
    if std::env::var_os("LPDF_UPDATE_GOLDENS").is_some() {
        if let Err(e) = std::fs::write(path, expected) {
            panic!("cannot write {}: {e}", path.display());
        }
    }
    let actual = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => panic!(
            "missing {} ({e}); run with LPDF_UPDATE_GOLDENS=1",
            path.display()
        ),
    };
    assert!(
        actual == expected,
        "{} drifted from the Rust definitions; rerun with LPDF_UPDATE_GOLDENS=1 if the change is intended",
        path.display()
    );
}

fn seg_json(s: &Segment) -> String {
    format!(
        "[{}, {}, {}, {}]",
        num(s.a.x),
        num(s.a.y),
        num(s.b.x),
        num(s.b.y)
    )
}

#[test]
fn studio_golden_with_owner_approval_note() {
    let star = star();
    let mut cases: Vec<(String, Vec<Pt>, HatchPatternId, f64)> = Vec::new();
    for (shape, pts) in [
        ("rect", RECT.to_vec()),
        ("L", L_POLY.to_vec()),
        ("star", star),
    ] {
        for p in HatchPatternId::ALL {
            cases.push((
                format!("{shape}/{}/scale-1", p.as_id()),
                pts.clone(),
                p,
                1.0,
            ));
        }
    }
    cases.push((
        "rect/diagonal-steel/scale-0.1".into(),
        RECT.to_vec(),
        HatchPatternId::DiagonalSteel,
        0.1,
    ));
    cases.push((
        "rect/diagonal-steel/scale-10".into(),
        RECT.to_vec(),
        HatchPatternId::DiagonalSteel,
        10.0,
    ));
    cases.push((
        "rect/crosshatch/scale-2.5".into(),
        RECT.to_vec(),
        HatchPatternId::Crosshatch,
        2.5,
    ));

    let mut out = String::from("{\n");
    out.push_str("  \"provenance\": \"lpdf-geom HatchEngine::Studio (own pattern families + even-odd clipping)\",\n");
    out.push_str("  \"note\": \"ERR-007 intentional change, approved by owner 2026-11-06. Replaces hatch_legacy.golden.json as what Studio save writes: every segment is clipped to the real loop (even-odd) and each pattern id has its own families. The legacy golden still pins the old bounding-box generator.\",\n");
    out.push_str("  \"segmentFormat\": \"[x1, y1, x2, y2] in PDF pt\",\n  \"cases\": [\n");
    for (ci, (name, pts, p, scale)) in cases.iter().enumerate() {
        let segs = match hatch_segments(*p, &[to_loop(pts)], *scale) {
            Ok(s) => s,
            Err(e) => panic!("{name}: {e}"),
        };
        let lp: Vec<String> = pts
            .iter()
            .map(|&(x, y)| format!("[{}, {}]", num(x), num(y)))
            .collect();
        out.push_str(&format!(
            "    {{\n      \"name\": \"{name}\",\n      \"pattern\": \"{}\",\n      \"scale\": {},\n      \"loop\": [{}],\n      \"segmentCount\": {},\n      \"segments\": [\n",
            p.as_id(),
            num(*scale),
            lp.join(", "),
            segs.len()
        ));
        for (si, s) in segs.iter().enumerate() {
            let comma = if si + 1 < segs.len() { "," } else { "" };
            out.push_str(&format!("        {}{comma}\n", seg_json(s)));
        }
        let comma = if ci + 1 < cases.len() { "," } else { "" };
        out.push_str(&format!("      ]\n    }}{comma}\n"));
    }
    out.push_str("  ]\n}\n");
    check_or_update(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/goldens/hatch_studio.golden.json"),
        &out,
    );
}

fn preview_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"provenance\": \"GENERATED by crates/lpdf-geom/tests/studio_golden.rs from lpdf-geom pattern definitions (own work). Do not edit; run LPDF_UPDATE_GOLDENS=1 cargo nextest run -p lpdf-geom.\",\n");
    out.push_str(&format!(
        "  \"unit\": {{ \"basePt\": {}, \"minPt\": {}, \"scaleMin\": {}, \"scaleMax\": {} }},\n",
        num(UNIT_BASE_PT),
        num(UNIT_MIN_PT),
        num(SCALE_MIN),
        num(SCALE_MAX)
    ));
    out.push_str(&format!(
        "  \"tileCells\": {},\n  \"patterns\": [\n",
        num(TILE_CELLS)
    ));
    for (pi, p) in HatchPatternId::ALL.iter().enumerate() {
        let fams: Vec<String> = pattern_families(*p)
            .iter()
            .map(|f| {
                let dash: Vec<String> = f.dash.iter().map(|d| num(*d)).collect();
                format!(
                    "{{ \"angleDeg\": {}, \"pitch\": {}, \"offset\": {}, \"phase\": {}, \"dash\": [{}] }}",
                    num(f.angle_deg),
                    num(f.pitch),
                    num(f.offset),
                    num(f.phase),
                    dash.join(", ")
                )
            })
            .collect();
        let tile = match pattern_tile(*p) {
            Ok(t) => t,
            Err(e) => panic!("{}: {e}", p.as_id()),
        };
        out.push_str(&format!(
            "    {{\n      \"id\": \"{}\",\n      \"families\": [{}],\n      \"tile\": [\n",
            p.as_id(),
            fams.join(", ")
        ));
        for (si, s) in tile.iter().enumerate() {
            let comma = if si + 1 < tile.len() { "," } else { "" };
            out.push_str(&format!("        {}{comma}\n", seg_json(s)));
        }
        let comma = if pi + 1 < HatchPatternId::ALL.len() {
            ","
        } else {
            ""
        };
        out.push_str(&format!("      ]\n    }}{comma}\n"));
    }
    out.push_str("  ]\n}\n");
    out
}

/// The preview file the UI reads is exactly what the Rust definitions produce.
#[test]
fn preview_parameters_file_matches_the_definitions() {
    check_or_update(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../app-tauri/src/lib/core/hatchPatterns.json"),
        &preview_json(),
    );
}

/// Merge collinear, touching segments so a tile boundary split does not count as a difference.
fn merged(segs: &[Segment]) -> Vec<(f64, f64, f64, f64)> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<(i64, i64), Vec<(f64, f64, f64, f64)>> = BTreeMap::new();
    for s in segs {
        let (a, b) = if (s.a.x, s.a.y) <= (s.b.x, s.b.y) {
            (s.a, s.b)
        } else {
            (s.b, s.a)
        };
        let key = if (b.x - a.x).abs() < 1e-9 {
            (i64::MAX, (a.x * 1e5).round() as i64)
        } else {
            let m = (b.y - a.y) / (b.x - a.x);
            (
                (m * 1e5).round() as i64,
                ((a.y - m * a.x) * 1e5).round() as i64,
            )
        };
        groups.entry(key).or_default().push((a.x, a.y, b.x, b.y));
    }
    let mut out = Vec::new();
    for mut g in groups.into_values() {
        g.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
        let mut cur = g[0];
        for s in g.into_iter().skip(1) {
            if (s.0, s.1) <= (cur.2 + 1e-6, f64::INFINITY)
                && s.0 <= cur.2 + 1e-6
                && (s.1 - cur.3).abs() < 1e-4 + (s.0 - cur.2).abs() * 2.0
            {
                if (s.2, s.3) > (cur.2, cur.3) {
                    cur.2 = s.2;
                    cur.3 = s.3;
                }
            } else {
                out.push(cur);
                cur = s;
            }
        }
        out.push(cur);
    }
    out.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// What the preview draws (the tile, repeated) equals what save writes (the real clipper over a
/// region of the same size): the tile repeated 2 x 2 at scale 1 (8 pt cells) is the hatch of the
/// 64 pt square, once collinear pieces are joined.
#[test]
fn tile_repeated_equals_hatching_the_region() {
    let cell = UNIT_BASE_PT; // scale 1
    let side = TILE_CELLS * cell;
    let region = to_loop(&[
        (0.0, 0.0),
        (2.0 * side, 0.0),
        (2.0 * side, 2.0 * side),
        (0.0, 2.0 * side),
    ]);
    for p in HatchPatternId::ALL {
        let tile = match pattern_tile(p) {
            Ok(t) => t,
            Err(e) => panic!("{}: {e}", p.as_id()),
        };
        assert!(!tile.is_empty(), "{}: empty tile", p.as_id());
        let mut repeated = Vec::new();
        for (ix, iy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
            for s in &tile {
                repeated.push(Segment {
                    a: PdfPt::new(s.a.x * cell + ix * side, s.a.y * cell + iy * side),
                    b: PdfPt::new(s.b.x * cell + ix * side, s.b.y * cell + iy * side),
                });
            }
        }
        let whole = match hatch_segments(p, &[region.clone()], 1.0) {
            Ok(s) => s,
            Err(e) => panic!("{}: {e}", p.as_id()),
        };
        let (a, b) = (merged(&repeated), merged(&whole));
        assert_eq!(
            a.len(),
            b.len(),
            "{}: {} vs {} pieces",
            p.as_id(),
            a.len(),
            b.len()
        );
        for (x, y) in a.iter().zip(b.iter()) {
            assert!(
                (x.0 - y.0).abs() < 1e-6
                    && (x.1 - y.1).abs() < 1e-6
                    && (x.2 - y.2).abs() < 1e-6
                    && (x.3 - y.3).abs() < 1e-6,
                "{}: {x:?} vs {y:?}",
                p.as_id()
            );
        }
    }
}

#[test]
fn every_pattern_has_its_own_families_and_tile() {
    let ids = HatchPatternId::ALL;
    for (i, a) in ids.iter().enumerate() {
        for b in ids.iter().skip(i + 1) {
            assert_ne!(
                pattern_families(*a),
                pattern_families(*b),
                "{} = {}",
                a.as_id(),
                b.as_id()
            );
            assert_ne!(
                pattern_tile(*a),
                pattern_tile(*b),
                "{} = {} (tiles)",
                a.as_id(),
                b.as_id()
            );
        }
    }
    // Every parameter divides the tile, so the pattern really repeats every TILE_CELLS.
    for p in ids {
        for f in pattern_families(p) {
            for v in [f.pitch, f.offset, f.phase]
                .into_iter()
                .chain(f.dash.iter().copied())
            {
                assert!(
                    (v * 4.0).fract() == 0.0,
                    "{}: {v} is not a quarter cell",
                    p.as_id()
                );
            }
            let period: f64 = f.dash.iter().sum();
            if period > 0.0 {
                assert!(
                    (TILE_CELLS / period).fract() == 0.0,
                    "{}: dash period {period}",
                    p.as_id()
                );
            }
            assert!(
                (TILE_CELLS / f.pitch).fract() == 0.0,
                "{}: pitch {}",
                p.as_id(),
                f.pitch
            );
        }
    }
}
