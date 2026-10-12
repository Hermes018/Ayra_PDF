use lpdf_fs_safety::*;
use std::fs;

#[test]
fn test_sanitize_filename() {
    assert_eq!(sanitize_filename("my doc (1).pdf"), "my_doc__1__pdf");
    assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
    assert_eq!(sanitize_filename(""), "export");
    assert_eq!(sanitize_filename("..."), "export");
}

#[test]
fn test_atomic_write() {
    let temp_dir = std::env::temp_dir().join(format!("lpdf_test_fs_{}", uuid_simple()));
    let file_path = temp_dir.join("test.txt");
    let content = b"Hello, World!";

    let res = atomic_write(&file_path, content);
    assert!(res.is_ok());
    assert!(file_path.exists());
    let read = fs::read(&file_path).unwrap();
    assert_eq!(read, content);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_create_safe_output_dir() {
    let base_dir = std::env::temp_dir().join(format!("lpdf_test_safe_dir_{}", uuid_simple()));
    let dir1 = create_safe_output_dir(&base_dir, "export_images").unwrap();
    assert!(dir1.exists());
    assert!(dir1.ends_with("export_images"));

    // Creating second time with same prefix should append _1
    let dir2 = create_safe_output_dir(&base_dir, "export_images").unwrap();
    assert!(dir2.exists());
    assert!(dir2.ends_with("export_images_1"));

    let _ = fs::remove_dir_all(&base_dir);
}
