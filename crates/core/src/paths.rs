use crate::error::WindhawkError;
use directories::UserDirs;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Returns the default export directory: `<Documents>/Windhawk/` (or `./` fallback).
pub fn default_export_dir() -> PathBuf {
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(docs) = user_dirs.document_dir() {
            return docs.join("Windhawk");
        }
    }
    PathBuf::from(".")
}

/// Returns the default export file path: `<Documents>/Windhawk/notes.csv`.
pub fn default_export_path() -> PathBuf {
    default_export_dir().join("notes.csv")
}

/// Ensures that the parent directory of `path` exists, creating any missing directories.
pub fn ensure_parent_dir(path: &Path) -> Result<(), WindhawkError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            debug!("Creating directory structure: {}", parent.display());
            fs::create_dir_all(parent).map_err(|e| {
                WindhawkError::PathError(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                ))
            })?;
        }
    }
    Ok(())
}

