# Desk closeout — Requirements

Lead-locked. Do not rewrite.

## REQ-DC-001: Title filter without a text query (INV-FILTER-TITLE)

**EARS:** WHEN `search_filtered` is given only a title substring, THE SYSTEM SHALL return sessions whose titles contain that substring (case-insensitive) and SHALL NOT require a transcript FTS query.

CT-filter-title.

## REQ-DC-002: Desk does not throw invoke errors in the browser shell

The Vite desk SHALL NOT call Tauri `invoke` unless `__TAURI_INTERNALS__` is present. Privacy, search, and settings actions in the shell SHALL show a recoverable hint, not a TypeError.

No new network. No silent cloud.
