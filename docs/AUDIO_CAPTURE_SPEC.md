# Audio capture

Capture meeting audio on macOS with clear consent.

## Modes

System only. Mic only. Mixed. Per-app later.

## Platform

macOS 14.4+ preferred for Core Audio taps. `Info.plist` already includes `NSMicrophoneUsageDescription` and `NSAudioCaptureUsageDescription`.

Wave 1: `recorder_stop_fixture` still encrypts `CONSULT-001.wav` for `make demo`. The desk Record button uses `recorder_stop` + microphone chunks (`live-record`). System-audio uses ScreenCaptureKit when Screen Recording is already granted; otherwise `CAPTURE_UNSUPPORTED` / `needs-permission`. Tests never prompt.

## Pipeline

1. Record (after consent).
2. Request permissions if missing (taps wave).
3. Capture session.
4. Chunked write to a temp file.
5. Finalize on stop.
6. Encrypt; delete temp plaintext.
7. Hand ciphertext path to STT (decrypt into a temp for the worker, then delete).

## Format

16 kHz mono WAV/FLAC for STT. Preserve original only if requested.

## Failures

Permission denied. Unsupported OS. Device gone. Disk full. Capture crash. Save partial audio when possible. Never lose completed audio because transcription failed.

## Consent

Visible recording LED. Pre-record disclosure. Custom text later.
