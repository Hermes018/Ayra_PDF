//! F4 preflight guard (BUG-018 a).
use lpdf_core_types::LpdfError;
use lpdf_engine_port::PdfEngine;
use lpdf_engine_switcher::guard::{
    check_declared_stream_lengths, preflight, DECLARED_STREAM_LENGTH,
};
use std::path::PathBuf;

fn corpus(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus")
        .join(name)
}

#[test]
fn bomb_trips_the_declared_stream_length_guard() {
    let bytes = std::fs::read(corpus("hostile-bombs.pdf")).unwrap();
    let trip = preflight(&bytes).unwrap_err();
    assert_eq!(trip.guard, DECLARED_STREAM_LENGTH);
    assert!(trip
        .to_string()
        .starts_with("F4 guard 'declared-stream-length' tripped:"));
    let generated = lpdf_engine_switcher::hostile::generate_flate_bomb_pdf();
    assert!(check_declared_stream_lengths(&generated).is_err());
}

#[test]
fn corpus_documents_pass_the_guards() {
    for name in [
        "standard-text.pdf",
        "standard-form.pdf",
        "synthetic-1k.pdf",
        "hostile-broken-xref.pdf",
    ] {
        let bytes = std::fs::read(corpus(name)).unwrap();
        assert_eq!(preflight(&bytes), Ok(()), "{name}");
    }
}

#[test]
fn indirect_lengths_and_font_lengths_are_not_checked() {
    let pdf = b"%PDF-1.4\n4 0 obj << /Length 5 0 R /Length1 99999999 /Length2 99999999 >> stream\nabc\nendstream endobj\n5 0 obj 3 endobj\n";
    assert_eq!(check_declared_stream_lengths(pdf), Ok(()));
    let small = b"%PDF-1.4\n4 0 obj << /Length 3 >> stream\nabc\nendstream endobj\n";
    assert_eq!(check_declared_stream_lengths(small), Ok(()));
    let big = b"%PDF-1.4\n4 0 obj << /Length 999999 >> stream\nabc\nendstream endobj\n";
    assert!(check_declared_stream_lengths(big).is_err());
    let huge_digits = b"<< /Length 99999999999999999999999999 >>";
    assert!(
        check_declared_stream_lengths(huge_digits).is_err(),
        "saturating parse"
    );
}

#[test]
fn switcher_refuses_the_bomb_before_either_engine() {
    let mut sw = lpdf_engine_switcher::EngineSwitcher::new();
    match sw.load(&corpus("hostile-bombs.pdf")) {
        Err(LpdfError::ResourceLimit(msg)) => {
            assert!(msg.contains(DECLARED_STREAM_LENGTH), "{msg}")
        }
        other => panic!("expected ResourceLimit, got {other:?}"),
    }
    assert!(!sw.is_degraded());
    assert!(sw.active_engine().is_none());
}
