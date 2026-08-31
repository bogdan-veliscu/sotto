# AGENTS.md — Sotto

You are implementing **Sotto**, a local-first macOS meeting recorder.

Read these before writing code:

1. `KIRO_BRIEF.md`
2. `harness/graph/domain.graph.json`
3. `harness/graph/task-dag.yaml`
4. `.kiro/steering/`

## Hard rules

- Specs live in `.kiro/specs/<name>/{requirements,design,tasks}.md`. Lead already wrote them. Do not `/spec new`. Do not rewrite locked EARS.
- Implement only the current wave. Named contract tests are the done gate.
- Golden files in `fixtures/` are immutable.
- Cloud STT is off by default. Fallback never selects cloud/api unless `cloud_mode=on`.
- Recording requires consent_state `acknowledged`.
- Finalized audio on disk must not look like a WAV.
- No transcript content in logs.
- Kiro is not a runtime dependency.

## Done means

`make graph` green and `make contract` green for every invariant whose wave you claim is complete.

## Git

Conventional commits, subject ≤72 chars. Do not force-push `main`.
