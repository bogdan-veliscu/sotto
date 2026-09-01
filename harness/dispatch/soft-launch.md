# Soft-launch cooperation — Codex

Read `docs/SOFT_LAUNCH.md` first. You implement. Lead reviews and merges.

## Now

Specs for K–O are on `docs/completeness-review`. Lead is landing that branch as a PR.

Your first implementation wave is **K**.

```
git status --short --branch
git checkout -B fix/judge-reliability
```

Branch from current `docs/completeness-review` (includes `2a65f54` and the soft-launch docs). If that branch later merges to `main`, rebase onto `main` before the K PR. Do not branch from stale `main` that lacks the K specs.

Execute `harness/dispatch/wave-39.md` exactly.

Stop when:

- `fix/judge-reliability` has conventional commit(s)
- `make graph`, `make contract`, desktop `cargo check --features desktop --bins`, and `npm run check` are green
- `make demo` completes without a Keychain prompt
- production desktop still uses Keychain (no silent file fallback)
- worktree clean
- nothing pushed

Then wait. Lead will review, PR, and merge. Do not start L until lead says so.

## Later (do not start now)

L → M → N → O as in `docs/SOFT_LAUNCH.md`. Same stop-for-review rule after each PR.

## Never

- Meeting bot, silent start, cloud STT default
- Rewrite locked EARS
- Commit model weights or fixtures
- Treat Linux CI as Mac hardware evidence
- Product Hunt / notarization / paid packaging work
