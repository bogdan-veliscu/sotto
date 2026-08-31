# Local STT — Requirements

Lead-locked. Do not rewrite.

## REQ-STT-001: Missing Whisper weights (INV-WHISPER-LOCAL-ONLY)

**EARS:** WHEN `transcribe_local` is asked for `whisper-large-v3-turbo` and the weights file is absent, THE SYSTEM SHALL return `ENGINE_NOT_INSTALLED` with `recoverable: true`, SHALL NOT return fixture-replay text, and SHALL NOT select a cloud/api engine.

CT-whisper-local-only.

## REQ-STT-002: Demo never downloads (INV-DEMO-NO-DOWNLOAD)

**EARS:** WHEN `demo_pipeline` runs, THE SYSTEM SHALL keep `network_calls` at 0, SHALL use `fixture-replay`, and SHALL NOT create Whisper weight files under the data directory.

CT-demo-no-download.

## REQ-STT-003: Local file, not a URL (INV-WHISPER-WEIGHTS-LOCAL)

**EARS:** WHEN a file exists at `whisper_weights_path` but is not a valid local Whisper model, THE SYSTEM SHALL return `ENGINE_MODEL_INVALID` and SHALL NOT fetch weights from the network.

CT-whisper-weights-are-local.

Fixture fallback to `fixture-replay` is allowed only when `SOTTO_ALLOW_FIXTURE_FALLBACK=1`. The UI path never silent-cloud.
