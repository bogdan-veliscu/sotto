# Wave 13–14 — notes-export

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/notes-export/requirements.md
4. .kiro/specs/notes-export/design.md
5. .kiro/specs/notes-export/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/notes.rs (stub, returns NOT_IMPLEMENTED)
8. src-tauri/src/store.rs (`export_markdown_file` and `privacy_settings` stubs; `persist_transcript`)
9. src-tauri/tests/contract.rs (RED contract tests)

## Do this wave

Implement `src-tauri/src/notes.rs` and wire Store so these tests pass:

- `ct_summary_from_transcript`
- `ct_export_file`
- `ct_settings_privacy`

`extract_notes` is extractive and local. Keep distinctive claims (`privileged`) and follow-ups (`follow up` / `engagement`). No LLM. No HTTP.

When `TranscriptResult.summary_text` is empty, fill notes via `extract_notes` before persist. Fixture-replay may keep golden notes.

`export_markdown_file` writes `export_markdown()` to a local path. Reject `http://` / `https://`. Create parent dirs.

`privacy_settings` reads existing keys. Fresh store: telemetry off, cloud_mode off. Do not change `ensure_defaults`.

`make demo` stays `fixture-replay`, `network_calls: 0`.

## Done gate

```
make graph
cd src-tauri && cargo test --no-default-features --test contract
make demo
```

All notes-export CTs green. `make demo` prints `network_calls: 0`.

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Call a cloud LLM
- Enable telemetry or cloud in defaults
- Log transcript text
- Commit or push
- Change the public function names in notes.rs

When tests pass, check the boxes in `.kiro/specs/notes-export/tasks.md` and stop.
