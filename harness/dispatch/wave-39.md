# Wave 39–40 — judge-reliability

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. docs/COMPLETENESS_REVIEW.md
4. .kiro/specs/judge-reliability/requirements.md
5. .kiro/specs/judge-reliability/design.md
6. .kiro/specs/judge-reliability/tasks.md
7. harness/graph/task-dag.yaml
8. Makefile
9. src-tauri/src/keys.rs
10. src-tauri/src/store.rs
11. src-tauri/src/lib.rs
12. src-tauri/tests/contract.rs
13. tests/contract/test_catalog.py

Git branch is `fix/judge-reliability`. Conventional commits, subject ≤72 chars. You MAY commit on this branch. Do not push. Do not touch `main`.

## Do this wave

1. Add a narrowly scoped test/judge keystore so `make demo`, `make contract`, and `make ci` do not call macOS Keychain or open Keychain UI. It must use the command's temporary data directory, persist a 32-byte key with mode 0600, and reuse it on reopen. Production desktop startup must still use Keychain and must not silently fall back.
2. Keep judge commands independent of microphone, Screen Recording, desktop UI, and real model weights. `demo_pipeline` remains CONSULT-001 fixture-replay with `network_calls: 0`.

Named tests to add in `src-tauri/tests/contract.rs`:

- `ct_keychain_test_deterministic`
- `ct_judge_completes`

Tests do not invoke Keychain UI, a decoder, microphone, or Screen Recording. Do not print key bytes.

## Done gate

```
make graph
make contract
cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bins
npm run check
```

All four commands are required. The purpose of this wave is to make the local judge green before later product waves.

## Do not

- Edit `fixtures/` or `harness/graph/fixture-lock.json`
- Replace production Keychain with the judge backend
- Fall back after a production Keychain error
- Download or commit model weights
- Change live engine selection or product features
- Call `start_live(Mic)` or request Screen Recording from tests
- Skip consent or start a meeting bot
