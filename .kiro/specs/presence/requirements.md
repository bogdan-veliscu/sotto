# Presence — Requirements

Lead-locked. Do not rewrite.

## REQ-PR-001: HUD state from recorder status (INV-HUD-RECORDING)

**EARS:** WHEN `hud_from_status` is given status `recording` and a positive elapsed time, THE SYSTEM SHALL return `led_on=true` and a `mm:ss` clock. WHEN status is `paused`, THE SYSTEM SHALL return `paused=true` and `led_on=false`.

CT-hud-recording.

## REQ-PR-002: Login-item backend is platform-honest (INV-LOGIN-ITEM-BACKEND)

**EARS:** WHEN `login_item_backend` runs on macOS, THE SYSTEM SHALL report `smappservice`. WHEN it runs elsewhere, THE SYSTEM SHALL report `unsupported` and SHALL NOT pretend a login item was registered.

CT-login-item-backend.

No network. No silent record. No meeting bot.
