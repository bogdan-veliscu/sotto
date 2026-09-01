# Mixed capture — Requirements

Lead-locked. Do not rewrite.

## REQ-MX-001: Mixed never silently becomes mic-only (INV-MIXED-NO-FALLBACK)

**EARS:** WHEN `start_live(Mixed)` cannot open both a system-audio tap and a microphone, THE SYSTEM SHALL return `MIXED_UNAVAILABLE` or `CAPTURE_UNSUPPORTED` with `recoverable: true`. It SHALL NOT open a live session that records microphone only. WHEN `system_tap_status` is not `available`, `start_live(Mixed)` SHALL fail.

CT-mixed-not-mic-only.

## REQ-MX-002: Mix is a sum of both lanes (INV-MIXED-PCM)

**EARS:** WHEN `mix_pcm` is given microphone and system samples, THE SYSTEM SHALL return saturating-averaged PCM of equal length to the longer input, padding the shorter lane with silence. It SHALL NOT return CONSULT-001 bytes.

CT-mix-pcm.

No network. No silent record. No meeting bot. Linux CI must stay green without Screen Recording or a microphone.
