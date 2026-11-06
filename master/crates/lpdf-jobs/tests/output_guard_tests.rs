//! BUG-050 / BUG-032: unit tests for the output-path guard (`lpdf_jobs::output_guard`).
use lpdf_jobs::output_guard::{check_output_path, OutputPathError};
use std::fs;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lpdf-guard-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).expect("io");
    d
}

#[test]
fn output_equal_to_input_is_refused() {
    let d = scratch("same");
    let input = d.join("doc");
    fs::write(&input, b"%PDF-1.4").expect("io");
    let err = check_output_path(&[input.clone()], &input).expect_err("same path");
    assert!(matches!(err, OutputPathError::SameAsInput { .. }));
    assert_eq!(fs::read(&input).expect("io"), b"%PDF-1.4");
}

#[test]
fn output_reaching_the_input_through_dotdot_or_a_symlink_is_refused() {
    let d = scratch("alias");
    let input = d.join("a.pdf");
    fs::write(&input, b"x").expect("io");
    fs::create_dir_all(d.join("sub")).expect("io");
    let sneaky = d.join("sub").join("..").join("a.pdf");
    assert!(matches!(
        check_output_path(&[input.clone()], &sneaky),
        Err(OutputPathError::SameAsInput { .. })
    ));
    #[cfg(unix)]
    {
        let link = d.join("link.pdf");
        std::os::unix::fs::symlink(&input, &link).expect("io");
        assert!(matches!(
            check_output_path(&[input], &link),
            Err(OutputPathError::SameAsInput { .. }) | Err(OutputPathError::AlreadyExists { .. })
        ));
    }
}

#[test]
fn an_existing_output_file_is_refused() {
    let d = scratch("exists");
    let input = d.join("a.pdf");
    let out = d.join("a_out.pdf");
    fs::write(&input, b"x").expect("io");
    fs::write(&out, b"old").expect("io");
    assert!(matches!(
        check_output_path(&[input], &out),
        Err(OutputPathError::AlreadyExists { .. })
    ));
    assert_eq!(fs::read(&out).expect("io"), b"old");
}

#[test]
fn a_fresh_name_next_to_the_input_is_allowed_and_an_output_directory_is_not_a_collision() {
    let d = scratch("fresh");
    let input = d.join("doc");
    fs::write(&input, b"x").expect("io");
    assert!(check_output_path(&[input.clone()], &d.join("doc.out.pdf")).is_ok());
    assert!(
        check_output_path(&[input], &d).is_ok(),
        "an existing directory is not a file collision"
    );
}

#[test]
fn merge_inputs_are_all_checked() {
    let d = scratch("merge");
    let a = d.join("a.pdf");
    let b = d.join("b.pdf");
    fs::write(&a, b"a").expect("io");
    fs::write(&b, b"b").expect("io");
    assert!(matches!(
        check_output_path(&[a, b.clone()], &b),
        Err(OutputPathError::SameAsInput { .. })
    ));
}

#[test]
fn the_error_message_names_the_problem_in_plain_words() {
    let e = OutputPathError::SameAsInput {
        path: "/x/doc".into(),
    };
    assert!(e.to_string().contains("original"));
}
