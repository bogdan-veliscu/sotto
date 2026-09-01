# Parakeet runtime — Design

On-device Parakeet TDT 0.6B v3 via `parakeet-rs`. It does not download weights.

Install still writes the checksum blob `cache_dir/models/parakeet-tdt-0.6b-v3.bin` (contract tests). That blob is **not** a model. Real decode needs a **directory**:

`cache_dir/models/parakeet-tdt-0.6b-v3/{encoder-model.onnx, decoder_joint-model.onnx, vocab.txt}`

`is_installed` / overlay `ready` if the blob **or** that TDT layout exists. `delete_model` removes both.

## Lib

`src-tauri/src/stt.rs` and `src-tauri/src/stt_parakeet.rs`:

- `parakeet_runtime_status() -> &'static str`
  - decoder not compiled (default, Linux CI, `--no-default-features`): `not-built`
  - `parakeet-rs` inference compiled in (`desktop` includes `parakeet`): `ready`
  - do not claim `ready` for a feature flag with no inference
- `transcribe_local("parakeet-tdt-0.6b-v3", wav, cache_dir)`
  - blob and TDT directory both missing: `ENGINE_NOT_INSTALLED` (unchanged)
  - URL-shaped path: `ENGINE_MODEL_INVALID` (unchanged)
  - present, decoder not compiled: `ENGINE_NOT_BUILT` recoverable
  - decoder compiled, dummy blob / incomplete dir: `ENGINE_MODEL_INVALID`
  - decoder compiled, valid TDT directory: on-device transcript with `engine_id = parakeet-tdt-0.6b-v3`
  - never copy `fixtures/sessions/CONSULT-001.transcript.json` text into the result
  - never HTTP

Optional Cargo feature `parakeet`, **off by default** and **off on Linux CI** (`cargo test --no-default-features`). Enabled from `desktop` so the Mac app has a second local engine. Do not pull a 1.2 GB model into git. Do not download weights in `make demo` or contract tests.

The test blob `parakeet-test-blob` is not a model. A compiled decoder must reject it as `ENGINE_MODEL_INVALID`, not invent speech.

`make demo` stays `fixture-replay`. Whisper stays the working engine when its weights exist.

## Desktop

Settings may show runtime status next to the Parakeet install row. Transcribe on the desk still requires an installed valid TDT directory. Consent is unrelated.

## Forbidden

- Replaying the golden transcript as Parakeet
- Silent cloud STT
- Downloading weights in `make demo` or contract tests
- Bundling the real 0.6B weights in the repo
