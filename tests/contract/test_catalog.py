"""INV-NO-CLOUD-DEFAULT — catalog and settings."""

import pytest

pytestmark = pytest.mark.contract

from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "fixtures/models.json"


def test_catalog_engines_are_local():
    data = json.loads(CATALOG.read_text())
    engines = data["engines"]
    assert engines, "catalog empty"
    assert all(e["mode"] == "local" for e in engines)
    assert any(e["id"] == "fixture-replay" and e["install_state"] == "ready" for e in engines)
    assert not any(e["mode"] in {"cloud", "api"} for e in engines)


def test_planned_models_are_not_installed():
    data = json.loads(CATALOG.read_text())
    by_id = {e["id"]: e for e in data["engines"]}
    assert by_id["parakeet-tdt-0.6b-v3"]["install_state"] == "not-installed"
    assert by_id["whisper-large-v3-turbo"]["install_state"] == "not-installed"
    assert by_id["apple-speech-ondevice"]["mode"] == "local"
    assert by_id["apple-speech-ondevice"]["install_state"] == "not-installed"
