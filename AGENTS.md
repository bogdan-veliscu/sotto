# Sotto agent instructions

Sotto is a local-first macOS meeting recorder. Work in this repository only.

## Scope and verification

- Read [KIRO_BRIEF.md](KIRO_BRIEF.md), the relevant [domain invariants](harness/graph/domain.graph.json), and the assigned wave in the [task DAG](harness/graph/task-dag.yaml).
- Work one wave at a time from its requirements, design, and tasks under `.kiro/specs/<name>/`. Follow requirements-first specs. Do not run `/spec new` or rewrite locked EARS requirements. Open an issue before adding a new invariant.
- Keep `fixtures/` and `harness/graph/fixture-lock.json` immutable. Fixtures are the only seed data. Keep model weights out of git.
- A wave is complete when `make graph` and `make contract` pass for its named invariants and its required tasks are done. Report checks run and any unfinished tasks. Python contract tests in `tests/contract/` use `@pytest.mark.contract`. Rust contracts live in `src-tauri/tests/contract.rs`.
- Run `make ci` before pushing. Linux core, macOS desktop build, hardware/TCC, and real local-model verification are separate evidence classes. Automated tests must not prompt for capture permissions or require live capture.
- Use conventional commits with subjects of at most 72 characters. Do not push directly to `main` or force-push.

## Privacy and product boundaries

- Start recording only after `consent_state` is `acknowledged`. Keep finalized audio AES-GCM encrypted at rest with no WAV header. Never log transcript content or encryption keys.
- Audio stays on-device unless the user explicitly enables a cloud engine. Cloud STT defaults to off. Fallback must never select a cloud/API engine unless `cloud_mode=on`.
- Keep telemetry off by default. Do not require cloud APIs. Do not use LLM API keys. Keep notes extractive and local. Do not add a chatbot, meeting bot, teams, any calendar features, or a browser extension. See [product decisions](docs/DECISIONS.md) for scope.
- Keep `make demo` on offline fixture replay after crate fetch, with `network_calls: 0`. Demo and contract tests must never download models. Kiro is a development tool only.

## Stack and desk UI

- Use Tauri 2, Rust, SvelteKit, and SQLite with FTS5. Use Node 22 and npm for the UI. Python 3.12 is for the harness and contract tests only.
- Design the desk as a quiet instrument panel for private meetings. Use Fraunces, IBM Plex Sans, and IBM Plex Mono with warm ink and paper. Avoid Inter, Roboto, Arial, and purple gradients. Keep the REC indicator and "on this Mac" chip visible. Use LED red only while recording.
