# Live record — Design

Desk Record/Pause/Stop drive a live `ChunkedRecorder`. The golden fixture stays on `demo_pipeline` and on `recorder_stop_fixture` (kept for offline demo; the desk does not call it).

## Begin

`recorder_begin` parses the session source:

- `system` → `start_live(System)` → `CAPTURE_UNSUPPORTED` (Core Audio taps are later).
- `mic` / `mixed` → start `ChunkedRecorder` under `{data_dir}/live/{session_id}` and, on macOS, a CPAL input stream that writes PCM into it. Mixed is mic-only until taps exist.

If the microphone backend is missing, return `CAPTURE_UNSUPPORTED` with `recoverable: true`. Do **not** fall back to CONSULT-001.

Pause/resume flip both session status and `ChunkedRecorder::pause` / `resume`.

## Stop

`recorder_stop` removes the live handle and calls `Store::finalize_live`. That stops the recorder (WAV from chunks) and encrypts those bytes. Missing live handle → `CAPTURE_NOT_STARTED`, never the fixture.

## Transcribe

`fixture-replay` compares WAV bytes to the golden file. Mismatch → `FIXTURE_AUDIO_MISMATCH`. The desk shows the hint and still keeps the encrypted audio. Install Whisper/Parakeet to transcribe a live take.

## Tests

Do not call `start_live(Mic)` inside `cargo test` (it would prompt for the mic on macOS). Inject PCM through `ChunkedRecorder::write_pcm`.

## Forbidden

- Silent fixture fallback from the Record button.
- Network.
- Core Audio process taps in this spec.
