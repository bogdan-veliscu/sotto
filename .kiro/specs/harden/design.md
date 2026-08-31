# Harden — Design

Module: `src-tauri/src/keys.rs`. Public API is locked by the contract tests. Do not rename.

```rust
pub struct KeyReport {
    pub backend: String, // "keychain" | "file"
    pub key_len: usize,
    pub fingerprint: String,
}

pub fn load_or_create(data_dir: &Path) -> Result<([u8; 32], &'static str)>;

impl Store {
    pub fn key_report(&self) -> Result<KeyReport>;
    pub fn apply_retention(&self) -> Result<u32>;
    pub fn scrub_plaintext_temps(&self) -> Result<u32>;
}
```

## KeyStore

- **macOS:** Keychain generic password. Service `com.bogdanveliscu.sotto`. Account is a digest of `data_dir` so tests isolate. If `master.key` already exists, migrate it into Keychain and remove the file.
- **elsewhere (Linux CI):** `master.key` next to SQLite, `chmod 0600`. Never world-readable.

Do not log the key. Fingerprint is a hex prefix of SHA-256(key), not the key.

`Store::open` uses `load_or_create`. `apply_retention` and `scrub_plaintext_temps` run on open.

## Retention

`retention_days` is already a setting. `0` keeps everything. Positive N deletes sessions with `CAST(created_at AS INTEGER) < now - N*86400`, via existing `delete_session` (audio + FTS + row).

## Temps

`scrub_plaintext_temps` deletes `audio/*` files whose contents look like a WAV (`RIFF…WAVE`) or whose name ends in `.wav` / `.tmp`. Encrypted `.sotto` files stay.

## Forbidden

- Logging key bytes
- HTTP
- SQLCipher (after v1)
- Editing fixtures
