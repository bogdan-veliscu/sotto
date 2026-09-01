# Completeness review — Sotto (lead dispatch to Codex)

You are Codex in `/Users/bogdan/kiro/sotto` on `main` (`69dd5e8` or later). Agent window is `til:codex` (`til:4`). Do this review yourself. Do not wait for Kiro. Do not send keys to other tmux windows.

Public repo: https://github.com/bogdan-veliscu/sotto

Bundle id: `com.bogdanveliscu.sotto`
Founder tool. Demand-first override is in `docs/DECISIONS.md`. Personal daily use. Never a meeting bot. Never silent-start. Never silent cloud STT.

## Job

1. **Complete review** of the entire implementation vs PRD, PRODUCT_BRIEF, specs, DAG, desk UI, and honesty (what the code actually does vs what docs claim).
2. **Completeness:** what a first user can do today with `make demo` / `make dev` / `make ci`, and what is still stub, missing, misleading, or untestable.
3. **New tasks:** ordered PRs / features that are still in-scope for the founder using this as a daily recorder. Out of scope stays out: meeting bot, cloud STT default, calendar, teams, Windows/Linux, streaming STT, diarization, SQLCipher unless you have a strong privacy reason and say so.
4. **Prepare all needed specs** in Kiro form. Do **not** implement the features in this pass. Specs + DAG + plan only.

## Read first

- `AGENTS.md`, `KIRO_BRIEF.md`, `docs/PRD.md`, `docs/PRODUCT_BRIEF.md`, `docs/PR_PLAN.md`, `docs/DECISIONS.md`, `docs/MODEL_ABSTRACTION.md`, `README.md`
- `harness/graph/{domain.graph.json,task-dag.yaml,progress.json}`
- Every spec under `.kiro/specs/`
- Desk: `src/routes/+page.svelte`, `src/lib/api.ts`
- Core: `src-tauri/src/{lib,store,stt,stt_parakeet,install,capture,commands,presence,hotkey,meeting}.rs`
- Tests: `src-tauri/tests/contract.rs`
- `Makefile`, `.github/workflows/`

Trust `git status` / `git log` over TUI chrome. Latest complete-product PRs on `main`: H `#14` Parakeet decode, I `#15` STT worker, J `#16` source picker.

## Method

- Requirements-First. EARS. Specs live in `.kiro/specs/<name>/{requirements,design,tasks}.md`.
- **Do not rewrite locked EARS** in existing specs.
- **Do not edit** `fixtures/` or `harness/graph/fixture-lock.json`.
- **Do not `/spec new` via Kiro CLI.** Write the markdown files yourself.
- Named contract tests are the done gate for any new wave. Propose `CT-*` names and DAG waves continuing after 38.
- Conventional commits, subject ≤72 chars. You MAY commit on a branch `docs/completeness-review`. Do not push. Do not touch `main` except via that branch. Do not force-push.
- `make graph` must stay green after you add nodes/edges/waves.
- Linux CI is `make ci` / `--no-default-features`. It does not compile `--features desktop --bins`. Do not claim desktop-only code is covered by GHA.

## Honesty gaps to verify (do not paper over)

Confirm or refute against the code. Add more if you find them.

- README / LAUNCH_PLAN still describe Wave 1 (fixture only; Parakeet/Whisper “not wired”). Product on `main` is further.
- Overlay `ready` for Parakeet if a dummy checksum `.bin` exists, even though decode needs a TDT **directory**. Desk “Install file” cannot place that directory.
- Live transcribe with default `fixture-replay` on real audio → `FIXTURE_AUDIO_MISMATCH` (intentional). First-run does not guide Whisper/TDT install.
- Parakeet weights (~1.2GB) and Whisper ggml are not in git and must never be downloaded by `make demo` or contract tests.
- Tray menu is Open desk only (Record from tray would skip consent).
- `login_item_backend()` may still report `smappservice` while Open at login uses LaunchAgent.
- Mixed/system need Screen Recording already granted; tests must never `CGRequestScreenCaptureAccess` or `start_live(Mic)`.
- Local `make demo` can hang on Keychain; GHA Linux is the merge gate.
- Notarization / signed bundle not done.
- Notes are extractive, not an LLM.

## Deliverables (this pass only)

1. `docs/COMPLETENESS_REVIEW.md` — findings, severity, evidence (file:line or test name), what works, what is left. No marketing. No claiming ready without a decoder / tap / permission.
2. New specs for every in-scope gap you recommend shipping next, each with locked EARS (2 invariants typical), design, tasks. Suggested candidates to evaluate (keep, split, drop, or replace after you read the code):
   - first-run / model-onboarding (Whisper file + Parakeet TDT **folder**, no download in demo)
   - parakeet-tdt-layout install (directory overlay, do not treat dummy `.bin` as a runnable model)
   - docs-readme closeout (honest What works today)
   - notarize / signed macOS app (only if you treat it as a real next founder task)
   - default-engine-for-live (do not leave fixture-replay as the live default without copy)
   Drop anything that is already specified and shipped. Do not spec a meeting bot.
3. Update `docs/PR_PLAN.md` with a new “After H–J” table: ordered PRs, branches, specs, merge gates. **PRs remaining / features** counts must match.
4. Extend `harness/graph/domain.graph.json` and `task-dag.yaml` for the new specs (waves 39+). `make graph` green. Do not mark them complete in `progress.json`.
5. Dispatch notes under `harness/dispatch/` for the first new wave only (`wave-39.md` + runner), same shape as `wave-37.md`.
6. Update `KIRO_BRIEF.md` current DAG wave to the first new wave.

Stop when the review doc + specs + DAG + plan are committed on `docs/completeness-review`. Do not implement product features. Do not push.
