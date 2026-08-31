"""INV-FIXTURE-LOCK"""

import pytest

pytestmark = pytest.mark.contract

from pathlib import Path
import json
import hashlib

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / "harness/graph/fixture-lock.json"
FIXTURES = ROOT / "fixtures"


def test_every_fixture_matches_lock():
    lock = json.loads(LOCK.read_text())
    for name, meta in lock["files"].items():
        path = FIXTURES / name
        assert path.is_file(), name
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        assert digest == meta["sha256"], name
        assert path.stat().st_size == meta["bytes"], name
