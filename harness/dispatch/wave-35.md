# Wave 35–36 — stt-worker

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/stt-worker/requirements.md
4. .kiro/specs/stt-worker/design.md
5. .kiro/specs/stt-worker/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/store.rs
8. src-tauri/src/commands.rs
9. src-tauri/tests/contract.rs

Git branch is `feat/stt-worker`. Conventional commits, subject ≤72 chars. You MAY commit on this branch. Do not push. Do not touch `main`.

## Do this wave

1. Split transcribe into prepare (short Store lock) → `transcribe_job` (no Store) → commit (short Store lock). `CT-stt-worker-releases-lock` must pass: after prepare, `Mutex<Store>::try_lock` succeeds and session list / settings still work.
2. `transcribe_job` on CONSULT-001 / fixture-replay must match `Store::transcribe`. Never `CLOUD_DISABLED`. `demo_pipeline` stays fixture-replay. Desktop `transcribe_run` must `spawn_blocking` so inference is off the Tauri command thread.

Named tests to add in `src-tauri/tests/contract.rs`:

- `ct_stt_worker_releases_lock`
- `ct_stt_worker_same_result`

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
- Hold the Store mutex during `transcribe_local`
- Start a meeting bot
- Implement source-picker (later spec)
