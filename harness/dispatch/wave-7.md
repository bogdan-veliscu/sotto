# Wave 7–8 — live-capture

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/live-capture/requirements.md
4. .kiro/specs/live-capture/design.md
5. .kiro/specs/live-capture/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/capture.rs (stub, returns NOT_IMPLEMENTED)
8. src-tauri/tests/contract.rs (RED contract tests)

## Do this wave

Implement `src-tauri/src/capture.rs` so these tests pass:

- `ct_capture_wav`
- `ct_pause_resume`
- `ct_crash_partial`
- `ct_demo_still_offline`
- `ct_mic_unsupported_is_recoverable`

`start_live(CaptureSource::System, …)` MUST return error code `CAPTURE_UNSUPPORTED` with recoverable true. Do not implement Core Audio taps in this wave. Optional: CPAL for Mic, but tests must not require a hardware mic.

`demo_pipeline` must stay on `fixtures/sessions/CONSULT-001.wav` and `fixture-replay`.

## Done gate

```
make graph
cd src-tauri && cargo test --test contract
make demo
```

All capture CTs green. `make demo` prints `network_calls: 0`.

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Download models
- Enable cloud STT
- Log transcript text
- Commit or push
- Change the public function names in capture.rs

When tests pass, check the boxes in `.kiro/specs/live-capture/tasks.md` and stop.
