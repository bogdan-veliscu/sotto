# Lead steer — completeness-review graph honesty

Codex: you finished the review commit `339cfd5` on `docs/completeness-review`. Stay on that branch. Specs+DAG+plan only. Do not implement K. Do not push. Do not rewrite locked EARS.

## Defect to fix now

`harness/graph/domain.graph.json` points these CTs at `tests/contract/test_catalog.py`:

- `CT-judge-completes`
- `CT-macos-desktop-gate`
- `CT-macos-hardware-e2e`
- `CT-docs-current`
- `CT-coverage-honesty`

That file only checks `fixtures/models.json` is local. Graph `path` is the enforcement home. Putting Mac certification and README truth there would either fail Linux `make ci` or pretend the catalog test covers them.

`harness/dispatch/wave-39.md` already names `ct_judge_completes` in `src-tauri/tests/contract.rs`. The graph disagrees. Fix the graph (and designs/tasks if they cite catalog.py).

## Required test homes

| CT | Path |
|---|---|
| `CT-keychain-test-deterministic`, `CT-judge-completes` | `src-tauri/tests/contract.rs` |
| `CT-model-*`, `CT-recovery-*` | already `contract.rs` — leave |
| `CT-docs-current`, `CT-coverage-honesty` | new `tests/contract/test_docs.py` (string/claim checks; no model download) |
| `CT-macos-desktop-gate` | new `tests/contract/test_macos_cert.py` — on Linux/GHA **skip**; do not fail `make ci` |
| `CT-macos-hardware-e2e` | same file — **skip unless** a human evidence manifest exists. Never call `CGRequestScreenCaptureAccess` or `start_live(Mic)`. Missing evidence = skip/`not-run`, not fail on Ubuntu. |

Update macos-founder-certification design: Linux `--no-default-features` CI must stay green. Hardware layer is explicit human-run; automated tests only assert the runner/manifest schema or skip.

## Also

- `make graph` green after the path fix.
- One conventional commit on this branch, e.g. `docs(spec): house post-J CTs in honest test files`
- Do not start `fix/judge-reliability` implementation until this lands.

Stop after the commit. Worktree clean. No product source changes.
