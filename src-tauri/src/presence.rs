use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HudView {
    pub led_on: bool,
    pub paused: bool,
    pub elapsed_ms: u64,
    pub clock: String,
    pub caption: String,
}

pub fn format_clock(elapsed_ms: u64) -> String {
    let total = elapsed_ms / 1000;
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// LED + caption for the notch HUD and the tray tooltip.
pub fn hud_from_status(status: &str, elapsed_ms: u64) -> HudView {
    let led_on = status == "recording";
    let paused = status == "paused";
    let caption = match status {
        "recording" => "on this Mac",
        "paused" => "paused",
        _ => "Sotto",
    };
    HudView {
        led_on,
        paused,
        elapsed_ms,
        clock: format_clock(elapsed_ms),
        caption: caption.to_string(),
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
