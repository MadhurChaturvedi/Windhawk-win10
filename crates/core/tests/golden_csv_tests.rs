use windhawk_core::{export_notes_to_bytes, Note, UTF8_BOM};

#[test]
fn test_golden_csv_output() {
    let notes = vec![
        Note::new(
            1,
            "Taskbar Tweak",
            "Classic layout enabled",
            "2026-01-01T00:00:00Z",
            "2026-01-02T12:00:00Z",
        ),
        Note::new(
            2,
            "Start Menu",
            "Disable web search in start",
            "2026-02-01T10:00:00Z",
            "2026-02-02T15:30:00Z",
        ),
    ];

    let bytes = export_notes_to_bytes(&notes).expect("Export should succeed");

    // Check UTF-8 BOM
    assert_eq!(&bytes[0..3], UTF8_BOM, "First 3 bytes must be UTF-8 BOM");

    // Convert payload after BOM to string
    let csv_text = std::str::from_utf8(&bytes[3..]).expect("CSV body must be valid UTF-8");

    let expected = "\
id,title,body,created_at,updated_at\r\n\
1,Taskbar Tweak,Classic layout enabled,2026-01-01T00:00:00Z,2026-01-02T12:00:00Z\r\n\
2,Start Menu,Disable web search in start,2026-02-01T10:00:00Z,2026-02-02T15:30:00Z\r\n";

    assert_eq!(
        csv_text, expected,
        "CSV output must match golden string exactly"
    );
}

#[test]
fn test_empty_notes_golden_csv() {
    let notes: Vec<Note> = Vec::new();
    let bytes = export_notes_to_bytes(&notes).expect("Exporting empty notes should succeed");

    assert_eq!(&bytes[0..3], UTF8_BOM);
    let csv_text = std::str::from_utf8(&bytes[3..]).expect("Valid UTF-8");
    let expected = "id,title,body,created_at,updated_at\r\n";
    assert_eq!(csv_text, expected);
}
