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
