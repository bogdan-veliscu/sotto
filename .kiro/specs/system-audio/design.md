# System audio — Design

Record the Mac's system mix (what you hear), not a meeting bot.

## Lib

`src-tauri/src/capture.rs` (and optional `capture_system.rs` behind `cfg(target_os = "macos")`):

- `system_tap_status() -> &'static str`
  - non-macOS: `unsupported`
  - macOS, Screen Recording off: `needs-permission` (`CGPreflightScreenCaptureAccess` only — never prompt)
  - macOS, Screen Recording already granted: `available`
- `start_live(CaptureSource::System, dir)`
  - non-macOS: `CAPTURE_UNSUPPORTED` recoverable (unchanged)
  - macOS without preflight: same recoverable error; do not call `SCShareableContent::get()`
  - macOS with preflight: ScreenCaptureKit audio tap into `ChunkedRecorder`
  - never copy `fixtures/sessions/CONSULT-001.wav`

`Mic` / `Mixed` stay on the existing CPAL thread. Mixing system+mic is a later spec. Tests never require Screen Recording. `make demo` stays fixture-replay.

## Desktop

Desk may show the status string. Do not silently switch source to mic. Consent still required before any capture.
