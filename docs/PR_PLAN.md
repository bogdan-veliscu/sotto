# v1 PR plan

Initial commit on `main` is **PR 0** (`b956583`): desk UI, privacy invariants, fixture pipeline. That is not the complete product.

Complete **v1** means the PRD success criteria: record on this Mac, transcribe locally with a real model, search, export, encrypted audio, no cloud by default. Windows, Linux, SQLCipher, notarization, streaming STT, diarization, and self-host sync are **after v1**.

The historical v1 sequence below was six PRs after PR 0. All are shipped. Contract tests remain the merge gate; do not skip the DAG.

| PR | Branch | Spec | What ships | Merge gate |
|----|--------|------|------------|------------|
| **0** | `main` | `session-store`, `capture-consent`, `search-notes` | Public desk. Fixture record → encrypt → search. | **Shipped** |
| **1** | `feat/live-capture` | `live-capture` | Real capture pipeline: chunked WAV, pause/resume, crash recover, mic via CPAL. System-audio tap may return `CAPTURE_UNSUPPORTED`. **Shipped** (`#1`). | `CT-capture-wav`, `CT-pause-resume`, `CT-crash-partial`, `CT-demo-still-offline` |
| **2** | `feat/local-whisper` | `local-stt` | Whisper Large-v3 Turbo **local weights**, batch transcribe. Never download inside `make demo`. **Shipped** (`#2`). | `CT-whisper-local-only`, `CT-demo-no-download` |
| **3** | `feat/parakeet-install` | `model-install` | Parakeet TDT 0.6B v3 as selectable default. User-initiated checksummed install, overlay, delete. **Shipped** (`#3`). | `CT-checksum`, `CT-parakeet-local`, `CT-no-silent-cloud` (still) |
| **4** | `feat/notes-export` | `notes-export` | Summary / action items / key points from the transcript (extractive local, no cloud LLM). Settings pane. Save-as markdown via dialog. **Shipped** (`#4`). | `CT-summary-from-transcript`, `CT-export-file`, `CT-settings-privacy` |
| **5** | `feat/search-filters` | `search-filters` | Title, date range, tags. **Shipped** (`#5`). | `CT-filter-date`, `CT-tag-roundtrip` |
| **6** | `feat/harden-keychain` | `harden` | Master key in macOS Keychain (file 0600 on Linux CI). Retention deletes. Crash-safe temps gone. **Shipped** (`#6`). | `CT-keychain`, `CT-retention` |

## Historical v1 counts

- **PRs to complete v1: 0.** PRs 0–6 are on `main`.
- **Features: 6** (one per PR).
- **New Kiro specs: 6** (`live-capture`, `local-stt`, `model-install`, `notes-export`, `search-filters`, `harden`). `live-capture` is two DAG waves in one PR.

## Order (do not parallelize 1→2→3)

Capture before STT. Whisper before Parakeet so there is always one real local engine. Install UI after an engine can run. Notes after there is a real transcript. Filters after notes. Keychain last so we do not churn crypto twice.

## After v1 (do not skip A; do not parallelize)

| Wave | Branch | Spec | What ships | Merge gate |
|------|--------|------|------------|------------|
| **A** | `feat/desk-closeout` | `desk-closeout` | Promised v1 APIs actually usable in the desk. No live Core Audio. | `CT-filter-title`, `make ci` |
| **B** | `feat/live-record` | `live-record` | Record/Pause/Stop drive `ChunkedRecorder` + mic. Fixture only for `make demo`. | `CT-live-stop-not-fixture`, `CT-fixture-audio-mismatch` |
| **C** | `feat/presence` | `presence` | Login item, menu bar, notch HUD (LED + timer). | `CT-hud-recording`, `CT-login-item-backend` |
| **D** | `feat/hotkey` | `hotkey` | Configurable toggle + optional PTT. | hotkey CTs |
| **E** | `feat/meeting-detect` | `meeting-detect` | Local process watch; ask before record; never silent-start. | meeting-detect CTs |

Still no bot, no silent cloud, no calendar sync.

## H–J closeout (shipped)

v1 PRs 0–6 plus Waves A–G (`#7`–`#13`) are on `main`. System audio and mixed capture shipped. The PRD still needs **two working local engines** (Parakeet decode, not only install), **STT off the UI thread**, and a **desk source picker**. Do not skip; do not parallelize.

| PR | Branch | Spec | What ships | Merge gate |
|----|--------|------|------------|------------|
| **F** | `feat/system-audio` | `system-audio` | ScreenCaptureKit tap on macOS. **Shipped** (`#12`). | `CT-system-tap-status`, `CT-system-not-fixture` |
| **G** | `feat/mixed-capture` | `mixed-capture` | Mix system + mic. Never mic-only fallback. **Shipped** (`#13`). | `CT-mixed-not-mic-only`, `CT-mix-pcm` |
| **H** | `feat/parakeet-runtime` | `parakeet-runtime` | On-device Parakeet decode. **Shipped** (`#14`). | `CT-parakeet-runtime-status`, `CT-parakeet-not-fixture` |
| **I** | `feat/stt-worker` | `stt-worker` | Batch transcribe off the Tauri command thread. **Shipped** (`#15`). | `CT-stt-worker-releases-lock`, `CT-stt-worker-same-result` |
| **J** | `feat/source-picker` | `source-picker` | Desk chooses system / mic / mixed. Permission copy. Consent still required. **Shipped** (`#16`). | `CT-source-unknown`, `CT-source-permission-copy` |

Order was **H → I → J**. Parakeet came before the worker so both engines share the worker. Source picker came last so mixed/system were real before the desk exposed them.

## After H–J

The implementation has the core pieces, but founder daily use is not yet a certified path. **Soft-launch bar:** `docs/SOFT_LAUNCH.md`. Continue serially after wave 38. One spec and two DAG waves per PR.

| PR | Branch | Spec | What ships | Merge gate |
|----|--------|------|------------|------------|
| **K** | `fix/judge-reliability` | `judge-reliability` | Deterministic macOS judge keystore so demo/contracts/CI complete without changing production Keychain or the fixture boundary. | `CT-keychain-test-deterministic`, `CT-judge-completes`, `make ci` |
| **L** | `feat/model-onboarding` | `model-onboarding` | Honest runnable-engine state, local Whisper file / Parakeet TDT directory import, and an explicit non-fixture engine for live transcription. Fixture remains demo-only. | `CT-model-runnable-ready`, `CT-model-import-local`, `CT-live-engine-runnable`, `make ci` |
| **M** | `feat/crash-recovery` | `crash-recovery` | Discover incomplete consented captures and recover them through encrypted finalization before cleanup. | `CT-recovery-discovery`, `CT-recovery-encrypted`, `make ci` |
| **N** | `chore/macos-founder-certification` | `macos-founder-certification` | Separate desktop-build evidence from human hardware/TCC/recovery/real-model evidence. | `CT-macos-desktop-gate`, `CT-macos-hardware-e2e`, `make ci` |
| **O** | `docs/readme-closeout` | `docs-readme-closeout` | Public docs describe post-J behavior, prerequisites, and verification limits without Wave 1 or Linux/macOS evidence drift. | `CT-docs-current`, `CT-coverage-honesty`, `make ci` |

**PRs remaining: 5. Features remaining: 5. New specs: 5. DAG waves: 39–48.**

Order: **K → L → M → N → O**. Restore the required judge first, make live recordings transcribable second, recover interrupted recordings third, certify the actual Mac path fourth, then publish only the claims supported by that evidence.

Candidate decisions from the completeness review:

- Add K before product work because every later wave requires a green local contract gate and the current macOS judge fails at Keychain creation.
- Combine first-run/model onboarding, Parakeet TDT directory import, and live default-engine behavior in L; they are one setup and selection journey.
- Keep crash recovery because the PRD promises crash-safe recording but startup does not expose the existing chunk recovery primitive.
- Replace immediate notarization with N's founder certification. Signing/notarization remains a later distribution gate.
- Keep docs closeout last so it can cite the certification result rather than predict it.

Still out of scope: meeting bot, cloud STT default, calendar, teams, Windows/Linux, notarization, streaming STT, diarization.

## Explicit non-goals in K–O

Meeting bot, cloud STT default, telemetry on, calendar, teams, browser extension, Windows/Linux, App Store notarization.
