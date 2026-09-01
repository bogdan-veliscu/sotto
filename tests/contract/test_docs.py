"""Future documentation-truth contracts for waves 47–48.

The assertions live here now so the domain graph names an honest enforcement
home. They are skipped until docs-readme-closeout is implemented; wave 47 owns
removing the skips. These tests perform filesystem string checks only.
"""

from pathlib import Path

import pytest

pytestmark = pytest.mark.contract

ROOT = Path(__file__).resolve().parents[2]

CURRENT_DOCS = (
    ROOT / "README.md",
    ROOT / "docs/PRODUCT_BRIEF.md",
    ROOT / "docs/PR_PLAN.md",
    ROOT / "docs/LAUNCH_PLAN.md",
    ROOT / "docs/API_CONTRACT.md",
    ROOT / "docs/ARCHITECTURE.md",
    ROOT / "docs/UX_FLOWS.md",
    ROOT / "KIRO_BRIEF.md",
)

STALE_CLAIMS = (
    "live system-audio / mic capture | next wave",
    "catalogued, not wired",
    "fixture path ships now. live capture next",
    "transcribe_run` | fixture engine",
    "wave 1 does not call the tap yet",
    "record a fixture capture to seed the desk",
)

EVIDENCE_CLASSES = (
    "linux core",
    "macos desktop",
    "hardware/tcc",
    "real local-model",
)


def _combined(paths: tuple[Path, ...]) -> str:
    return "\n".join(path.read_text().lower() for path in paths)


@pytest.mark.skip(reason="wave 47 docs-readme-closeout is pending")
def test_docs_current():
    """CT-docs-current: shipped work is not described as future work."""
    body = _combined(CURRENT_DOCS)
    for stale in STALE_CLAIMS:
        assert stale not in body, f"stale product claim remains: {stale}"
    assert "make demo" in body and "fixture-replay" in body
    assert "screen recording" in body and "local model" in body


@pytest.mark.skip(reason="wave 48 evidence-scope docs are pending")
def test_coverage_honesty():
    """CT-coverage-honesty: each evidence class stays explicit."""
    body = _combined((ROOT / "README.md", ROOT / "docs/PR_PLAN.md"))
    for label in EVIDENCE_CLASSES:
        assert label in body, f"missing verification class: {label}"
