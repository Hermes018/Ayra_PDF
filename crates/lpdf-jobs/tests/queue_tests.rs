use lpdf_jobs::events::job_event_channel;
use lpdf_jobs::queue::{JobManager, JobPriority, JobRequest, JobStatus};

#[tokio::test]
async fn test_submit_batch_and_list() {
    let (event_tx, _event_rx) = job_event_channel();
    let manager = JobManager::new(2, event_tx);

    let reqs = vec![
        JobRequest::GetInfo {
            input_path: "/tmp/nonexistent1.pdf".to_string(),
        },
        JobRequest::GetInfo {
            input_path: "/tmp/nonexistent2.pdf".to_string(),
        },
    ];

    let ids = manager.submit_batch(reqs, Some(JobPriority::High));
    assert_eq!(ids.len(), 2);

    let jobs = manager.list_jobs();
    assert_eq!(jobs.len(), 2);
    assert_eq!(jobs[0].priority, JobPriority::High);
    assert_eq!(jobs[1].priority, JobPriority::High);
}

#[tokio::test]
async fn test_cancel_job() {
    let (event_tx, _event_rx) = job_event_channel();
    // 0 concurrency so it stays queued and can be cancelled deterministically
    let manager = JobManager::new(0, event_tx);

    let req = JobRequest::GetInfo {
        input_path: "/tmp/nonexistent.pdf".to_string(),
    };
    let job_id = manager.submit(req);

    assert_eq!(manager.status(&job_id), Some(JobStatus::Queued));
    let cancelled = manager.cancel(&job_id);
    assert!(cancelled);
}

#[tokio::test]
async fn test_clear_finished_and_retry_job() {
    let (event_tx, mut event_rx) = job_event_channel();
    let manager = JobManager::new(2, event_tx);

    // Submitting with nonexistent path will fail execution quickly
    let req = JobRequest::GetInfo {
        input_path: "/tmp/nonexistent_fail.pdf".to_string(),
    };
    let job_id = manager.submit(req);

    // Wait until failed
    while let Some(ev) = event_rx.recv().await {
        if let lpdf_jobs::events::JobEvent::Failed { job_id: ev_id, .. } = ev {
            if ev_id == job_id {
                break;
            }
        }
    }

    assert_eq!(manager.status(&job_id), Some(JobStatus::Failed));

    // Retry job
    let retry_id = manager.retry_job(&job_id).expect("retry should succeed");
    assert_ne!(retry_id, job_id);

    // Clear finished jobs (the old failed job should be cleared)
    let cleared = manager.clear_finished_jobs();
    assert!(cleared >= 1);
    assert_eq!(manager.status(&job_id), None);
}

#[test]
fn test_builtin_pipeline_presets() {
    let presets = lpdf_jobs::pipeline::builtin_pipeline_presets();
    assert_eq!(presets.len(), 3);
    assert_eq!(presets[0].id, "archive_searchable");
    assert_eq!(presets[1].id, "confidential_release");
    assert_eq!(presets[2].id, "clean_web_optimize");
    for preset in &presets {
        assert!(!preset.steps.is_empty());
        assert!(!preset.name.is_empty());
        assert!(!preset.description.is_empty());
    }
}

#[tokio::test]
async fn test_run_pipeline_chaining() {
    let corpus_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-text.pdf");
    assert!(corpus_path.exists(), "Corpus file must exist");

    let temp_dir =
        std::env::temp_dir().join(format!("lpdf_pipeline_test_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let out_path = temp_dir.join("pipeline_output.pdf");

    let (event_tx, mut event_rx) = job_event_channel();
    let manager = JobManager::new(2, event_tx);

    let steps = vec![
        lpdf_jobs::pipeline::PipelineStep::Repair,
        lpdf_jobs::pipeline::PipelineStep::FlattenComments,
        lpdf_jobs::pipeline::PipelineStep::Compress {
            profile: lpdf_engine_port::OptimizeProfile::Medium,
        },
    ];

    let req = JobRequest::RunPipeline {
        input_path: corpus_path.to_string_lossy().to_string(),
        output_path: out_path.to_string_lossy().to_string(),
        steps,
        preset_name: Some("Clean & Web Optimize".to_string()),
    };

    let job_id = manager.submit(req);

    while let Some(ev) = event_rx.recv().await {
        match ev {
            lpdf_jobs::events::JobEvent::Completed { job_id: ev_id, .. } => {
                if ev_id == job_id {
                    break;
                }
            }
            lpdf_jobs::events::JobEvent::Failed {
                job_id: ev_id,
                error_code,
                error_message,
            } => {
                if ev_id == job_id {
                    panic!("Pipeline job failed: {} - {}", error_code, error_message);
                }
            }
            _ => {}
        }
    }

    assert!(out_path.exists());
    let bytes = std::fs::read(&out_path).unwrap();
    assert!(bytes.starts_with(b"%PDF-"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_run_pipeline_fail_closed() {
    let temp_dir =
        std::env::temp_dir().join(format!("lpdf_pipeline_fail_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let out_path = temp_dir.join("failed_output.pdf");

    let (event_tx, mut event_rx) = job_event_channel();
    let manager = JobManager::new(2, event_tx);

    let req = JobRequest::RunPipeline {
        input_path: "/tmp/this_file_definitely_does_not_exist_12345.pdf".to_string(),
        output_path: out_path.to_string_lossy().to_string(),
        steps: vec![lpdf_jobs::pipeline::PipelineStep::Repair],
        preset_name: None,
    };

    let job_id = manager.submit(req);

    while let Some(ev) = event_rx.recv().await {
        if let lpdf_jobs::events::JobEvent::Failed { job_id: ev_id, .. } = ev {
            if ev_id == job_id {
                break;
            }
        }
    }

    // Fail closed: output file must NOT exist
    assert!(!out_path.exists());

    let _ = std::fs::remove_dir_all(&temp_dir);
}
