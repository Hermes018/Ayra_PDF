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

    ImageToPdf {
        input_path: String,
        output_path: String,
    },
    DocxToPdf {
        input_path: String,
        output_path: String,
    },
    HtmlToPdf {
        input_path: String,
        output_path: String,
    },

    PdfToImages {
        input_path: String,
        output_path: String,
    },
    ExtractAllImages {
        input_path: String,
        output_path: String,
    },
    PdfToText {
        input_path: String,
        output_path: String,
    },
    PdfToDocx {
        input_path: String,
        output_path: String,
    },
    PdfToHtml {
        input_path: String,
        output_path: String,
    },
    PdfToRtf {
        input_path: String,
        output_path: String,
    },
    PdfToPdfA {
        input_path: String,
        output_path: String,
        conformance: lpdf_engine_port::PdfAConformance,
    },
    OfficeToPdf {
        input_path: String,
        output_path: String,
        format: lpdf_engine_port::OfficeInputFormat,
    },
    PdfToOffice {
        input_path: String,
        output_path: String,
        format: lpdf_engine_port::OfficeOutputFormat,
    },
    MarkdownToPdf {
        input_path: String,
        output_path: String,
    },
    PdfToMarkdown {
        input_path: String,
        output_path: String,
    },
    MarkdownToDoc {
        input_path: String,
        output_path: String,
        format: lpdf_engine_port::MarkdownDocFormat,
    },
    DocToMarkdown {
        input_path: String,
        output_path: String,
        format: lpdf_engine_port::MarkdownDocFormat,
    },
    BatchMarkdownConvert {
        input_dir: String,
        output_dir: String,
        options: lpdf_engine_port::MarkdownBatchOptions,
    },
    RunPipeline {
        input_path: String,
        output_path: String,
        steps: Vec<crate::pipeline::PipelineStep>,
        preset_name: Option<String>,
    },
    SummarizeExtractive {
        input_path: PathBuf,
        output_path: PathBuf,
        length: lpdf_engine_port::SummaryLength,
    },
}

/// Execution priority for queued jobs.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, ts_rs::TS,
)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum JobPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// Comprehensive record of a background job for queue inspection and control.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub struct JobRecord {
    pub job_id: JobId,
    pub title: String,
    pub status: JobStatus,
    pub progress: u8,
    pub priority: JobPriority,
    pub input_path: Option<String>,
    pub output_path: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub created_at_epoch_ms: u64,
    pub request: JobRequest,
}

impl JobRequest {
    /// Returns (title, input_path, output_path) for summary display.
    pub fn summary(&self) -> (String, Option<String>, Option<String>) {
        match self {
            JobRequest::GetInfo { input_path } => {
                ("Document Info".to_string(), Some(input_path.clone()), None)
            }
            JobRequest::DeletePages {
                input_path,
                output_path,
                ..
            } => (
                "Delete Pages".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::RenderPage {
                input_path,
                output_path,
                ..
            } => (
                "Render Page".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::Merge {
                input_paths,
                output_path,
            } => {
                let first = input_paths.first().cloned();
                ("Merge PDFs".to_string(), first, Some(output_path.clone()))
            }
            JobRequest::ExtractPages {
                input_path,
                output_path,
                ..
            } => (
                "Extract Pages".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::RotatePages {
                input_path,
                output_path,
                ..
            } => (
                "Rotate Pages".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::RotateDocument {
                input_path,
                output_path,
                ..
            } => (
                "Rotate Document".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::Compress {
                input_path,
                output_path,
                ..
            } => (
                "Compress PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::Repair {
                input_path,
                output_path,
            } => (
                "Repair PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::Ocr {
                input_path,
                output_path,
                ..
            } => (
                "OCR Document".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::ProtectDocument {
                input_path,
                output_path,
                ..
            } => (
                "Protect PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::UnlockDocument {
                input_path,
                output_path,
                ..
            } => (
                "Unlock PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::AnnotateDocument {
                input_path,
                output_path,
                ..
            } => (
                "Annotate PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::WatermarkDocument {
                input_path,
                output_path,
                ..
            } => (
                "Watermark PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::FlattenComments {
                input_path,
                output_path,
            } => (
                "Flatten Comments".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::FlattenForms {
                input_path,
                output_path,
            } => (
                "Flatten Forms".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::FillForm {
                input_path,
                output_path,
                ..
            } => (
                "Fill Form".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::SignDocument {
                input_path,
                output_path,
                ..
            } => (
                "Sign PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::RedactDocument {
                input_path,
                output_path,
                ..
            } => (
                "Redact PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::ImageToPdf {
                input_path,
                output_path,
            } => (
                "Image to PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::DocxToPdf {
                input_path,
                output_path,
            } => (
                "DOCX to PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::HtmlToPdf {
                input_path,
                output_path,
            } => (
                "HTML to PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToImages {
                input_path,
                output_path,
            } => (
                "PDF to Images".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::ExtractAllImages {
                input_path,
                output_path,
            } => (
                "Extract Images".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToText {
                input_path,
                output_path,
            } => (
                "PDF to Text".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToDocx {
                input_path,
                output_path,
            } => (
                "PDF to DOCX".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToHtml {
                input_path,
                output_path,
            } => (
                "PDF to HTML".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToRtf {
                input_path,
                output_path,
            } => (
                "PDF to RTF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToPdfA {
                input_path,
                output_path,
                ..
            } => (
                "PDF to PDF/A".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::OfficeToPdf {
                input_path,
                output_path,
                ..
            } => (
                "Office to PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToOffice {
                input_path,
                output_path,
                ..
            } => (
                "PDF to Office".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::MarkdownToPdf {
                input_path,
                output_path,
            } => (
                "Markdown to PDF".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::PdfToMarkdown {
                input_path,
                output_path,
            } => (
                "PDF to Markdown".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::MarkdownToDoc {
                input_path,
                output_path,
                ..
            } => (
                "Markdown to Document".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::DocToMarkdown {
                input_path,
                output_path,
                ..
            } => (
                "Document to Markdown".to_string(),
                Some(input_path.clone()),
                Some(output_path.clone()),
            ),
            JobRequest::BatchMarkdownConvert {
                input_dir,
                output_dir,
                ..
            } => (
                "Batch Markdown Convert".to_string(),
                Some(input_dir.clone()),
                Some(output_dir.clone()),
            ),
            JobRequest::RunPipeline {
                input_path,
                output_path,
                steps,
                preset_name,
            } => {
                let title = match preset_name {
                    Some(name) => format!("Workflow: {}", name),
                    None => format!("Pipeline ({} steps)", steps.len()),
                };
                (title, Some(input_path.clone()), Some(output_path.clone()))
            }
            JobRequest::SummarizeExtractive {
                input_path,
                output_path,
                ..
            } => (
                "Summarize (Extractive)".to_string(),
                Some(input_path.to_string_lossy().to_string()),
                Some(output_path.to_string_lossy().to_string()),
            ),
        }
    }
}

/// A tracked job entry in the manager.
#[derive(Debug)]
struct JobEntry {
    record: JobRecord,
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

    /// Submit a job request with default normal priority. Returns the job ID immediately (< 1ms).
    pub fn submit(&self, request: JobRequest) -> JobId {
        self.submit_with_priority(request, JobPriority::Normal)
    }

    /// Submit a job request with an explicit priority. Returns the job ID immediately.
    pub fn submit_with_priority(&self, request: JobRequest, priority: JobPriority) -> JobId {
        let job_id: JobId = uuid::Uuid::new_v4().to_string();
        let cancel_token = CancellationToken::new();

        let (title, input_path, output_path) = request.summary();
        let now_epoch_ms = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        {
            Ok(dur) => dur.as_millis() as u64,
            Err(_) => 0,
        };

        let record = JobRecord {
            job_id: job_id.clone(),
            title,
            status: JobStatus::Queued,
            progress: 0,
            priority,
            input_path,
            output_path,
            error_code: None,
            error_message: None,
            created_at_epoch_ms: now_epoch_ms,
            request: request.clone(),
        };

        {
            let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
            jobs.insert(
                job_id.clone(),
                JobEntry {
                    record,
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

            // Proxy channel to capture progress updates into memory
            let (proxy_tx, mut proxy_rx) = tokio::sync::mpsc::unbounded_channel::<JobEvent>();
            let proxy_tx_arc = Arc::new(proxy_tx);

            let jobs_map_proxy = Arc::clone(&jobs_map);
            let forward_event_tx = Arc::clone(&event_tx);
            let proxy_task = tokio::spawn(async move {
                while let Some(ev) = proxy_rx.recv().await {
                    if let JobEvent::Progress {
                        ref job_id,
                        percent,
                        ..
                    } = ev
                    {
                        Self::set_progress(&jobs_map_proxy, job_id, percent);
                    }
                    let _ = forward_event_tx.send(ev);
                }
            });

            // Run the blocking PDF work on a dedicated thread
            let result = tokio::task::spawn_blocking({
                let jid = jid.clone();
                let p_tx = Arc::clone(&proxy_tx_arc);
                let cancel_token = cancel_token.clone();
                move || Self::execute_job(&jid, &request, &p_tx, &cancel_token)
            })
            .await;

            drop(proxy_tx_arc);
            let _ = proxy_task.await;

            match result {
                Ok(Ok(output_info)) => {
                    Self::set_completed(&jobs_map, &jid, output_info.output_path.clone());
                    let _ = event_tx.send(JobEvent::Completed {
                        job_id: jid,
                        output_path: output_info.output_path,
                        degraded: output_info.degraded,
                    });
                }
                Ok(Err(e)) => {
                    let code = Self::error_code(&e);
                    let msg = e.to_string();
                    Self::set_failed(&jobs_map, &jid, code.clone(), msg.clone());
                    let _ = event_tx.send(JobEvent::Failed {
                        job_id: jid,
                        error_code: code,
                        error_message: msg,
                    });
                }
                Err(_join_err) => {
                    let code: String = "TASK_PANIC".to_string();
                    let msg: String = "Job task panicked".to_string();
                    Self::set_failed(&jobs_map, &jid, code.clone(), msg.clone());
                    let _ = event_tx.send(JobEvent::Failed {
                        job_id: jid,
                        error_code: code,
                        error_message: msg,
                    });
                }
            }
        });

        job_id
    }

    /// Submit a batch of requests with an optional priority.
    pub fn submit_batch(
        &self,
        requests: Vec<JobRequest>,
        priority: Option<JobPriority>,
    ) -> Vec<JobId> {
        let p = match priority {
            Some(p) => p,
            None => JobPriority::Normal,
        };
        requests
            .into_iter()
            .map(|req| self.submit_with_priority(req, p))
            .collect()
    }

    /// Return all tracked jobs sorted newest first.
    pub fn list_jobs(&self) -> Vec<JobRecord> {
        let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        let mut records: Vec<JobRecord> = jobs.values().map(|e| e.record.clone()).collect();
        records.sort_by(|a, b| {
            b.created_at_epoch_ms
                .cmp(&a.created_at_epoch_ms)
                .then_with(|| a.job_id.cmp(&b.job_id))
        });
        records
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

    /// Alias for `cancel`.
    pub fn cancel_job(&self, job_id: &str) -> bool {
        self.cancel(job_id)
    }

    /// Clear all completed, failed, or cancelled jobs from memory.
    /// Returns the number of removed entries.
    pub fn clear_finished_jobs(&self) -> usize {
        let mut jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        let initial_len = jobs.len();
        jobs.retain(|_, entry| match entry.record.status {
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled => false,
            JobStatus::Queued | JobStatus::Running => true,
        });
        initial_len.saturating_sub(jobs.len())
    }

    /// Re-enqueue a failed or cancelled job.
    pub fn retry_job(&self, job_id: &str) -> lpdf_core_types::Result<JobId> {
        let (request, priority) = {
            let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
            let entry = match jobs.get(job_id) {
                Some(e) => e,
                None => {
                    return Err(LpdfError::InvalidInput(format!("Job '{job_id}' not found")));
                }
            };
            match entry.record.status {
                JobStatus::Failed | JobStatus::Cancelled => {
                    (entry.record.request.clone(), entry.record.priority)
                }
                JobStatus::Queued | JobStatus::Running => {
                    return Err(LpdfError::InvalidInput(format!(
                        "Job '{job_id}' is currently active"
                    )));
                }
                JobStatus::Completed => {
                    return Err(LpdfError::InvalidInput(format!(
                        "Job '{job_id}' already completed"
                    )));
                }
            }
        };

        Ok(self.submit_with_priority(request, priority))
    }

    /// Get the current status of a job.
    pub fn status(&self, job_id: &str) -> Option<JobStatus> {
        let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.get(job_id).map(|e| e.record.status)
    }

    /// Get the full record of a job.
    pub fn get_job(&self, job_id: &str) -> Option<JobRecord> {
        let jobs = self.jobs.lock().unwrap_or_else(|e| e.into_inner());
        jobs.get(job_id).map(|e| e.record.clone())
    }

    fn set_status(jobs: &Arc<Mutex<HashMap<JobId, JobEntry>>>, job_id: &str, status: JobStatus) {
        let mut jobs = jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = jobs.get_mut(job_id) {
            entry.record.status = status;
            if status == JobStatus::Running && entry.record.progress == 0 {
                entry.record.progress = 5;
            }
        }
    }

    fn set_progress(jobs: &Arc<Mutex<HashMap<JobId, JobEntry>>>, job_id: &str, percent: u8) {
        let mut jobs = jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = jobs.get_mut(job_id) {
            entry.record.progress = percent;
        }
    }

    fn set_completed(
        jobs: &Arc<Mutex<HashMap<JobId, JobEntry>>>,
        job_id: &str,
        output_path: Option<String>,
    ) {
        let mut jobs = jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = jobs.get_mut(job_id) {
            entry.record.status = JobStatus::Completed;
            entry.record.progress = 100;
            if output_path.is_some() {
                entry.record.output_path = output_path;
            }
        }
    }

    fn set_failed(
        jobs: &Arc<Mutex<HashMap<JobId, JobEntry>>>,
        job_id: &str,
        error_code: String,
        error_message: String,
    ) {
        let mut jobs = jobs.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = jobs.get_mut(job_id) {
            entry.record.status = JobStatus::Failed;
            entry.record.error_code = Some(error_code);
            entry.record.error_message = Some(error_message);
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
            LpdfError::ExternalDependencyError(_) => "EXTERNAL_DEPENDENCY_ERROR".into(),
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

            JobRequest::ImageToPdf {
                input_path,
                output_path,
            } => {
                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.image_to_pdf(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::DocxToPdf {
                input_path,
                output_path,
            } => {
                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.structural_docx_to_pdf(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::HtmlToPdf {
                input_path,
                output_path,
            } => {
                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                engine.html_to_pdf(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToImages {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Rendering PDF pages as images...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                engine.pdf_to_images(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::ExtractAllImages {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Extracting embedded images...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                engine.extract_all_images(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToText {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Extracting text...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                engine.pdf_to_text(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToDocx {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Exporting to structural Word (.docx)...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                engine.pdf_to_structural_docx(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToHtml {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Exporting to HTML...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                engine.pdf_to_html(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToRtf {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Exporting to RTF...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                engine.pdf_to_rtf(&path, &out)?;

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToPdfA {
                input_path,
                output_path,
                conformance,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: format!("Converting to {:?} archival format...", conformance),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let report = engine.pdf_to_pdfa(&path, *conformance, &out)?;

                let summary = format!(
                    "PDF/A Preflight: {} (fixed: {}, remaining: {})",
                    if report.is_compliant {
                        "Compliant"
                    } else {
                        "Non-compliant"
                    },
                    report.fixed_issues.len(),
                    report.remaining_issues.len(),
                );

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: summary,
                });

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::OfficeToPdf {
                input_path,
                output_path,
                format,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: format!(
                        "Converting {:?} document to PDF via headless sidecar...",
                        format
                    ),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 40,
                    message: "Rendering high-fidelity layout...".into(),
                });

                let result_path = engine.office_to_pdf(&path, *format, &out)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: "Office conversion complete".into(),
                });

                Ok(JobOutput {
                    output_path: Some(result_path.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToOffice {
                input_path,
                output_path,
                format,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: format!("Converting PDF to {:?} via headless sidecar...", format),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 40,
                    message: "Importing flowable layout and generating document...".into(),
                });

                let result_path = engine.pdf_to_office(&path, *format, &out)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: "Office export complete".into(),
                });

                Ok(JobOutput {
                    output_path: Some(result_path.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::MarkdownToPdf {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Converting Markdown to PDF via Pandoc sidecar...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Rendering standalone document to PDF...".into(),
                });

                let result_path = engine.markdown_to_pdf(&path, &out)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: "Markdown to PDF conversion complete".into(),
                });

                Ok(JobOutput {
                    output_path: Some(result_path.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::PdfToMarkdown {
                input_path,
                output_path,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Converting PDF to Markdown via Pandoc sidecar...".into(),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Extracting structural layout and images to assets/...".into(),
                });

                let result_path = engine.pdf_to_markdown(&path, &out)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: "PDF to Markdown conversion complete".into(),
                });

                Ok(JobOutput {
                    output_path: Some(result_path.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::MarkdownToDoc {
                input_path,
                output_path,
                format,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: format!("Converting Markdown to {:?} via Pandoc sidecar...", format),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let result_path = engine.markdown_to_doc(&path, *format, &out)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: format!("Markdown to {:?} conversion complete", format),
                });

                Ok(JobOutput {
                    output_path: Some(result_path.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::DocToMarkdown {
                input_path,
                output_path,
                format,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: format!("Converting {:?} to Markdown via Pandoc sidecar...", format),
                });

                let engine = EngineSwitcher::new();
                let path = PathBuf::from(input_path);
                let out = PathBuf::from(output_path);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let result_path = engine.doc_to_markdown(&path, *format, &out)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: format!("{:?} to Markdown conversion complete", format),
                });

                Ok(JobOutput {
                    output_path: Some(result_path.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::BatchMarkdownConvert {
                input_dir,
                output_dir,
                options,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: format!(
                        "Scanning directory and starting batch conversion to {:?}...",
                        options.target
                    ),
                });

                let engine = EngineSwitcher::new();
                let in_p = PathBuf::from(input_dir);
                let out_p = PathBuf::from(output_dir);

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Batch converting documents with Pandoc and asset rewriting...".into(),
                });

                let report = engine.batch_markdown_convert(&in_p, &out_p, options.clone())?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: format!(
                        "Batch conversion complete: {}/{} files converted",
                        report.converted_files, report.total_files
                    ),
                });

                Ok(JobOutput {
                    output_path: Some(out_p.to_string_lossy().to_string()),
                    degraded: engine.is_degraded(),
                })
            }

            JobRequest::RunPipeline {
                input_path,
                output_path,
                steps,
                ..
            } => {
                if steps.is_empty() {
                    return Err(LpdfError::InvalidInput("Pipeline has no steps".into()));
                }

                let out_path = PathBuf::from(output_path);
                let parent_dir = out_path.parent().unwrap_or_else(|| Path::new("."));
                let stage_id = uuid::Uuid::new_v4().to_string();
                let stage_dir = parent_dir.join(format!(".pipeline_stage_{}", stage_id));
                std::fs::create_dir_all(&stage_dir).map_err(LpdfError::Io)?;

                // RAII cleaner ensuring stage_dir is removed on return/cancellation/error
                struct StageCleaner<'a>(&'a Path);
                impl<'a> Drop for StageCleaner<'a> {
                    fn drop(&mut self) {
                        let _ = std::fs::remove_dir_all(self.0);
                    }
                }
                let _cleaner = StageCleaner(&stage_dir);

                let total_steps = steps.len();
                let mut current_input = PathBuf::from(input_path);

                for (idx, step) in steps.iter().enumerate() {
                    if cancel_token.is_cancelled() {
                        return Err(LpdfError::Engine("Job cancelled".into()));
                    }

                    let step_base_percent = ((idx as f64 / total_steps as f64) * 90.0) as u8 + 5;
                    let _ = event_tx.send(JobEvent::Progress {
                        job_id: job_id.into(),
                        percent: step_base_percent,
                        message: format!("Pipeline step {}/{}: {:?}", idx + 1, total_steps, step),
                    });

                    let step_output = stage_dir.join(format!("step_{}.pdf", idx + 1));
                    let engine = EngineSwitcher::new();

                    match step {
                        crate::pipeline::PipelineStep::Repair => {
                            let _ = engine.repair(&current_input, &step_output)?;
                        }
                        crate::pipeline::PipelineStep::Compress { profile } => {
                            engine.compress(&current_input, *profile, &step_output)?;
                        }
                        crate::pipeline::PipelineStep::Ocr => {
                            engine.ocr(&current_input, "eng", &step_output)?;
                        }
                        crate::pipeline::PipelineStep::Rotate { degrees, pages } => {
                            if pages.is_empty() {
                                engine.rotate_document(&current_input, *degrees, &step_output)?;
                            } else {
                                let usize_pages: Vec<usize> =
                                    pages.iter().map(|&p| p as usize).collect();
                                engine.rotate_pages(
                                    &current_input,
                                    &usize_pages,
                                    *degrees,
                                    &step_output,
                                )?;
                            }
                        }
                        crate::pipeline::PipelineStep::FlattenComments => {
                            engine.flatten_comments(&current_input, &step_output)?;
                        }
                        crate::pipeline::PipelineStep::FlattenForms => {
                            engine.flatten_forms(&current_input, &step_output)?;
                        }
                        crate::pipeline::PipelineStep::Watermark { settings } => {
                            engine.add_watermark(&current_input, settings, &step_output)?;
                        }
                        crate::pipeline::PipelineStep::Protect {
                            password,
                            permissions,
                        } => {
                            engine.protect(
                                &current_input,
                                password,
                                permissions.clone(),
                                &step_output,
                            )?;
                        }
                        crate::pipeline::PipelineStep::PdfToPdfA { conformance } => {
                            let _ =
                                engine.pdf_to_pdfa(&current_input, *conformance, &step_output)?;
                        }
                    }

                    let meta = std::fs::metadata(&step_output).map_err(LpdfError::Io)?;
                    if meta.len() == 0 {
                        return Err(LpdfError::Engine(format!(
                            "Pipeline step {} produced an empty file",
                            idx + 1
                        )));
                    }

                    current_input = step_output;
                }

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let final_bytes = std::fs::read(&current_input).map_err(LpdfError::Io)?;
                if !final_bytes.starts_with(b"%PDF-") {
                    return Err(LpdfError::ParseError(
                        "Pipeline final output is not a valid PDF".into(),
                    ));
                }

                lpdf_fs_safety::atomic_write(&out_path, &final_bytes)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: "Pipeline completed successfully".into(),
                });

                Ok(JobOutput {
                    output_path: Some(output_path.clone()),
                    degraded: false,
                })
            }

            JobRequest::SummarizeExtractive {
                input_path,
                output_path,
                length,
            } => {
                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 10,
                    message: "Extracting text and scoring informative content...".into(),
                });

                let engine = EngineSwitcher::new();

                if cancel_token.is_cancelled() {
                    return Err(LpdfError::Engine("Job cancelled".into()));
                }

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 50,
                    message: "Generating extractive summary report...".into(),
                });

                let report = engine.summarize_extractive(input_path, output_path, *length)?;

                let _ = event_tx.send(JobEvent::Progress {
                    job_id: job_id.into(),
                    percent: 100,
                    message: format!(
                        "Extractive summary generated ({} key points, {} words)",
                        report.key_points.len(),
                        report.summary_word_count
                    ),
                });

                Ok(JobOutput {
                    output_path: Some(report.output_path.to_string_lossy().to_string()),
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
