use serde::Serialize;

use crate::error::{Result, SottoError};
use crate::store::Store;

pub const DEFAULT_TOGGLE: &str = "CommandOrControl+Shift+Space";
pub const DEFAULT_MODE: &str = "toggle";
/// Hold Fn at least this long to talk; a shorter press toggles record.
pub const FN_HOLD_MS: u64 = 280;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HotkeyView {
    pub shortcut: String,
    pub mode: String,
}

pub fn parse_hotkey(raw: &str) -> Result<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(SottoError::app(
            "HOTKEY_INVALID",
            "A recording shortcut cannot be empty.",
            true,
            "Use a modifier plus a key, for example Command+Shift+Space.",
        ));
    }
    Ok(t.to_string())
}

/// Fn tap vs hold. The event tap uses this so tests never grab the key.
pub fn fn_gesture(held_ms: u64) -> &'static str {
    if held_ms >= FN_HOLD_MS {
        "ptt"
    } else {
        "toggle"
    }
}

pub fn is_fn_shortcut(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "fn" | "globe" | "fn-key"
    )
}

pub fn parse_hotkey_mode(raw: &str) -> Result<String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "toggle" => Ok("toggle".into()),
        "ptt" | "push-to-talk" => Ok("ptt".into()),
        other => Err(SottoError::app(
            "HOTKEY_INVALID",
            format!("Unknown hotkey mode {other}."),
            true,
            "Use toggle or ptt. The hotkey still cannot skip the consent card.",
        )),
    }
}

pub fn stored_shortcut(store: &Store) -> Result<String> {
    match store.get_setting("hotkey_toggle")? {
        Some(raw) if !raw.trim().is_empty() => parse_hotkey(&raw),
        _ => Ok(DEFAULT_TOGGLE.to_string()),
    }
}

pub fn stored_mode(store: &Store) -> Result<String> {
    match store.get_setting("hotkey_mode")? {
        Some(raw) => parse_hotkey_mode(&raw),
        None => Ok(DEFAULT_MODE.to_string()),
    }
}

pub fn view(store: &Store) -> Result<HotkeyView> {
    Ok(HotkeyView {
        shortcut: stored_shortcut(store)?,
        mode: stored_mode(store)?,
    })
}
