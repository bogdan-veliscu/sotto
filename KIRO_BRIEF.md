# Kiro brief — Sotto

You are Kiro CLI V3. Work only in this repository. Requirements-First. Do not use Vibe mode for product features.

Sotto is a **local-first meeting recorder**. Headline: a meeting recorded on this Mac must never leave the device unless the user explicitly enables a cloud engine.

## Current product slice

PR 0 (shipped): fixture capture → AES-GCM → fixture-replay → FTS5 → delete-all.

**Current DAG wave: 15–16 (`search-filters`).** Implement `src-tauri/src/search.rs` and `Store::search_filtered`. Local only. See `docs/PR_PLAN.md`.

## Commands

```
make graph
make contract
make demo
make dev
```

## Do not

- Download models in `make demo`.
- Add telemetry on by default.
- Join a meeting as a bot.
- Edit `fixtures/` or `harness/graph/fixture-lock.json`.
- Copy invoice / claims / four-eyes domains from sibling repos.
