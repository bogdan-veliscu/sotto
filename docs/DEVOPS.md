# DevOps

## Repo

Flat Tauri app (easiest for contributors), not a multi-package monorepo.

```
src/  src-tauri/  fixtures/  docs/  harness/  tests/  .kiro/
```

## CI

Lint is `npm run check`. Tests are `make graph` + `make contract` (Rust pipeline + Python catalog). Full `tauri build` is macOS-only and not required to merge core changes.

## Release (later)

Signed app bundle, notarization, checksums. Signing keys never in git.

## Telemetry

Off by default. Opt-in only. No transcript content. No audio samples.

## Future self-host

Postgres optional, object storage optional, compose, admin token. Not v1.
