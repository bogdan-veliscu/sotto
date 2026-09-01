# STT worker — Design

Whisper and Parakeet inference can take a long time. Today `transcribe_run` locks `Mutex<Store>` for the whole call, so the desk, HUD, and hotkey cannot use the store.

## Split

`src-tauri/src/store.rs` + `src-tauri/src/stt.rs`:

- `TranscribeJob { engine_id, wav, cache_dir }` — Send, no Store.
- `Store::prepare_transcribe(session_id, model)` — resolve engine, decrypt WAV. Short lock.
- `transcribe_job(job)` — `transcribe_local`. No mutex.
- `Store::commit_transcript(session_id, result)` — persist + detail. Short lock.
- `Store::transcribe` — prepare + job + commit on `&self` for demo and existing tests.

## Desktop

`transcribe_run` is async. It prepares under the mutex, `spawn_blocking` for `transcribe_job`, then commits under the mutex. The IPC thread is not blocked on ggml/ONNX.

HUD and settings commands keep locking the store independently.

## Forbidden

- Silent cloud STT
- Downloading weights
- Holding `Mutex<Store>` across inference
- Changing `demo_pipeline` off fixture-replay
