# Meeting detect — Requirements

Lead-locked. Do not rewrite.

## REQ-MD-001: Local process names classify meeting apps (INV-MEETING-DETECT)

**EARS:** WHEN `classify_processes` is given names that include Zoom, Microsoft Teams, or Slack, THE SYSTEM SHALL return those kinds. WHEN given only unrelated names, THE SYSTEM SHALL return an empty list. Classification SHALL NOT start recording.

CT-meeting-detect-apps.

## REQ-MD-002: Detection asks; it never silent-starts (INV-MEETING-ASK)

**EARS:** WHEN meeting detect is enabled and a classified app is present and the desk is idle, THE SYSTEM SHALL recommend a prompt. WHEN detect is off, already recording, or no app is classified, THE SYSTEM SHALL NOT recommend a prompt. Detecting a meeting SHALL NOT bypass `CONSENT_REQUIRED`.

CT-meeting-never-silent.

No calendar. No network. No meeting bot. Default is off.
