//! BUG-050 / BUG-032 backend contract: a job whose output is its own input (even an input without a `.pdf`
//! extension) or an existing file ends `Failed` with the OUTPUT_PATH_UNSAFE message, and nothing is written.

use lpdf_jobs::events::{job_event_channel, JobEvent};
use lpdf_jobs::{JobManager, JobRequest};
use std::path::PathBuf;

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lpdf_bug050_{tag}_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

async fn run(
    manager: &JobManager,
    rx: &mut lpdf_jobs::events::JobEventReceiver,
    req: JobRequest,
) -> JobEvent {
    let id = manager.submit(req);
    while let Some(ev) = rx.recv().await {
        match &ev {
            JobEvent::Completed { job_id, .. }
            | JobEvent::Failed { job_id, .. }
            | JobEvent::Cancelled { job_id }
                if *job_id == id =>
            {
                return ev;
            }
            _ => {}
        }
    }
    panic!("event channel closed before job {id} finished");
}

#[tokio::test(flavor = "multi_thread")]
async fn output_equal_to_input_without_pdf_extension_fails_and_leaves_the_original_untouched() {
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);
    let d = temp_dir("same");
    let input = d.join("doc"); // no .pdf extension
    std::fs::write(&input, b"original bytes").unwrap();
    let path = input.to_string_lossy().to_string();
    let ev = run(
        &manager,
        &mut rx,
        JobRequest::Repair {
            input_path: path.clone(),
            output_path: path.clone(),
        },
    )
    .await;
    match ev {
        JobEvent::Failed { error_message, .. } => {
            assert!(
                error_message.contains("OUTPUT_PATH_UNSAFE"),
                "{error_message}"
            )
        }
        other => panic!("expected Failed, got {other:?}"),
    }
    assert_eq!(std::fs::read(&input).unwrap(), b"original bytes");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_existing_output_file_is_never_replaced() {
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);
    let d = temp_dir("exists");
    let input = d.join("a.pdf");
    let out = d.join("a_repaired.pdf");
    std::fs::write(&input, b"in").unwrap();
    std::fs::write(&out, b"previous output").unwrap();
    let ev = run(
        &manager,
        &mut rx,
        JobRequest::Repair {
            input_path: input.to_string_lossy().to_string(),
            output_path: out.to_string_lossy().to_string(),
        },
    )
    .await;
    assert!(matches!(ev, JobEvent::Failed { .. }), "{ev:?}");
    assert_eq!(std::fs::read(&out).unwrap(), b"previous output");
}
