# Crash recovery — Requirements

Lead-locked. Do not rewrite.

## REQ-CR-001: Discover recoverable captures (INV-RECOVERY-DISCOVERY)

**EARS:** WHEN Sotto opens after capture stopped without finalization and a consented session has flushed `chunk-*.pcm` files, THE SYSTEM SHALL report that session as recoverable. It SHALL NOT silently discard the chunks, silently resume recording, or attach them to a different session.

CT-recovery-discovery.

## REQ-CR-002: Recovery finalizes encrypted audio (INV-RECOVERY-ENCRYPTED)

**EARS:** WHEN the user chooses Recover for a discovered capture, THE SYSTEM SHALL rebuild the WAV from that session's chunks, encrypt it through the normal finalization path, remove the plaintext chunks only after encrypted persistence succeeds, and leave the session `recorded` for local transcription. The finalized file SHALL NOT begin with `RIFF` or `WAVE`.

CT-recovery-encrypted.

Discard SHALL require a separate explicit action. Recovery SHALL NOT select fixture or cloud STT.
