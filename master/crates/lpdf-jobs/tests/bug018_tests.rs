//! BUG-018 backend contract (fixes builder's frontend depends on it):
//! (a) GetInfo on a hostile file ends `Failed` with error_code `F4_RESOURCE_GUARD` and a message
//!     naming the guard; an open that succeeds only through the fallback ends `Completed{degraded:true}`.
//! (c) `JobManager::delete_session_intermediate` deletes only `*.organizing-*.pdf` files that a job
//!     of this session wrote, and never an original.

use lpdf_engine_port::PdfEngine;
use lpdf_jobs::events::{job_event_channel, JobEvent};
use lpdf_jobs::{JobManager, JobRequest};
use std::path::{Path, PathBuf};

fn corpus(name: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus")
        .join(name);
    assert!(
        p.exists(),
        "corpus file {} missing (corpus/fetch.sh)",
        p.display()
    );
    p
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lpdf_bug018_{tag}_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Submit and wait for the terminal event of that job.
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

fn get_info(p: &Path) -> JobRequest {
    JobRequest::GetInfo {
        input_path: p.to_string_lossy().to_string(),
    }
}

#[tokio::test]
async fn get_info_on_hostile_bomb_fails_naming_the_f4_guard() {
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);
    match run(&manager, &mut rx, get_info(&corpus("hostile-bombs.pdf"))).await {
        JobEvent::Failed {
            error_code,
            error_message,
            ..
        } => {
            assert_eq!(error_code, "F4_RESOURCE_GUARD");
            assert!(
                error_message.contains("F4 guard 'declared-stream-length' tripped"),
                "{error_message}"
            );
            assert!(error_message.contains("1073741824"), "{error_message}");
        }
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn get_info_on_normal_files_completes_not_degraded() {
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);
    for name in ["standard-text.pdf", "standard-form.pdf"] {
        match run(&manager, &mut rx, get_info(&corpus(name))).await {
            JobEvent::Completed { degraded, .. } => assert!(!degraded, "{name}"),
            other => panic!("{name}: expected Completed, got {other:?}"),
        }
    }
}

/// Every candidate that the switcher can only open through the fallback must report
/// `Completed{degraded:true}`; at least one candidate must take that path (non-vacuous).
#[tokio::test]
async fn fallback_open_reports_degraded_true() {
    let dir = temp_dir("fallback");
    let mut candidates = vec![corpus("hostile-broken-xref.pdf")];
    for (name, bytes) in [
        (
            "gen-broken-xref.pdf",
            lpdf_engine_switcher::hostile::generate_broken_xref_pdf(),
        ),
        (
            "gen-cyclic.pdf",
            lpdf_engine_switcher::hostile::generate_cyclic_page_tree_pdf(),
        ),
        (
            "gen-deep.pdf",
            lpdf_engine_switcher::hostile::generate_deep_page_tree_pdf(500),
        ),
    ] {
        let p = dir.join(name);
        std::fs::write(&p, bytes).unwrap();
        candidates.push(p);
    }
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);
    let mut via_fallback = 0;
    for p in &candidates {
        let mut sw = lpdf_engine_switcher::EngineSwitcher::new();
        let opened = sw.load(p).is_ok() && sw.info().is_ok();
        let ev = run(&manager, &mut rx, get_info(p)).await;
        if opened && sw.is_degraded() {
            via_fallback += 1;
            assert!(
                matches!(ev, JobEvent::Completed { degraded: true, .. }),
                "{}: {ev:?}",
                p.display()
            );
        } else if opened {
            assert!(
                matches!(
                    ev,
                    JobEvent::Completed {
                        degraded: false,
                        ..
                    }
                ),
                "{}: {ev:?}",
                p.display()
            );
        } else {
            assert!(
                matches!(ev, JobEvent::Failed { .. }),
                "{}: {ev:?}",
                p.display()
            );
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        via_fallback >= 1,
        "no candidate exercised the fallback path"
    );
}

#[tokio::test]
async fn delete_intermediate_only_for_session_created_organizing_files() {
    let dir = temp_dir("intermediate");
    let original = dir.join("plan.pdf");
    std::fs::copy(corpus("standard-text.pdf"), &original).unwrap();
    let original_bytes = std::fs::read(&original).unwrap();

    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(2, tx);

    // A job of this session writes the intermediate.
    let inter = dir.join("plan.organizing-1a2b.pdf");
    let ev = run(
        &manager,
        &mut rx,
        JobRequest::ExtractPages {
            input_path: original.to_string_lossy().to_string(),
            output_path: inter.to_string_lossy().to_string(),
            pages: vec![0],
        },
    )
    .await;
    assert!(matches!(ev, JobEvent::Completed { .. }), "{ev:?}");
    assert!(inter.exists());
    assert!(manager.created_this_session(&inter));

    // Refusals leave files untouched.
    assert!(
        manager.delete_session_intermediate(&original).is_err(),
        "original must never be deleted"
    );
    assert!(
        manager
            .delete_session_intermediate(Path::new("plan.organizing-1a2b.pdf"))
            .is_err(),
        "relative"
    );
    let foreign = dir.join("other.organizing-zz.pdf"); // matches the pattern, not written by a job
    std::fs::copy(&original, &foreign).unwrap();
    assert!(manager.delete_session_intermediate(&foreign).is_err());
    assert!(foreign.exists());
    let other_out = dir.join("plan_organized.pdf"); // written by a job, wrong name
    let ev = run(
        &manager,
        &mut rx,
        JobRequest::ExtractPages {
            input_path: original.to_string_lossy().to_string(),
            output_path: other_out.to_string_lossy().to_string(),
            pages: vec![0],
        },
    )
    .await;
    assert!(matches!(ev, JobEvent::Completed { .. }), "{ev:?}");
    assert!(manager.delete_session_intermediate(&other_out).is_err());
    assert!(other_out.exists());
    #[cfg(unix)]
    {
        let link = dir.join("link.organizing-9.pdf");
        std::os::unix::fs::symlink(&inter, &link).unwrap();
        assert!(
            manager.delete_session_intermediate(&link).is_err(),
            "symlinks are refused"
        );
        assert!(inter.exists());
    }

    // The real intermediate is deleted, once.
    manager.delete_session_intermediate(&inter).unwrap();
    assert!(!inter.exists());
    assert!(manager.delete_session_intermediate(&inter).is_err());
    assert_eq!(
        std::fs::read(&original).unwrap(),
        original_bytes,
        "original unchanged"
    );

    // A second manager (a new app session) cannot delete what the first one wrote.
    let (tx2, _rx2) = job_event_channel();
    let other_session = JobManager::new(1, tx2);
    let again = dir.join("plan.organizing-2.pdf");
    let ev = run(
        &manager,
        &mut rx,
        JobRequest::ExtractPages {
            input_path: original.to_string_lossy().to_string(),
            output_path: again.to_string_lossy().to_string(),
            pages: vec![0],
        },
    )
    .await;
    assert!(matches!(ev, JobEvent::Completed { .. }), "{ev:?}");
    assert!(other_session.delete_session_intermediate(&again).is_err());
    assert!(again.exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn intermediate_name_pattern() {
    use lpdf_jobs::is_organizing_intermediate as m;
    assert!(m(Path::new("/x/plan.organizing-1a2b.pdf")));
    assert!(m(Path::new("/x/plan.v2.organizing-abc_9.PDF")));
    assert!(!m(Path::new("/x/plan.pdf")));
    assert!(!m(Path::new("/x/.organizing-1.pdf")), "empty stem");
    assert!(!m(Path::new("/x/plan.organizing-.pdf")), "empty token");
    assert!(!m(Path::new("/x/plan.organizing-1.pdf.bak")));
    assert!(!m(Path::new("/x/plan.organizing-1 2.pdf")), "token chars");
}

/// A file replaced (new inode) under the intermediate's name after the job wrote it is refused,
/// left in place, and forgotten by the registry.
#[cfg(unix)]
#[tokio::test]
async fn delete_intermediate_refuses_a_replaced_file() {
    let dir = temp_dir("replaced");
    let original = dir.join("plan.pdf");
    std::fs::copy(corpus("standard-text.pdf"), &original).unwrap();
    let (tx, mut rx) = job_event_channel();
    let manager = JobManager::new(1, tx);
    let inter = dir.join("plan.organizing-r1.pdf");
    let ev = run(
        &manager,
        &mut rx,
        JobRequest::ExtractPages {
            input_path: original.to_string_lossy().to_string(),
            output_path: inter.to_string_lossy().to_string(),
            pages: vec![0],
        },
    )
    .await;
    assert!(matches!(ev, JobEvent::Completed { .. }), "{ev:?}");
    // Swap in a different file under the same name (new inode).
    let swap = dir.join("swap.tmp");
    std::fs::write(&swap, b"%PDF-1.4\nnot ours\n").unwrap();
    std::fs::rename(&swap, &inter).unwrap();
    let err = manager.delete_session_intermediate(&inter).unwrap_err();
    assert!(err.to_string().contains("replaced"), "{err}");
    assert!(inter.exists(), "the replacement is left untouched");
    assert!(
        !manager.created_this_session(&inter),
        "registry entry dropped"
    );
    assert!(manager.delete_session_intermediate(&inter).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}
