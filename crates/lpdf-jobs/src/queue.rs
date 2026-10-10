//! Background job queue with bounded concurrency and cancellation.
//!
//! Jobs are dispatched to `tokio::task::spawn_blocking` so the async
//! runtime is never starved. Each job gets a `CancellationToken` for
//! cooperative cancellation.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use lpdf_core_types::LpdfError;
use lpdf_engine_port::PdfEngine;
use lpdf_engine_switcher::EngineSwitcher;

use crate::events::{JobEvent, JobEventSender};

/// Unique identifier for a job, based on UUID v4.
pub type JobId = String;

/// Current status of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Describes the operation a job should perform.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "op")]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum JobRequest {
    /// Load a PDF and retrieve its info.
    GetInfo { input_path: String },

    /// Delete pages from a PDF and save to a new file.
    DeletePages {
        input_path: String,
        output_path: String,
        pages: Vec<usize>,
    },

    /// Render a page to an image file.
    RenderPage {
        input_path: String,
        output_path: String,
        page_index: usize,
        dpi: u32,
    },

    /// Merge multiple PDFs into a single document.
    Merge {
        input_paths: Vec<String>,
        output_path: String,
    },

    /// Extract specific pages from a PDF.
    ExtractPages {
        input_path: String,
        output_path: String,
        pages: Vec<usize>, // 1-indexed for the user interface
    },

    /// Rotate specific pages in a PDF.
    RotatePages {
        input_path: String,
        output_path: String,
        pages: Vec<usize>, // 1-indexed
        degrees: i32,
    },

    /// Rotate an entire PDF document.
    RotateDocument {
        input_path: String,
        output_path: String,
        degrees: i32,
    },

    /// Compress a PDF document.
    Compress {
        input_path: String,
        output_path: String,
        profile: lpdf_engine_port::OptimizeProfile,
    },

    /// Repair a damaged PDF document.
    Repair {
        input_path: String,
        output_path: String,
    },

    /// Extract text using OCR.
    Ocr {
        input_path: String,
        output_path: String,
        lang: String,
    },

    /// Protect a PDF with a password and permissions.
    ProtectDocument {
        input_path: String,
        output_path: String,
        password: String,
        permissions: lpdf_engine_port::Permissions,
    },

    /// Unlock a password-protected PDF.
    UnlockDocument {
        input_path: String,
        output_path: String,
        password: String,
    },

    /// Add an annotation to the document.
    AnnotateDocument {
        input_path: String,
        output_path: String,
        annotation: lpdf_engine_port::AnnotationDef,
    },

    /// Add a watermark to the document.
    WatermarkDocument {
        input_path: String,
        output_path: String,
        settings: lpdf_engine_port::WatermarkSettings,
    },

    /// Flatten comments into the page content.
    FlattenComments {
        input_path: String,
        output_path: String,
    },

    /// Flatten form fields into the page content.
    FlattenForms {
        input_path: String,
        output_path: String,
    },

    /// Fill form fields with values.
    FillForm {
        input_path: String,
        output_path: String,
        fields: std::collections::HashMap<String, String>,
    },

    SignDocument {
        input_path: String,
        output_path: String,
        p12_path: String,
        password: String,
    },

    /// Apply semantic redactions to the document.
    RedactDocument {
        input_path: String,
        output_path: String,
        regions: Vec<lpdf_engine_port::RedactionRegion>,
    },
}

/// A tracked job entry in the manager.
#[derive(Debug)]
struct JobEntry {
    status: JobStatus,
    cancel_token: CancellationToken,
}

/// Manages a bounded queue of async PDF jobs.
///
/// Concurrency is controlled by a semaphore whose size is set based on
/// the hardware tier (Phase 1.5).
pub struct JobManager {
    jobs: Arc<Mutex<HashMap<JobId, JobEntry>>>,
    semaphore: Arc<Semaphore>,
    event_tx: JobEventSender,
}

impl JobManager {
    /// Create a new job manager.
    ///
    /// `max_concurrent` controls the number of jobs that may run
    /// simultaneously (typically derived from the hardware tier).
    pub fn new(max_concurrent: usize, event_tx: JobEventSender) -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            event_tx,
        }
    }

    /// Submit a job request. Returns the job ID immediately (< 1ms).
    pub fn submit(&self, request: JobRequest) -> JobId {
        let job_id: JobId = uuid::Uuid::new_v4().to_string();
        let cancel_token = CancellationToken::new();

        {
            let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
            jobs.insert(
                job_id.clone(),
                JobEntry {
                    status: JobStatus::Queued,
                    cancel_token: cancel_token.clone(),
                },
            );
        }

        let _ = self.event_tx.send(JobEvent::Queued {
            job_id: job_id.clone(),
        });

        // Spawn the job on the async runtime
        let jobs_map = Arc::clone(&self.jobs);
        let semaphore = Arc::clone(&self.semaphore);
        let event_tx = Arc::clone(&self.event_tx);
        let jid = job_id.clone();

        tokio::spawn(async move {
            // Acquire a permit (respects concurrency limit)
            let _permit = semaphore.acquire().await;

            // Check if cancelled while waiting
            if cancel_token.is_cancelled() {
                Self::set_status(&jobs_map, &jid, JobStatus::Cancelled);
                let _ = event_tx.send(JobEvent::Cancelled {
                    job_id: jid.clone(),
                });
                return;
            }

            Self::set_status(&jobs_map, &jid, JobStatus::Running);
            let _ = event_tx.send(JobEvent::Started {
                job_id: jid.clone(),
            });

            // Run the blocking PDF work on a dedicated thread
            let result = tokio::task::spawn_blocking({
                let jid = jid.clone();
                let event_tx = Arc::clone(&event_tx);
                let cancel_token = cancel_token.clone();
                move || Self::execute_job(&jid, &request, &event_tx, &cancel_token)
            })
            .await;

            match result {
                Ok(Ok(output_info)) => {
                    Self::set_status(&jobs_map, &jid, JobStatus::Completed);
                    let _ = event_tx.send(JobEvent::Completed {
                        job_id: jid,
                        output_path: output_info.output_path,
                        degraded: output_info.degraded,
                    });
                }
                Ok(Err(e)) => {
                    Self::set_status(&jobs_map, &jid, JobStatus::Failed);
                    let _ = event_tx.send(JobEvent::Failed {
                        job_id: jid,
                        error_code: Self::error_code(&e),
                        error_message: e.to_string(),
                    });
                }
                Err(_join_err) => {
                    Self::set_status(&jobs_map, &jid, JobStatus::Failed);
                    let _ = event_tx.send(JobEvent::Failed {
                        job_id: jid,
                        error_code: "TASK_PANIC".into(),
                        error_message: "Job task panicked".into(),
                    });
                }
            }
        });

        job_id
    }

    /// Request cancellation of a running or queued job.
    pub fn cancel(&self, job_id: &str) -> bool {
        let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = jobs.get(job_id) {
            entry.cancel_token.cancel();
            true
        } else {
            false
        }
    }

    /// Get the current status of a job.
    pub fn status(&self, job_id: &str) -> Option<JobStatus> {
        let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.get(job_id).map(|e| e.status)
    }

    fn set_status(jobs: &Arc<Mutex<HashMap<JobId, JobEntry>>>, job_id: &str, status: JobStatus) {
        let mut jobs = jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = jobs.get_mut(job_id) {
            entry.status = status;
        }
    }

    fn error_code(err: &LpdfError) -> String {
        match err {
            LpdfError::Engine(_) => "ENGINE_ERROR".into(),
            LpdfError::Io(_) => "IO_ERROR".into(),
            LpdfError::InvalidInput(_) => "INVALID_INPUT".into(),
            LpdfError::ParseError(_) => "PARSE_ERROR".into(),
            LpdfError::ConfigError(_) => "CONFIG_ERROR".into(),
            LpdfError::Unsupported(_) => "UNSUPPORTED".into(),
            LpdfError::OcrInferenceError(_) => "OCR_INFERENCE_ERROR".into(),
        }
    }

    /// Execute the actual PDF work (runs on a blocking thread).
    fn execute_job(
        job_id: &str,
        request: &JobRequest,
        event_tx: &JobEventSender,
        cancel_token: &CancellationToken,
    ) -> lpdf_core_types::Result<JobOutput> {
        match request {
            JobRequest::GetInfo { input_path } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Loading document...".into(),
                });

                let mut engine = EngineSwitcher::new();
                engine.load(&PathBuf::from(input_path))?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 80,
                    message: "Reading metadata...".into(),
                });

                let _info = engine.info()?;

                Ok(JobOutput {
                    output_path: None,
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::DeletePages {
                input_path,
                output_path,
                pages,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting delete operation...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Deleting pages...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.delete_pages(&path, pages, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::RenderPage {
                input_path,
                output_path,
                page_index,
                dpi,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Loading document...".into(),
                });

                let mut engine = EngineSwitcher::new();
                engine.load(&PathBuf::from(input_path))?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Rendering page...".into(),
                });

                let opts = lpdf_engine_port::RenderOptions {
                    dpi: *dpi,
                    format: "png".into(),
                };
                let out = PathBuf::from(output_path);
                engine.render(*page_index, &opts, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::Merge {
                input_paths,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting merge operation...".into(),
                });

                // Merge operation does not require loading a document first
                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Merging documents...".into(),
                });

                let paths: Vec<&Path> = input_paths.iter().map(Path::new).collect();
                let out = PathBuf::from(output_path);

                engine.merge(&paths, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::ExtractPages {
                input_path,
                output_path,
                pages,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting extraction operation...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Extracting pages...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.extract_pages(&path, pages, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::RotatePages {
                input_path,
                output_path,
                pages,
                degrees,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting rotate operation...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Rotating pages...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.rotate_pages(&path, pages, *degrees, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::RotateDocument {
                input_path,
                output_path,
                degrees,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting document rotation...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Rotating document...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.rotate_document(&path, *degrees, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::Compress {
                input_path,
                output_path,
                profile,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting compression...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Compressing document...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.compress(&path, *profile, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::Repair {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting repair...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Repairing document...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                let log = engine.repair(&path, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 90,
                    message: format!("Repair complete. {} issues fixed.", log.len()),
                });

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::Ocr {
                input_path,
                output_path,
                lang,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting OCR...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Recognizing text...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.ocr(&path, lang, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::ProtectDocument {
                input_path,
                output_path,
                password,
                permissions,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting protection...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Encrypting document...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.protect(&path, password, permissions.clone(), &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::UnlockDocument {
                input_path,
                output_path,
                password,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Starting unlock...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Decrypting document...".into(),
                });

                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.unlock(&path, password, &out)?;

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::AnnotateDocument {
                input_path,
                output_path,
                annotation,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Applying annotation...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.add_annotation(&path, annotation, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::WatermarkDocument {
                input_path,
                output_path,
                settings,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Adding watermark...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.add_watermark(&path, settings, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::FlattenComments {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Flattening comments...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.flatten_comments(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::FlattenForms {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Flattening forms...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.flatten_forms(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::FillForm {
                input_path,
                output_path,
                fields,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Filling form...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.fill_form(&path, fields, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::SignDocument {
                input_path,
                output_path,
                p12_path,
                password,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Signing document...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let p12 = PathBuf::from(p12_path);
                let out = PathBuf::from(output_path);

                engine.sign_document(&path, &p12, password, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::RedactDocument {
                input_path,
                output_path,
                regions,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Applying redactions...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.redact(&path, regions.clone(), &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }
        }
    }
}

/// Output metadata from a completed job.
struct JobOutput {
    output_path: Option<String>,
    degraded: bool,
}
