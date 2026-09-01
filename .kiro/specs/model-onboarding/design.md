# Model onboarding — Design

Make the first live recording path honest and usable without changing the offline demo.

## Runtime readiness

Introduce a readiness view that combines build capability and on-disk layout. `fixture-replay` remains ready only for the golden demo. Whisper is runnable only when the decoder is compiled and a locally selected model passes existing validation. Parakeet is runnable only when the decoder is compiled and `parakeet_tdt_layout_ok` succeeds. The historical checksum `.bin` remains a contract-test artifact and must not make the desk show Parakeet as ready.

## Local import

The desk uses a file picker for Whisper and a directory picker for Parakeet. A backend import command validates the source, copies into a sibling staging path, verifies the final layout, and atomically activates it under the app data `models/` directory. Failed imports remove staging data and leave the previous runnable model untouched.

There is no URL input and no HTTP client. Large weights are supplied by the user and never enter git, fixtures, `make demo`, or contract tests.

## First-run and live selection

Onboarding explains the difference between the fixture judge path and live transcription, shows decoder/layout state, and offers local import. The desk may still record before a model is installed, but Stop must leave encrypted audio available for a later retry and show `ENGINE_SETUP_REQUIRED` instead of presenting fixture mismatch as the normal first-run result.

For live sessions, resolve an explicitly selected runnable non-fixture local engine. If none is selected and exactly one runnable non-fixture engine exists, the desk may offer it but must not silently persist the choice. `demo_pipeline` continues to request `fixture-replay` explicitly.

## Tests

Use tiny synthetic directory/file layouts. Do not include real weights and do not invoke a decoder. Tests must not prompt for microphone or Screen Recording access.

## Forbidden

- Treating the Parakeet checksum blob as runnable weights
- Downloading models or adding model URLs
- Falling back from live audio to fixture or cloud STT
- Editing `fixtures/` or `fixture-lock.json`
