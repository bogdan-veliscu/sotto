# Source picker — Requirements

Lead-locked. Do not rewrite.

## REQ-SP-001: Only known capture sources (INV-SOURCE-KNOWN)

**EARS:** WHEN a capture source is parsed, THE SYSTEM SHALL accept only `mic`, `system`, or `mixed`. Any other value SHALL return `SOURCE_UNKNOWN` with `recoverable: true`. It SHALL NOT silently treat an unknown source as mixed or start a meeting bot.

CT-source-unknown.

## REQ-SP-002: Permission copy and consent stay required (INV-SOURCE-CONSENT-COPY)

**EARS:** WHEN `source_permission_hint` runs for a known source, THE SYSTEM SHALL return copy that mentions consent is still required. WHEN the source is `mixed`, the copy SHALL say mixed will not fall back to microphone-only. It SHALL NOT say capture may start without consent.

CT-source-permission-copy.

No network. No silent record. No meeting bot. Linux CI stays green without Screen Recording or a microphone.
