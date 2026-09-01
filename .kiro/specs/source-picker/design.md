# Source picker — Design

The desk currently hardcodes `mic`. Mixed and system taps already exist. This spec lets the user pick a lane before Record. Consent is unchanged.

## Lib

`src-tauri/src/capture.rs`:

- `CaptureSource::try_parse` — `mic` / `system` / `mixed` only. Else `SOURCE_UNKNOWN` recoverable.
- `CaptureSource::as_str`
- `source_permission_hint(source, tap_status) -> String` — always mentions consent; mixed mentions no mic-only fallback. Uses `system_tap_status` values already defined.

`recorder_start` validates the source before `create_session`. Default when omitted: `mic` (current desk). `recorder_begin` uses `try_parse` on the stored session source.

Tests never call `start_live(Mic)`. Tests never prompt for Screen Recording.

## Desktop

Header select: Microphone / What you hear / Mixed. Persist `capture_source` in settings. Consent modal shows the hint, then the same consent card. Hotkey Record uses the saved source. Settings copy no longer says Record is always mic.

## Forbidden

- Silent-start / skipping consent
- Meeting bot
- Mixed falling back to mic-only
- Downloading models
