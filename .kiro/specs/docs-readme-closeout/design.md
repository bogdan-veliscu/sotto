# Docs and README closeout — Design

Make the public entry points agree with the post-J implementation and the evidence produced by the preceding waves.

## Canonical status

`docs/COMPLETENESS_REVIEW.md` is the gap baseline. README owns the short “What works today” table and quick start. `docs/PR_PLAN.md` owns ordered remaining work. `KIRO_BRIEF.md` owns the current wave. Detailed architecture/API/UX documents describe current behavior, while dated historical decisions remain explicitly historical.

## Claim matrix

Use four labels consistently:

- portable core contract (`--no-default-features`),
- macOS desktop compile/build,
- macOS hardware/TCC probe,
- real local-model transcription.

Do not collapse these into “CI green” or “ready.” Record the exact command or named test beside consequential claims. `make demo` is always identified as the golden fixture path.

## Scope

Close stale Wave 1 wording in README, PRODUCT_BRIEF, LAUNCH_PLAN, API_CONTRACT, ARCHITECTURE, UX_FLOWS, IMPLEMENTATION_ROADMAP, MODEL_ABSTRACTION, and relevant contributor-facing docs. Preserve locked EARS and historical decision records. Update PR/feature counts from the DAG.

## Contract home

`tests/contract/test_docs.py` owns `CT-docs-current` and `CT-coverage-honesty`. The tests are string/claim checks only: no network, model import, decoder, or desktop launch. They remain explicitly skipped while wave 47 is pending; wave 47 removes the skip as part of making the documented claim set pass.

## Forbidden

- Claiming real capture from fixture evidence
- Claiming real STT from catalog/install-state evidence
- Claiming macOS coverage from Linux CI
- Rewriting existing locked requirements
