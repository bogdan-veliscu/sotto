# macOS founder certification — Design

Turn the current split between portable contracts and the real Mac app into an explicit, repeatable gate.

## Certification runner

Add a macOS-only runner and evidence manifest with two layers:

1. Automated: `make graph`, portable contracts, `cargo check --features desktop --bins`, `npm run check`, and an unsigned `.app` build.
2. Human-gated hardware/TCC: verify no silent start, consent card, microphone capture, system capture with pre-granted Screen Recording, mixed no-fallback behavior, encrypted output, and one real Whisper or Parakeet transcript.

The runner must preflight permissions and report `not-run`, `needs-permission`, `pass`, or `fail`. It never requests Screen Recording from a contract test and never downloads weights. Evidence records commands, build SHA, engine id, and outcome, but no transcript text or audio.

## Contract home and skip policy

`tests/contract/test_macos_cert.py` owns both certification CTs. Linux/GHA always skips them because Ubuntu core CI is not macOS evidence. On macOS, absence of `harness/evidence/macos-founder-certification.json` is `not-run` and skips rather than fails. If the manifest exists, automated tests validate schema `sotto/macos-founder-certification/v1`, the recorded commit, required outcome names, and the absence of transcript/audio content. They do not perform capture or request TCC access.

`CT-macos-desktop-gate` requires `desktop_build`, `ui_check`, and `app_bundle` outcomes to be `pass`. `CT-macos-hardware-e2e` requires `consent`, `mic`, `system`, `mixed`, `pause_resume`, `recovery`, `encrypted_audio`, and `real_local_stt` outcomes to be `pass`. Hardware execution remains an explicit human-run step that writes only the content-free manifest.

Signing/notarization is not part of founder daily-use certification. It remains a later distribution gate.

## Forbidden

- Treating Linux CI as macOS runtime evidence
- Prompting for TCC access from automated tests
- Storing transcript/audio content in evidence or logs
- Treating judge-keystore success as production Keychain evidence
