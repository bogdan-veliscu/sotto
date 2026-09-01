# Parakeet runtime — Requirements

Lead-locked. Do not rewrite.

## REQ-PK-001: Runtime status is honest (INV-PARAKEET-RUNTIME-STATUS)

**EARS:** WHEN `parakeet_runtime_status` runs without a compiled Parakeet decoder, THE SYSTEM SHALL return `not-built`. WHEN a decoder is compiled in, THE SYSTEM SHALL return `ready`. It SHALL NOT claim `ready` unless on-device Parakeet inference is compiled into the binary.

CT-parakeet-runtime-status.

## REQ-PK-002: Parakeet never replays the fixture (INV-PARAKEET-NOT-FIXTURE)

**EARS:** WHEN `transcribe_local` runs for `parakeet-tdt-0.6b-v3` with a local weights file present, THE SYSTEM SHALL either return a `TranscriptResult` whose `engine_id` is `parakeet-tdt-0.6b-v3` and whose text is not the CONSULT-001 golden transcript, or return `ENGINE_NOT_BUILT` or `ENGINE_MODEL_INVALID` with `recoverable: true`. It SHALL NOT return `CLOUD_DISABLED`. `demo_pipeline` SHALL still use `fixture-replay`.

CT-parakeet-not-fixture.

No network. No silent cloud. Linux CI (`--no-default-features`) must stay `not-built` / `ENGINE_NOT_BUILT` without downloading weights.
