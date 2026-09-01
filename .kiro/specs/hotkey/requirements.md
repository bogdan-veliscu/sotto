# Hotkey — Requirements

Lead-locked. Do not rewrite.

## REQ-HK-001: Shortcut strings are explicit (INV-HOTKEY-PARSE)

**EARS:** WHEN `parse_hotkey` is given a non-empty shortcut string, THE SYSTEM SHALL return the trimmed string. WHEN it is given an empty or whitespace-only string, THE SYSTEM SHALL return `HOTKEY_INVALID`.

CT-hotkey-parse.

## REQ-HK-002: Mode is toggle or PTT (INV-HOTKEY-MODE)

**EARS:** WHEN `parse_hotkey_mode` is given `toggle` or `ptt`, THE SYSTEM SHALL accept it. WHEN given any other value, THE SYSTEM SHALL return `HOTKEY_INVALID`. Default mode is `toggle`. Default shortcut is `CommandOrControl+Shift+Space`.

CT-hotkey-mode.

The hotkey SHALL NOT start capture without the existing consent card. Fn-as-lone-modifier is not this PR (needs an event tap). No network.
