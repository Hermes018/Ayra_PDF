use directories::ProjectDirs;
use lpdf_control_bus::ControlMessage;
use lpdf_fs_safety::atomic_write;
use pdfcraft_engine::{
    block_overlapping, rect_quad, AddedText, BlockStyle, Edit, FontFamily, LineEnding, Markup,
    NewAnnotation, OverlayFont, OverlayLook, Session, Shape, Style, TextAlign,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Studio UI preferences persisted atomically to disk (`ADR-012` Phase D1/D2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioPrefs {
    pub dock_edge: String,
    pub float_x: f64,
    pub float_y: f64,
    pub shortcuts: std::collections::BTreeMap<String, String>,
    pub favorites: Vec<String>,
    pub history_limit: usize,
}

impl Default for StudioPrefs {
    fn default() -> Self {
        Self {
            dock_edge: "left".to_string(),
            float_x: 16.0,
            float_y: 80.0,
            shortcuts: std::collections::BTreeMap::new(),
            favorites: vec![
                "preset.pen.a".to_string(),
                "preset.highlighter".to_string(),
                "preset.eraser.partial".to_string(),
            ],
            history_limit: 500,
        }
    }
}

/// A single point in PDF user-space coordinates (`pt`), with optional stylus pressure & timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioPointDto {
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_pressure")]
    pub p: f64,
    #[serde(default)]
    pub t: Option<f64>,
}

fn default_pressure() -> f64 {
    0.5
}

fn default_opacity() -> f64 {
    1.0
}

fn default_width() -> f64 {
    1.5
}

/// Serializable annotation DTO passed between the Svelte 5 studio canvas and the Rust `pdfcraft` engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioAnnotationDto {
    pub id: String,
    /// Kind of annotation: "ink" | "line" | "arrow" | "polyline" | "rect" | "ellipse" | "polygon"
    /// | "cloud" | "text" | "highlight" | "underline" | "squiggle" | "strikethrough" | "redact" | "snapshot"
    pub kind: String,
    pub page_index: u32,
    #[serde(default)]
    pub points: Vec<StudioPointDto>,
    /// Bounding box in PDF user-space coordinates `[x0, y0, x1, y1]` (pt).
    #[serde(default)]
    pub bbox_pt: [f64; 4],
    #[serde(default)]
    pub color_rgb: [u8; 3],
    #[serde(default)]
    pub fill_rgb: Option<[u8; 3]>,
    #[serde(default = "default_width")]
    pub width_pt: f64,
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    #[serde(default)]
    pub line_style: Option<String>,
    #[serde(default)]
    pub start_ending: Option<String>,
    #[serde(default)]
    pub end_ending: Option<String>,
    #[serde(default)]
    pub hatch_pattern: Option<String>,
    #[serde(default)]
    pub hatch_scale: Option<f64>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub font_size: Option<f64>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub rotation_deg: Option<f64>,
    #[serde(default)]
    pub snapshot_data_url: Option<String>,
}

/// Result returned by studio persistence and structural edit commands.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioSaveResult {
    pub output_path: String,
    pub saved_count: usize,
    pub flattened: bool,
    pub page_count: usize,
}

fn studio_prefs_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "localpdf", "studio") {
        let config_dir = proj_dirs.config_dir();
        let _ = fs::create_dir_all(config_dir);
        config_dir.join("studio_prefs.json")
    } else {
        PathBuf::from("studio_prefs.json")
    }
}

fn studio_sidecar_path(pdf_path: &Path) -> PathBuf {
    let mut os = pdf_path.as_os_str().to_os_string();
    os.push(".studio.json");
    PathBuf::from(os)
}

fn parse_line_ending(raw: Option<&str>) -> LineEnding {
    match raw.unwrap_or("none") {
        "open-arrow" | "open" => LineEnding::OpenArrow,
        "closed-arrow" | "closed" => LineEnding::ClosedArrow,
        "circle" => LineEnding::Circle,
        "square" => LineEnding::Square,
        "diamond" => LineEnding::Diamond,
        "slash" => LineEnding::Slash,
        "butt" => LineEnding::Butt,
        _ => LineEnding::None,
    }
}

fn normalize_rect(r: [f64; 4]) -> [f64; 4] {
    let x0 = r[0].min(r[2]);
    let y0 = r[1].min(r[3]);
    let x1 = r[0].max(r[2]).max(x0 + 1.0);
    let y1 = r[1].max(r[3]).max(y0 + 1.0);
    [x0, y0, x1, y1]
}

fn bbox_from_points_or_rect(points: &[StudioPointDto], fallback: [f64; 4]) -> [f64; 4] {
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

fn rgb_to_f64(rgb: [u8; 3]) -> [f64; 3] {
    [
        f64::from(rgb[0]) / 255.0,
        f64::from(rgb[1]) / 255.0,
        f64::from(rgb[2]) / 255.0,
    ]
}

/// Generate procedural hatch lines inside `[x0, y0, x1, y1]` so flattened/standard PDF annotations
/// preserve procedural hatch appearances inside PDF viewers.
fn generate_hatch_strokes(
    rect: [f64; 4],
    pattern: &str,
    scale: f64,
) -> Vec<Vec<[f64; 2]>> {
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

fn verify_written_pdf(path: &Path) -> Result<usize, String> {
    let written = fs::read(path)
        .map_err(|e| format!("Verify-after-write failed reading {:?}: {}", path, e))?;
    if written.len() < 8 {
        return Err("Verify-after-write failed: output file is truncated".to_string());
    }
    let mut verify_session = Session::new();
    let doc_id = verify_session
        .open(
            "verify.pdf".to_string(),
            None,
            Arc::new(written),
            None,
        )
        .map_err(|e| format!("Verify-after-write failed parsing PDF: {}", e))?;
    let page_count = verify_session
        .get(doc_id)
        .map(|d| d.info.pages.len())
        .unwrap_or(0);
    if page_count == 0 {
        return Err("Verify-after-write failed: output PDF has 0 pages".to_string());
    }
    Ok(page_count)
}

#[tauri::command]
pub fn prefs_load() -> Result<StudioPrefs, String> {
    let path = studio_prefs_path();
    if !path.exists() {
        return Ok(StudioPrefs::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read studio_prefs.json: {}", e))?;
    serde_json::from_str::<StudioPrefs>(&content)
        .map_err(|e| format!("Failed to parse studio_prefs.json: {}", e))
}

#[tauri::command]
pub fn prefs_save(prefs: StudioPrefs) -> Result<(), String> {
    let path = studio_prefs_path();
    let bytes = serde_json::to_vec_pretty(&prefs)
        .map_err(|e| format!("Failed to serialize studio preferences: {}", e))?;
    atomic_write(&path, &bytes)
        .map_err(|e| format!("Failed to atomically write studio_prefs.json: {}", e))
}

#[tauri::command]
pub fn control_validate(msg: ControlMessage) -> Result<bool, String> {
    msg.validate().map(|()| true).map_err(|e| e.to_string())
}

/// Persist studio annotations (ink, shapes, polylines, clouds, hatch regions, text boxes,
/// review markups, and cryptographic redactions) onto the target PDF via `pdfcraft`.
#[tauri::command]
pub fn studio_save_annotations(
    input_path: String,
    output_path: String,
    annotations: Vec<StudioAnnotationDto>,
    flatten: bool,
) -> Result<StudioSaveResult, String> {
    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);

    let raw_bytes = fs::read(&in_path)
        .map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
    let doc_name = in_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut session = Session::new();
    let doc_id = session
        .open(
            doc_name,
            Some(input_path.clone()),
            Arc::new(raw_bytes),
            None,
        )
        .map_err(|e| format!("Failed to open PDF in pdfcraft session: {}", e))?;

    let page_total = session
        .get(doc_id)
        .map(|d| d.info.pages.len())
        .unwrap_or(1)
        .max(1);

    let mut has_redactions = false;
    let mut saved_count = 0usize;

    for annot in &annotations {
        let page = (annot.page_index as usize).min(page_total.saturating_sub(1));
        let color = rgb_to_f64(annot.color_rgb);
        let fill = annot.fill_rgb.map(rgb_to_f64);
        let width = annot.width_pt.clamp(0.25, 72.0);
        let opacity = annot.opacity.clamp(0.05, 1.0);
        let rect = bbox_from_points_or_rect(&annot.points, annot.bbox_pt);
        let start_end = parse_line_ending(annot.start_ending.as_deref());
        let end_end = parse_line_ending(annot.end_ending.as_deref());

        let kind = annot.kind.as_str();
        let shape_opt: Option<Shape> = match kind {
            "ink" | "pen" | "pen.a" | "pen.b" | "highlighter" => {
                let pts: Vec<[f64; 2]> = if annot.points.len() >= 2 {
                    annot.points.iter().map(|p| [p.x, p.y]).collect()
                } else {
                    vec![[rect[0], rect[1]], [rect[2], rect[3]]]
                };
                Some(Shape::Ink { strokes: vec![pts] })
            }
            "line" | "shape.line" | "arrow" | "shape.arrow" => {
                let (from, to) = if annot.points.len() >= 2 {
                    let first = &annot.points[0];
                    let last = &annot.points[annot.points.len() - 1];
                    ([first.x, first.y], [last.x, last.y])
                } else {
                    ([rect[0], rect[1]], [rect[2], rect[3]])
                };
                let effective_end = if (kind == "arrow" || kind == "shape.arrow")
                    && matches!(end_end, LineEnding::None)
                {
                    LineEnding::OpenArrow
                } else {
                    end_end
                };
                Some(Shape::Line {
                    from,
                    to,
                    start: start_end,
                    end: effective_end,
                })
            }
            "polyline" | "shape.polyline" => {
                let vertices: Vec<[f64; 2]> = if annot.points.len() >= 2 {
                    annot.points.iter().map(|p| [p.x, p.y]).collect()
                } else {
                    vec![[rect[0], rect[1]], [rect[2], rect[3]]]
                };
                Some(Shape::PolyLine {
                    vertices,
                    start: start_end,
                    end: end_end,
                })
            }
            "rect" | "shape.rect" | "snapshot" | "select.snapshot" => {
                Some(Shape::Rectangle { rect })
            }
            "ellipse" | "shape.ellipse" => Some(Shape::Oval { rect }),
            "polygon" | "shape.polygon" | "cloud" | "shape.cloud" | "hatch.region" => {
                let vertices: Vec<[f64; 2]> = if annot.points.len() >= 3 {
                    annot.points.iter().map(|p| [p.x, p.y]).collect()
                } else {
                    vec![
                        [rect[0], rect[1]],
                        [rect[2], rect[1]],
                        [rect[2], rect[3]],
                        [rect[0], rect[3]],
                    ]
                };
                let cloud = kind == "cloud" || kind == "shape.cloud";
                Some(Shape::Polygon { vertices, cloud })
            }
            "text" | "insert.text" | "insert.textBox" => {
                let font_size = annot.font_size.unwrap_or(12.0).clamp(6.0, 96.0);
                Some(Shape::TextBox { rect, font_size })
            }
            "highlight" | "review.highlight" => Some(Shape::TextMarkup {
                kind: Markup::Highlight,
                quads: vec![rect_quad(rect)],
            }),
            "underline" | "review.underline" => Some(Shape::TextMarkup {
                kind: Markup::Underline,
                quads: vec![rect_quad(rect)],
            }),
            "squiggle" | "review.squiggle" | "review.squiggly" => Some(Shape::TextMarkup {
                kind: Markup::Squiggly,
                quads: vec![rect_quad(rect)],
            }),
            "strikethrough" | "review.strikethrough" | "review.strikeout" => {
                Some(Shape::TextMarkup {
                    kind: Markup::StrikeOut,
                    quads: vec![rect_quad(rect)],
                })
            }
            "redact" | "review.redact" => {
                has_redactions = true;
                Some(Shape::Redact {
                    quads: vec![rect_quad(rect)],
                    overlay: annot.text.clone().unwrap_or_default(),
                    look: OverlayLook {
                        font: OverlayFont::Helvetica,
                        size: 10.0,
                        color: [1.0, 1.0, 1.0],
                        align: 1,
                        repeat: false,
                    },
                })
            }
            _ => None,
        };

        if let Some(shape) = shape_opt {
            let style = Style {
                color,
                opacity,
                width,
                fill,
            };
            let contents = annot.text.clone().unwrap_or_default();
            let new_annot = NewAnnotation {
                page,
                shape,
                style,
                contents,
                author: "LocalPDF Studio".to_string(),
            };
            session
                .apply(doc_id, Edit::AddAnnotation(new_annot))
                .map_err(|e| format!("Failed to apply annotation '{}': {}", annot.id, e))?;
            saved_count += 1;

            // If a procedural hatch pattern is configured, also emit hatch line strokes so
            // standard PDF viewers and flattened streams display the architectural hatching.
            if let Some(ref pat) = annot.hatch_pattern {
                if pat != "none" && pat != "solid" {
                    let hatch_strokes =
                        generate_hatch_strokes(rect, pat, annot.hatch_scale.unwrap_or(1.0));
                    if !hatch_strokes.is_empty() {
                        let hatch_annot = NewAnnotation {
                            page,
                            shape: Shape::Ink {
                                strokes: hatch_strokes,
                            },
                            style: Style {
                                color,
                                opacity: (opacity * 0.65).clamp(0.1, 1.0),
                                width: 0.6,
                                fill: None,
                            },
                            contents: String::new(),
                            author: "LocalPDF Studio Hatch".to_string(),
                        };
                        let _ = session.apply(doc_id, Edit::AddAnnotation(hatch_annot));
                    }
                }
            }
        }
    }

    // Execute true cryptographic stream sanitization if any redaction annotations were committed.
    if has_redactions {
        session
            .apply(doc_id, Edit::ApplyRedactions { pages: None })
            .map_err(|e| format!("Failed to apply cryptographic redaction sanitization: {}", e))?;
    }

    if flatten {
        session
            .apply(
                doc_id,
                Edit::Flatten {
                    comments: true,
                    fields: false,
                },
            )
            .map_err(|e| format!("Failed to flatten annotations into page stream: {}", e))?;
        let sidecar = studio_sidecar_path(&out_path);
        if sidecar.exists() {
            let _ = fs::remove_file(&sidecar);
        }
    } else {
        let sidecar_bytes = serde_json::to_vec_pretty(&annotations)
            .map_err(|e| format!("Failed to serialize structured studio annotations: {}", e))?;
        let sidecar_path = studio_sidecar_path(&out_path);
        atomic_write(&sidecar_path, &sidecar_bytes)
            .map_err(|e| format!("Failed to write structured annotation sidecar: {}", e))?;
        let _ = session.apply(
            doc_id,
            Edit::AddAttachment {
                name: "localpdf-studio-annotations.json".to_string(),
                bytes: sidecar_bytes,
                description: Some("LocalPDF Studio editable vector annotations".to_string()),
                mime: Some("application/json".to_string()),
            },
        );
    }

    let pdf_bytes = session
        .save_bytes(doc_id)
        .map_err(|e| format!("Failed to serialize modified PDF bytes: {}", e))?;

    atomic_write(&out_path, &pdf_bytes)
        .map_err(|e| format!("Failed to atomically write output PDF: {}", e))?;

    let page_count = verify_written_pdf(&out_path)?;

    Ok(StudioSaveResult {
        output_path,
        saved_count,
        flattened: flatten,
        page_count,
    })
}

/// Load persisted structured annotations for a PDF so reopening restores full vector editability.
#[tauri::command]
pub fn studio_load_annotations(input_path: String) -> Result<Vec<StudioAnnotationDto>, String> {
    let pdf_path = PathBuf::from(&input_path);
    let sidecar = studio_sidecar_path(&pdf_path);
    if !sidecar.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&sidecar)
        .map_err(|e| format!("Failed to read annotation sidecar {:?}: {}", sidecar, e))?;
    serde_json::from_str::<Vec<StudioAnnotationDto>>(&content)
        .map_err(|e| format!("Failed to parse annotation sidecar {:?}: {}", sidecar, e))
}

/// Structural PDF Text Edit IPC: replaces an existing overlapping PDF text block in-place
/// (rewrapped to paragraph bounds) or places a native PDF text block stream on `page_index`,
/// then writes atomically with verify-after-write.
#[tauri::command]
pub fn studio_edit_pdf_text_block(
    input_path: String,
    output_path: String,
    page_index: u32,
    bbox_pt: [f64; 4],
    new_text: String,
    font_size: f64,
    color_rgb: [u8; 3],
) -> Result<StudioSaveResult, String> {
    if new_text.trim().is_empty() {
        return Err("Replacement text cannot be empty".to_string());
    }

    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);

    let raw_bytes = fs::read(&in_path)
        .map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
    let doc_name = in_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut session = Session::new();
    let doc_id = session
        .open(
            doc_name,
            Some(input_path.clone()),
            Arc::new(raw_bytes),
            None,
        )
        .map_err(|e| format!("Failed to open PDF in pdfcraft session: {}", e))?;

    let page_total = session
        .get(doc_id)
        .map(|d| d.info.pages.len())
        .unwrap_or(1)
        .max(1);
    let page = (page_index as usize).min(page_total.saturating_sub(1));
    let rect = normalize_rect(bbox_pt);
    let color = rgb_to_f64(color_rgb);
    let clamped_size = font_size.clamp(6.0, 96.0);

    // Check if an existing PDF paragraph overlaps `rect`; if so, attempt native block rewrite first.
    let overlapping_block = session
        .get(doc_id)
        .map(|d| d.text_blocks(page))
        .and_then(|blocks| block_overlapping(&blocks, rect));

    let mut applied = false;
    if let Some(block_idx) = overlapping_block {
        let style = BlockStyle {
            size: Some(clamped_size),
            color: Some(color),
            ..BlockStyle::default()
        };
        if session
            .apply(
                doc_id,
                Edit::EditTextBlock {
                    page,
                    block: block_idx,
                    text: new_text.clone(),
                    style,
                },
            )
            .is_ok()
        {
            applied = true;
        }
    }

    if !applied {
        let added = AddedText {
            rect,
            text: new_text,
            family: FontFamily::Helvetica,
            bold: false,
            italic: false,
            size: clamped_size,
            color,
            align: TextAlign::Left,
            char_spacing: 0.0,
        };
        session
            .apply(doc_id, Edit::AddText { page, text: added })
            .map_err(|e| format!("Failed to write native PDF text block: {}", e))?;
    }

    let pdf_bytes = session
        .save_bytes(doc_id)
        .map_err(|e| format!("Failed to serialize modified PDF bytes: {}", e))?;

    atomic_write(&out_path, &pdf_bytes)
        .map_err(|e| format!("Failed to atomically write output PDF: {}", e))?;

    let page_count = verify_written_pdf(&out_path)?;

    Ok(StudioSaveResult {
        output_path,
        saved_count: 1,
        flattened: true,
        page_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sample_pdf(path: &Path) {
        let pdf_bytes = b"%PDF-1.4\n\
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
        let _ = atomic_write(path, pdf_bytes);
    }

    #[test]
    fn test_studio_save_and_load_annotations_roundtrip() {
        let tmp_dir = std::env::temp_dir().join(format!("lpdf_studio_d4_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp_dir);
        let input_pdf = tmp_dir.join("input.pdf");
        let output_pdf = tmp_dir.join("output_editable.pdf");
        let flat_pdf = tmp_dir.join("output_flat.pdf");
        create_sample_pdf(&input_pdf);

        let annots = vec![
            StudioAnnotationDto {
                id: "annot-1".to_string(),
                kind: "cloud".to_string(),
                page_index: 0,
                points: vec![
                    StudioPointDto { x: 100.0, y: 100.0, p: 0.5, t: None },
                    StudioPointDto { x: 240.0, y: 100.0, p: 0.5, t: None },
                    StudioPointDto { x: 240.0, y: 200.0, p: 0.5, t: None },
                    StudioPointDto { x: 100.0, y: 200.0, p: 0.5, t: None },
                ],
                bbox_pt: [100.0, 100.0, 240.0, 200.0],
                color_rgb: [220, 38, 38],
                fill_rgb: Some([254, 226, 226]),
                width_pt: 2.0,
                opacity: 0.9,
                line_style: Some("dashed".to_string()),
                start_ending: None,
                end_ending: None,
                hatch_pattern: Some("diagonal-steel".to_string()),
                hatch_scale: Some(1.0),
                text: Some("Revision A".to_string()),
                font_size: None,
                group_id: Some("grp-1".to_string()),
                rotation_deg: Some(0.0),
                snapshot_data_url: None,
            },
        ];

        let save_res = studio_save_annotations(
            input_pdf.to_string_lossy().to_string(),
            output_pdf.to_string_lossy().to_string(),
            annots.clone(),
            false,
        );
        assert!(save_res.is_ok(), "save failed: {:?}", save_res);
        let res = save_res.unwrap();
        assert_eq!(res.saved_count, 1);
        assert!(!res.flattened);
        assert_eq!(res.page_count, 1);

        let loaded = studio_load_annotations(output_pdf.to_string_lossy().to_string());
        assert!(loaded.is_ok());
        let loaded_vec = loaded.unwrap();
        assert_eq!(loaded_vec.len(), 1);
        assert_eq!(loaded_vec[0].id, "annot-1");
        assert_eq!(loaded_vec[0].hatch_pattern.as_deref(), Some("diagonal-steel"));

        // Flatten test
        let flat_res = studio_save_annotations(
            input_pdf.to_string_lossy().to_string(),
            flat_pdf.to_string_lossy().to_string(),
            annots,
            true,
        );
        assert!(flat_res.is_ok(), "flatten failed: {:?}", flat_res);
        assert!(flat_res.unwrap().flattened);

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_studio_edit_pdf_text_block() {
        let tmp_dir =
            std::env::temp_dir().join(format!("lpdf_studio_text_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp_dir);
        let input_pdf = tmp_dir.join("input_text.pdf");
        let output_pdf = tmp_dir.join("output_text.pdf");
        create_sample_pdf(&input_pdf);

        let res = studio_edit_pdf_text_block(
            input_pdf.to_string_lossy().to_string(),
            output_pdf.to_string_lossy().to_string(),
            0,
            [70.0, 700.0, 300.0, 740.0],
            "Updated Structural PDF Text".to_string(),
            14.0,
            [15, 23, 42],
        );
        assert!(res.is_ok(), "text block edit failed: {:?}", res);
        let save_info = res.unwrap();
        assert_eq!(save_info.page_count, 1);
        assert!(output_pdf.exists());

        let _ = fs::remove_dir_all(&tmp_dir);
    }
}
