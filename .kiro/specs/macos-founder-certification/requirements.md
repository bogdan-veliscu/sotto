# macOS founder certification — Requirements

Lead-locked. Do not rewrite.

## REQ-MC-001: Desktop build evidence (INV-MACOS-DESKTOP-GATE)

**EARS:** WHEN a PR claims the macOS desktop is buildable, THE SYSTEM SHALL have current evidence for the `desktop` Rust targets, desk UI check, and unsigned `.app` bundle. Linux `--no-default-features` results SHALL NOT satisfy this gate.

CT-macos-desktop-gate.

## REQ-MC-002: Founder hardware path evidence (INV-MACOS-HARDWARE-E2E)

**EARS:** WHEN Sotto is certified for founder daily use, THE SYSTEM SHALL have current content-free evidence for consented mic/system/mixed capture outcomes, encrypted finalization, pause/resume, recovery, and at least one real local decoder/model transcription. A compile-only result SHALL NOT satisfy this gate.

CT-macos-hardware-e2e.

Tests SHALL NOT call `CGRequestScreenCaptureAccess` or silently start microphone capture. Hardware/TCC steps are explicit human-run probes.
