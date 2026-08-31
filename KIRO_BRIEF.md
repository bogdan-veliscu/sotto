# Kiro brief — Sotto

You are Kiro CLI V3. Work only in this repository. Requirements-First. Do not use Vibe mode for product features.

Sotto is a **local-first meeting recorder**. Headline: a meeting recorded on this Mac must never leave the device unless the user explicitly enables a cloud engine.

## Current product slice (wave 1)

Fixture capture → AES-GCM audio → fixture-replay transcript → FTS5 search → delete-all.

Live Core Audio taps and Parakeet/Whisper weights are specified, not this wave.

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
