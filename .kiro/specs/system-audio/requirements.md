# System audio — Requirements

Lead-locked. Do not rewrite.

## REQ-SA-001: Tap status is platform-honest (INV-SYSTEM-TAP-STATUS)

**EARS:** WHEN `system_tap_status` runs off macOS, THE SYSTEM SHALL return `unsupported`. WHEN it runs on macOS, THE SYSTEM SHALL return one of `unsupported`, `needs-permission`, or `available`. It SHALL NOT claim `available` unless a system-audio backend is compiled in.

CT-system-tap-status.

## REQ-SA-002: System capture never fakes the fixture (INV-SYSTEM-NOT-FIXTURE)

**EARS:** WHEN `start_live(System)` cannot capture, THE SYSTEM SHALL return `CAPTURE_UNSUPPORTED` with `recoverable: true`. WHEN it can capture, THE SYSTEM SHALL open a live session and SHALL NOT write `CONSULT-001.wav` bytes. `demo_pipeline` SHALL still use the golden fixture.

CT-system-not-fixture.

No network. No silent record. No meeting bot. Linux CI must stay green without Screen Recording.
