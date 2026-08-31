# Contributing to Sotto

Thank you. Sotto is built in public. The bar is: **a stranger can clone, run `make demo`, and trust the privacy invariants.**

## Before you write code

1. Read `README.md`, `AGENTS.md`, `KIRO_BRIEF.md`.
2. Read the current wave in `harness/graph/task-dag.yaml`.
3. Read the spec under `.kiro/specs/<name>/requirements.md`. Do not rewrite locked EARS.
4. Run `make graph` then `make contract`.

## Way of working

This repo uses the same method that worked on the Kiro labs:

- Requirements-First Feature Specs (EARS).
- Domain graph + wave DAG.
- Contract tests are the done gate.
- Golden fixtures are content-addressed.
- One wave per PR unless the PR is docs-only.

Do not `/spec new`. If the graph needs a new invariant, open an issue first.

## Dev setup

- macOS 14.4+ for the desktop app.
- Rust stable, Node 22+, Python 3.12+.
- `npm install`
- `make demo` must pass before you push.

Windows/Linux can run `make graph`, the Python tests, and `cargo test` for the core. Live capture is macOS-only.

## PR checklist

- [ ] Conventional commit subject ≤72 chars (`feat:`, `fix:`, `docs:`, `test:`, `chore:`).
- [ ] `make graph` green.
- [ ] `make contract` green.
- [ ] No edits to `fixtures/` unless the issue says so. If you must, set `SOTTO_ALLOW_FIXTURE_MUTATION=1` and run `make lock`.
- [ ] UI stays quiet: no Inter/Roboto/purple-gradient slop. Fraunces + IBM Plex, warm ink.
- [ ] Do not add a cloud engine that can be selected by default.
- [ ] Do not log transcript text.

## Good first contributions

- Wire Core Audio taps behind `recorder.start` (see `docs/AUDIO_CAPTURE_SPEC.md`).
- Implement Parakeet / Whisper behind `TranscriptionEngine` without changing UI types.
- SQLCipher for the metadata DB.
- macOS Keychain for `master.key`.
- Export to a file via a save dialog.
- Tests for pause/resume elapsed time.

## Code of conduct

`CODE_OF_CONDUCT.md`. Harassment is out.

## Security

`SECURITY.md`. Do not file a public issue for a decryption or upload bug — email is in that file.
