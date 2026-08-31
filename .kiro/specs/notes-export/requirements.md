# Notes export — Requirements

Lead-locked. Do not rewrite.

## REQ-NE-001: Extractive notes (INV-SUMMARY-FROM-TRANSCRIPT)

**EARS:** WHEN `extract_notes` is given a transcript that contains a distinctive claim and an imperative follow-up, THE SYSTEM SHALL return a local summary that includes the distinctive claim AND action items that include the follow-up, without calling the network.

CT-summary-from-transcript.

## REQ-NE-002: File export (INV-EXPORT-FILE)

**EARS:** WHEN `export_markdown_file` runs for a transcribed session, THE SYSTEM SHALL write a markdown file at the given filesystem path that contains the transcript text. The path SHALL NOT be a URL.

CT-export-file.

## REQ-NE-003: Privacy settings (INV-SETTINGS-PRIVACY)

**EARS:** WHEN a store is opened, `privacy_settings` SHALL report `telemetry=off` and `cloud_mode=off`. Explicit `set_setting` may turn a toggle on. Demo SHALL keep both off.

CT-settings-privacy.

No cloud LLM. No telemetry on by default.
