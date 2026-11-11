use serde::{Deserialize, Serialize};

/// Represents a single user note for Windhawk modding/system tweaks.
///
/// NOTE: This schema (`id`, `title`, `body`, `created_at`, `updated_at`) is an
/// assumed design implementation per the project specification (Phase 2 found
/// no upstream notes export feature in Windhawk v1.7.3 or 2.0.0-alpha.8).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier for the note
    pub id: u64,
    /// Note title or subject
    pub title: String,
    /// Detailed text body of the note
    pub body: String,
    /// ISO 8601 / RFC 3339 creation timestamp (e.g. "2026-11-11T12:00:00Z")
    pub created_at: String,
    /// ISO 8601 / RFC 3339 last update timestamp
    pub updated_at: String,
}

impl Note {
    pub fn new(
        id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            body: body.into(),
            created_at: created_at.into(),
            updated_at: updated_at.into(),
        }
    }

    /// Default sample notes representing typical Windhawk Windows 10 modding tweaks.
    pub fn sample_notes() -> Vec<Self> {
        vec![
            Self::new(
                1,
                "Classic Taskbar Labels",
                "Ensure never-combine taskbar buttons is enabled with text labels visible.\nTested on Windows 10 Build 19045.",
                "2026-01-15T09:30:00Z",
                "2026-02-10T14:22:00Z",
            ),
            Self::new(
                2,
                "Explorer Details Pane",
                "Hook shell32 to restore bottom details pane layout in File Explorer.\n\"Important: run with elevated broker.\"",
                "2026-03-01T11:00:00Z",
                "2026-03-05T16:45:00Z",
            ),
            Self::new(
                3,
                "Volume Scroll on Taskbar",
                "Change system volume by scrolling wheel over empty taskbar areas.\nSmooth steps: 2% per tick.",
                "2026-04-12T08:15:00Z",
                "2026-05-20T10:00:00Z",
            ),
            Self::new(
                4,
                "Custom Clock Format",
                "Format: \"ddd, MMM d, yyyy - HH:mm:ss\"\nMulti-monitor secondary taskbar support enabled.",
                "2026-06-01T13:40:00Z",
                "2026-06-02T09:10:00Z",
            ),
        ]
    }
}

