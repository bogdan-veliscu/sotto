# Wave 17–18 — harden

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/harden/requirements.md
4. .kiro/specs/harden/design.md
5. .kiro/specs/harden/tasks.md
6. src-tauri/src/keys.rs (stub)
7. src-tauri/src/store.rs
8. src-tauri/tests/contract.rs

## Do this wave

Implement KeyStore + retention so these tests pass:

- `ct_keychain`
- `ct_retention`

macOS → Keychain. Linux CI → `master.key` mode 0600. Same key on reopen. `retention_days=0` deletes nothing. Positive days delete old sessions and audio. Scrub leftover plaintext WAVs. Never log the key. Never HTTP.

## Done gate

```
make graph
cd src-tauri && cargo test --no-default-features --test contract
make demo
```

## Do not

- Edit fixtures/
- Enable telemetry
- Commit or push
- Change public names in keys.rs

When tests pass, check the boxes in `.kiro/specs/harden/tasks.md` and stop.
