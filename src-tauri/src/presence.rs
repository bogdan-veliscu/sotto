use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HudView {
    pub led_on: bool,
    pub paused: bool,
    pub elapsed_ms: u64,
    pub clock: String,
    pub caption: String,
    pub status_label: String,
    pub source: String,
    pub source_label: String,
    pub title: String,
    pub session_id: String,
    pub level: u8,
}

pub fn format_clock(elapsed_ms: u64) -> String {
    let total = elapsed_ms / 1000;
    format!("{:02}:{:02}", total / 60, total % 60)
}

pub fn source_label(source: &str) -> &'static str {
    match source {
        "mic" => "Mic",
        "system" => "System",
        "mixed" => "Mixed",
        _ => "",
    }
}

/// LED + caption for the notch HUD and the tray tooltip.
pub fn hud_from_status(status: &str, elapsed_ms: u64) -> HudView {
    hud_view(status, elapsed_ms, "", "", "", 0)
}

pub fn hud_view(
    status: &str,
    elapsed_ms: u64,
    source: &str,
    title: &str,
    session_id: &str,
    level: u8,
) -> HudView {
    let led_on = status == "recording";
    let paused = status == "paused";
    let caption = match status {
        "recording" => "on this Mac",
        "paused" => "paused",
        _ => "Sotto",
    };
    let status_label = if paused {
        "Paused"
    } else if led_on {
        "Rec"
    } else {
        "Sotto"
    };
    let title: String = title.chars().take(32).collect();
    HudView {
        led_on,
        paused,
        elapsed_ms,
        clock: format_clock(elapsed_ms),
        caption: caption.to_string(),
        status_label: status_label.to_string(),
        source: source.to_string(),
        source_label: source_label(source).to_string(),
        title,
        session_id: session_id.to_string(),
        level,
    }
}

/// Honest backend name. Tests must not register a real login item.
pub fn login_item_backend() -> &'static str {
    if cfg!(target_os = "macos") {
        "smappservice"
    } else {
        "unsupported"
    }
}
