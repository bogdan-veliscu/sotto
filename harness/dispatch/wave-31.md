# Wave 31–32 — mixed-capture

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/mixed-capture/requirements.md
4. .kiro/specs/mixed-capture/design.md
5. .kiro/specs/mixed-capture/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/capture.rs
8. src-tauri/tests/contract.rs

Git branch is `feat/mixed-capture`. Conventional commits, subject ≤72 chars. You MAY commit on this branch. Do not push. Do not touch `main`.

## Do this wave

1. Add `mix_pcm` so `CT-mix-pcm` passes.
2. Change `start_live(Mixed)` so it never falls through to microphone-only. Off macOS, or on macOS when `system_tap_status` is not `available`, it must be `MIXED_UNAVAILABLE` or `CAPTURE_UNSUPPORTED` recoverable. When both backends can open, mix into `ChunkedRecorder`. Never write CONSULT-001.

Named tests to add in `src-tauri/tests/contract.rs`:

- `ct_mixed_not_mic_only`
- `ct_mix_pcm`

## Done gate

```
make graph
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --lib --test contract
cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bins
```

Do not wait forever on `make demo` if Keychain blocks. Demo must remain fixture-replay / `network_calls: 0` when it runs.

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Download models
- Enable cloud STT
- Log transcript text
- Push
- Add a desk source picker (later spec)
- Fake mixed audio by copying CONSULT-001 or by recording mic-only

When tests pass, check the boxes in `.kiro/specs/mixed-capture/tasks.md` and stop.
