# Wave 29–30 — system-audio

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/system-audio/requirements.md
4. .kiro/specs/system-audio/design.md
5. .kiro/specs/system-audio/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/capture.rs
8. src-tauri/tests/contract.rs

Git branch is `feat/system-audio`. Conventional commits, subject ≤72 chars. You MAY commit on this branch. Do not push. Do not touch `main`.

## Do this wave

1. Add `system_tap_status()` so `CT-system-tap-status` passes.
2. Change `start_live(System)` so it never writes or returns the golden fixture. Off macOS it must still be `CAPTURE_UNSUPPORTED` recoverable. On macOS you may open a real tap; if you cannot, keep the recoverable error. Update `ct_mic_unsupported_is_recoverable` if a successful tap would panic it — Linux CI must still prove System is unsupported.

Named tests to add in `src-tauri/tests/contract.rs`:

- `ct_system_tap_status`
- `ct_system_not_fixture`

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
- Mix mic+system (later spec)
- Fake system audio by copying CONSULT-001

When tests pass, check the boxes in `.kiro/specs/system-audio/tasks.md` and stop.
