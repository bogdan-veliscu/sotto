# Sotto

Working name: **Sotto**. Bundle id: `com.bogdanveliscu.sotto`.

A local-first macOS desk for recording meetings without a bot in the call.

## In scope (wave 1)

- Tauri 2 + SvelteKit desk UI.
- SQLite + FTS5.
- Consent gate.
- AES-GCM audio files.
- Fixture-replay engine so `make demo` is offline.
- Public docs and a Kiro DAG.

## Out of scope (this wave)

Live Core Audio taps, Parakeet/Whisper runtime, SQLCipher, Keychain, teams, calendar, browser extension, cloud sync.

## How we build

Lead writes locked EARS + RED contract tests + graph. Kiro implements one wave per prompt. Hard stop on named CTs. Do not `/spec new`. Do not edit `fixtures/`.
