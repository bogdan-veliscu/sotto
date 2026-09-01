# Sotto completeness review

**Review date:** 2026-09-01

**Reviewed commit:** `69dd5e8` (`main`, PR #16 merged)

**Criteria:** product completeness and documentation honesty

**Depth:** deep

**Method:** `directory-audit`

**Scope:** PRD, PRODUCT_BRIEF, all Kiro specs, domain graph, DAG/progress, desk, Rust core/desktop commands, contract tests, Make targets, CI, and current documentation.
**Verdict:** The post-J code contains the intended local capture, encryption, search/export, and local STT building blocks, but Sotto is not yet a complete or certified founder daily recorder. Model onboarding is not runnable end to end, interrupted recordings are not surfaced for recovery, local macOS judge commands fail at Keychain creation in this environment, and macOS hardware/model paths have no retained end-to-end evidence.

## Gate results observed in this review

| Gate | Result | What it proves |
|---|---|---|
| `make graph` | **PASS** | Current domain graph, DAG, and immutable fixture lock are consistent. |
| `cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bins` | **PASS** | The macOS desktop-feature Rust targets compile on this Mac. It does not prove taps, permissions, model files, or transcription. |
| `npm run check` | **PASS** | Svelte/TypeScript diagnostics: 0 errors, 0 warnings. |
| `make ci` | **FAIL** | On this Mac, 14 core unit tests passed, then `store::tests::{defaults_are_private,consent_blocks_record}` failed with `KEYCHAIN`: “One or more parameters passed to a function were not valid.” Demo and later gates did not run. |
| `make demo` | **FAIL** | The judge path failed opening its Store with the same Keychain error. No demo report was produced. |
| `make dev` | **NOT RUN** | Interactive desktop launch would require UI/TCC/model handling; compile success is recorded separately above. |

The GitHub workflow runs Ubuntu with `--no-default-features` (`.github/workflows/ci.yml:8-30`). It is the configured merge gate, but it does not compile `desktop`, `whisper`, or `parakeet`, and it cannot exercise macOS ScreenCaptureKit, CPAL, TCC, Keychain, tray, hotkey, HUD, or real model weights (`src-tauri/Cargo.toml:23-32`).

## What a first user can do in the implementation

The following capabilities exist in code and portable contracts, subject to the evidence limits above:

- Create a session, acknowledge consent, and block recording before acknowledgement (`src-tauri/src/store.rs:222-270`; `CT-consent-before-record`).
- Capture microphone, system, or mixed sources on macOS through CPAL/ScreenCaptureKit code paths; mixed fails instead of degrading to mic-only (`src-tauri/src/capture.rs:539-669`; `CT-system-not-fixture`, `CT-mixed-not-mic-only`). No hardware capture was executed in this review.
- Pause/resume chunk capture, encrypt finalized WAV bytes into `.sotto`, and remove a completed live directory (`src-tauri/src/capture.rs:305-415`; `src-tauri/src/store.rs:289-360`; `CT-pause-resume`, `CT-audio-encrypted`).
- Run Whisper or Parakeet inference when the matching desktop feature and valid user-supplied local model are present (`src-tauri/src/stt.rs:164-321`; `src-tauri/src/stt_parakeet.rs:1-50`). No real weights or real transcript were exercised.
- Persist timestamped segments, extract local non-LLM notes, search by text/title/date/tag, export markdown, apply retention, delete a session, and delete all (`src-tauri/src/store.rs:395-815`; named notes/search/delete contracts).
- Use the desk source picker, consent card, Record/Pause/Stop, local search, settings, model row, export, delete, hotkey, optional meeting-app prompt, HUD, and Open-desk tray entry (`src/routes/+page.svelte:64-725`; `src-tauri/src/lib.rs:115-195`). UI compile/check passed; interactive behavior was not certified.

What the commands mean today:

- `make demo` is intentionally only the golden fixture pipeline, never a real meeting or model download (`Makefile:29-30`; `src-tauri/src/lib.rs:49-79`). In this review it did not complete because of Keychain.
- `make ci` is a portable/core-oriented command and fails locally on this Mac before its demo step. The GitHub version is Linux-only.
- `make dev` launches the real desktop path, but a fresh user still needs permissions and valid local model assets that the first-run flow cannot fully install or validate.

## Findings

### P0 — blocks founder daily use or a trustworthy done gate

1. **The first live recording defaults to an engine that cannot transcribe live audio.** Fresh stores set `default_model=fixture-replay` (`src-tauri/src/store.rs:160-168`), Stop calls transcription without an explicit model (`src/routes/+page.svelte:177-187`), and resolution therefore selects that default (`src-tauri/src/store.rs:375-390`). Fixture replay correctly rejects non-golden audio with `FIXTURE_AUDIO_MISMATCH` (`src-tauri/src/stt.rs:111-121`; `CT-fixture-audio-mismatch`). The safety behavior is correct, but the normal first-run path ends in a recoverable error instead of a transcript. **Ship L (`model-onboarding`).**

2. **Parakeet install/readiness is internally contradictory and the desk cannot install the runnable layout.** `is_installed` accepts either a valid TDT directory or the checksum `.bin`, and overlay turns either into `ready` (`src-tauri/src/install.rs:156-184`). Actual decode requires the TDT directory (`src-tauri/src/stt.rs:202-238`). The desk picker accepts one file and calls `model_install_file` (`src/routes/+page.svelte:297-313`; `src-tauri/src/commands.rs:365-380`), so it cannot place the three-file TDT folder it tells the user to provide. A dummy contract blob can therefore display `ready` and then fail decode. **Ship L; a checksum blob remains test evidence, not a runnable model.**

3. **The documented local judge path is red on this Mac.** `make ci` and `make demo` both failed during Store creation because the macOS target selects Keychain even under `--no-default-features` (`src-tauri/src/keys.rs:23-105`). This prevents a founder from using the documented judge commands reliably, blocks the required done gate for later waves, and prevents this review from confirming the demo report. **Ship K (`judge-reliability`) with an isolated test-only keystore; do not weaken production Keychain.**

### P1 — required for a dependable founder recorder

1. **Crash-safe capture exists only as a library primitive, not a product flow.** `ChunkedRecorder::Drop` flushes and `recover` rebuilds a WAV (`src-tauri/src/capture.rs:394-415`), but startup only scrubs the encrypted audio directory (`src-tauri/src/lib.rs:100-105`; `src-tauri/src/store.rs:852-875`). No command or desk flow discovers `{data}/live/<session>` after a crash. The PRD promises crash-safe partial save (`docs/PRD.md:43-45`), while `CT-crash-partial` proves only a direct function call (`src-tauri/tests/contract.rs:59-73`). **Ship M (`crash-recovery`).**

2. **macOS runtime claims are not covered by the merge gate.** GHA is Ubuntu/no-default-features, while `desktop` enables Tauri, Whisper, and Parakeet (`.github/workflows/ci.yml:8-30`; `src-tauri/Cargo.toml:23-32`). Existing contracts intentionally avoid microphones and permission prompts. This is correct test hygiene, but it leaves real mic/system/mixed capture, TCC handling, Keychain, login item, hotkey, HUD, recovery, and real-model transcription unproven as a combined daily path. **Ship N with separate automated and human-gated evidence.**

3. **Public and in-app copy materially understates or misstates the post-J implementation.** README says live capture is next and local engines are not wired (`README.md:7-25`), PRODUCT_BRIEF calls live capture next (`docs/PRODUCT_BRIEF.md:39-45`), LAUNCH_PLAN says fixture now/live next (`docs/LAUNCH_PLAN.md:7-13`), API_CONTRACT still labels `transcribe_run` fixture-only (`docs/API_CONTRACT.md:5-26`), and the desk says to record a fixture and describes Wave 1 (`src/routes/+page.svelte:446-499`). PR_PLAN also previously left J and its counts open even though `69dd5e8` merged #16 and progress says wave 38. **Ship O (`docs-readme-closeout`) after Mac certification.**

### P2 — honesty or polish gaps that should not pre-empt K–O

1. **Login-item backend naming is false.** The app initializes `tauri-plugin-autostart` with `MacosLauncher::LaunchAgent` (`src-tauri/src/lib.rs:93-99`) while `login_item_backend()` reports `smappservice` (`src-tauri/src/presence.rs:35-41`) and `presence_login_get` infers `applied` from that label (`src-tauri/src/commands.rs:420-434`). Correct the label/contract during N; do not claim SMAppService.

2. **Presence design promises tray Record/Stop, but the implementation intentionally exposes Open desk only.** The safe implementation is `Open desk` (`src-tauri/src/lib.rs:156-195`); direct tray recording would need to surface the consent card. Keep Open-only unless a future design routes tray intent to the visible consent flow, and correct the old design claim in O.

3. **Cloud and telemetry toggles are visible without implemented consumers.** Defaults are safely off and no catalog engine is cloud, so this is not a privacy defect. The UI can mislead users into thinking an operational cloud path or telemetry sink exists (`src/routes/+page.svelte:615-639`). O should label these reserved settings or hide them until there is separately approved scope.

4. **HUD elapsed state is not authoritative.** Pause/resume commands emit HUD state with elapsed `0` (`src-tauri/src/commands.rs:159-189`), while the desk and HUD each advance local timers. This can reset or drift after pause/resume. It is daily-use polish, but not ahead of transcribability, recovery, or certification.

### Info — explicit boundaries

- Notes are extractive local rules, not an LLM (`.kiro/specs/notes-export/design.md:15-25`). This is honest and in scope.
- Real Whisper and Parakeet weights are absent by design and must remain outside git, fixtures, demo, and contracts.
- Signing/notarization is not required for the founder to run a locally built app. It remains a distribution gate (`docs/DEVOPS.md:15-17`), after K–O unless external users need a downloadable binary.
- SQLCipher, calendar, teams, Windows/Linux, streaming STT, diarization, cloud-default STT, and a meeting bot remain out of scope.

## PRD and brief completeness matrix

| Requirement | Implementation status | Evidence boundary | Remaining work |
|---|---|---|---|
| Manual start/stop + consent | Implemented | Portable consent contracts; desk compile only | Hardware/TCC certification in N |
| Mic/system/mixed capture | Implemented in macOS code | No retained live hardware result in this review | N |
| Pause/resume | Implemented | `CT-pause-resume` uses injected PCM | N validates live behavior |
| Encrypted audio at rest | Implemented | `CT-audio-encrypted`, `CT-live-stop-not-fixture` | Recovery must reuse it in M |
| Crash-safe partial save | Primitive only | `CT-crash-partial` calls recovery directly | M productizes discovery/recovery |
| Real local transcription | Whisper + Parakeet decoder code exists | No real weights/transcript evidence; setup broken for Parakeet directory | L, then N |
| Two selectable local engines | Catalog/runtime code exists | `ready` can mean dummy blob; fresh default is fixture | L |
| Timestamped transcript | Implemented | Fixture contracts and decoder code | Real-model check in N |
| Local summary/action items/key points | Implemented extractively | Portable contracts | Document non-LLM behavior in O |
| Search title/text/date/tags | Implemented | Portable contracts | None before daily-use gate |
| Markdown export | Implemented | `CT-export-file`; UI compile | N desk smoke |
| Retention/delete-all | Implemented | Portable contracts | N desk smoke |
| Low-friction first run | Incomplete | Privacy modal only; no runnable model setup | L |
| Fast/stable private desktop | Uncertified | Desktop compile passes; local judge fails Keychain | K, then N |

## Ordered remaining scope

Soft-launch bar: `docs/SOFT_LAUNCH.md`. Same order. L is the product P0; K unblocks the judge.

1. **K, waves 39–40 — judge-reliability:** make demo/contract/CI deterministic on macOS with an isolated judge keystore while production desktop keeps Keychain.
2. **L, waves 41–42 — model-onboarding:** make runnable state truthful; import Whisper file or Parakeet TDT directory; require explicit non-fixture live engine while keeping recorded audio retryable.
3. **M, waves 43–44 — crash-recovery:** discover exact-session chunks and encrypt before cleanup; keep Discard explicit.
4. **N, waves 45–46 — macos-founder-certification:** retain separate desktop build and content-free hardware/TCC/recovery/real-model evidence.
5. **O, waves 47–48 — docs-readme-closeout:** reconcile README/product/API/architecture/UX/launch docs and publish exact evidence scope.

No product feature was implemented in this review. New nodes are intentionally absent from `harness/graph/progress.json`; wave 38 remains the latest completed wave.

## Audit counts

P0=3 P1=3 P2=4 INFO=4

## Inventory snapshot

- Tracked files at reviewed commit: 197.
- Rust/Svelte/TypeScript/Python implementation and test LOC: 6,358.
- Test files under `src-tauri/tests/` and `tests/`: 3.
- Last reviewed commit: `69dd5e8` (2026-09-01), merge PR #16 source-picker.
