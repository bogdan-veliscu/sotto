#!/usr/bin/env python3
"""Block writes to content-addressed fixtures and the lockfile."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GUARDED = (
    ROOT / "fixtures",
    ROOT / "harness/graph/fixture-lock.json",
)


def mentions_guarded_path(payload: str) -> bool:
    text = payload.lower().replace("\\", "/")
    return "fixtures/" in text or "fixture-lock.json" in text


def stdin_targets_guarded_path() -> bool:
    if sys.stdin.isatty():
        return False
    payload = sys.stdin.read()
    try:
        event = json.loads(payload)
    except json.JSONDecodeError:
        return mentions_guarded_path(payload)

    tool_input = event.get("tool_input") if isinstance(event, dict) else None
    if isinstance(tool_input, dict) and isinstance(tool_input.get("path"), str):
        path = tool_input["path"].replace("\\", "/")
        target = Path(str((ROOT / path).resolve()).lower())
        return any(target.is_relative_to(Path(str(guard).lower())) for guard in GUARDED)
    return mentions_guarded_path(payload)


if os.environ.get("SOTTO_ALLOW_FIXTURE_MUTATION") != "1" and (
    mentions_guarded_path(os.environ.get("KIRO_HOOK_INPUT", ""))
    or any(mentions_guarded_path(arg) for arg in sys.argv[1:])
    or stdin_targets_guarded_path()
):
    print("blocked: golden fixtures are content-addressed", file=sys.stderr)
    raise SystemExit(2)
