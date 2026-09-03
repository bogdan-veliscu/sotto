# API contract

Local Tauri IPC only.

## Commands

| Command | Notes |
|---|---|
| `settings_get` / `settings_set` | key/value |
| `engines_list` | catalog with honest `live_ready` |
| `recorder_start` | creates session, still pending consent |
| `recorder_consent` | acknowledge disclosure |
| `recorder_begin` / `pause` / `resume` | status + live `ChunkedRecorder` |
| `recorder_stop` | encrypt live chunks (never CONSULT-001) |
| `recorder_stop_fixture` | encrypt golden WAV (`make demo` only) |
| `transcribe_run` | explicit local engine; live audio never uses fixture-replay |
| `recovery_list` / `recovery_recover` / `recovery_discard` | crashed live chunks |
| `model_import_local` | Whisper file or Parakeet TDT directory; no network |
| `sessions_list` / `sessions_get` | |
| `search_query` | FTS5 |
| `sessions_rename` / `sessions_export` / `sessions_delete` | |
| `data_delete_all` | |

Errors return `{ code, message, recoverable, action_hint }`.

## Planned events

`recording.started`, `recording.stopped`, `transcription.progress`, `transcription.completed`, `transcription.failed`, `model.install.*`
