# System audio — Design

Record the Mac's system mix (what you hear), not a meeting bot.

## Lib

`src-tauri/src/capture.rs` (and optional `capture_system.rs` behind `cfg(target_os = "macos")`):

- `system_tap_status() -> &'static str`
  - non-macOS: `unsupported`
  - macOS without a tap backend compiled: `unsupported`
  - macOS with a tap backend, permission missing: `needs-permission`
  - macOS with a tap backend, permission granted or not yet prompted: `available`
- `start_live(CaptureSource::System, dir)`
  - non-macOS: `CAPTURE_UNSUPPORTED` recoverable (unchanged)
  - macOS: try the tap; on failure return the same recoverable error
  - never copy `fixtures/sessions/CONSULT-001.wav`

`Mic` / `Mixed` stay on the existing CPAL thread. Mixing system+mic is a later spec. Tests never require Screen Recording. `make demo` stays fixture-replay.

## Desktop

Desk may show the status string. Do not silently switch source to mic. Consent still required before any capture.
