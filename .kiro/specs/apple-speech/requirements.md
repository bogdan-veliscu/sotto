# Apple Speech — Requirements

## REQ-AS-001: On-device Apple Speech (INV-APPLE-SPEECH-ONDEVICE)

**EARS:** WHEN the catalog includes `apple-speech-ondevice`, THE SYSTEM SHALL treat it as a local engine. Live-ready SHALL require an on-device Apple speech recognizer. Transcription SHALL NOT send audio to Apple servers. Linux CI and `make demo` SHALL NOT invoke Apple Speech, SHALL keep `network_calls` at 0, and SHALL keep demo on `fixture-replay`.

CT-apple-speech-ondevice.

No cloud default. No meeting bot. No silent server fallback.
