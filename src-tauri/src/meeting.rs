use serde::Serialize;

use crate::error::{Result, SottoError};
use crate::store::Store;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeetingKind {
    Zoom,
    Teams,
    Slack,
}

impl MeetingKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Zoom => "zoom",
            Self::Teams => "teams",
            Self::Slack => "slack",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Zoom => "Zoom",
            Self::Teams => "Teams",
            Self::Slack => "Slack",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DetectedMeeting {
    pub kind: String,
    pub label: String,
    pub process: String,
}

fn stem(name: &str) -> String {
    let trimmed = name.trim().trim_matches('"');
    let file = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    file.trim()
        .trim_end_matches(".exe")
        .trim_end_matches(".app")
        .to_ascii_lowercase()
}

pub fn classify_name(name: &str) -> Option<MeetingKind> {
    match stem(name).as_str() {
        "zoom.us" | "zoom" => Some(MeetingKind::Zoom),
        "slack" => Some(MeetingKind::Slack),
        "microsoft teams" | "teams" | "ms-teams" => Some(MeetingKind::Teams),
        _ => None,
    }
}

pub fn classify_processes(names: &[impl AsRef<str>]) -> Vec<DetectedMeeting> {
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for name in names {
        let raw = name.as_ref();
        if let Some(kind) = classify_name(raw) {
            if seen.insert(kind) {
                out.push(DetectedMeeting {
                    kind: kind.as_str().to_string(),
                    label: kind.label().to_string(),
                    process: raw.trim().to_string(),
                });
            }
        }
    }
    out
}

pub fn should_prompt(detected: &[DetectedMeeting], recording: bool, enabled: bool) -> bool {
    enabled && !recording && !detected.is_empty()
}

pub fn prompt_copy(detected: &[DetectedMeeting]) -> String {
    if detected.is_empty() {
        return String::new();
    }
    let names = detected
        .iter()
        .map(|d| d.label.as_str())
        .collect::<Vec<_>>()
        .join(" and ");
    format!(
        "{names} looks open. Record this meeting on this Mac? Sotto will not start until you consent."
    )
}

pub fn parse_detect_enabled(raw: &str) -> Result<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "off" => Ok(false),
        "on" => Ok(true),
        other => Err(SottoError::app(
            "MEETING_DETECT_INVALID",
            format!("Unknown meeting-detect setting {other}."),
            true,
            "Use on or off. Detection still cannot skip the consent card.",
        )),
    }
}

pub fn detect_enabled(store: &Store) -> Result<bool> {
    match store.get_setting("meeting_detect")? {
        Some(raw) => parse_detect_enabled(&raw),
        None => Ok(false),
    }
}

/// macOS `ps` only. Tests inject names and never call this.
pub fn list_process_names() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("ps")
            .args(["-caxo", "comm="])
            .output()
        {
            if out.status.success() {
                return String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}
