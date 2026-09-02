# Model abstraction

Stable interface so local (and later optional cloud) backends swap without changing UI or storage.

## Interface

See `src-tauri/src/engines.rs` (`Engine`, `resolve_engine`, `TranscriptResult`).

Each engine must eventually implement: availability, install/download, health, version, transcribe, error mapping, fallback.

## v1 engines

| id | Role |
|---|---|
| `fixture-replay` | Always-ready local demo. Golden transcript. |
| `apple-speech-ondevice` | macOS on-device SpeechAnalyzer. Audio is not sent to Apple servers. |
| `parakeet-tdt-0.6b-v3` | Local TDT. Import a folder or user-started INT8/FP32 download. Dummy checksum blob is not a model. |
| `whisper-large-v3-turbo` | Compatibility baseline. Local ggml import. Not the API. |

## Selection policy

1. Try the user's selected model.
2. If unavailable, fall back to a ready **local** model.
3. If hardware is insufficient, recommend a smaller local model.
4. Never silently switch to cloud unless `cloud_mode=on`.

## Downloads

User-started Parakeet INT8/FP32 from pinned Hugging Face files. Staging + layout check + atomic activate. Never in `make demo`. `import_local` still refuses URLs.
