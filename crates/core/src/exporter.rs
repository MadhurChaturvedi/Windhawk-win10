use crate::error::WindhawkError;
use crate::model::Note;
use crate::paths::ensure_parent_dir;
use csv::{QuoteStyle, Terminator, WriterBuilder};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{debug, info};

/// UTF-8 Byte Order Mark (BOM) bytes: [0xEF, 0xBB, 0xBF]
pub const UTF8_BOM: &[u8; 3] = &[0xEF, 0xBB, 0xBF];

/// Ordered CSV header fields conforming to RFC 4180
pub const CSV_HEADERS: &[&str] = &["id", "title", "body", "created_at", "updated_at"];

/// Serializes notes to CSV format conforming to RFC 4180 with a leading UTF-8 BOM.
///
/// Output columns:
/// 1. `id` (integer)
/// 2. `title` (text)
/// 3. `body` (text)
/// 4. `created_at` (RFC 3339 text)
/// 5. `updated_at` (RFC 3339 text)
pub fn export_notes_to_csv<W: Write>(notes: &[Note], mut writer: W) -> Result<(), WindhawkError> {
    debug!("Writing UTF-8 BOM to CSV destination");
    writer.write_all(UTF8_BOM)?;

    let mut csv_writer = WriterBuilder::new()
        .delimiter(b',')
        .quote_style(QuoteStyle::Necessary)
        .terminator(Terminator::CRLF)
        .has_headers(false)
        .from_writer(writer);

    // Always emit the header record first
    csv_writer.write_record(CSV_HEADERS)?;

    for note in notes {
        csv_writer.serialize(note)?;
    }

    csv_writer.flush()?;
    Ok(())
}

/// Exports notes into an in-memory byte buffer (prefixed with UTF-8 BOM).
pub fn export_notes_to_bytes(notes: &[Note]) -> Result<Vec<u8>, WindhawkError> {
    let mut buffer = Vec::new();
    export_notes_to_csv(notes, &mut buffer)?;
    Ok(buffer)
}

/// Exports notes directly to a file path, ensuring parent directories exist.
/// Returns the number of notes exported.
pub fn export_notes_to_file<P: AsRef<Path>>(
    notes: &[Note],
    path: P,
) -> Result<usize, WindhawkError> {
    let path = path.as_ref();
    info!(
        "Exporting {} notes to CSV file: {}",
        notes.len(),
        path.display()
    );

    ensure_parent_dir(path)?;

    let file = File::create(path)?;
    export_notes_to_csv(notes, file)?;

    info!(
        "Successfully exported {} notes to {}",
        notes.len(),
        path.display()
    );
    Ok(notes.len())
}
