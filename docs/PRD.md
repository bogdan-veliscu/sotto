# Product requirements

## Goals

Private meeting recorder for macOS that records, transcribes locally, and stores outputs securely.

## Target launch

macOS only. Manual start/stop. Local transcription. Local summaries. Local search. Encrypted audio at rest.

## User stories

- Start recording with one click (after consent).
- Pause and resume.
- Transcribe a completed recording locally.
- Choose between supported models.
- See transcript segments with timestamps.
- Search across all meetings.
- Export transcript and summary.

## Recording

Capture system audio on supported macOS versions. Capture microphone optionally. Pause/resume/stop. Save audio to local encrypted storage.

Wave 1: fixture capture stands in for Core Audio taps so the rest of the pipeline is real.

## Transcription

Fully on-device by default. At least two model backends in the catalog. Batch transcription for recorded files. Streamed transcription later.

## Notes

Summary, action items, key points.

## Search

Full-text across transcript text. Title / date / tags later.

## Settings

Model selection. Audio input (later). Retention. Export format. Telemetry privacy toggle (default off).

## Non-functional

Low setup friction. Crash-safe recording (partial save). Fast startup. Offline first. No cloud dependency for core. Clear permissions messaging.

## Privacy

Audio never leaves the device unless the user enables a cloud mode. No telemetry by default. Encrypted audio at rest. Easy delete-all.

## Platforms

v1 macOS 14.4+ preferred. v2 Windows. v3 Linux.

## Out of scope

Real-time conferencing bot. Server-side transcription in v1. Multi-user teams. Browser extension.
