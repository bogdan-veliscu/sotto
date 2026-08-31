# Security

Sotto stores meeting audio and transcripts on the local machine.

## Default posture

- Telemetry off.
- Cloud mode off.
- `make demo` makes zero network calls after crate download.
- Audio files are AES-256-GCM. Wave 1 keeps `master.key` beside the database.

## Please report privately

If you find a way to:

- read another user's audio without their key,
- upload audio without `cloud_mode=on`,
- recover deleted sessions from FTS leftovers,

email **bogdan.veliscu@outlook.com**. Do not open a public GitHub issue for those classes of bug.

## Threat notes (honest)

- Wave 1 does not use SQLCipher. Metadata and transcripts sit in SQLite. Enable FileVault.
- Wave 1 does not use the Keychain. Anyone with the app data directory has the key and the ciphertext.
- The UI runs in a webview. Do not `eval` search snippets.

## Supported versions

`main` only until a tagged 0.2.
