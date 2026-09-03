# Navigator brief — L (Codex)

You are **navigator**, not driver. Read `harness/dispatch/PAIR.md`.

Branch: `feat/model-onboarding`. Goal: prove L is **not** soft-launch ready, or say SHIP.

## Claim to attack

Live-ready requires a compiled decoder plus a runnable local layout. A Parakeet checksum `.bin` is not live-ready. Import copies a Whisper file or Parakeet TDT directory without network. Live Stop never runs `fixture-replay`; it uses an explicit live-ready engine or leaves encrypted audio `recorded` with `ENGINE_SETUP_REQUIRED`. `demo_pipeline` still requests fixture-replay with `network_calls: 0`.

## Do this

1. `git status --short --branch` and `git log --oneline main..HEAD`
2. Read `src-tauri/src/install.rs`, `src-tauri/src/store.rs` `prepare_transcribe`, `src/routes/+page.svelte` Stop/import, and the three new tests in `src-tauri/tests/contract.rs`
3. Try to break the claim. Useful angles:
   - dummy `.bin` overlaying as `live_ready`
   - live Stop still hitting `FIXTURE_AUDIO_MISMATCH` / fixture-replay
   - `demo_pipeline` no longer fixture or network_calls ≠ 0
   - import from `https://` succeeding
   - failed import clobbering a previous Whisper file
   - Linux CI requiring a real decoder
4. Do **not** start M. Do **not** rewrite EARS. Do **not** push. Do **not** edit product source unless you have a single `FIX` required for SHIP.

Reply with `SHIP` or `FIX` (one defect, file:line, expected gate). Then release the keyboard.
