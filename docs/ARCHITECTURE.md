# Architecture

Thin UI, Rust coordination, local AI worker, encrypted local data.

## Stack

| Layer | Choice |
|---|---|
| Desktop shell | Tauri 2 |
| UI | SvelteKit (static adapter, SPA) |
| Coordination | Rust (`src-tauri`) |
| Database | SQLite |
| Search | SQLite FTS5 |
| Audio at rest | AES-256-GCM blobs |
| Packaging | Tauri bundle |

Why: Tauri stays light. Svelte is small. Rust is good for file safety. SQLite + FTS5 is the right local search primitive. No extra search engine.

## macOS audio capture

Prefer Apple's supported system-audio path (Core Audio taps) on modern macOS, with the usage description in `Info.plist`. Wave 1 does not call the tap yet; `NSAudioCaptureUsageDescription` is already in the bundle plist.

## Model layer

One interface. UI sees label, languages, speed, accuracy, hardware hints, install state. Never model-specific UI branches.

## Data flow

1. User starts recording (consent first).
2. Capture writes local audio (fixture WAV in wave 1).
3. Stop finalizes and encrypts.
4. Worker transcribes.
5. Segments + summary persist.
6. FTS index updates.
7. UI loads transcript and search.

## Updates

Signed bundles. Silent updates only if the user opts in.

## Observability

Minimal crash reporting later. No content logging. Opt-in diagnostics with redaction.
