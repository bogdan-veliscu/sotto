use std::path::Path;

use crate::error::{Result, SottoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notes {
    pub summary: String,
    pub action_items: String,
    pub key_points: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrivacySettings {
    pub telemetry: String,
    pub cloud_mode: String,
    pub retention_days: String,
}

/// Extractive local notes. Never calls the network.
pub fn extract_notes(_transcript: &str) -> Result<Notes> {
    Err(SottoError::app(
        "NOT_IMPLEMENTED",
        "extractive notes are not implemented in this wave",
        true,
        "Wait for notes-export GREEN.",
    ))
}

pub fn looks_like_url(path: &Path) -> bool {
    let s = path.to_string_lossy().to_ascii_lowercase();
    s.starts_with("http://") || s.starts_with("https://")
}
