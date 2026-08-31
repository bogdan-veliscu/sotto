#!/usr/bin/env python3
"""Block writes to content-addressed fixtures and the lockfile."""

from __future__ import annotations

import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GUARDED = (
    ROOT / "fixtures",
    ROOT / "harness/graph/fixture-lock.json",
)

payload = os.environ.get("KIRO_HOOK_INPUT", "") + " ".join(sys.argv[1:])
text = payload.lower()
blocked = ("fixtures/", "fixture-lock.json")
if any(token in text.replace("\\", "/") for token in blocked):
    if os.environ.get("SOTTO_ALLOW_FIXTURE_MUTATION") != "1":
        print("blocked: golden fixtures are content-addressed", file=sys.stderr)
        raise SystemExit(2)
raise SystemExit(0)
