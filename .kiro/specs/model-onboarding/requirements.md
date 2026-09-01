# Model onboarding — Requirements

Lead-locked. Do not rewrite.

## REQ-MO-001: Runnable readiness (INV-MODEL-RUNNABLE-READY)

**EARS:** WHEN the engine catalog reports a local engine as `ready` for live transcription, THE SYSTEM SHALL require both a compiled decoder and a runnable local model layout. A Parakeet checksum `.bin` without `encoder-model.onnx`, `decoder_joint-model.onnx`, and `vocab.txt` SHALL NOT be reported as ready.

CT-model-runnable-ready.

## REQ-MO-002: Local import only (INV-MODEL-IMPORT-LOCAL)

**EARS:** WHEN a user imports local model weights, THE SYSTEM SHALL accept a Whisper model file or a complete Parakeet TDT directory, SHALL validate the selected layout before activation, and SHALL NOT fetch replacement files from the network. `make demo` and contract tests SHALL NOT import or download model weights.

CT-model-import-local.

## REQ-MO-003: Live transcription never defaults to fixture replay (INV-LIVE-ENGINE-RUNNABLE)

**EARS:** WHEN a live recording is stopped, THE SYSTEM SHALL transcribe with an explicitly selected runnable non-fixture local engine or keep the encrypted recording in a recoverable `recorded` state with `ENGINE_SETUP_REQUIRED`. It SHALL NOT run `fixture-replay` on live audio. `demo_pipeline` SHALL continue to use `fixture-replay` with zero network calls.

CT-live-engine-runnable.

No cloud default. No auto-download. No model weights in git. No meeting bot.
