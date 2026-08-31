# Decisions

## Demand-first gate — Sotto

- **Tier 1 exempt:** yes — founder explicit override. This is a tool Bogdan needs for himself (sensitive conversations, no bot in the room). First user is the founder. Built in public so others with the same constraint can use and contribute.
- **Tier 2 interview count:** 0 / 5 (personal tool; interviews are launch-plan work, not a blocker for the fixture MVP)
- **Tier 3 fallback signal:** none required (Tier 1)
- **Evidence:** this file; user request 2026-09-01 to build the private meeting recorder in public

**Verdict:** ALLOW

Caveat: do not expand into cloud sync, teams, or paid packaging until there is evidence beyond the founder.

## Name

**Sotto** (sotto voce). Repo: `bogdan-veliscu/sotto`. Bundle id: `com.bogdanveliscu.sotto`.

Rejected: generic "Private Meeting Recorder", anything with Whisper in the product name, another `*-desk` clone.

## Wave 1 cuts

- Capture is a golden WAV fixture, not Core Audio taps yet.
- Transcription is `fixture-replay`, not Parakeet/Whisper weights.
- Metadata SQLite is not SQLCipher yet. Audio files are AES-GCM.
- Master key is a file, not Keychain yet.

These are specified so the next waves have a contract, not so wave 1 pretends they shipped.
