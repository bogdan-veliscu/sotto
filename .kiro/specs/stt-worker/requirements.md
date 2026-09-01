# STT worker — Requirements

Lead-locked. Do not rewrite.

## REQ-SW-001: Inference does not hold the store mutex (INV-STT-WORKER-LOCK)

**EARS:** WHEN batch transcription runs, THE SYSTEM SHALL release the Store mutex before on-device inference and SHALL NOT hold it during `transcribe_local`. Other desk reads (session list, settings) SHALL be able to lock the store while inference runs.

CT-stt-worker-releases-lock.

## REQ-SW-002: Worker result matches in-process transcribe (INV-STT-WORKER-RESULT)

**EARS:** WHEN `transcribe_job` runs for a CONSULT-001 session with `fixture-replay`, THE SYSTEM SHALL return the same `engine_id` and transcript text as `Store::transcribe`. It SHALL NOT return `CLOUD_DISABLED`. `demo_pipeline` SHALL still use `fixture-replay`.

CT-stt-worker-same-result.

No network. No silent cloud. Consent is unrelated. Linux CI stays `--no-default-features`.
