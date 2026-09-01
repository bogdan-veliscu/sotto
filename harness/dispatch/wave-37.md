# Wave 37–38 — source-picker

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/source-picker/requirements.md
4. .kiro/specs/source-picker/design.md
5. .kiro/specs/source-picker/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/capture.rs
8. src-tauri/src/commands.rs
9. src/routes/+page.svelte
10. src-tauri/tests/contract.rs

Git branch is `feat/source-picker`. Conventional commits, subject ≤72 chars. You MAY commit on this branch. Do not push. Do not touch `main`.

## Do this wave

1. `CaptureSource::try_parse` accepts only `mic`, `system`, `mixed`. Anything else is `SOURCE_UNKNOWN` recoverable. `recorder_start` must validate. Never silent mixed. Never a meeting bot.
2. `source_permission_hint` always says consent is still required. Mixed copy says it will not fall back to mic-only. Desk Record uses the picker (not hardcoded mic). Consent card still required before `recorder_begin`.

Named tests to add in `src-tauri/tests/contract.rs`:

- `ct_source_unknown`
- `ct_source_permission_copy`

## Done gate

```
make graph
cargo test --manifest-path src-tauri/Cargo.toml --no-default-features --lib --test contract
cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bins
npm run check
```

Do not wait forever on `make demo` if Keychain blocks. Demo must remain fixture-replay / `network_calls: 0` when it runs.

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Call `start_live(Mic)` from tests
- Skip consent
- Start a meeting bot
