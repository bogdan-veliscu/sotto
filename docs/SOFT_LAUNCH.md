# Soft launch bar — trusted circle

**Goal date context:** 2026-09-01. This is the shared done definition for lead + Codex.

Soft launch means a few trusted people can record a real meeting on a Mac and get a local transcript. It is not Product Hunt, not a paid plan, and not a notarized download.

## A first user can

1. Build with `make dev` or run the unsigned `.app` from `make build` (Gatekeeper: right-click Open is acceptable).
2. Grant Microphone. Grant Screen Recording only if they pick system or mixed.
3. Import a local Whisper ggml **file** or a Parakeet TDT **directory**. The desk shows honest `ready` only when that engine can actually decode.
4. Acknowledge consent, record, Stop, and get an on-device transcript. Live Stop must not land on `fixture-replay` / `FIXTURE_AUDIO_MISMATCH`.
5. Search the transcript and export markdown.
6. If the app dies mid-record, see the incomplete session and Recover (encrypt) or Discard. No silent wipe of consented chunks.
7. Keep audio encrypted at rest. No bot in the call. No silent cloud STT.

## Ship this, in order

| PR | Why it is on the bar |
|----|----------------------|
| **K** judge-reliability | Local `make demo` / `make ci` must finish so later PRs can be judged. Production Keychain stays. |
| **L** model-onboarding | **The product gap.** Without it, live Stop cannot transcribe. |
| **M** crash-recovery | A meeting recorder that drops the last session is not launchable. |
| **N** macos-founder-certification | One retained Mac hardware pass. Linux CI stays skip/`not-run`. |
| **O** docs-readme-closeout | README must describe the path above, not Wave 1 / “live capture next”. |

K unblocks shipping. **L is the launch P0.** Do not skip L to polish N.

## Explicitly not this bar

Notarization, App Store, DMG/Sparkle, Stripe, landing page, Product Hunt, calendar, teams, Windows/Linux, meeting bot, cloud STT default, streaming STT, diarization, SQLCipher.

Interviews-before-charging stays in `docs/LAUNCH_PLAN.md` Phase 0. Soft launch may go to trusted people **without charging**. Do not add paid packaging.

## Evidence

- Merge gate remains GitHub Ubuntu `make ci` (portable). It does not prove taps, TCC, or real models.
- Founder daily-use evidence is N’s content-free manifest plus a human-run transcript on this Mac.
- Dummy checksum `.bin` is never a runnable model.

## Roles

- **Lead (Cursor):** keep this bar, steer Codex, review diffs, open/merge PRs, run the N hardware pass.
- **Codex (`til:codex`):** implement one PR at a time from locked specs. Stop for review. Do not push. Do not rewrite EARS. Do not parallelize K–O.
