# Wave 33–34 — parakeet-runtime

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/parakeet-runtime/requirements.md
4. .kiro/specs/parakeet-runtime/design.md
5. .kiro/specs/parakeet-runtime/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/stt.rs
8. src-tauri/tests/contract.rs

Git branch is `feat/parakeet-runtime`. Conventional commits, subject ≤72 chars. You MAY commit on this branch. Do not push. Do not touch `main`.

## Do this wave

1. Add `parakeet_runtime_status()` so `CT-parakeet-runtime-status` passes. Off a compiled decoder it must be `not-built`. Do not claim `ready` without a decoder compiled in.
2. Change `transcribe_local` for `parakeet-tdt-0.6b-v3` so an installed weights file never yields the CONSULT-001 golden transcript. Dummy `parakeet-test-blob` is not a model: `ENGINE_NOT_BUILT` or `ENGINE_MODEL_INVALID`, recoverable. Never `CLOUD_DISABLED`. `demo_pipeline` stays fixture-replay.

Named tests to add in `src-tauri/tests/contract.rs`:

- `ct_parakeet_runtime_status`
- `ct_parakeet_not_fixture`

Export `parakeet_runtime_status` from the lib like `system_tap_status`. Optional Cargo feature `parakeet` may exist but must stay **off** by default so Linux CI (`--no-default-features`) stays `not-built`.

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
- Bundle real 0.6B weights
- Replay CONSULT-001 text as Parakeet
- Move transcribe off the Tauri thread (later spec)
- Add a desk source picker (later spec)

When tests pass, check the boxes in `.kiro/specs/parakeet-runtime/tasks.md` and stop.
