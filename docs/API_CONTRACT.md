# API contract

Local Tauri IPC only.

## Commands (wave 1)

| Command | Notes |
|---|---|
| `settings_get` / `settings_set` | key/value |
| `engines_list` | catalog |
| `recorder_start` | creates session, still pending consent |
| `recorder_consent` | acknowledge disclosure |
| `recorder_begin` / `pause` / `resume` | status + live `ChunkedRecorder` |
| `recorder_stop` | encrypt live chunks (never CONSULT-001) |
| `recorder_stop_fixture` | encrypt golden WAV (`make demo` only) |
| `transcribe_run` | fixture engine |
| `sessions_list` / `sessions_get` | |
| `search_query` | FTS5 |
| `sessions_rename` / `sessions_export` / `sessions_delete` | |
| `data_delete_all` | |

Errors return `{ code, message, recoverable, action_hint }`.

## Planned events

`recording.started`, `recording.stopped`, `transcription.progress`, `transcription.completed`, `transcription.failed`, `model.install.*`
