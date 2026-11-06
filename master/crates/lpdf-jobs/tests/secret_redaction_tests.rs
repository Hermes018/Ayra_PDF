//! BUG-058 (P0): a password in a job spec must never reach a log, a Progress event, a Debug/Display string or the
//! job list the UI reads. It serializes only where the engine needs it (the sandboxed worker payload).

use lpdf_core_types::Secret;
use lpdf_engine_port::Permissions;
use lpdf_jobs::events::{job_event_channel, JobEvent};
use lpdf_jobs::{JobManager, JobRequest, PipelineStep};
use std::path::PathBuf;

const PW: &str = "hunter2-TOP-SECRET-pw";

fn perms() -> Permissions {
    Permissions {
        allow_print: true,
        allow_copy: false,
        allow_edit: false,
    }
}

fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-text.pdf")
}

#[test]
fn secret_debug_and_display_never_show_the_value() {
    let s = Secret::new(PW);
    assert_eq!(format!("{s:?}"), "[redacted]");
    assert_eq!(format!("{s}"), "[redacted]");
    assert_eq!(format!("{s:#?}"), "[redacted]");
    assert_eq!(s.expose(), PW);
}

#[test]
fn job_requests_and_pipeline_steps_redact_in_debug() {
    let reqs = vec![
        JobRequest::ProtectDocument {
            input_path: "/a.pdf".into(),
            output_path: "/b.pdf".into(),
            password: Secret::new(PW),
            permissions: perms(),
        },
        JobRequest::UnlockDocument {
            input_path: "/a.pdf".into(),
            output_path: "/b.pdf".into(),
            password: Secret::new(PW),
        },
        JobRequest::SignDocument {
            input_path: "/a.pdf".into(),
            output_path: "/b.pdf".into(),
            p12_path: "/id.p12".into(),
            password: Secret::new(PW),
        },
        JobRequest::RunPipeline {
            input_path: "/a.pdf".into(),
            output_path: "/b.pdf".into(),
            steps: vec![PipelineStep::Protect {
                password: Secret::new(PW),
                permissions: perms(),
            }],
            preset_name: None,
        },
    ];
    for r in &reqs {
        assert!(!format!("{r:?}").contains(PW), "Debug leaked: {r:?}");
        assert!(!format!("{r:#?}").contains(PW));
    }
}

#[test]
fn the_worker_payload_still_carries_the_password_but_the_job_list_does_not() {
    let req = JobRequest::UnlockDocument {
        input_path: "/a.pdf".into(),
        output_path: "/b.pdf".into(),
        password: Secret::new(PW),
    };
    // the engine/worker needs it
    assert!(serde_json::to_string(&req).unwrap().contains(PW));
    // a JobRecord (what list_jobs sends to the UI) never carries it
    let (tx, _rx) = job_event_channel();
    let manager = JobManager::new(1, tx);
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let json = rt.block_on(async {
        let _id = manager.submit(req);
        serde_json::to_string(&manager.list_jobs()).unwrap()
    });
    assert!(!json.contains(PW), "list_jobs leaked the password: {json}");
    assert!(json.contains("redacted"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_pipeline_with_a_protect_step_never_puts_the_password_in_an_event_or_the_log_line() {
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);
    let d = std::env::temp_dir().join(format!("lpdf_bug058_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    let input = d.join("in.pdf");
    std::fs::copy(corpus(), &input).unwrap();
    let id = manager.submit(JobRequest::RunPipeline {
        input_path: input.to_string_lossy().to_string(),
        output_path: d.join("out.pdf").to_string_lossy().to_string(),
        steps: vec![PipelineStep::Protect {
            password: Secret::new(PW),
            permissions: perms(),
        }],
        preset_name: None,
    });
    let mut progress_seen = 0;
    while let Some(ev) = rx.recv().await {
        // exactly what the app logs: tracing::debug!("Job event: {:?}", event)
        let log_line = format!("Job event: {ev:?}");
        assert!(
            !log_line.contains(PW),
            "log line leaked the password: {log_line}"
        );
        if let JobEvent::Progress { ref message, .. } = ev {
            progress_seen += 1;
            assert!(!message.contains(PW), "Progress message leaked: {message}");
            assert!(
                !message.contains("Protect {"),
                "Progress must use a short label, not Debug output: {message}"
            );
        }
        match &ev {
            JobEvent::Completed { job_id, .. }
            | JobEvent::Failed { job_id, .. }
            | JobEvent::Cancelled { job_id }
                if *job_id == id =>
            {
                break
            }
            _ => {}
        }
    }
    assert!(progress_seen > 0);
    let _ = std::fs::remove_dir_all(&d);
}
