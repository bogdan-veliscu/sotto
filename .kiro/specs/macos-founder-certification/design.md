# macOS founder certification — Design

Turn the current split between portable contracts and the real Mac app into an explicit, repeatable gate.

## Certification runner

Add a macOS-only runner and evidence manifest with two layers:

1. Automated: `make graph`, portable contracts, `cargo check --features desktop --bins`, `npm run check`, and an unsigned `.app` build.
2. Human-gated hardware/TCC: verify no silent start, consent card, microphone capture, system capture with pre-granted Screen Recording, mixed no-fallback behavior, encrypted output, and one real Whisper or Parakeet transcript.

The runner must preflight permissions and report `not-run`, `needs-permission`, `pass`, or `fail`. It never requests Screen Recording from a contract test and never downloads weights. Evidence records commands, build SHA, engine id, and outcome, but no transcript text or audio.

Signing/notarization is not part of founder daily-use certification. It remains a later distribution gate.

## Forbidden

- Treating Linux CI as macOS runtime evidence
- Prompting for TCC access from automated tests
- Storing transcript/audio content in evidence or logs
- Treating judge-keystore success as production Keychain evidence
