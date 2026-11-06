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
    /// | "measure.length" | "measure.polylength" | "measure.rectArea" | "measure.polyArea" | "measure.perimeter" | "measure.protractor"
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
    #[serde(default, alias = "measurement_kind")]
    pub measurement_kind: Option<String>,
    #[serde(default, alias = "calibrated_value")]
    pub calibrated_value: Option<f64>,
    #[serde(default, alias = "unit_label")]
    pub unit_label: Option<String>,
    #[serde(default, alias = "perimeter_value")]
    pub perimeter_value: Option<f64>,
    #[serde(default, alias = "volume_value")]
    pub volume_value: Option<f64>,
    #[serde(default, alias = "page_scale")]
    pub page_scale: Option<PageScaleDto>,
    #[serde(default)]
    pub author: Option<String>,
}

/// A single vector line segment in PDF user-space points (`pt`) for OSNAP content snapping (`Phase D5`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PdfSnapSegmentDto {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Extracted vector geometry for a single PDF page (`Phase D5`), capped at 4,096 segments for 120Hz OSNAP.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageSnapGeometryDto {
    #[serde(alias = "page_index")]
    pub page_index: u32,
    #[serde(alias = "width_pt")]
    pub width_pt: f64,
    #[serde(alias = "height_pt")]
    pub height_pt: f64,
    pub segments: Vec<PdfSnapSegmentDto>,
}

/// Calibrated scale configuration for a PDF page (`Phase D5`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageScaleDto {
    #[serde(alias = "page_index")]
    pub page_index: u32,
    #[serde(alias = "pt_distance")]
    pub pt_distance: f64,
    #[serde(alias = "real_distance")]
    pub real_distance: f64,
    pub unit: String,
    pub precision: u8,
    #[serde(default, alias = "default_depth")]
    pub default_depth: Option<f64>,
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

pub(crate) fn studio_prefs_path() -> PathBuf {
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

fn studio_scales_sidecar_path(pdf_path: &Path) -> PathBuf {
    let mut os = pdf_path.as_os_str().to_os_string();
    os.push(".scales.studio.json");
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

/// The boundary loop handed to the hatch adapter (`lpdf-geom`, ADR-018): the annotation's own
/// vertices when it has at least three finite ones (a polygon is clipped to itself, ERR-007), else
/// the corners of its (normalised) rectangle (two-point shapes such as rectangles and ellipses).
fn hatch_loops(points: &[StudioPointDto], rect: [f64; 4]) -> Vec<lpdf_geom::Loop> {
    if points
        .iter()
        .filter(|p| p.x.is_finite() && p.y.is_finite())
        .count()
        >= 3
    {
        vec![lpdf_geom::Loop::new(
            points
                .iter()
                .map(|p| lpdf_geom::PdfPt::new(p.x, p.y))
                .collect(),
        )]
    } else {
        vec![lpdf_geom::Loop::from_rect(rect)]
    }
}

/// Procedural hatch lines for an annotation, as two-point ink strokes, so flattened/standard PDF
/// annotations preserve the hatch appearance in other PDF viewers. `None` for fills (`solid`,
/// `none`) and unknown pattern ids.
fn hatch_strokes_for(
    pattern: &str,
    points: &[StudioPointDto],
    rect: [f64; 4],
    scale: f64,
) -> Result<Option<Vec<Vec<[f64; 2]>>>, lpdf_geom::GeomError> {
    let Some(pattern) = lpdf_geom::HatchPatternId::from_id(pattern) else {
        return Ok(None);
    };
    let segments = match lpdf_geom::hatch_segments(pattern, &hatch_loops(points, rect), scale) {
        Ok(segments) => segments,
        // Only reachable with a non-finite `bbox_pt` and no finite points (JSON cannot carry
        // NaN/inf, so not over IPC). The pre-adapter code drew no hatch here, so keep that, logged.
        Err(lpdf_geom::GeomError::NoFiniteGeometry) => {
            tracing::warn!(
                code = "GEOM_NO_FINITE_GEOMETRY",
                "hatch skipped: region has no finite points"
            );
            return Ok(None);
        }
        Err(e) => return Err(e),
    };
    Ok(Some(
        segments
            .into_iter()
            .map(|s| vec![[s.a.x, s.a.y], [s.b.x, s.b.y]])
            .collect(),
    ))
}

fn verify_written_pdf(path: &Path) -> Result<usize, String> {
    let written = fs::read(path)
        .map_err(|e| format!("Verify-after-write failed reading {:?}: {}", path, e))?;
    if written.len() < 8 {
        return Err("Verify-after-write failed: output file is truncated".to_string());
    }
    let mut verify_session = Session::new();
    let doc_id = verify_session
        .open("verify.pdf".to_string(), None, Arc::new(written), None)
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

/// BUG-050 / BUG-032: refuse an output that is the input (any spelling) or an existing file, before anything is
/// read or written. Typed `OutputPathError` text, `OUTPUT_PATH_UNSAFE: ...`.
fn guard_save_output(input_path: &str, output_path: &str) -> Result<(), String> {
    lpdf_jobs::output_guard::check_output_path(&[PathBuf::from(input_path)], Path::new(output_path))
        .map_err(|e| e.to_string())
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
    guard_save_output(&input_path, &output_path)?;
    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);

    let raw_bytes =
        fs::read(&in_path).map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
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

    // BUG-054: a redaction on a page that does not exist is refused up front (typed error, nothing written). It used
    // to be clamped onto the LAST page, which redacted the wrong page. (Pages here are 0-based.)
    for annot in &annotations {
        if matches!(annot.kind.as_str(), "redact" | "review.redact")
            && (annot.page_index as usize) >= page_total
        {
            return Err(format!(
                "REDACTION_PAGE_OUT_OF_RANGE: page {} does not exist; this document has {} page(s) (pages start at 1)",
                annot.page_index as usize + 1,
                page_total
            ));
        }
    }

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
            "line" | "shape.line" | "arrow" | "shape.arrow" | "measure.length" => {
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
            "polyline" | "shape.polyline" | "measure.polylength" | "measure.perimeter"
            | "measure.protractor" => {
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
            "rect" | "shape.rect" | "snapshot" | "select.snapshot" | "measure.rectArea"
            | "area.rect" => Some(Shape::Rectangle { rect }),
            "ellipse" | "shape.ellipse" => Some(Shape::Oval { rect }),
            "polygon" | "shape.polygon" | "cloud" | "shape.cloud" | "hatch.region"
            | "measure.polyArea" | "area.polygon" => {
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
            "text" | "insert.text" | "insert.textBox" | "insert.note" | "insert.latex"
            | "insert.signature" | "insert.audio" | "insert.stamp" | "insert.image"
            | "callout.single" | "callout.multi" => {
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
            let author = annot
                .author
                .clone()
                .filter(|a| !a.trim().is_empty())
                .unwrap_or_else(|| "LocalPDF Studio".to_string());
            let new_annot = NewAnnotation {
                page,
                shape,
                style,
                contents,
                author,
            };
            session
                .apply(doc_id, Edit::AddAnnotation(new_annot))
                .map_err(|e| format!("Failed to apply annotation '{}': {}", annot.id, e))?;
            saved_count += 1;

            // If a procedural hatch pattern is configured, also emit hatch line strokes so
            // standard PDF viewers and flattened streams display the architectural hatching.
            if let Some(ref pat) = annot.hatch_pattern {
                if pat != "none" && pat != "solid" {
                    let hatch_strokes = hatch_strokes_for(
                        pat,
                        &annot.points,
                        rect,
                        annot.hatch_scale.unwrap_or(1.0),
                    )
                    .map_err(|e| {
                        format!(
                            "Failed to generate hatch for annotation '{}' ({}): {}",
                            annot.id,
                            e.code(),
                            e
                        )
                    })?
                    .unwrap_or_default();
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
                        // BUG-024: a failed hatch annotation fails the save (no silent loss).
                        session
                            .apply(doc_id, Edit::AddAnnotation(hatch_annot))
                            .map_err(|e| {
                                format!(
                                    "Failed to apply hatch for annotation '{}': {}",
                                    annot.id, e
                                )
                            })?;
                    }
                }
            }
        }
    }

    // Execute true cryptographic stream sanitization if any redaction annotations were committed.
    if has_redactions {
        session
            .apply(doc_id, Edit::ApplyRedactions { pages: None })
            .map_err(|e| {
                format!(
                    "Failed to apply cryptographic redaction sanitization: {}",
                    e
                )
            })?;
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

        // Also propagate or collect page scales into `<out_path>.scales.studio.json`
        let mut collected_scales: Vec<PageScaleDto> = Vec::new();
        for annot in &annotations {
            if let Some(ref sc) = annot.page_scale {
                if !collected_scales
                    .iter()
                    .any(|s| s.page_index == sc.page_index)
                {
                    collected_scales.push(sc.clone());
                }
            }
        }
        if collected_scales.is_empty() {
            let in_scales = studio_scales_sidecar_path(&in_path);
            if in_scales.exists() {
                if let Ok(raw) = fs::read_to_string(&in_scales) {
                    if let Ok(parsed) = serde_json::from_str::<Vec<PageScaleDto>>(&raw) {
                        collected_scales = parsed;
                    }
                }
            }
        }
        if !collected_scales.is_empty() {
            if let Ok(scale_bytes) = serde_json::to_vec_pretty(&collected_scales) {
                let out_scales = studio_scales_sidecar_path(&out_path);
                let _ = atomic_write(&out_scales, &scale_bytes);
            }
        }
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

/// Save calibrated page scales for a PDF document atomically (`Phase D5`).
#[tauri::command]
pub fn studio_save_page_scales(
    input_path: String,
    page_scales: Vec<PageScaleDto>,
) -> Result<(), String> {
    let pdf_path = PathBuf::from(&input_path);
    let sidecar = studio_scales_sidecar_path(&pdf_path);
    let bytes = serde_json::to_vec_pretty(&page_scales)
        .map_err(|e| format!("Failed to serialize page scales: {}", e))?;
    atomic_write(&sidecar, &bytes)
        .map_err(|e| format!("Failed to atomically write page scales sidecar: {}", e))
}

/// Load calibrated page scales for a PDF document (`Phase D5`).
#[tauri::command]
pub fn studio_load_page_scales(input_path: String) -> Result<Vec<PageScaleDto>, String> {
    let pdf_path = PathBuf::from(&input_path);
    let sidecar = studio_scales_sidecar_path(&pdf_path);
    if sidecar.exists() {
        let content = fs::read_to_string(&sidecar)
            .map_err(|e| format!("Failed to read page scales sidecar {:?}: {}", sidecar, e))?;
        return serde_json::from_str::<Vec<PageScaleDto>>(&content)
            .map_err(|e| format!("Failed to parse page scales sidecar {:?}: {}", sidecar, e));
    }

    // Fallback: extract any page_scale embedded inside `.studio.json` annotations
    let annot_sidecar = studio_sidecar_path(&pdf_path);
    if annot_sidecar.exists() {
        if let Ok(content) = fs::read_to_string(&annot_sidecar) {
            if let Ok(annots) = serde_json::from_str::<Vec<StudioAnnotationDto>>(&content) {
                let mut scales = Vec::new();
                for a in annots {
                    if let Some(sc) = a.page_scale {
                        if !scales
                            .iter()
                            .any(|s: &PageScaleDto| s.page_index == sc.page_index)
                        {
                            scales.push(sc);
                        }
                    }
                }
                return Ok(scales);
            }
        }
    }

    Ok(Vec::new())
}

const MAX_SNAP_SEGMENTS_PER_PAGE: usize = 4_096;

fn push_segment_capped(out: &mut Vec<PdfSnapSegmentDto>, x1: f64, y1: f64, x2: f64, y2: f64) {
    if out.len() >= MAX_SNAP_SEGMENTS_PER_PAGE {
        return;
    }
    if !x1.is_finite() || !y1.is_finite() || !x2.is_finite() || !y2.is_finite() {
        return;
    }
    if (x2 - x1).hypot(y2 - y1) < 1e-3 {
        return;
    }
    out.push(PdfSnapSegmentDto { x1, y1, x2, y2 });
}

/// Extract vector line/rect/path segments on `page_index` in PDF user-space points (`pt`) for 120Hz OSNAP (`Phase D5`).
/// Caps output at 4,096 segments per page and always includes the 4 page-border edges and persisted vector annotations.
#[tauri::command]
pub fn studio_extract_snap_geometry(
    input_path: String,
    page_index: u32,
) -> Result<PageSnapGeometryDto, String> {
    let in_path = PathBuf::from(&input_path);
    let raw_bytes =
        fs::read(&in_path).map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
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

    let doc = session
        .get(doc_id)
        .ok_or_else(|| "Document session unavailable".to_string())?;

    let page_total = doc.info.pages.len().max(1);
    let page = (page_index as usize).min(page_total.saturating_sub(1));

    let (width_pt, height_pt) = doc
        .info
        .pages
        .get(page)
        .map(|p| (f64::from(p.width).max(1.0), f64::from(p.height).max(1.0)))
        .unwrap_or((612.0, 792.0));

    let mut segments: Vec<PdfSnapSegmentDto> = Vec::with_capacity(256);

    // 1. Always include the 4 page-border edges in PDF user-space points (`pt`).
    push_segment_capped(&mut segments, 0.0, 0.0, width_pt, 0.0);
    push_segment_capped(&mut segments, width_pt, 0.0, width_pt, height_pt);
    push_segment_capped(&mut segments, width_pt, height_pt, 0.0, height_pt);
    push_segment_capped(&mut segments, 0.0, height_pt, 0.0, 0.0);

    // 2. Extract native vector path segments from the page content stream & Form XObjects via `pdfcraft`.
    if let Ok(geom) = doc.measurement_paths(page) {
        for [a, b] in geom.segments {
            if segments.len() >= MAX_SNAP_SEGMENTS_PER_PAGE {
                break;
            }
            push_segment_capped(&mut segments, a[0], a[1], b[0], b[1]);
        }
    }

    // 3. Also include persisted structured annotation segments on this page if `.studio.json` exists.
    if let Ok(annots) = studio_load_annotations(input_path) {
        for annot in annots.into_iter().filter(|a| a.page_index == page_index) {
            if segments.len() >= MAX_SNAP_SEGMENTS_PER_PAGE {
                break;
            }
            if annot.points.len() >= 2 {
                for pair in annot.points.windows(2) {
                    push_segment_capped(&mut segments, pair[0].x, pair[0].y, pair[1].x, pair[1].y);
                }
                if annot.kind.contains("polygon")
                    || annot.kind.contains("cloud")
                    || annot.kind.contains("hatch")
                    || annot.kind.contains("Area")
                {
                    if let (Some(first), Some(last)) = (annot.points.first(), annot.points.last()) {
                        push_segment_capped(&mut segments, last.x, last.y, first.x, first.y);
                    }
                }
            } else {
                let [x0, y0, x1, y1] = normalize_rect(annot.bbox_pt);
                push_segment_capped(&mut segments, x0, y0, x1, y0);
                push_segment_capped(&mut segments, x1, y0, x1, y1);
                push_segment_capped(&mut segments, x1, y1, x0, y1);
                push_segment_capped(&mut segments, x0, y1, x0, y0);
            }
        }
    }

    Ok(PageSnapGeometryDto {
        page_index,
        width_pt,
        height_pt,
        segments,
    })
}

/// DTO representing an extracted or editable structural PDF text block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfTextBlockDto {
    pub id: String,
    pub page_index: u32,
    pub bbox_pt: [f64; 4], // [x, y, width, height] in top-left PDF user points
    pub text: String,
    pub font_size: f64,
}

/// Extracts all readable text blocks and their bounding boxes on `page_index`.
#[tauri::command]
pub fn studio_extract_text_blocks(
    input_path: String,
    page_index: u32,
) -> Result<Vec<PdfTextBlockDto>, String> {
    let in_path = PathBuf::from(&input_path);
    if !in_path.exists() {
        return Ok(Vec::new());
    }

    let raw_bytes =
        fs::read(&in_path).map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
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

    let blocks = session
        .get(doc_id)
        .map(|d| d.text_blocks(page))
        .unwrap_or_default();

    let mut result = Vec::new();
    for (idx, b) in blocks.into_iter().enumerate() {
        let x0 = b.rect[0].min(b.rect[2]);
        let y0 = b.rect[1].min(b.rect[3]);
        let x1 = b.rect[0].max(b.rect[2]);
        let y1 = b.rect[1].max(b.rect[3]);
        let w = (x1 - x0).abs().max(1.0);
        let h = (y1 - y0).abs().max(1.0);
        let text = if !b.editor_text.is_empty() {
            b.editor_text
        } else {
            b.text
        };
        if text.trim().is_empty() {
            continue;
        }
        result.push(PdfTextBlockDto {
            id: format!("pdf-block-p{}-{}", page_index, idx),
            page_index,
            bbox_pt: [x0, y0, w, h],
            text,
            font_size: if b.size > 0.0 { b.size } else { 12.0 },
        });
    }

    Ok(result)
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
    guard_save_output(&input_path, &output_path)?;
    if new_text.trim().is_empty() {
        return Err("Replacement text cannot be empty".to_string());
    }

    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);

    let raw_bytes =
        fs::read(&in_path).map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
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

/// Phase D6 Quantity Takeoff CSV Record (`studio_export_takeoff_csv`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeasurementRecord {
    #[serde(default)]
    pub id: String,
    pub page_index: u32,
    pub measurement_type: String,
    pub raw_pt: f64,
    pub calibrated_value: f64,
    pub unit: String,
    pub color: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// Phase D6 Reusable Markup Library Item (`studio_markup_library`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkupLibraryItemDto {
    pub id: String,
    pub title: String,
    pub category: String,
    pub tool_id: String,
    pub svg_path: String,
    pub color_token: String,
    pub stroke_width: f64,
    #[serde(default)]
    pub latex_source: Option<String>,
    #[serde(default)]
    pub text_label: Option<String>,
    #[serde(default)]
    pub data_url: Option<String>,
    pub created_at_ms: u64,
}

pub(crate) fn get_markup_library_path() -> PathBuf {
    if let Ok(custom_dir) = std::env::var("LPDF_STUDIO_CONFIG_DIR") {
        let dir = PathBuf::from(custom_dir);
        let _ = fs::create_dir_all(&dir);
        return dir.join("studio_markup_library.json");
    }
    if let Some(proj_dirs) = ProjectDirs::from("", "", "localpdf") {
        let config_dir = proj_dirs.config_dir();
        let _ = fs::create_dir_all(config_dir);
        config_dir.join("studio_markup_library.json")
    } else {
        PathBuf::from("studio_markup_library.json")
    }
}

/// Escape an arbitrary string for RFC 4180 CSV with spreadsheet formula-injection protection.
fn csv_field(s: &str) -> String {
    let trimmed = s.trim_start();
    let needs_prefix = trimmed.starts_with('=')
        || trimmed.starts_with('+')
        || trimmed.starts_with('-')
        || trimmed.starts_with('@');
    let escaped = s.replace('"', "\"\"");
    if needs_prefix {
        format!("\"'{}'\"", escaped)
    } else {
        format!("\"{}\"", escaped)
    }
}

/// Phase D6 Audio Notes & Embeds: embeds WebM/WAV audio bytes as a PDF `/Sound` / `/FileAttachment`
/// stream and stamps an audio note marker on `page_index` at `(x_pt, y_pt)`.
#[tauri::command]
pub fn studio_attach_audio(
    input_path: String,
    output_path: String,
    page_index: u32,
    x_pt: f64,
    y_pt: f64,
    audio_bytes: Vec<u8>,
) -> Result<StudioSaveResult, String> {
    guard_save_output(&input_path, &output_path)?;
    if audio_bytes.is_empty() {
        return Err("Audio payload cannot be empty".to_string());
    }

    let in_path = PathBuf::from(&input_path);
    let out_path = PathBuf::from(&output_path);

    let raw_bytes =
        fs::read(&in_path).map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
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

    // Detect WAV vs WebM header for MIME metadata
    let is_wav = audio_bytes.len() >= 4 && &audio_bytes[0..4] == b"RIFF";
    let ext = if is_wav { "wav" } else { "webm" };
    let mime = if is_wav { "audio/wav" } else { "audio/webm" };
    let attachment_name = format!(
        "studio-audio-p{}-{}-{}.{}",
        page + 1,
        x_pt.round() as i64,
        y_pt.round() as i64,
        ext
    );

    session
        .apply(
            doc_id,
            Edit::AddAttachment {
                name: attachment_name.clone(),
                bytes: audio_bytes.clone(),
                description: Some(format!(
                    "LocalPDF Studio Audio Note (Page {}, {:.1}pt, {:.1}pt)",
                    page + 1,
                    x_pt,
                    y_pt
                )),
                mime: Some(mime.to_string()),
            },
        )
        .map_err(|e| format!("Failed to embed audio stream into PDF: {}", e))?;

    // Place a visual sound pin annotation at `(x_pt, y_pt)` referencing the embedded audio stream
    let pin_rect = normalize_rect([x_pt - 12.0, y_pt - 12.0, x_pt + 84.0, y_pt + 16.0]);
    let sound_annot = NewAnnotation {
        page,
        shape: Shape::TextBox {
            rect: pin_rect,
            font_size: 10.0,
        },
        style: Style {
            color: [0.05, 0.58, 0.53],
            opacity: 0.95,
            width: 1.5,
            fill: Some([0.80, 0.98, 0.95]),
        },
        contents: format!(
            "[Audio Note: {} ({} bytes)]",
            attachment_name,
            audio_bytes.len()
        ),
        author: "LocalPDF Studio Audio".to_string(),
    };

    session
        .apply(doc_id, Edit::AddAnnotation(sound_annot))
        .map_err(|e| format!("Failed to attach audio pin annotation: {}", e))?;

    let pdf_bytes = session
        .save_bytes(doc_id)
        .map_err(|e| format!("Failed to serialize PDF with embedded audio: {}", e))?;

    atomic_write(&out_path, &pdf_bytes)
        .map_err(|e| format!("Failed to atomically write output PDF: {}", e))?;

    let page_count = verify_written_pdf(&out_path)?;

    Ok(StudioSaveResult {
        output_path,
        saved_count: 1,
        flattened: false,
        page_count,
    })
}

/// Phase D6 Quantity Takeoff Export: writes a clean RFC 4180 CSV spreadsheet of all measurements
/// (Page, Type, Raw Pt, Calibrated Value, Unit, Color, Label) plus per-unit summary totals.
#[tauri::command]
pub fn studio_export_takeoff_csv(
    measurements: Vec<MeasurementRecord>,
    output_path: String,
    protected_paths: Option<Vec<String>>,
    overwrite: Option<bool>,
) -> Result<(), String> {
    let out_path = PathBuf::from(&output_path);
    // A takeoff CSV is never written to a PDF path or an open document, and never over an existing file unless
    // the UI confirmed the overwrite (BUG-051 follow-up).
    if lpdf_jobs::output_guard::targets_pdf(&out_path) {
        return Err(format!(
            "OUTPUT_PATH_UNSAFE: a takeoff CSV cannot be written to a PDF path ({output_path}). Nothing was written."
        ));
    }
    let protected: Vec<PathBuf> = protected_paths
        .unwrap_or_default()
        .iter()
        .map(PathBuf::from)
        .collect();
    let guard = if overwrite == Some(true) {
        lpdf_jobs::output_guard::check_output_path_confirmed(&protected, &out_path)
    } else {
        lpdf_jobs::output_guard::check_output_path(&protected, &out_path)
    };
    guard.map_err(|e| e.to_string())?;
    let mut csv_out = String::with_capacity(measurements.len() * 96 + 256);

    csv_out.push_str(
        "\"ID\",\"Page\",\"Type\",\"Raw Pt\",\"Calibrated Value\",\"Unit\",\"Color\",\"Label\"\r\n",
    );

    let mut unit_totals: std::collections::BTreeMap<(String, String), f64> =
        std::collections::BTreeMap::new();

    for (idx, m) in measurements.iter().enumerate() {
        let row_id = if m.id.trim().is_empty() {
            format!("M-{:03}", idx + 1)
        } else {
            m.id.clone()
        };
        let label = m.label.as_deref().unwrap_or("");
        let line = format!(
            "{},{},{},{:.2},{:.4},{},{},{}\r\n",
            csv_field(&row_id),
            m.page_index + 1,
            csv_field(&m.measurement_type),
            m.raw_pt,
            m.calibrated_value,
            csv_field(&m.unit),
            csv_field(&m.color),
            csv_field(label),
        );
        csv_out.push_str(&line);

        if m.calibrated_value.is_finite() {
            let key = (m.measurement_type.clone(), m.unit.clone());
            let entry = unit_totals.entry(key).or_insert(0.0);
            *entry += m.calibrated_value;
        }
    }

    // Append summary section for construction quantity takeoff
    if !unit_totals.is_empty() {
        csv_out.push_str(
            "\r\n\"SUMMARY\",\"\",\"Type\",\"\",\"Total Calibrated\",\"Unit\",\"\",\"\"\r\n",
        );
        for ((mtype, unit), total) in unit_totals {
            let summary_line = format!(
                "\"TOTAL\",\"\",{},\"\",{:.4},{},\"\",\"\"\r\n",
                csv_field(&mtype),
                total,
                csv_field(&unit)
            );
            csv_out.push_str(&summary_line);
        }
    }

    atomic_write(&out_path, csv_out.as_bytes()).map_err(|e| {
        format!(
            "Failed to atomically write takeoff CSV {:?}: {}",
            out_path, e
        )
    })
}

/// Phase D6 Reusable Markup Library Vault (`studio_markup_library`).
/// Supports actions: `"list"` / `"load"`, `"save"` / `"add"`, `"delete"`, `"clear"`.
#[tauri::command]
pub fn studio_markup_library(action: String, payload: Option<String>) -> Result<String, String> {
    let lib_path = get_markup_library_path();
    let mut items: Vec<MarkupLibraryItemDto> = if lib_path.exists() {
        let raw = fs::read_to_string(&lib_path)
            .map_err(|e| format!("Failed to read markup library {:?}: {}", lib_path, e))?;
        serde_json::from_str::<Vec<MarkupLibraryItemDto>>(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };

    match action.as_str() {
        "list" | "load" => serde_json::to_string(&items)
            .map_err(|e| format!("Failed to serialize markup library: {}", e)),
        "save" | "add" => {
            let raw_payload = payload
                .ok_or_else(|| "Missing payload for studio_markup_library save".to_string())?;
            if let Ok(single_item) = serde_json::from_str::<MarkupLibraryItemDto>(&raw_payload) {
                items.retain(|existing| existing.id != single_item.id);
                items.push(single_item);
            } else if let Ok(batch_items) =
                serde_json::from_str::<Vec<MarkupLibraryItemDto>>(&raw_payload)
            {
                items = batch_items;
            } else {
                return Err("Invalid JSON payload for studio_markup_library".to_string());
            }
            let bytes = serde_json::to_vec_pretty(&items)
                .map_err(|e| format!("Failed to serialize markup library: {}", e))?;
            atomic_write(&lib_path, &bytes)
                .map_err(|e| format!("Failed to atomically save markup library: {}", e))?;
            serde_json::to_string(&items)
                .map_err(|e| format!("Failed to serialize updated markup library: {}", e))
        }
        "delete" => {
            let target_id = payload
                .ok_or_else(|| "Missing item id for studio_markup_library delete".to_string())?;
            items.retain(|item| item.id != target_id);
            let bytes = serde_json::to_vec_pretty(&items)
                .map_err(|e| format!("Failed to serialize markup library: {}", e))?;
            atomic_write(&lib_path, &bytes)
                .map_err(|e| format!("Failed to atomically save markup library: {}", e))?;
            serde_json::to_string(&items)
                .map_err(|e| format!("Failed to serialize updated markup library: {}", e))
        }
        "clear" => {
            items.clear();
            let bytes = serde_json::to_vec_pretty(&items)
                .map_err(|e| format!("Failed to serialize empty markup library: {}", e))?;
            atomic_write(&lib_path, &bytes)
                .map_err(|e| format!("Failed to atomically clear markup library: {}", e))?;
            Ok("[]".to_string())
        }
        other => Err(format!(
            "Unsupported studio_markup_library action '{}'",
            other
        )),
    }
}

/// Phase D6.1 Document Tags, Folders, Favorites & Recent Files Entry (`7.10`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkspaceDocumentEntryDto {
    pub path: String,
    pub title: String,
    pub folder: String,
    pub tags: Vec<String>,
    pub favorite: bool,
    pub last_opened_ms: u64,
}

fn get_workspace_library_path() -> PathBuf {
    if let Ok(custom_dir) = std::env::var("LPDF_STUDIO_CONFIG_DIR") {
        let dir = PathBuf::from(custom_dir);
        let _ = fs::create_dir_all(&dir);
        return dir.join("studio_workspace_documents.json");
    }
    if let Some(proj_dirs) = ProjectDirs::from("", "", "localpdf") {
        let config_dir = proj_dirs.config_dir();
        let _ = fs::create_dir_all(config_dir);
        config_dir.join("studio_workspace_documents.json")
    } else {
        PathBuf::from("studio_workspace_documents.json")
    }
}

/// Phase D6.1 Document Library Manager (`7.10`): manages local document tags, folders, favorites & recent files.
#[tauri::command]
pub fn studio_workspace_library(action: String, payload: Option<String>) -> Result<String, String> {
    let lib_path = get_workspace_library_path();
    let mut docs: Vec<WorkspaceDocumentEntryDto> = if lib_path.exists() {
        let raw = fs::read_to_string(&lib_path)
            .map_err(|e| format!("Failed to read workspace library {:?}: {}", lib_path, e))?;
        serde_json::from_str::<Vec<WorkspaceDocumentEntryDto>>(&raw).unwrap_or_default()
    } else {
        Vec::new()
    };

    match action.as_str() {
        "list" | "load" => serde_json::to_string(&docs)
            .map_err(|e| format!("Failed to serialize workspace library: {}", e)),
        "upsert" | "save" => {
            let raw_payload = payload
                .ok_or_else(|| "Missing payload for studio_workspace_library upsert".to_string())?;
            if let Ok(entry) = serde_json::from_str::<WorkspaceDocumentEntryDto>(&raw_payload) {
                docs.retain(|d| d.path != entry.path);
                docs.insert(0, entry);
            } else if let Ok(batch) =
                serde_json::from_str::<Vec<WorkspaceDocumentEntryDto>>(&raw_payload)
            {
                docs = batch;
            } else {
                return Err("Invalid JSON payload for studio_workspace_library".to_string());
            }
            let bytes = serde_json::to_vec_pretty(&docs)
                .map_err(|e| format!("Failed to serialize workspace library: {}", e))?;
            atomic_write(&lib_path, &bytes)
                .map_err(|e| format!("Failed to atomically write workspace library: {}", e))?;
            serde_json::to_string(&docs)
                .map_err(|e| format!("Failed to serialize updated workspace library: {}", e))
        }
        "delete" => {
            let target_path = payload
                .ok_or_else(|| "Missing path for studio_workspace_library delete".to_string())?;
            docs.retain(|d| d.path != target_path);
            let bytes = serde_json::to_vec_pretty(&docs)
                .map_err(|e| format!("Failed to serialize workspace library: {}", e))?;
            atomic_write(&lib_path, &bytes)
                .map_err(|e| format!("Failed to atomically write workspace library: {}", e))?;
            serde_json::to_string(&docs)
                .map_err(|e| format!("Failed to serialize updated workspace library: {}", e))
        }
        other => Err(format!(
            "Unsupported studio_workspace_library action '{}'",
            other
        )),
    }
}

/// Phase D6.1 Local Watch-Folder Auto-Ingest Scanner (`8.7`, `ADR-012 §8`):
/// Scans a local directory for dropped PDFs and images (`.pdf`, `.png`, `.jpg`, `.jpeg`, `.svg`)
/// with zero network access.
#[tauri::command]
pub fn studio_scan_watch_folder(folder_path: String) -> Result<Vec<String>, String> {
    let dir = PathBuf::from(&folder_path);
    if !dir.exists() || !dir.is_dir() {
        return Err(format!(
            "Watch folder does not exist or is not a directory: {:?}",
            dir
        ));
    }
    let entries =
        fs::read_dir(&dir).map_err(|e| format!("Failed to read watch folder {:?}: {}", dir, e))?;

    let mut discovered: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_file() {
            continue;
        }
        let allowed = p
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| {
                matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "pdf" | "png" | "jpg" | "jpeg" | "svg"
                )
            })
            .unwrap_or(false);
        if allowed {
            discovered.push(p.to_string_lossy().to_string());
        }
    }
    discovered.sort();
    Ok(discovered)
}

/// Phase D6.1 Auto Table & Form Field Region Detection DTO (`8.12`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetectedRegionDto {
    pub id: String,
    /// `"table"` or `"form_field"`
    pub kind: String,
    pub page_index: u32,
    pub bbox_pt: [f64; 4],
    pub rows: usize,
    pub cols: usize,
    pub label: String,
}

/// Phase D6.1 Auto Table / Form Field Detection (`8.12`):
/// Detects tabular grid structures from page vector geometry, and fillable form fields with
/// pdfcraft's field detection (`Session::detect_fields`). Returns only what was found.
#[tauri::command]
pub fn studio_detect_tables_and_fields(
    input_path: String,
    page_index: u32,
) -> Result<Vec<DetectedRegionDto>, String> {
    let in_path = PathBuf::from(&input_path);
    let raw_bytes =
        fs::read(&in_path).map_err(|e| format!("Failed to read input PDF {:?}: {}", in_path, e))?;
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

    let doc = session
        .get(doc_id)
        .ok_or_else(|| "Document session unavailable".to_string())?;

    let page_total = doc.info.pages.len().max(1);
    let page = (page_index as usize).min(page_total.saturating_sub(1));

    let mut detected: Vec<DetectedRegionDto> = Vec::new();

    // 1. Analyze vector line segments for horizontal/vertical grid intersections (tables) and underlines (form fields)
    if let Ok(geom) = doc.measurement_paths(page) {
        let mut h_lines: Vec<[f64; 4]> = Vec::new();
        let mut v_lines: Vec<[f64; 4]> = Vec::new();
        for [a, b] in geom.segments {
            let dx = (b[0] - a[0]).abs();
            let dy = (b[1] - a[1]).abs();
            if dy < 1.5 && dx >= 36.0 {
                h_lines.push([a[0].min(b[0]), a[1], a[0].max(b[0]), b[1]]);
            } else if dx < 1.5 && dy >= 14.0 {
                v_lines.push([a[0], a[1].min(b[1]), b[0], a[1].max(b[1])]);
            }
        }

        if h_lines.len() >= 2 && v_lines.len() >= 2 {
            let min_x = h_lines.iter().map(|l| l[0]).fold(f64::INFINITY, f64::min);
            let max_x = h_lines
                .iter()
                .map(|l| l[2])
                .fold(f64::NEG_INFINITY, f64::max);
            let min_y = v_lines.iter().map(|l| l[1]).fold(f64::INFINITY, f64::min);
            let max_y = v_lines
                .iter()
                .map(|l| l[3])
                .fold(f64::NEG_INFINITY, f64::max);
            if min_x.is_finite() && max_x > min_x && min_y.is_finite() && max_y > min_y {
                let rows = h_lines.len().saturating_sub(1).max(1);
                let cols = v_lines.len().saturating_sub(1).max(1);
                detected.push(DetectedRegionDto {
                    id: format!("tbl-p{}-1", page + 1),
                    kind: "table".to_string(),
                    page_index,
                    bbox_pt: [min_x, min_y, max_x, max_y],
                    rows,
                    cols,
                    label: format!("Detected Table ({}x{})", rows, cols),
                });
            }
        }
    }

    // 2. Form fields: pdfcraft's own detection (DO-25 "Detect fields"; matrix R8), which reads
    //    labels, boxes, rules and combs (or pixels on scanned pages) and skips existing widgets.
    //    BUG-023: nothing is invented when nothing is found; an empty result means "none found".
    for (idx, (_, c)) in session
        .detect_fields(doc_id, &[page])
        .into_iter()
        .enumerate()
    {
        let kind_label = match c.kind {
            pdfcraft_engine::detect::Kind::Text => "Text field".to_string(),
            pdfcraft_engine::detect::Kind::CheckBox => "Checkbox".to_string(),
            pdfcraft_engine::detect::Kind::Comb { cells } => format!("Comb field ({cells} cells)"),
        };
        detected.push(DetectedRegionDto {
            id: format!("fld-p{}-{}", page + 1, idx + 1),
            kind: "form_field".to_string(),
            page_index,
            bbox_pt: normalize_rect(c.rect),
            rows: 1,
            cols: 1,
            label: format!("{kind_label}: {}", c.name),
        });
    }

    Ok(detected)
}

/// BUG-018 (b): read-only existence check for an absolute local path (no read, no network).
/// Relative paths and paths that cannot be checked (permissions, I/O error) report `false`.
#[tauri::command]
pub fn path_exists(path: String) -> bool {
    path_exists_at(Path::new(&path))
}

fn path_exists_at(path: &Path) -> bool {
    path.is_absolute() && path.try_exists().unwrap_or(false)
}

/// BUG-018 (c): delete an Organize intermediate `<stem>.organizing-<token>.pdf` that a job of this
/// app session wrote. Anything else (originals, other names, files this session did not create,
/// symlinks, relative paths) is refused and left untouched.
#[tauri::command]
pub(crate) fn delete_organize_intermediate(
    state: tauri::State<'_, crate::AppState>,
    path: String,
) -> Result<(), String> {
    state
        .job_manager
        .delete_session_intermediate(Path::new(&path))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_exists_is_absolute_only_and_read_only() {
        let dir = std::env::temp_dir().join(format!("lpdf_path_exists_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("present.pdf");
        fs::write(&file, b"%PDF-1.4\n").unwrap();
        let before = fs::read(&file).unwrap();
        assert!(path_exists(file.to_string_lossy().to_string()));
        assert!(path_exists(dir.to_string_lossy().to_string()));
        assert!(!path_exists(
            dir.join("absent.pdf").to_string_lossy().to_string()
        ));
        assert!(
            !path_exists("present.pdf".to_string()),
            "relative paths are never resolved"
        );
        assert!(!path_exists(String::new()));
        assert_eq!(
            fs::read(&file).unwrap(),
            before,
            "the check does not modify the file"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn takeoff_csv_export_refuses_open_documents_pdf_paths_and_existing_files_unless_confirmed() {
        let dir = std::env::temp_dir().join(format!("lpdf_csv_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let doc = dir.join("plan.pdf");
        fs::write(&doc, b"%PDF-1.4 original").unwrap();
        let d = doc.to_string_lossy().to_string();
        let unsafe_err =
            |r: Result<(), String>| r.as_ref().is_err_and(|e| e.contains("OUTPUT_PATH_UNSAFE"));

        // never the open document, never a .pdf path, even with a confirmed overwrite
        assert!(unsafe_err(studio_export_takeoff_csv(
            vec![],
            d.clone(),
            Some(vec![d.clone()]),
            Some(true)
        )));
        assert!(unsafe_err(studio_export_takeoff_csv(
            vec![],
            d.clone(),
            None,
            Some(true)
        )));
        assert_eq!(fs::read(&doc).unwrap(), b"%PDF-1.4 original");

        let csv = dir.join("plan.takeoff.csv");
        fs::write(&csv, b"old rows").unwrap();
        let c = csv.to_string_lossy().to_string();
        assert!(unsafe_err(studio_export_takeoff_csv(
            vec![],
            c.clone(),
            None,
            None
        )));
        assert_eq!(fs::read(&csv).unwrap(), b"old rows");
        assert!(studio_export_takeoff_csv(vec![], c, None, Some(true)).is_ok());
        let fresh = dir.join("plan.takeoff-2.csv").to_string_lossy().to_string();
        assert!(studio_export_takeoff_csv(vec![], fresh, None, None).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    // BUG-050 / BUG-032: the save entry points refuse an output that is the input (any spelling, no `.pdf`
    // extension needed) or an existing file, with a typed message, and write nothing.
    #[test]
    fn save_entry_points_refuse_overwriting_the_input_or_an_existing_file() {
        let dir = std::env::temp_dir().join(format!("lpdf_bug050_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("doc"); // no .pdf extension
        fs::write(&input, b"original bytes").unwrap();
        let p = input.to_string_lossy().to_string();

        let r = studio_save_annotations(p.clone(), p.clone(), vec![], false);
        assert!(
            r.as_ref().is_err_and(|e| e.contains("OUTPUT_PATH_UNSAFE")),
            "{r:?}"
        );
        let r = studio_edit_pdf_text_block(
            p.clone(),
            p.clone(),
            0,
            [0.0; 4],
            "x".into(),
            12.0,
            [0, 0, 0],
        );
        assert!(
            r.as_ref().is_err_and(|e| e.contains("OUTPUT_PATH_UNSAFE")),
            "{r:?}"
        );
        let r = studio_attach_audio(p.clone(), p.clone(), 0, 0.0, 0.0, vec![1, 2, 3]);
        assert!(
            r.as_ref().is_err_and(|e| e.contains("OUTPUT_PATH_UNSAFE")),
            "{r:?}"
        );
        assert_eq!(fs::read(&input).unwrap(), b"original bytes");

        let existing = dir.join("out.pdf");
        fs::write(&existing, b"previous").unwrap();
        let r = studio_save_annotations(p, existing.to_string_lossy().to_string(), vec![], false);
        assert!(
            r.as_ref().is_err_and(|e| e.contains("OUTPUT_PATH_UNSAFE")),
            "{r:?}"
        );
        assert_eq!(fs::read(&existing).unwrap(), b"previous");
        let _ = fs::remove_dir_all(&dir);
    }

    // BUG-054: Studio redactions are 0-based and checked: an out-of-range page is refused with the typed error and
    // nothing is written (it used to be clamped onto the LAST page, redacting the wrong page).
    #[test]
    fn studio_redaction_on_a_page_that_does_not_exist_is_refused_not_clamped() {
        let dir = std::env::temp_dir().join(format!("lpdf_bug054_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("one-page.pdf");
        create_sample_pdf(&input);
        let annot = |page: u32| -> StudioAnnotationDto {
            serde_json::from_value(serde_json::json!({
                "id": "r1", "kind": "redact", "pageIndex": page,
                "bboxPt": [10.0, 10.0, 200.0, 100.0]
            }))
            .unwrap()
        };
        let i = input.to_string_lossy().to_string();

        for bad in [1u32, 7] {
            let out = dir.join(format!("out-{bad}.pdf"));
            let r = studio_save_annotations(
                i.clone(),
                out.to_string_lossy().to_string(),
                vec![annot(bad)],
                true,
            );
            assert!(
                r.as_ref()
                    .is_err_and(|e| e.contains("REDACTION_PAGE_OUT_OF_RANGE")),
                "page {bad} of a 1-page document must be refused: {r:?}"
            );
            assert!(!out.exists(), "nothing may be written");
        }
        // the last real page (index 0 of a 1-page document) still works
        let ok = dir.join("ok.pdf");
        let r = studio_save_annotations(i, ok.to_string_lossy().to_string(), vec![annot(0)], true);
        assert!(r.is_ok(), "{r:?}");
        assert!(ok.exists());
        let _ = fs::remove_dir_all(&dir);
    }

    // A batch with ONE invalid redaction page writes NOTHING (all-or-nothing): the valid redaction in the same
    // commit must not be saved either.
    #[test]
    fn a_redaction_batch_with_one_invalid_page_writes_nothing() {
        let dir = std::env::temp_dir().join(format!("lpdf_bug054b_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let input = dir.join("one-page.pdf");
        create_sample_pdf(&input);
        let before = fs::read(&input).unwrap();
        let annot = |id: &str, page: u32| -> StudioAnnotationDto {
            serde_json::from_value(serde_json::json!({
                "id": id, "kind": "redact", "pageIndex": page,
                "bboxPt": [10.0, 10.0, 200.0, 100.0]
            }))
            .unwrap()
        };
        let out = dir.join("out.pdf");
        let r = studio_save_annotations(
            input.to_string_lossy().to_string(),
            out.to_string_lossy().to_string(),
            vec![annot("ok", 0), annot("bad", 4)],
            true,
        );
        assert!(
            r.as_ref()
                .is_err_and(|e| e.contains("REDACTION_PAGE_OUT_OF_RANGE")),
            "{r:?}"
        );
        assert!(
            !out.exists(),
            "no output file, not even for the valid redaction"
        );
        assert!(!out.with_extension("tmp").exists());
        assert_eq!(
            fs::read(&input).unwrap(),
            before,
            "the original is untouched"
        );
        let _ = fs::remove_dir_all(&dir);
    }

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

        let annots = vec![StudioAnnotationDto {
            id: "annot-1".to_string(),
            kind: "cloud".to_string(),
            page_index: 0,
            points: vec![
                StudioPointDto {
                    x: 100.0,
                    y: 100.0,
                    p: 0.5,
                    t: None,
                },
                StudioPointDto {
                    x: 240.0,
                    y: 100.0,
                    p: 0.5,
                    t: None,
                },
                StudioPointDto {
                    x: 240.0,
                    y: 200.0,
                    p: 0.5,
                    t: None,
                },
                StudioPointDto {
                    x: 100.0,
                    y: 200.0,
                    p: 0.5,
                    t: None,
                },
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
            measurement_kind: Some("area.polygon".to_string()),
            calibrated_value: Some(14.0),
            unit_label: Some("m²".to_string()),
            perimeter_value: Some(15.2),
            volume_value: Some(42.0),
            page_scale: Some(PageScaleDto {
                page_index: 0,
                pt_distance: 72.0,
                real_distance: 1.0,
                unit: "m".to_string(),
                precision: 2,
                default_depth: Some(3.0),
            }),
            author: Some("A. Architect".to_string()),
        }];

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
        assert_eq!(
            loaded_vec[0].hatch_pattern.as_deref(),
            Some("diagonal-steel")
        );
        assert_eq!(loaded_vec[0].calibrated_value, Some(14.0));
        assert_eq!(loaded_vec[0].unit_label.as_deref(), Some("m²"));

        let loaded_scales = studio_load_page_scales(output_pdf.to_string_lossy().to_string());
        assert!(loaded_scales.is_ok());
        assert_eq!(loaded_scales.unwrap().len(), 1);

        let snap_geom = studio_extract_snap_geometry(output_pdf.to_string_lossy().to_string(), 0);
        assert!(snap_geom.is_ok(), "snap extraction failed: {:?}", snap_geom);
        let geom = snap_geom.unwrap();
        assert_eq!(geom.page_index, 0);
        assert!((geom.width_pt - 612.0).abs() < 1e-3);
        assert!((geom.height_pt - 792.0).abs() < 1e-3);
        assert!(geom.segments.len() >= 4);

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
        let tmp_dir = std::env::temp_dir().join(format!("lpdf_studio_text_{}", std::process::id()));
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

    #[test]
    fn test_studio_extract_and_edit_text_blocks_roundtrip() {
        let tmp_dir =
            std::env::temp_dir().join(format!("lpdf_studio_text_rt_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp_dir);
        let input_pdf = tmp_dir.join("input_rt.pdf");
        let output_pdf = tmp_dir.join("output_rt.pdf");
        create_sample_pdf(&input_pdf);

        let initial_blocks = studio_extract_text_blocks(input_pdf.to_string_lossy().to_string(), 0);
        assert!(
            initial_blocks.is_ok(),
            "initial extraction failed: {:?}",
            initial_blocks
        );

        let edit_res = studio_edit_pdf_text_block(
            input_pdf.to_string_lossy().to_string(),
            output_pdf.to_string_lossy().to_string(),
            0,
            [70.0, 700.0, 200.0, 40.0],
            "Architectural Structural Revision A1".to_string(),
            14.0,
            [20, 30, 40],
        );
        assert!(edit_res.is_ok(), "text block edit failed: {:?}", edit_res);

        let extracted = studio_extract_text_blocks(output_pdf.to_string_lossy().to_string(), 0);
        assert!(
            extracted.is_ok(),
            "extract text blocks failed: {:?}",
            extracted
        );
        let blocks = extracted.unwrap();
        assert!(!blocks.is_empty(), "extracted blocks should not be empty");
        let found = blocks
            .iter()
            .any(|b| b.text.contains("Architectural Structural Revision"));
        assert!(
            found,
            "newly edited text block should be found in extracted blocks"
        );

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn test_studio_d6_audio_takeoff_csv_and_markup_library() {
        let tmp_dir = std::env::temp_dir().join(format!("lpdf_studio_d6_{}", std::process::id()));
        let _ = fs::create_dir_all(&tmp_dir);
        std::env::set_var("LPDF_STUDIO_CONFIG_DIR", &tmp_dir);

        let input_pdf = tmp_dir.join("input_d6.pdf");
        let output_audio_pdf = tmp_dir.join("output_audio_d6.pdf");
        let csv_path = tmp_dir.join("takeoff_d6.csv");
        create_sample_pdf(&input_pdf);

        // 1. Audio attachment test (minimal RIFF WAV header + samples)
        let fake_wav = b"RIFF$\x00\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x01\x00D\xac\x00\x00\x88X\x01\x00\x02\x00\x10\x00data\x00\x00\x00\x00".to_vec();
        let audio_res = studio_attach_audio(
            input_pdf.to_string_lossy().to_string(),
            output_audio_pdf.to_string_lossy().to_string(),
            0,
            144.0,
            520.0,
            fake_wav,
        );
        assert!(audio_res.is_ok(), "audio attach failed: {:?}", audio_res);
        assert_eq!(audio_res.unwrap().page_count, 1);

        // 2. Quantity Takeoff CSV export test (including formula injection guard)
        let records = vec![
            MeasurementRecord {
                id: "M-001".to_string(),
                page_index: 0,
                measurement_type: "measure.distance".to_string(),
                raw_pt: 144.0,
                calibrated_value: 2.0,
                unit: "m".to_string(),
                color: "var(--measure-line)".to_string(),
                label: Some("=SUM(A1:A2)".to_string()),
            },
            MeasurementRecord {
                id: "M-002".to_string(),
                page_index: 0,
                measurement_type: "measure.distance".to_string(),
                raw_pt: 216.0,
                calibrated_value: 3.0,
                unit: "m".to_string(),
                color: "var(--measure-line)".to_string(),
                label: Some("North Wall".to_string()),
            },
        ];
        let csv_res =
            studio_export_takeoff_csv(records, csv_path.to_string_lossy().to_string(), None, None);
        assert!(csv_res.is_ok(), "csv export failed: {:?}", csv_res);
        let csv_content = fs::read_to_string(&csv_path).unwrap();
        assert!(csv_content.contains("\"ID\",\"Page\",\"Type\""));
        assert!(csv_content.contains("\"'=SUM(A1:A2)'\""));
        assert!(csv_content.contains("\"TOTAL\",\"\",\"measure.distance\",\"\",5.0000,\"m\""));

        // 3. Markup Library roundtrip test
        let item = MarkupLibraryItemDto {
            id: "lib-item-1".to_string(),
            title: "Structural Approval Stamp".to_string(),
            category: "Stamps".to_string(),
            tool_id: "insert.stamp".to_string(),
            svg_path: "M 0 0 L 100 0 L 100 40 L 0 40 Z".to_string(),
            color_token: "--accent".to_string(),
            stroke_width: 2.0,
            latex_source: None,
            text_label: Some("APPROVED".to_string()),
            data_url: None,
            created_at_ms: 1700000000000,
        };
        let payload = serde_json::to_string(&item).unwrap();
        let save_lib = studio_markup_library("save".to_string(), Some(payload));
        assert!(save_lib.is_ok());
        let list_lib = studio_markup_library("list".to_string(), None).unwrap();
        assert!(list_lib.contains("Structural Approval Stamp"));

        let del_lib =
            studio_markup_library("delete".to_string(), Some("lib-item-1".to_string())).unwrap();
        assert_eq!(del_lib.trim(), "[]");

        // 4. Phase D6.1: Workspace Document Library (7.10), Watch Folder Auto-Ingest (8.7) & Table/Field Detection (8.12)
        let doc_entry = WorkspaceDocumentEntryDto {
            path: input_pdf.to_string_lossy().to_string(),
            title: "Structural Plan Set A".to_string(),
            folder: "Structural".to_string(),
            tags: vec!["Blueprint".to_string(), "IFC".to_string()],
            favorite: true,
            last_opened_ms: 1700000005000,
        };
        let doc_json = serde_json::to_string(&doc_entry).unwrap();
        let saved_docs = studio_workspace_library("upsert".to_string(), Some(doc_json)).unwrap();
        assert!(saved_docs.contains("Structural Plan Set A"));

        let scanned = studio_scan_watch_folder(tmp_dir.to_string_lossy().to_string()).unwrap();
        assert!(scanned.iter().any(|p| p.ends_with("input_d6.pdf")));

        // BUG-023: the one-line sample page has no table and no field, so nothing is reported
        // (it used to return a fabricated "auto" field).
        let detected =
            studio_detect_tables_and_fields(input_pdf.to_string_lossy().to_string(), 0).unwrap();
        assert!(
            detected.iter().all(|d| !d.id.ends_with("-auto")),
            "no fabricated regions: {detected:?}"
        );

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    // ---------------------------------------------------------------------------------------
    // ADR-018 R2 step 1: characterization goldens of CURRENT behaviour (before any engine swap).
    // Regenerate only for an intended, explained change: `LPDF_UPDATE_GOLDENS=1 cargo nextest run -p app-tauri golden_`.
    // ---------------------------------------------------------------------------------------

    fn manifest_path(rel: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
    }

    fn corpus_pdf(name: &str) -> PathBuf {
        let p = manifest_path("../../corpus").join(name);
        assert!(
            p.exists(),
            "corpus file {} missing: run corpus/fetch.sh",
            p.display()
        );
        p
    }

    /// Compare `actual` with the golden file at `rel` (relative to this crate), or write it when
    /// `LPDF_UPDATE_GOLDENS` is set. Numbers are written by serde_json (shortest round-trip), so
    /// equal text means bit-equal values.
    fn check_golden(rel: &str, actual: &str) {
        let path = manifest_path(rel);
        if std::env::var_os("LPDF_UPDATE_GOLDENS").is_some() {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).unwrap();
            }
            fs::write(&path, actual).unwrap();
        }
        let expected = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "missing golden {} ({e}); run with LPDF_UPDATE_GOLDENS=1",
                path.display()
            )
        });
        assert!(
            expected == actual,
            "output drifted from golden {} (diff it after LPDF_UPDATE_GOLDENS=1 to review)",
            path.display()
        );
    }

    fn num(v: f64) -> String {
        serde_json::to_string(&v).unwrap()
    }

    /// Snap geometry extracted from corpus/standard-form.pdf (IRS W-4) page 1. Also the input of
    /// the TypeScript snap goldens (`src/lib/core/__fixtures__`).
    #[test]
    fn golden_snap_geometry_standard_form_p0() {
        let pdf = corpus_pdf("standard-form.pdf");
        let geom = studio_extract_snap_geometry(pdf.to_string_lossy().to_string(), 0).unwrap();
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str("  \"source\": \"corpus/standard-form.pdf sha256 92444d8856ce55d9e25dca8b6d1420634fc68b11e1ab1f760916ea29ddd312b2\",\n");
        out.push_str("  \"producer\": \"studio_extract_snap_geometry(page 0) @ 562e4f9 (pdfcraft-measure measurement_paths, 4096 cap)\",\n");
        out.push_str(&format!("  \"pageIndex\": {},\n", geom.page_index));
        out.push_str(&format!("  \"widthPt\": {},\n", num(geom.width_pt)));
        out.push_str(&format!("  \"heightPt\": {},\n", num(geom.height_pt)));
        out.push_str(&format!("  \"segmentCount\": {},\n", geom.segments.len()));
        out.push_str("  \"segments\": [\n");
        for (i, s) in geom.segments.iter().enumerate() {
            let comma = if i + 1 < geom.segments.len() { "," } else { "" };
            out.push_str(&format!(
                "    [{}, {}, {}, {}]{}\n",
                num(s.x1),
                num(s.y1),
                num(s.x2),
                num(s.y2),
                comma
            ));
        }
        out.push_str("  ]\n}\n");
        check_golden(
            "../src/lib/core/__fixtures__/snap-geometry.standard-form.p0.golden.json",
            &out,
        );
    }

    /// Text blocks of corpus/standard-text.pdf page 1, and the blocks after an in-place edit of
    /// the first block (`studio_edit_pdf_text_block`).
    #[test]
    fn golden_text_blocks_extract_and_edit_standard_text() {
        let pdf = corpus_pdf("standard-text.pdf");
        let before = studio_extract_text_blocks(pdf.to_string_lossy().to_string(), 0).unwrap();
        assert!(!before.is_empty(), "standard-text.pdf page 1 has text");

        let tmp_dir = std::env::temp_dir().join(format!("lpdf_golden_text_{}", std::process::id()));
        fs::create_dir_all(&tmp_dir).unwrap();
        let out_pdf = tmp_dir.join("edited.pdf");
        let edit = studio_edit_pdf_text_block(
            pdf.to_string_lossy().to_string(),
            out_pdf.to_string_lossy().to_string(),
            0,
            before[0].bbox_pt,
            "LocalPDF golden replacement text".to_string(),
            12.0,
            [0, 0, 0],
        )
        .unwrap();
        let after = studio_extract_text_blocks(out_pdf.to_string_lossy().to_string(), 0).unwrap();
        let _ = fs::remove_dir_all(&tmp_dir);

        let golden = serde_json::json!({
            "source": "corpus/standard-text.pdf sha256 3df79d34abbca99308e79cb94461c1893582604d68329a41fd4bec1885e6adb4",
            "producer": "studio_extract_text_blocks / studio_edit_pdf_text_block @ 562e4f9 (pdfcraft text_blocks, EditTextBlock or AddText fallback)",
            "before": before,
            "edit": { "bboxPt": before[0].bbox_pt, "newText": "LocalPDF golden replacement text", "fontSize": 12.0, "pageCount": edit.page_count, "savedCount": edit.saved_count },
            "after": after,
        });
        let text = format!("{}\n", serde_json::to_string_pretty(&golden).unwrap());
        check_golden(
            "tests/goldens/text-blocks.standard-text.p0.golden.json",
            &text,
        );
    }

    /// Quantity takeoff CSV bytes (header, rows, formula-injection guard, per-unit totals).
    #[test]
    fn golden_takeoff_csv() {
        let rec =
            |id: &str, page: u32, t: &str, raw: f64, cal: f64, unit: &str, label: Option<&str>| {
                MeasurementRecord {
                    id: id.to_string(),
                    page_index: page,
                    measurement_type: t.to_string(),
                    raw_pt: raw,
                    calibrated_value: cal,
                    unit: unit.to_string(),
                    color: "var(--measure-line)".to_string(),
                    label: label.map(str::to_string),
                }
            };
        let records = vec![
            rec("M-001", 0, "length", 144.0, 2.0, "m", Some("North wall")),
            rec(
                "",
                0,
                "length",
                216.123456,
                3.0017146,
                "m",
                Some("=SUM(A1:A2)"),
            ),
            rec(
                "A-1",
                1,
                "area.polygon",
                7200.5,
                1.3889853395061728,
                "m²",
                Some("Slab \"B\""),
            ),
            rec("A-2", 1, "area.rect", 5184.0, 1.0, "m²", None),
            rec(
                "P-1",
                2,
                "perimeter",
                360.0,
                5.0,
                "ft",
                Some("+cmd|' /C calc'!A0"),
            ),
            rec("D-1", 2, "protractor", 90.0, 90.0, "deg", Some("@angle")),
            rec(
                "X-1",
                3,
                "length",
                f64::NAN,
                f64::NAN,
                "m",
                Some("-not a number"),
            ),
        ];
        let tmp_dir = std::env::temp_dir().join(format!("lpdf_golden_csv_{}", std::process::id()));
        fs::create_dir_all(&tmp_dir).unwrap();
        let csv_path = tmp_dir.join("takeoff.csv");
        studio_export_takeoff_csv(records, csv_path.to_string_lossy().to_string(), None, None)
            .unwrap();
        let csv = fs::read_to_string(&csv_path).unwrap();
        let _ = fs::remove_dir_all(&tmp_dir);
        let golden = serde_json::json!({
            "producer": "studio_export_takeoff_csv @ 562e4f9",
            "csv": csv,
        });
        let text = format!("{}\n", serde_json::to_string_pretty(&golden).unwrap());
        check_golden("tests/goldens/takeoff.golden.json", &text);
    }

    /// `studio_save_annotations` hatches through the lpdf-geom adapter, byte-identical to the
    /// Studio golden. ERR-007 intentional change, approved by owner 2026-11-06: this used to pin
    /// `hatch_legacy.golden.json` (bounding-box hatching); it now pins the clipped Studio engine.
    #[test]
    fn golden_studio_hatch_goes_through_adapter() {
        let raw = fs::read_to_string(manifest_path(
            "../../crates/lpdf-geom/tests/goldens/hatch_studio.golden.json",
        ))
        .unwrap();
        let golden: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let cases = golden["cases"].as_array().unwrap();
        assert!(!cases.is_empty());
        for case in cases {
            let name = case["name"].as_str().unwrap();
            let pts: Vec<StudioPointDto> = case["loop"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| StudioPointDto {
                    x: p[0].as_f64().unwrap(),
                    y: p[1].as_f64().unwrap(),
                    p: 0.5,
                    t: None,
                })
                .collect();
            let rect = bbox_from_points_or_rect(&pts, [0.0; 4]);
            let strokes = hatch_strokes_for(
                case["pattern"].as_str().unwrap(),
                &pts,
                rect,
                case["scale"].as_f64().unwrap(),
            )
            .unwrap()
            .unwrap();
            // serde_json (no float_roundtrip) can read a long decimal one ulp off, so compare
            // the numbers, not their text.
            let want: Vec<[f64; 4]> = case["segments"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| {
                    [
                        s[0].as_f64().unwrap(),
                        s[1].as_f64().unwrap(),
                        s[2].as_f64().unwrap(),
                        s[3].as_f64().unwrap(),
                    ]
                })
                .collect();
            assert_eq!(strokes.len(), want.len(), "{name}: segment count");
            for (s, w) in strokes.iter().zip(want.iter()) {
                let got = [s[0][0], s[0][1], s[1][0], s[1][1]];
                for (g, w) in got.iter().zip(w.iter()) {
                    assert!((g - w).abs() < 1e-9, "{name}: {got:?} vs {w:?}");
                }
            }
        }
        // Rect-only annotations (no points) hatch their normalised bbox_pt.
        let rect = bbox_from_points_or_rect(&[], [100.0, 100.0, 240.0, 200.0]);
        let from_rect = hatch_strokes_for("timber", &[], rect, 1.0)
            .unwrap()
            .unwrap();
        assert!(from_rect.len() > 8, "{} timber lines", from_rect.len());
        assert!(from_rect
            .iter()
            .flatten()
            .all(|p| (100.0..=240.0).contains(&p[0]) && (100.0..=200.0).contains(&p[1])));
        // Fills and unknown ids draw no line hatch.
        assert_eq!(hatch_strokes_for("solid", &[], rect, 1.0).unwrap(), None);
        assert_eq!(hatch_strokes_for("ANSI31", &[], rect, 1.0).unwrap(), None);
    }

    /// GEOM_NO_FINITE_GEOMETRY: a region without one finite point draws no hatch, as the
    /// pre-adapter code did (it hatched a NaN rect, which emitted no lines); save does not fail.
    #[test]
    fn hatch_without_finite_geometry_draws_nothing() {
        let nan_pts = vec![StudioPointDto {
            x: f64::NAN,
            y: 1.0,
            p: 0.5,
            t: None,
        }];
        let nan_rect = [f64::NAN; 4];
        assert_eq!(
            hatch_strokes_for("timber", &nan_pts, nan_rect, 1.0),
            Ok(None)
        );
        assert_eq!(
            hatch_strokes_for("crosshatch", &[], nan_rect, 1.0),
            Ok(None)
        );
    }

    /// GEOM_WORK_LIMIT: a region too large to hatch fails the save with a coded message instead
    /// of hanging (the pre-adapter code looped without bound).
    #[test]
    fn hatch_work_limit_is_an_error_with_code() {
        let rect = [0.0, 0.0, 1.0e9, 1.0e9];
        let err = hatch_strokes_for("timber", &[], rect, 1.0).unwrap_err();
        assert_eq!(err.code(), "GEOM_WORK_LIMIT");
    }
}
