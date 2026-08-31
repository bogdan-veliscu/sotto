# Search filters — Requirements

Lead-locked. Do not rewrite.

## REQ-SF-001: Date range (INV-FILTER-DATE)

**EARS:** WHEN `search_filtered` is given a created-at range, THE SYSTEM SHALL return only sessions whose `created_at` falls inside that inclusive range.

CT-filter-date.

## REQ-SF-002: Tags (INV-TAG-ROUNDTRIP)

**EARS:** WHEN tags are set on a session, THE SYSTEM SHALL persist them, return the same set from `list_tags`, and include that session in `search_filtered` for one of those tags.

CT-tag-roundtrip.

Title substring filter is part of the same `SearchFilter` API. No network. No cloud index.
