# Decisions

## Demand-first gate — Sotto

- **Tier 1 exempt:** yes — founder explicit override. This is a tool Bogdan needs for himself (sensitive conversations, no bot in the room). First user is the founder. Built in public so others with the same constraint can use and contribute.
- **Tier 2 interview count:** 0 / 5 (personal tool; interviews are launch-plan work, not a blocker for the fixture MVP)
- **Tier 3 fallback signal:** none required (Tier 1)
- **Evidence:** this file; user request 2026-09-01 to build the private meeting recorder in public

**Verdict:** ALLOW

Caveat: do not expand into cloud sync, teams, or paid packaging until there is evidence beyond the founder.

## Soft launch — trusted circle (2026-09-01)

Founder override: proceed to a **trusted-circle** soft launch, not a public paid launch.

Done means `docs/SOFT_LAUNCH.md`: a first user can grant TCC, import a local Whisper file or Parakeet TDT directory, record with consent, get an on-device transcript, recover or discard a crashed capture, and follow an honest README.

Ship K → L → M → N → O. L is the product P0. Notarization, Product Hunt, Stripe, and interviews-before-charging stay out of this bar. Interviews remain required before **charging** (`docs/LAUNCH_PLAN.md` Phase 0).

Lead (Cursor) and Codex (`til:codex`) cooperate: Codex implements one PR at a time; lead steers, reviews, and merges.

## Presence and hotkeys (2026-09-01)

Founder override for personal daily use: login item, configurable global shortcut (start/stop/toggle), notch HUD while recording, and optional local meeting-app detection (Zoom / Teams / Slack). Still no bot, no silent cloud, no calendar sync unless a later override says so.

Meeting detection must **ask** before recording (Wispr-style card). Never start capture without the consent disclosure.


## Name

**Sotto** (sotto voce). Repo: `bogdan-veliscu/sotto`. Bundle id: `com.bogdanveliscu.sotto`.

Rejected: generic "Private Meeting Recorder", anything with Whisper in the product name, another `*-desk` clone.

## Wave 1 cuts

- Capture is a golden WAV fixture, not Core Audio taps yet.
- Transcription is `fixture-replay`, not Parakeet/Whisper weights.
- Metadata SQLite is not SQLCipher yet. Audio files are AES-GCM.
- Master key is a file, not Keychain yet.

These are specified so the next waves have a contract, not so wave 1 pretends they shipped.
