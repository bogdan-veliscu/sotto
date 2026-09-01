# Hotkey — Design

Global toggle (and optional push-to-talk) for the same Record/Stop path as the desk.

## Lib

`src-tauri/src/hotkey.rs` (no desktop feature):

- `DEFAULT_TOGGLE = "CommandOrControl+Shift+Space"`
- `parse_hotkey` / `parse_hotkey_mode`
- Settings keys `hotkey_toggle`, `hotkey_mode`

## Desktop

`tauri-plugin-global-shortcut` registers the stored shortcut. On press, emit `sotto://hotkey` with `{ mode, state: "pressed"|"released" }`. The desk listens: toggle starts/stops via the existing consent flow; PTT pressed resumes-or-prompts, released pauses. Never silent-start.

Tests do not register a real OS shortcut.
