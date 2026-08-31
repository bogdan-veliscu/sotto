# Wave 9–10 — local-stt

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/local-stt/requirements.md
4. .kiro/specs/local-stt/design.md
5. .kiro/specs/local-stt/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/stt.rs (stub, returns NOT_IMPLEMENTED)
8. src-tauri/src/engines.rs
9. src-tauri/src/store.rs (`transcribe` still returns ENGINE_NOT_WIRED for non-fixture)
10. src-tauri/tests/contract.rs (RED contract tests)

## Do this wave

Implement `src-tauri/src/stt.rs` (and wire `Store::transcribe` / `resolve_engine`) so these tests pass:

- `ct_whisper_local_only`
- `ct_demo_no_download`
- `ct_whisper_weights_are_local`

Missing Whisper weights → `ENGINE_NOT_INSTALLED`. Present garbage file → `ENGINE_MODEL_INVALID`. Never HTTP. Never silent cloud.

`demo_pipeline` must stay on `fixtures/sessions/CONSULT-001.wav` and `fixture-replay`. Do not download weights. Do not add 1.6 GB models to git.

Fixture fallback to `fixture-replay` only when `SOTTO_ALLOW_FIXTURE_FALLBACK=1`.

Linux CI runs `cargo test --no-default-features`. Optional `whisper` Cargo feature may pull `whisper-rs`; core tests must pass without compiling ggml.

## Done gate

```
make graph
cd src-tauri && cargo test --no-default-features --test contract
make demo
```

All local-stt CTs green. `make demo` prints `network_calls: 0`.

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Download models
- Enable cloud STT
- Log transcript text
- Commit or push
- Change the public function names in stt.rs

When tests pass, check the boxes in `.kiro/specs/local-stt/tasks.md` and stop.
