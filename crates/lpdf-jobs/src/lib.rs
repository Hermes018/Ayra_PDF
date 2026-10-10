//! Async job queue for long-running PDF operations.
//!
//! Design rules:
//! - The Tauri main thread is NEVER blocked (DO-8: command handlers < 10ms).
//! - Every job runs on a tokio blocking task to avoid starving the async runtime.
//! - Progress, completion, and errors are communicated via typed events.
//! - Every job supports cancellation via a `CancellationToken`.

#![forbid(unsafe_code)]

pub mod events;
pub mod queue;

pub use events::{JobEvent, JobEventSender};
pub use queue::{JobId, JobManager, JobRequest, JobStatus};
