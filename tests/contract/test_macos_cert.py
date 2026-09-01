"""Manifest-only macOS founder certification contracts.

These tests never start capture, request TCC access, run a decoder, or inspect
audio/transcript content. Linux/GHA and absent macOS evidence are `not-run`.
"""

import json
import sys
from pathlib import Path
from typing import Any

import pytest

pytestmark = pytest.mark.contract

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "harness/evidence/macos-founder-certification.json"
SCHEMA = "sotto/macos-founder-certification/v1"

DESKTOP_OUTCOMES = ("desktop_build", "ui_check", "app_bundle")
HARDWARE_OUTCOMES = (
    "consent",
    "mic",
    "system",
    "mixed",
    "pause_resume",
    "recovery",
    "encrypted_audio",
    "real_local_stt",
)
FORBIDDEN_CONTENT_KEYS = {"audio", "audio_bytes", "transcript", "transcript_text"}


def _manifest() -> dict[str, Any]:
    if sys.platform != "darwin":
        pytest.skip("macOS certification is not a Linux/GHA gate")
    if not MANIFEST.is_file():
        pytest.skip("macOS founder evidence is absent: not-run")
    data = json.loads(MANIFEST.read_text())
    assert data.get("schema") == SCHEMA
    assert isinstance(data.get("commit"), str) and data["commit"].strip()
    _assert_content_free(data)
    return data


def _assert_content_free(value: Any) -> None:
    if isinstance(value, dict):
        for key, nested in value.items():
            assert key.lower() not in FORBIDDEN_CONTENT_KEYS
            _assert_content_free(nested)
    elif isinstance(value, list):
        for nested in value:
            _assert_content_free(nested)


def _assert_passes(section: Any, required: tuple[str, ...]) -> None:
    assert isinstance(section, dict)
    for name in required:
        assert section.get(name) == "pass", f"{name} is not certified"


def test_macos_desktop_gate():
    """CT-macos-desktop-gate: validate retained automated Mac evidence."""
    data = _manifest()
    _assert_passes(data.get("automated"), DESKTOP_OUTCOMES)


def test_macos_hardware_e2e():
    """CT-macos-hardware-e2e: validate human-run evidence, never hardware."""
    data = _manifest()
    _assert_passes(data.get("hardware"), HARDWARE_OUTCOMES)
