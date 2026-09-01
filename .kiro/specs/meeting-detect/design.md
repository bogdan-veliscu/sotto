# Meeting detect — Design

Watch local process names (Zoom / Teams / Slack). Ask before record. Never silent-start.

## Lib

`src-tauri/src/meeting.rs` (no desktop feature):

- `classify_processes` — exact process stems: `zoom.us` / `zoom`, `slack`, `microsoft teams` / `teams` / `ms-teams`
- `should_prompt(detected, recording, enabled)` — true only when enabled, idle, and something classified
- `prompt_copy` — names the apps and says consent is still required
- Setting `meeting_detect` is `on` / `off`. Default off.

Tests inject process names. They never spawn Zoom.

## Desktop

`meeting_detect_get` / `meeting_detect_set` scan via `ps -caxo comm=` on macOS (empty list elsewhere). The desk shows a card; Record still opens the existing consent disclosure.
