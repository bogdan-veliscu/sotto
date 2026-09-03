# Sotto

Local-first meeting recorder for macOS. **No bot joins the call.** Audio stays on this Mac by default, then becomes a searchable transcript.

Sotto is built in public. It is a tool I need for sensitive conversations, and the repo is public so others can run it, inspect the privacy invariants, and contribute.

Trusted-circle soft launch: `docs/SOFT_LAUNCH.md`. `make demo` is the golden **fixture-replay** path. Live capture and a real **local model** are separate evidence.

## Why this name

*Sotto voce*: under the breath. Notes without a stranger in the room.

## What works today

| Capability | Status | Evidence |
|---|---|---|
| Local SQLite store + FTS5 search | **Done** | linux core (`make demo`) |
| Consent gate before record | **Done** | linux core + desk |
| AES-GCM audio at rest (not a plaintext WAV) | **Done** | linux core |
| Engine catalog; never silent-cloud fallback | **Done** | linux core |
| Fixture-replay transcription | **Done** — `make demo` only | linux core |
| Quiet desktop UI (Tauri 2 + SvelteKit) | **Done** | macos desktop |
| Live mic / system / mixed capture | **Done** on macOS | hardware/tcc (human) |
| Apple on-device Speech, Whisper file, Parakeet TDT | **Done** when a runnable local model is present | real local-model |
| Crash recover / discard leftover chunks | **Done** | linux core + desk |
| SQLCipher for the metadata DB | Later | — |
| Cloud STT | Off. Never on unless you set `cloud_mode=on` | — |

Linux GitHub Actions is **linux core** only (`cargo --no-default-features`). It does not prove taps, Screen Recording, or a real decoder.

## First user on this Mac

1. `make build` or `make dev`. Unsigned `.app`: Gatekeeper → right-click Open.
2. Grant Microphone. Grant **Screen Recording** only if you pick system or mixed.
3. Import a Whisper ggml file or a Parakeet TDT directory, or use Apple on-device Speech. The desk shows `ready` only when that engine can decode. No auto-download.
4. Acknowledge consent, record, Stop. Live Stop does not use `fixture-replay`.
5. Search the transcript and export markdown.
6. If the app dies mid-record, Recover (encrypt) or Discard. No silent wipe of consented chunks.

`make demo` never downloads a model and always stays on fixture-replay with `network_calls: 0`.

## Quick start

macOS 14.4+ recommended. You need **Rust**, **Node 22+**, and **Python 3.12+**.

```bash
# one-time
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then
git clone https://github.com/bogdan-veliscu/sotto.git
cd sotto
npm install
make demo          # fixture-replay privacy invariants, JSON on stdout, no API keys
make dev           # desktop app
```

`make demo` must succeed with the network unplugged after crates are cached.

### Make targets

| Target | What it does |
|---|---|
| `make graph` | Validate domain graph + SHA-256 fixture lock |
| `make contract` | Graph + core Rust tests + Python catalog tests |
| `make demo` | Offline fixture → encrypt → transcribe → search → delete-all |
| `make ci` | Same gates as GitHub Actions (linux core, no GTK/WebKit) |
| `make cert-desktop` | Record macos desktop evidence (compile, UI check, unsigned `.app`) |
| `make dev` | `npm run tauri dev` |
| `make build` | Unsigned macOS `.app` |
| `make test` | verify + contract |

Roadmap and remaining human hardware pass: `docs/PR_PLAN.md`, `docs/SOFT_LAUNCH.md`.

## How we work (Kiro)

Same method as the Ready / Spec / Ship labs: **Requirements-First specs, a domain DAG, contract tests as the done gate.**

1. Invariants live in `harness/graph/domain.graph.json`.
2. Waves live in `harness/graph/task-dag.yaml`.
3. Specs live in `.kiro/specs/<name>/`.
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
harness/evidence/       Content-free macos desktop / hardware/tcc manifests
tests/contract/         Python invariant tests
docs/                   Product spec pack
.kiro/                  Steering, specs, hooks, agent `scribe`
```

## Privacy

- No telemetry unless you turn it on (default **off**).
- No network in the default demo path (`network_calls: 0`).
- Audio is AES-256-GCM. Production desktop uses macOS Keychain; linux core / `make demo` uses an isolated file keystore.
- You are responsible for telling other people in the room when the law requires consent to record.

See `SECURITY.md` and `docs/PRD.md`.

## Contributing

`CONTRIBUTING.md`. Small PRs against one DAG wave. Contract tests must stay green.

## License

MIT. See `LICENSE`.
