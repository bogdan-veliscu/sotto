# v1 PR plan

Initial commit on `main` is **PR 0** (`b956583`): desk UI, privacy invariants, fixture pipeline. That is not the complete product.

Complete **v1** means the PRD success criteria: record on this Mac, transcribe locally with a real model, search, export, encrypted audio, no cloud by default. Windows, Linux, SQLCipher, notarization, streaming STT, diarization, and self-host sync are **after v1**.

Six PRs remain. One spec (or two DAG waves) per PR. Contract tests are the merge gate. Do not skip the DAG.

| PR | Branch | Spec | What ships | Merge gate |
|----|--------|------|------------|------------|
| **0** | `main` | `session-store`, `capture-consent`, `search-notes` | Public desk. Fixture record → encrypt → search. | **Shipped** |
| **1** | `feat/live-capture` | `live-capture` | Real capture pipeline: chunked WAV, pause/resume, crash recover, mic via CPAL. System-audio tap may return `CAPTURE_UNSUPPORTED`. **Shipped** (`#1`). | `CT-capture-wav`, `CT-pause-resume`, `CT-crash-partial`, `CT-demo-still-offline` |
| **2** | `feat/local-whisper` | `local-stt` | Whisper Large-v3 Turbo **local weights**, batch transcribe. Never download inside `make demo`. **Shipped** (`#2`). | `CT-whisper-local-only`, `CT-demo-no-download` |
| **3** | `feat/parakeet-install` | `model-install` | Parakeet TDT 0.6B v3 as selectable default. User-initiated checksummed install, overlay, delete. **Shipped** (`#3`). | `CT-checksum`, `CT-parakeet-local`, `CT-no-silent-cloud` (still) |
| **4** | `feat/notes-export` | `notes-export` | Summary / action items / key points from the transcript (extractive local, no cloud LLM). Settings pane. Save-as markdown via dialog. **Shipped** (`#4`). | `CT-summary-from-transcript`, `CT-export-file`, `CT-settings-privacy` |
| **5** | `feat/search-filters` | `search-filters` | Title, date range, tags. **Shipped** (`#5`). | `CT-filter-date`, `CT-tag-roundtrip` |
| **6** | `feat/harden-keychain` | `harden` | Master key in macOS Keychain (file 0600 on Linux CI). Retention deletes. Crash-safe temps gone. **Shipped** (`#6`). | `CT-keychain`, `CT-retention` |

## Counts

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

## Complete product (after A–G)

v1 PRs 0–6 plus Waves A–G (`#7`–`#13`) are on `main`. System audio and mixed capture shipped. The PRD still needs **two working local engines** (Parakeet decode, not only install), **STT off the UI thread**, and a **desk source picker**. Do not skip; do not parallelize.

| PR | Branch | Spec | What ships | Merge gate |
|----|--------|------|------------|------------|
| **F** | `feat/system-audio` | `system-audio` | ScreenCaptureKit tap on macOS. **Shipped** (`#12`). | `CT-system-tap-status`, `CT-system-not-fixture` |
| **G** | `feat/mixed-capture` | `mixed-capture` | Mix system + mic. Never mic-only fallback. **Shipped** (`#13`). | `CT-mixed-not-mic-only`, `CT-mix-pcm` |
| **H** | `feat/parakeet-runtime` | `parakeet-runtime` | On-device Parakeet decode. **Shipped** (`#14`). | `CT-parakeet-runtime-status`, `CT-parakeet-not-fixture` |
| **I** | `feat/stt-worker` | `stt-worker` | Batch transcribe off the Tauri command thread so the desk/HUD stay live. | `CT-stt-worker-releases-lock`, `CT-stt-worker-same-result` |
| **J** | `feat/source-picker` | `source-picker` | Desk chooses system / mic / mixed. Permission copy. Consent still required. | source-picker CTs |

**PRs remaining: 2. Features: 2.** One spec per PR.

Order: **H → I → J**. Parakeet before the worker so both engines share the worker. Source picker last so mixed/system are real before the desk exposes them.

Still out of scope: meeting bot, cloud STT default, calendar, teams, Windows/Linux, notarization, streaming STT, diarization.

## Explicit non-goals in these six PRs

Meeting bot, cloud STT default, telemetry on, calendar, teams, browser extension, Windows/Linux, App Store notarization.
