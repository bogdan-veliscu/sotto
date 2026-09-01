# Parakeet runtime — Design

On-device Parakeet TDT 0.6B v3. Install already writes `cache_dir/models/parakeet-tdt-0.6b-v3.bin`. This spec decodes that file. It does not download it.

## Lib

`src-tauri/src/stt.rs` (optional `src-tauri/src/stt_parakeet.rs`):

- `parakeet_runtime_status() -> &'static str`
  - decoder not compiled (`parakeet` Cargo feature off, including Linux CI): `not-built`
  - decoder compiled: `ready` (weights may still be missing — that is `ENGINE_NOT_INSTALLED` at transcribe time)
- `transcribe_local("parakeet-tdt-0.6b-v3", wav, cache_dir)`
  - missing file: `ENGINE_NOT_INSTALLED` (unchanged)
  - URL-shaped path: `ENGINE_MODEL_INVALID` (unchanged)
  - file present, decoder not compiled: `ENGINE_NOT_BUILT` recoverable
  - file present, decoder compiled, not a valid Parakeet/ONNX payload (including the contract-test blob): `ENGINE_MODEL_INVALID`
  - file present, decoder compiled, valid model: on-device transcript with `engine_id = parakeet-tdt-0.6b-v3`
  - never copy `fixtures/sessions/CONSULT-001.transcript.json` text into the result
  - never HTTP

Optional Cargo feature `parakeet`, **off by default** and **off on Linux CI** (`cargo test --no-default-features`). Do not pull a 1.2 GB model into git. Do not enable the feature from `desktop` until decode is proven local.

The test blob `parakeet-test-blob` is not a model. A compiled decoder must reject it as `ENGINE_MODEL_INVALID`, not invent speech.

`make demo` stays `fixture-replay`. Whisper stays the working engine when its weights exist.

## Desktop

Settings may show runtime status next to the Parakeet install row. Transcribe on the desk still requires an installed valid model. Consent is unrelated.

## Forbidden

- Replaying the golden transcript as Parakeet
- Silent cloud STT
- Downloading weights in `make demo` or contract tests
- Bundling the real 0.6B weights in the repo
