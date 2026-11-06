//! BUG-053 (P0): redaction pages are 1-based and checked. Page 0 or a page beyond the document is refused with a typed
//! error (never mapped to another page by saturating arithmetic), and redacting page N removes the text on page N and
//! leaves every other page's text intact (checked independently with `pdftotext`, not with the engine).

use lpdf_engine_pdfcraft::PdfcraftAdapter;
use lpdf_engine_port::{PdfEngine, RedactionRegion};
use std::path::{Path, PathBuf};
use std::process::Command;

fn corpus() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/standard-form.pdf");
    assert!(
        p.exists(),
        "corpus file missing (corpus/fetch.sh): {}",
        p.display()
    );
    p
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lpdf_bug053_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn page_text(pdf: &Path, page: usize) -> String {
    let out = Command::new("pdftotext")
        .args(["-f", &page.to_string(), "-l", &page.to_string(), "-layout"])
        .arg(pdf)
        .arg("-")
        .output()
        .expect("pdftotext must be installed (poppler-utils)");
    assert!(out.status.success());
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn pages(pdf: &Path) -> usize {
    let out = Command::new("pdfinfo").arg(pdf).output().expect("pdfinfo");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|l| l.strip_prefix("Pages:").map(|v| v.trim().parse().unwrap()))
        .expect("Pages: line")
}

fn whole_page(page: usize) -> RedactionRegion {
    RedactionRegion {
        page,
        x: 0.0,
        y: 0.0,
        width: 5000.0,
        height: 5000.0,
    }
}

fn words(text: &str) -> usize {
    text.split_whitespace().count()
}

#[test]
fn redacting_page_n_removes_page_n_text_and_keeps_every_other_page() {
    let input = corpus();
    let n_pages = pages(&input);
    assert!(n_pages >= 3, "corpus file needs at least 3 pages");
    let d = temp_dir("n");
    for target in [2usize, 3] {
        let out = d.join(format!("redacted-{target}.pdf"));
        PdfcraftAdapter::new()
            .redact(&input, vec![whole_page(target)], &out)
            .expect("redaction of a valid page");
        for p in 1..=n_pages {
            let before = page_text(&input, p);
            let after = page_text(&out, p);
            if p == target {
                assert!(
                    words(&after) * 10 < words(&before),
                    "page {p} text must be (almost) gone: {} -> {} words",
                    words(&before),
                    words(&after)
                );
            } else {
                assert_eq!(
                    words(&before),
                    words(&after),
                    "page {p} must be untouched when page {target} is redacted"
                );
            }
        }
    }
}

#[test]
fn page_zero_is_refused_with_a_typed_error_and_nothing_is_written() {
    let d = temp_dir("zero");
    let out = d.join("out.pdf");
    let r = PdfcraftAdapter::new().redact(&corpus(), vec![whole_page(0)], &out);
    let msg = r.expect_err("page 0 must be refused").to_string();
    assert!(msg.contains("REDACTION_PAGE_OUT_OF_RANGE"), "{msg}");
    assert!(!out.exists(), "nothing may be written");
}

#[test]
fn a_page_beyond_the_document_is_refused() {
    let input = corpus();
    let d = temp_dir("beyond");
    let out = d.join("out.pdf");
    let n = pages(&input);
    let r = PdfcraftAdapter::new().redact(&input, vec![whole_page(n + 1)], &out);
    let msg = r.expect_err("page > count must be refused").to_string();
    assert!(msg.contains("REDACTION_PAGE_OUT_OF_RANGE"), "{msg}");
    assert!(
        msg.contains(&n.to_string()),
        "the message names the real page count: {msg}"
    );
    assert!(!out.exists());
}

#[test]
fn an_invalid_region_is_refused_not_defaulted() {
    let d = temp_dir("invalid");
    let out = d.join("out.pdf");
    for bad in [
        RedactionRegion {
            page: 1,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 10.0,
        },
        RedactionRegion {
            page: 1,
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: -5.0,
        },
        RedactionRegion {
            page: 1,
            x: f64::NAN,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        },
    ] {
        let r = PdfcraftAdapter::new().redact(&corpus(), vec![bad], &out);
        let msg = r.expect_err("invalid region must be refused").to_string();
        assert!(msg.contains("REDACTION_REGION_INVALID"), "{msg}");
        assert!(!out.exists());
    }
}
