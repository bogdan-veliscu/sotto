# Model abstraction

Stable interface so local (and later optional cloud) backends swap without changing UI or storage.

## Interface

See `src-tauri/src/engines.rs` (`Engine`, `resolve_engine`, `TranscriptResult`).

Each engine must eventually implement: availability, install/download, health, version, transcribe, error mapping, fallback.

## v1 engines

| id | Role |
|---|---|
| `fixture-replay` | Always-ready local demo. Golden transcript. |
| `parakeet-tdt-0.6b-v3` | Planned default local model. Not installed. |
| `whisper-large-v3-turbo` | Planned robustness baseline. Not installed. Local weights, not the API. |

## Selection policy

1. Try the user's selected model.
2. If unavailable, fall back to a ready **local** model.
3. If hardware is insufficient, recommend a smaller local model.
4. Never silently switch to cloud unless `cloud_mode=on`.

## Downloads (later)

App-managed cache, checksum, progress, delete/reinstall. Never in `make demo`.
