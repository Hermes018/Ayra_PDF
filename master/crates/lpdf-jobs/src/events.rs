//! Typed job events for progress, completion, and error reporting.
//!
//! These events are serialized and pushed to the frontend via Tauri's
//! event system.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::queue::JobId;

/// A typed job event that flows from worker → supervisor → Tauri frontend.
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[serde(tag = "kind", content = "data")]
#[ts(export, export_to = "../../../app-tauri/src/lib/bindings/")]
pub enum JobEvent {
    /// Job has been enqueued and is waiting.
    Queued { job_id: JobId },

    /// Job has started executing.
    Started { job_id: JobId },

    /// Progress update (0–100).
    Progress {
        job_id: JobId,
        percent: u8,
        message: String,
    },

    /// Job completed successfully.
    Completed {
        job_id: JobId,
        output_path: Option<String>,
        degraded: bool,
    },

    /// Job failed with a typed error.
    Failed {
        job_id: JobId,
        error_code: String,
        error_message: String,
    },

    /// Job was cancelled by the user.
    Cancelled { job_id: JobId },
}

/// Sender half for job events. Cloneable so workers can push events.
pub type JobEventSender = Arc<mpsc::UnboundedSender<JobEvent>>;

/// Receiver half for job events. Owned by the event loop that forwards
/// events to the Tauri frontend.
pub type JobEventReceiver = mpsc::UnboundedReceiver<JobEvent>;

/// Create a new (sender, receiver) pair for job events.
pub fn job_event_channel() -> (JobEventSender, JobEventReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();
    (Arc::new(tx), rx)
}
