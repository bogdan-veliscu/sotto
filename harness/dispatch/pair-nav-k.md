# Navigator brief — K (Codex)

You are **navigator**, not driver. Read `harness/dispatch/PAIR.md`.

Branch: `fix/judge-reliability`. Goal: prove K is **not** soft-launch ready, or say SHIP.

## Claim to attack

Isolated judge keystore makes local `make demo` / `make contract` / `make ci` finish on macOS without Keychain, production desktop still uses Keychain, Linux GHA stays green, fixture demo unchanged.

## Do this

1. `git status --short --branch` and `git log --oneline main..HEAD`
2. Read `src-tauri/src/keys.rs`, `Makefile`, `.github/workflows/ci.yml`, the two new tests in `src-tauri/tests/contract.rs`
3. Try to break the claim. Useful angles:
   - desktop feature + `SOTTO_JUDGE_KEYSTORE=isolated-file` still using file backend
   - GHA cargo steps missing the env (should be set at job level now)
   - `make dev` accidentally selecting the judge backend
   - demo no longer `fixture-replay` / `network_calls: 0`
   - key bytes printed
4. Do **not** start L. Do **not** rewrite EARS. Do **not** push. Do **not** edit product source unless you have a single `FIX` that is required for SHIP.

Reply with `SHIP` or `FIX` (one defect, file:line, expected gate). Then release the keyboard.
