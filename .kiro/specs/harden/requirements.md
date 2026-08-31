# Harden keychain — Requirements

Lead-locked. Do not rewrite.

## REQ-HD-001: Key storage (INV-KEYCHAIN)

**EARS:** WHEN a store is opened, THE SYSTEM SHALL load or create a 32-byte master key via `KeyStore`. On macOS the backend SHALL be `keychain`. Elsewhere the backend SHALL be `file` with mode `0600`. Reopening the same data directory SHALL use the same key.

CT-keychain.

## REQ-HD-002: Retention (INV-RETENTION)

**EARS:** WHEN `retention_days` is a positive integer and `apply_retention` runs, THE SYSTEM SHALL delete sessions whose `created_at` is older than that many days, including their audio files. `retention_days=0` SHALL delete nothing.

CT-retention.

Plaintext WAV leftovers in the audio directory SHALL be removed by `scrub_plaintext_temps`. No HTTP.
