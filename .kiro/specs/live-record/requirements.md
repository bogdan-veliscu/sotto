# Live record — Requirements

Lead-locked. Do not rewrite.

## REQ-LR-001: Stop encrypts live chunks, not the golden fixture (INV-LIVE-STOP-NOT-FIXTURE)

**EARS:** WHEN a consented session is finalized through `Store::finalize_live` with a `ChunkedRecorder` that received PCM, THE SYSTEM SHALL encrypt that recorder's WAV and SHALL NOT substitute `CONSULT-001.wav`.

CT-live-stop-not-fixture.

`make demo` / `demo_pipeline` SHALL still ingest the golden fixture. System-audio taps remain `CAPTURE_UNSUPPORTED`.

## REQ-LR-002: Fixture-replay only on the golden WAV (INV-FIXTURE-AUDIO-MISMATCH)

**EARS:** WHEN `transcribe_local` is asked to use `fixture-replay` on audio that is not the golden CONSULT-001 WAV, THE SYSTEM SHALL return `FIXTURE_AUDIO_MISMATCH` and SHALL NOT attach the golden transcript.

CT-fixture-audio-mismatch.

No network. No silent cloud. No meeting bot.
