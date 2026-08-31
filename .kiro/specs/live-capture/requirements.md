# Live capture — Requirements

Lead-locked. Do not rewrite.

## REQ-LC-001: Valid WAV (INV-CAPTURE-WAV)

**EARS:** WHEN `record_sine` runs for a positive duration, THE SYSTEM SHALL return bytes whose header is `RIFF`/`WAVE`, PCM, 16-bit, mono, 16 kHz.

CT-capture-wav.

## REQ-LC-002: Pause / resume (INV-PAUSE-RESUME)

**EARS:** WHEN a `ChunkedRecorder` is paused, THE SYSTEM SHALL ignore PCM until resume, AND WHEN it stops, `duration_ms` SHALL equal the unpaused audio length within 50 ms.

CT-pause-resume.

## REQ-LC-003: Crash partial (INV-CRASH-PARTIAL)

**EARS:** WHEN a `ChunkedRecorder` flushes at least one chunk and is dropped without `stop`, THE SYSTEM SHALL recover a valid WAV from that directory via `ChunkedRecorder::recover`.

CT-crash-partial.

## REQ-LC-004: Demo stays offline fixture (INV-DEMO-STILL-OFFLINE)

**EARS:** WHEN `demo_pipeline` runs, THE SYSTEM SHALL still use `fixture-replay`, SHALL keep `network_calls` at 0, and SHALL NOT require a microphone.

CT-demo-still-offline.

## REQ-LC-005: Mic backend (wave 8)

**EARS:** WHEN capture source is `Mic` and a platform backend is available, THE SYSTEM SHALL write PCM through `ChunkedRecorder`. WHEN the backend is missing, THE SYSTEM SHALL return `CAPTURE_UNSUPPORTED` with `recoverable: true`.

CT-mic-unsupported-is-recoverable.

System-audio taps may return the same error in this PR. Do not fake system audio by uploading anywhere.
