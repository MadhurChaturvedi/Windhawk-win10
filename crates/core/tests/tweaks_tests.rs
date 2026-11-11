use std::collections::HashSet;
use windhawk_core::exporter::export_notes_to_bytes;
use windhawk_core::tweaks::{get_available_tweaks, read_tweak_status};
use windhawk_core::UTF8_BOM;

#[test]
fn test_get_available_tweaks_uniqueness_and_content() {
    let tweaks = get_available_tweaks();
    assert!(
        !tweaks.is_empty(),
        "Should provide curated Windows 10 tweaks"
    );

    let mut ids = HashSet::new();
    for tweak in &tweaks {
        assert!(ids.insert(tweak.id), "Tweak ID {} must be unique", tweak.id);
        assert!(!tweak.title.is_empty(), "Title must not be empty");
        assert!(
            !tweak.description.is_empty(),
            "Description must not be empty"
        );
        assert!(
            !tweak.registry_path.is_empty(),
            "Registry path must not be empty"
        );
        assert!(!tweak.value_name.is_empty(), "Value name must not be empty");
    }
}

#[test]
fn test_tweak_to_note_conversion() {
    let tweaks = get_available_tweaks();
    let first = &tweaks[0];
    let note = first.to_note(Some(true));

    assert_eq!(note.id, first.id);
    assert_eq!(note.title, first.title);
    assert!(note.body.contains("ACTIVE [Enabled]"));
    assert!(note.body.contains(&first.registry_path));
    assert!(note.body.contains(&first.value_name));
}

#[cfg(windows)]
#[test]
fn test_read_live_tweak_status_does_not_panic() {
    let tweaks = get_available_tweaks();
    for tweak in &tweaks {
        // Querying Windows 10 registry should either return Some(bool) or None
        let status = read_tweak_status(tweak);
        println!("Tweak '{}' live status: {:?}", tweak.title, status);
    }
}

#[test]
fn test_tweaks_export_to_valid_csv() {
    let tweaks = get_available_tweaks();
    let notes: Vec<_> = tweaks.iter().map(|t| t.to_note(Some(false))).collect();

    let bytes = export_notes_to_bytes(&notes).expect("Export should succeed");
    assert_eq!(&bytes[0..3], UTF8_BOM);

    let csv_text = std::str::from_utf8(&bytes[3..]).expect("Valid UTF-8");
    assert!(csv_text.starts_with("id,title,body,created_at,updated_at\r\n"));

    let mut reader = csv::ReaderBuilder::new().from_reader(&bytes[3..]);
    let records_count = reader.records().count();
    assert_eq!(records_count, tweaks.len());
}

#[cfg(windows)]
#[test]
fn test_taskbar_color_apis() {
    use windhawk_core::tweaks::{get_taskbar_color_prevalence, get_taskbar_current_color};
    let current_color = get_taskbar_current_color();
    println!("Current Taskbar RGB color: {:?}", current_color);
    let prevalence = get_taskbar_color_prevalence();
    println!("Current ColorPrevalence: {}", prevalence);
}
