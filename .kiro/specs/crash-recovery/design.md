# Crash recovery — Design

Connect the existing `ChunkedRecorder::recover` primitive to persisted sessions and the desk.

## Discovery

On startup, scan only `{app_data}/live/<session_id>/`. A candidate is valid when `<session_id>` exists in the store, consent is acknowledged, status is `recording` or `paused`, and at least one chunk is present. Return metadata only: session id, title, chunk count, and recoverable duration. Do not include PCM or transcript content in logs.

Unknown directories are quarantined for explicit cleanup; they are never attached by inference. Empty candidates remain errors, not fake audio.

## Recover

`Store::recover_live(session_id)` calls `ChunkedRecorder::recover`, then the same AES-GCM finalization invariant as a normal stop. Cleanup order is transactional in spirit: persist encrypted asset and session state first, then remove the live directory. On any failure, retain the chunks so the user can retry.

The desk presents Recover and Discard separately. Recover does not auto-transcribe; the normal runnable local-engine path may be invoked after encrypted finalization. Discard requires confirmation and deletes only the selected session's live directory.

## Tests

Tests inject PCM into a temporary `ChunkedRecorder`, drop it, reopen the Store, and recover without microphone, Screen Recording, Keychain UI, model weights, or network.

## Forbidden

- Deleting chunks before encrypted persistence succeeds
- Automatic discard or automatic recording resume
- Fixture/cloud transcription during recovery
- Logging audio or transcript content
