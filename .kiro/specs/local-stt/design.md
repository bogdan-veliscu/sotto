# Local STT — Design

Module: `src-tauri/src/stt.rs`. Public API is locked by the contract tests. Do not rename.

```rust
pub const WHISPER_ENGINE_ID: &str = "whisper-large-v3-turbo";
pub const FIXTURE_FALLBACK_ENV: &str = "SOTTO_ALLOW_FIXTURE_FALLBACK";

pub fn whisper_weights_path(cache_dir: &Path) -> PathBuf;
pub fn transcribe_local(
    engine_id: &str,
    wav: &[u8],
    cache_dir: &Path,
) -> Result<TranscriptResult>;
```

## Weights path

`cache_dir/models/ggml-large-v3-turbo.bin`. Filesystem only. Reject `http://` and `https://` in the path. Never download inside `transcribe_local` or `demo_pipeline`.

User-initiated download belongs to spec `model-install` (PR 3). This PR only consumes weights that are already on disk.

## `transcribe_local`

| engine_id | weights | result |
|-----------|---------|--------|
| `fixture-replay` | n/a | golden fixture transcript |
| `whisper-large-v3-turbo` | missing | `ENGINE_NOT_INSTALLED`, recoverable, hint to install locally |
| `whisper-large-v3-turbo` | present, invalid | `ENGINE_MODEL_INVALID`, recoverable |
| `whisper-large-v3-turbo` | present, valid ggml | `TranscriptResult` with `engine_id` = whisper id, timestamps kept |
| any cloud/api id | n/a | `CLOUD_DISABLED` unless `cloud_mode` is on (existing policy) |

Do not fall back to fixture-replay unless `SOTTO_ALLOW_FIXTURE_FALLBACK` is `"1"`.

## `resolve_engine`

If the requested engine is local and not `Ready`, return `ENGINE_NOT_INSTALLED` instead of silently picking fixture-replay — unless the fixture-fallback env is set.

Catalog JSON still lists whisper as `not-installed`. Runtime readiness is the weights file, not the JSON.

## Store

`Store::transcribe` must call `transcribe_local` with the store's data directory as `cache_dir`. Stop returning `ENGINE_NOT_WIRED` for whisper.

## Demo

`demo_pipeline` keeps `fixture-replay` and `CONSULT-001.wav`. Set or rely on fixture fallback so demo cannot start a download. `network_calls` stays 0.

## Inference (wave 10)

Batch, not streaming. Optional Cargo feature `whisper` may pull `whisper-rs` so `--no-default-features` Linux CI does not compile ggml. Without that feature, a present-but-invalid file still maps to `ENGINE_MODEL_INVALID` (the file was consulted locally). With the feature and real Large-v3 Turbo weights, transcribe on-device.

Keep `make demo` free of network after crates are cached.

## Forbidden

- Hugging Face / OpenAI / any HTTP fetch from this module
- Silent cloud STT
- Editing `fixtures/`
- Bundling 1.6 GB weights in git
