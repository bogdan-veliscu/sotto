use std::path::Path;

use serde::Serialize;

use crate::error::{Result, SottoError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notes {
    pub summary: String,
    pub action_items: String,
    pub key_points: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrivacySettings {
    pub telemetry: String,
    pub cloud_mode: String,
    pub retention_days: String,
}

/// Extractive local notes. Never calls the network.
pub fn extract_notes(transcript: &str) -> Result<Notes> {
    let text = transcript.trim();
    if text.is_empty() {
        return Ok(Notes {
            summary: String::new(),
            action_items: String::new(),
            key_points: String::new(),
        });
    }

    let sentences = split_sentences(text);
    let mut actions: Vec<String> = Vec::new();
    let mut claims: Vec<String> = Vec::new();
    for sentence in &sentences {
        if looks_like_follow_up(sentence) {
            actions.push(sentence.clone());
        } else {
            claims.push(sentence.clone());
        }
    }

    let summary = if claims.is_empty() {
        join_limited(&sentences, 2, 240)
    } else {
        join_limited(&claims, 2, 240)
    };
    let action_items = actions.join("\n");
    let key_points = if claims.len() > 2 {
        claims[2..].join("\n")
    } else if !claims.is_empty() {
        claims.join("\n")
    } else {
        sentences.join("\n")
    };

    Ok(Notes {
        summary,
        action_items,
        key_points,
    })
}

pub fn looks_like_url(path: &Path) -> bool {
    let s = path.to_string_lossy().to_ascii_lowercase();
    s.starts_with("http://") || s.starts_with("https://")
}

pub fn reject_remote_dest(path: &Path) -> Result<()> {
    if looks_like_url(path) {
        return Err(SottoError::app(
            "EXPORT_REMOTE",
            "Export destination must be a local filesystem path, not a URL.",
            true,
            "Choose a folder on this Mac.",
        ));
    }
    Ok(())
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        buf.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            let sentence = buf.trim().to_string();
            if !sentence.is_empty() {
                out.push(sentence);
            }
            buf.clear();
        }
    }
    let rest = buf.trim();
    if !rest.is_empty() {
        out.push(rest.to_string());
    }
    out
}

fn looks_like_follow_up(sentence: &str) -> bool {
    let lower = sentence.to_ascii_lowercase();
    let trimmed = lower.trim_start();
    trimmed.contains("follow up")
        || trimmed.contains("todo")
        || trimmed.contains("next:")
        || trimmed.starts_with("- ")
}

fn join_limited(parts: &[String], count: usize, max_chars: usize) -> String {
    let joined = parts
        .iter()
        .take(count)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");
    if joined.chars().count() <= max_chars {
        return joined;
    }
    let truncated: String = joined.chars().take(max_chars).collect();
    truncated.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn empty_transcript_is_empty_notes() {
        let notes = extract_notes("   ").unwrap();
        assert!(notes.summary.is_empty());
        assert!(notes.action_items.is_empty());
        assert!(notes.key_points.is_empty());
    }

    #[test]
    fn http_dest_is_rejected() {
        assert!(looks_like_url(Path::new("https://example.com/notes.md")));
        assert!(!looks_like_url(Path::new("/tmp/notes.md")));
    }
}
