# Mixed capture — Design

Record what you hear and what you say into one `ChunkedRecorder`. Never pretend mixed is mic-only.

## Lib

`src-tauri/src/capture.rs` + `capture_mix.rs`:

- `mix_pcm(mic, system) -> Vec<i16>` — saturating average; pad the shorter slice with zeros
- `start_live(CaptureSource::Mixed, dir)`
  - non-macOS: `MIXED_UNAVAILABLE` recoverable
  - macOS without `CGPreflightScreenCaptureAccess`: same error; do not open CPAL
  - macOS with preflight: ScreenCaptureKit + CPAL both write into a `MixBus`; the bus emits mixed PCM
  - if either backend fails after preflight, drop the other and return the recoverable error
  - never copy `fixtures/sessions/CONSULT-001.wav`

`Mic` and `System` stay single-source. Tests never prompt: Mixed returns before `SCShareableContent::get()` and before CPAL when Screen Recording is off. `make demo` stays fixture-replay.

## Desktop

Desk Record still uses `mic` until the source-picker spec. Settings may say mixed needs both Screen Recording and microphone. Consent still required.
