# Capture consent — Requirements

Lead-locked. Do not rewrite.

## REQ-CA-001: Consent (INV-CONSENT-BEFORE-RECORD)

**EARS:** WHEN `start_recording` runs, THE SYSTEM SHALL require `consent_state` equal to `acknowledged` and SHALL return `CONSENT_REQUIRED` otherwise.

CT-consent-before-record.

## REQ-CA-002: Encrypted audio (INV-AUDIO-ENCRYPTED)

**EARS:** WHEN audio is finalized, THE SYSTEM SHALL write a file whose first bytes are not `RIFF`/`WAVE`.

CT-audio-encrypted.
