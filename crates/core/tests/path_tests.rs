use std::fs;
use tempfile::tempdir;
use windhawk_core::{default_export_path, ensure_parent_dir, export_notes_to_file, Note};

#[test]
fn test_default_export_path_has_correct_file_name() {
    let path = default_export_path();
    assert_eq!(path.file_name().and_then(|f| f.to_str()), Some("notes.csv"));
}

#[test]
fn test_ensure_parent_dir_creates_nested_directories() {
    let tmp = tempdir().expect("tempdir failed");
    let deep_path = tmp.path().join("sub1").join("sub2").join("notes.csv");

    assert!(!deep_path.parent().unwrap().exists());
    ensure_parent_dir(&deep_path).expect("ensure_parent_dir failed");
    assert!(deep_path.parent().unwrap().exists());
}

#[test]
fn test_export_notes_to_file_creates_file_and_directories() {
    let tmp = tempdir().expect("tempdir failed");
    let target_file = tmp.path().join("export_dir").join("custom_notes.csv");

    let notes = Note::sample_notes();
    let count = export_notes_to_file(&notes, &target_file).expect("Export to file should succeed");

    assert_eq!(count, notes.len());
    assert!(target_file.exists());

    let metadata = fs::metadata(&target_file).expect("File metadata");
    assert!(metadata.len() > 0);
}
