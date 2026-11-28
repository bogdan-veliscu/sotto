"""INV-FIXTURE-LOCK"""

import hashlib
import json
import os
import shlex
import subprocess
from pathlib import Path

import pytest

pytestmark = pytest.mark.contract

ROOT = Path(__file__).resolve().parents[2]
LOCK = ROOT / "harness/graph/fixture-lock.json"
FIXTURES = ROOT / "fixtures"
GUARD_HOOK = ROOT / ".kiro/hooks/guard-fixtures.json"


def test_every_fixture_matches_lock():
    lock = json.loads(LOCK.read_text())
    for name, meta in lock["files"].items():
        path = FIXTURES / name
        assert path.is_file(), name
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        assert digest == meta["sha256"], name
        assert path.stat().st_size == meta["bytes"], name


@pytest.mark.parametrize("source", ["stdin", "argv", "env"])
@pytest.mark.parametrize(
    ("path", "exit_code"),
    [
        ("fixtures/models.json", 2),
        ("harness/graph/fixture-lock.json", 2),
        ("docs/DECISIONS.md", 0),
    ],
)
def test_fixture_guard(source, path, exit_code):
    hook = json.loads(GUARD_HOOK.read_text())["hooks"][0]
    command = shlex.split(hook["action"]["command"])
    env = os.environ.copy()
    env.pop("SOTTO_ALLOW_FIXTURE_MUTATION", None)
    env.pop("KIRO_HOOK_INPUT", None)
    stdin = ""
    event = {
        "hook_event_name": "PreToolUse",
        "cwd": str(ROOT),
        "session_id": "fixture-guard-test",
        "tool_name": "fs_write",
        "tool_input": {"path": path},
    }
    if source == "stdin":
        stdin = json.dumps(event)
    elif source == "argv":
        command.append(path)
    else:
        env["KIRO_HOOK_INPUT"] = json.dumps(event)

    result = subprocess.run(
        command, input=stdin, env=env, cwd=ROOT, capture_output=True, text=True,
    )
    assert result.returncode == exit_code, result.stderr
