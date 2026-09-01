# Presence — Design

Menu bar + notch HUD + login item. Capture stays Wave B. Hotkeys are Wave D.

## HUD model (lib)

`src-tauri/src/presence.rs` is compiled without the desktop feature so Linux CI can lock the copy:

```rust
pub struct HudView { pub led_on: bool, pub paused: bool, pub elapsed_ms: u64, pub clock: String, pub caption: String }
pub fn hud_from_status(status: &str, elapsed_ms: u64) -> HudView;
```

The desk and a borderless `hud` webview both render this. The HUD window is shown while recording or paused, hidden when idle. Clock ticks in the HUD webview.

## Login item

Preference `launch_at_login` is `on`/`off` in the store. Applying it uses `tauri-plugin-autostart` with `MacosLauncher::LaunchAgent` (this plugin version has no SMAppService API). Tests never register a real login item. `login_item_backend()` is `smappservice` on macOS (the intended OS service) and `unsupported` elsewhere.

## Tray

Tauri tray icon, menu: Open desk, Record, Stop. Same consent path as the desk Record button (do not silent-start).

## Forbidden

- Starting capture from login without the consent card.
- Network.
- Notch HUD that looks like a meeting bot.
