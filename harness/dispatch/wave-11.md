# Wave 11–12 — model-install

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/model-install/requirements.md
4. .kiro/specs/model-install/design.md
5. .kiro/specs/model-install/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/install.rs (stub, returns NOT_IMPLEMENTED)
8. src-tauri/src/stt.rs
9. src-tauri/src/store.rs (`list_engines` still returns the frozen catalog)
10. src-tauri/tests/contract.rs (RED contract tests)

## Do this wave

Implement `src-tauri/src/install.rs` (and wire overlay into `Store::list_engines` / `transcribe_local` for Parakeet) so these tests pass:

- `ct_checksum`
- `ct_parakeet_local`

Wrong SHA-256 → `CHECKSUM_MISMATCH` and no file left on disk. Matching SHA-256 writes `parakeet-tdt-0.6b-v3.bin`. Overlay marks it `ready`. Delete removes it. Missing Parakeet → `ENGINE_NOT_INSTALLED`. Never HTTP. Never silent cloud. Never treat fixture-replay as Parakeet.

`demo_pipeline` must not call `install_bytes`. `make demo` stays `fixture-replay`, `network_calls: 0`.

Linux CI runs `cargo test --no-default-features`. Do not download 1.2 GB models. Do not add a live download URL that CI can hit.

## Done gate

```
make graph
cd src-tauri && cargo test --no-default-features --test contract
make demo
```

All model-install CTs green. `make demo` prints `network_calls: 0`.

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Download models
- Enable cloud STT
- Log transcript text
- Commit or push
- Change the public function names in install.rs

When tests pass, check the boxes in `.kiro/specs/model-install/tasks.md` and stop.
