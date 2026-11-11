use windhawk_core::{export_notes_to_bytes, Note, UTF8_BOM};

#[test]
fn test_quoting_commas_and_newlines() {
    let notes = vec![Note::new(
        10,
        "Tweak, with comma",
        "Line 1\nLine 2\r\nLine 3, with another comma",
        "2026-05-01T10:00:00Z",
        "2026-05-01T11:00:00Z",
    )];

    let bytes = export_notes_to_bytes(&notes).expect("Export should succeed");
    assert_eq!(&bytes[0..3], UTF8_BOM);

    let csv_text = std::str::from_utf8(&bytes[3..]).expect("Valid UTF-8");

    // Must be quoted because title has a comma, and body has newlines and commas
    assert!(csv_text.contains("\"Tweak, with comma\""));
    assert!(csv_text.contains("\"Line 1\nLine 2\r\nLine 3, with another comma\""));

    // Round-trip verification with csv Reader
    let mut reader = csv::ReaderBuilder::new().from_reader(&bytes[3..]);
    let records: Vec<Note> = reader
        .deserialize()
        .map(|r| r.expect("Record must deserialize cleanly"))
        .collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], notes[0]);
}

#[test]
fn test_escaping_double_quotes_rfc4180() {
    let notes = vec![Note::new(
        20,
        "Quote \"Test\"",
        "She said: \"Windhawk is fast, reliable, and native!\"",
        "2026-06-01T12:00:00Z",
        "2026-06-01T12:00:00Z",
    )];

    let bytes = export_notes_to_bytes(&notes).expect("Export should succeed");
    let csv_text = std::str::from_utf8(&bytes[3..]).expect("Valid UTF-8");

    // RFC 4180 requires internal double-quotes to be escaped as two double-quotes ("")
    assert!(csv_text.contains("\"Quote \"\"Test\"\"\""));
    assert!(csv_text.contains("\"She said: \"\"Windhawk is fast, reliable, and native!\"\"\""));

    // Verify round-trip parsing
    let mut reader = csv::ReaderBuilder::new().from_reader(&bytes[3..]);
    let records: Vec<Note> = reader
        .deserialize()
        .map(|r| r.expect("Record must deserialize cleanly"))
        .collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].title, "Quote \"Test\"");
    assert_eq!(
        records[0].body,
        "She said: \"Windhawk is fast, reliable, and native!\""
    );
}

#[test]
fn test_unicode_and_utf8_fidelity() {
    let notes = vec![Note::new(
        30,
        "多言語テスト: Windhawk 🦅",
        "Windows 10 カスタマイズ — accented chars: äöü, éèê, ñ, emojis: 🚀✨",
        "2026-07-01T08:00:00Z",
        "2026-07-01T08:30:00Z",
    )];

    let bytes = export_notes_to_bytes(&notes).expect("Export should succeed");
    assert_eq!(&bytes[0..3], UTF8_BOM);

    let csv_text = std::str::from_utf8(&bytes[3..]).expect("Must preserve valid UTF-8");
    assert!(csv_text.contains("多言語テスト: Windhawk 🦅"));
    assert!(csv_text.contains("Windows 10 カスタマイズ"));
    assert!(csv_text.contains("🚀✨"));

    let mut reader = csv::ReaderBuilder::new().from_reader(&bytes[3..]);
    let records: Vec<Note> = reader
        .deserialize()
        .map(|r| r.expect("Record must deserialize cleanly"))
        .collect();

    assert_eq!(records[0], notes[0]);
}
