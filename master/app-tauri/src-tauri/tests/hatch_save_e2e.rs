//! ERR-007 / BUG-015 end to end: save a hatched concave polygon to a NEW PDF (flattened, so the
//! hatch lands in the page content stream), read the segment coordinates back out of that stream,
//! and check they lie inside the polygon and are exactly what the hatch engine produced.

use app_tauri_lib::commands::studio::{studio_extract_snap_geometry, studio_save_annotations};
use std::fs;
use std::path::{Path, PathBuf};

type Pt = (f64, f64);

/// Concave L, same as the lpdf-geom tests.
const L_POLY: [Pt; 6] = [
    (100.0, 100.0),
    (240.0, 100.0),
    (240.0, 140.0),
    (150.0, 140.0),
    (150.0, 200.0),
    (100.0, 200.0),
];

const SAMPLE_PDF: &[u8] = b"%PDF-1.4\n\
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n\
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >> endobj\n\
4 0 obj << /Length 44 >> stream\n\
BT /F1 12 Tf 72 720 Td (Original PDF Text) Tj ET\n\
endstream endobj\n\
5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n\
xref\n\
0 6\n\
0000000000 65535 f \n\
0000000009 00000 n \n\
0000000058 00000 n \n\
0000000115 00000 n \n\
0000000241 00000 n \n\
0000000335 00000 n \n\
trailer << /Size 6 /Root 1 0 R >>\n\
startxref\n\
403\n\
%%EOF\n";

fn inside_or_on(p: Pt, poly: &[Pt], tol: f64) -> bool {
    let n = poly.len();
    let mut c = false;
    let mut near = f64::INFINITY;
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
        let (vx, vy) = (b.0 - a.0, b.1 - a.1);
        let t = (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / (vx * vx + vy * vy)).clamp(0.0, 1.0);
        near = near.min((p.0 - (a.0 + t * vx)).hypot(p.1 - (a.1 + t * vy)));
    }
    c || near <= tol
}

fn annotation(pattern: &str) -> serde_json::Value {
    let points: Vec<_> = L_POLY
        .iter()
        .map(|&(x, y)| serde_json::json!({ "x": x, "y": y, "p": 0.5 }))
        .collect();
    serde_json::json!({
        "id": format!("hatch-{pattern}"),
        "kind": "hatch.region",
        "pageIndex": 0,
        "points": points,
        "bboxPt": [100.0, 100.0, 240.0, 200.0],
        "colorRgb": [20, 20, 20],
        "widthPt": 1.0,
        "opacity": 1.0,
        "hatchPattern": pattern,
        "hatchScale": 1.0,
    })
}

struct TmpDir(PathBuf);
impl Drop for TmpDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn near(a: [f64; 2], b: Pt, tol: f64) -> bool {
    (a[0] - b.0).abs() <= tol && (a[1] - b.1).abs() <= tol
}

#[test]
fn saved_hatch_of_a_concave_polygon_stays_inside_in_the_new_pdf() {
    let dir = TmpDir(std::env::temp_dir().join(format!("lpdf_hatch_e2e_{}", std::process::id())));
    assert!(fs::create_dir_all(&dir.0).is_ok());
    let input = dir.0.join("input.pdf");
    assert!(fs::write(&input, SAMPLE_PDF).is_ok());

    for pattern in lpdf_geom::HatchPatternId::ALL.map(lpdf_geom::HatchPatternId::as_id) {
        let output: PathBuf = dir.0.join(format!("out-{pattern}.pdf"));
        let annots = match serde_json::from_value(serde_json::json!([annotation(pattern)])) {
            Ok(a) => a,
            Err(e) => panic!("annotation DTO: {e}"),
        };
        let res = studio_save_annotations(
            input.to_string_lossy().to_string(),
            output.to_string_lossy().to_string(),
            annots,
            true,
        );
        assert!(
            res.as_ref().is_ok_and(|r| r.flattened),
            "{pattern}: {res:?}"
        );
        assert!(
            output.exists() && output != input,
            "{pattern}: not a new file"
        );
        assert_eq!(
            fs::read(&input).ok().as_deref(),
            Some(SAMPLE_PDF),
            "original modified"
        );

        // Re-extract the line segments from the saved page's content stream.
        let geom = match studio_extract_snap_geometry(output.to_string_lossy().to_string(), 0) {
            Ok(g) => g,
            Err(e) => panic!("{pattern}: extract: {e}"),
        };
        // The first four are the page border the extractor always adds.
        let drawn: Vec<_> = geom.segments.iter().skip(4).collect();
        assert!(
            drawn.len() > 8,
            "{pattern}: only {} segments in the content",
            drawn.len()
        );
        let tol = 0.05; // content streams round coordinates
        for s in &drawn {
            for p in [
                (s.x1, s.y1),
                (s.x2, s.y2),
                ((s.x1 + s.x2) / 2.0, (s.y1 + s.y2) / 2.0),
            ] {
                assert!(
                    inside_or_on(p, &L_POLY, tol),
                    "{pattern}: {p:?} of {s:?} is outside the polygon (ERR-007)"
                );
            }
        }

        // And it is exactly the engine's hatch, not just something inside.
        let loop_ = lpdf_geom::Loop::new(
            L_POLY
                .iter()
                .map(|&(x, y)| lpdf_geom::PdfPt::new(x, y))
                .collect(),
        );
        let want = match lpdf_geom::hatch_segments(
            lpdf_geom::HatchPatternId::from_id(pattern)
                .unwrap_or(lpdf_geom::HatchPatternId::Timber),
            &[loop_],
            1.0,
        ) {
            Ok(w) => w,
            Err(e) => panic!("{pattern}: {e}"),
        };
        for w in want {
            let found = drawn.iter().any(|s| {
                (near([s.x1, s.y1], (w.a.x, w.a.y), tol) && near([s.x2, s.y2], (w.b.x, w.b.y), tol))
                    || (near([s.x1, s.y1], (w.b.x, w.b.y), tol)
                        && near([s.x2, s.y2], (w.a.x, w.a.y), tol))
            });
            assert!(
                found,
                "{pattern}: engine segment {w:?} is not in the saved content"
            );
        }
        let _ = Path::new(&output);
    }
}
