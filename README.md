# Sotto

Local-first meeting recorder for macOS. **No bot joins the call.** Audio stays on this Mac by default, then becomes a searchable transcript.

Sotto is built in public. It is a tool I need for sensitive conversations, and the repo is public so others can run it, inspect the privacy invariants, and contribute.

> Wave 1 records a **golden fixture**, encrypts it, transcribes it offline, and searches it. Core Audio taps (live system audio) and Parakeet / Whisper weights are specified and catalogued, not downloaded.

## Why this name

*Sotto voce*: under the breath. Notes without a stranger in the room.

## What works today

| Capability | Status |
|---|---|
| Local SQLite store + FTS5 search | **Done** |
| Consent gate before record | **Done** |
| AES-GCM audio at rest (not a plaintext WAV) | **Done** |
| Engine catalog; never silent-cloud fallback | **Done** |
| Fixture-replay transcription (offline demo) | **Done** |
| Quiet desktop UI (Tauri 2 + SvelteKit) | **Done** |
| Live system-audio / mic capture | Next wave |
| Parakeet TDT 0.6B v3 / Whisper Large-v3 Turbo | Catalogued, not wired |
| SQLCipher for the metadata DB | Later |
| Cloud STT | Off. Never on unless you set `cloud_mode=on` |

## Quick start

macOS 14.4+ recommended. You need **Rust**, **Node 22+**, and **Python 3.12+**.

```bash
# one-time
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then
git clone https://github.com/bogdan-veliscu/sotto.git
cd sotto
npm install
make demo          # privacy invariants, JSON on stdout, no API keys
make dev           # desktop app
```

`make demo` must succeed with the network unplugged after crates are cached. It never downloads a model.

### Make targets

| Target | What it does |
|---|---|
| `make graph` | Validate domain graph + SHA-256 fixture lock |
| `make contract` | Graph + Rust pipeline + Python catalog tests |
| `make demo` | Offline fixture → encrypt → transcribe → search → delete-all |
| `make dev` | `npm run tauri dev` |
| `make test` | verify + contract |

## How we work (Kiro)

Same method as the Ready / Spec / Ship labs: **Requirements-First specs, a domain DAG, contract tests as the done gate.**

1. Invariants live in `harness/graph/domain.graph.json`.
2. Waves live in `harness/graph/task-dag.yaml`.
3. Specs live in `.kiro/specs/{session-store,capture-consent,search-notes}/`.
4. Golden files in `fixtures/` are content-addressed. A PreToolUse hook blocks edits unless `SOTTO_ALLOW_FIXTURE_MUTATION=1`.
5. Kiro is the engineering environment, not a runtime. You do not install Kiro to run Sotto.

Read `AGENTS.md` and `KIRO_BRIEF.md` before writing code. One wave per change. Named contract tests are done.

```
INV-NO-CLOUD-DEFAULT        telemetry off, cloud_mode off
INV-NO-SILENT-CLOUD         fallback never picks cloud
INV-CONSENT-BEFORE-RECORD   no record without disclosure ack
INV-AUDIO-ENCRYPTED         on-disk audio is not RIFF/WAVE
INV-FTS-SEARCH              distinctive term is findable
INV-DELETE-ALL              search goes empty after wipe
INV-FIXTURE-LOCK            SHA-256 of fixtures/
```

## Repository map

```
src/                    SvelteKit desk UI
src-tauri/              Rust store, crypto, engines, IPC, sotto-demo
fixtures/               Golden models catalog + CONSULT-001
harness/graph/          Domain graph, DAG, fixture lock
tests/contract/         Python invariant tests
docs/                   Product spec pack
.kiro/                  Steering, specs, hooks, agent `scribe`
```

## Privacy

- No telemetry unless you turn it on (default **off**).
- No network in the default demo path (`network_calls: 0`).
- Audio is AES-256-GCM. The master key lives next to the database in the app data directory for wave 1 (Keychain comes later). Keep them together.
- You are responsible for telling other people in the room when the law requires consent to record.

See `SECURITY.md` and `docs/PRD.md`.

## Contributing

`CONTRIBUTING.md`. Small PRs against one DAG wave. Contract tests must stay green.

## License

MIT. See `LICENSE`.
