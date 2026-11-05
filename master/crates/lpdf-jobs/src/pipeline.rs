//! Pipeline step definitions and built-in workflow presets.

use serde::{Deserialize, Serialize};

/// Represents a single chainable PDF transformation step.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "type")]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum PipelineStep {
    Repair,
    Compress {
        profile: lpdf_engine_port::OptimizeProfile,
    },
    Ocr,
    Rotate {
        degrees: i32,
        pages: Vec<u32>,
    },
    FlattenComments,
    FlattenForms,
    Watermark {
        settings: lpdf_engine_port::WatermarkSettings,
    },
    Protect {
        password: String,
        permissions: lpdf_engine_port::Permissions,
    },
    PdfToPdfA {
        conformance: lpdf_engine_port::PdfAConformance,
    },
}

/// A named workflow preset comprising ordered transformation steps.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct PipelinePreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<PipelineStep>,
}

/// Built-in pipeline presets equivalent to iLovePDF Workflows.
pub fn builtin_pipeline_presets() -> Vec<PipelinePreset> {
    vec![
        PipelinePreset {
            id: "archive_searchable".to_string(),
            name: "Archive & Searchable (OCR + PDF/A-2b)".to_string(),
            description: "Repairs document structure, performs OCR text recognition, optimizes stream compression, and converts to archival PDF/A-2b.".to_string(),
            steps: vec![
                PipelineStep::Repair,
                PipelineStep::Ocr,
                PipelineStep::Compress {
                    profile: lpdf_engine_port::OptimizeProfile::Medium,
                },
                PipelineStep::PdfToPdfA {
                    conformance: lpdf_engine_port::PdfAConformance::PdfA2b,
                },
            ],
        },
        PipelinePreset {
            id: "confidential_release".to_string(),
            name: "Flatten & Confidential Release".to_string(),
            description: "Flattens interactive form fields and reviewer comments, applies a prominent diagonal CONFIDENTIAL watermark, and optimizes size.".to_string(),
            steps: vec![
                PipelineStep::FlattenForms,
                PipelineStep::FlattenComments,
                PipelineStep::Watermark {
                    settings: lpdf_engine_port::WatermarkSettings {
                        text: "CONFIDENTIAL".to_string(),
                        opacity: 0.35,
                        rotation: 45.0,
                        color: [0.8, 0.2, 0.2],
                    },
                },
                PipelineStep::Compress {
                    profile: lpdf_engine_port::OptimizeProfile::Medium,
                },
            ],
        },
        PipelinePreset {
            id: "clean_web_optimize".to_string(),
            name: "Clean & Web Optimize".to_string(),
            description: "Repairs damaged xref tables, flattens review comments, and applies maximum compression for rapid web rendering.".to_string(),
            steps: vec![
                PipelineStep::Repair,
                PipelineStep::FlattenComments,
                PipelineStep::Compress {
                    profile: lpdf_engine_port::OptimizeProfile::High,
                },
            ],
        },
    ]
}
