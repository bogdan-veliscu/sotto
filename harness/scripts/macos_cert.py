#!/usr/bin/env python3
"""macOS founder-certification runner.

Records content-free evidence only. Never starts capture, never calls
CGRequestScreenCaptureAccess, never downloads weights, never stores
transcript or audio bytes.

Usage:
  python3 harness/scripts/macos_cert.py --automated
  python3 harness/scripts/macos_cert.py --set recovery=pass encrypted_audio=pass
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "harness/evidence/macos-founder-certification.json"
SCHEMA = "sotto/macos-founder-certification/v1"
APP_BUNDLE = ROOT / "src-tauri/target/release/bundle/macos/Sotto.app"
ALLOWED = ("pass", "fail", "not-run", "needs-permission")
DESKTOP = ("desktop_build", "ui_check", "app_bundle")
HARDWARE = (
    "consent",
    "mic",
    "system",
    "mixed",
    "pause_resume",
    "recovery",
    "encrypted_audio",
    "real_local_stt",
)
FORBIDDEN = {"audio", "audio_bytes", "transcript", "transcript_text"}


def _run(args: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )


def _git_commit() -> str:
    proc = _run(["git", "rev-parse", "HEAD"])
    if proc.returncode != 0:
        raise SystemExit("git rev-parse HEAD failed")
    return proc.stdout.strip()


def _blank() -> dict:
    return {
        "schema": SCHEMA,
        "commit": _git_commit(),
        "recorded_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platform": sys.platform,
        "automated": {name: "not-run" for name in DESKTOP},
        "hardware": {name: "not-run" for name in HARDWARE},
        "commands": {},
        "notes": "Content-free founder certification. No transcript or audio.",
    }


def _load() -> dict:
    if MANIFEST.is_file():
        data = json.loads(MANIFEST.read_text())
    else:
        data = _blank()
    data.setdefault("automated", {})
    data.setdefault("hardware", {})
    data.setdefault("commands", {})
    return data


def _assert_content_free(value: object) -> None:
    if isinstance(value, dict):
        for key, nested in value.items():
            if key.lower() in FORBIDDEN:
                raise SystemExit(f"refusing to write forbidden key {key}")
            _assert_content_free(nested)
    elif isinstance(value, list):
        for nested in value:
            _assert_content_free(nested)


def _write(data: dict) -> None:
    data["schema"] = SCHEMA
    data["commit"] = _git_commit()
    data["recorded_at"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    data["platform"] = sys.platform
    _assert_content_free(data)
    MANIFEST.parent.mkdir(parents=True, exist_ok=True)
    MANIFEST.write_text(json.dumps(data, indent=2) + "\n")
    print(f"wrote {MANIFEST.relative_to(ROOT)}")


def _outcome(proc: subprocess.CompletedProcess[str]) -> str:
    return "pass" if proc.returncode == 0 else "fail"


def run_automated() -> None:
    if sys.platform != "darwin":
        raise SystemExit("automated desktop evidence is macOS-only")
    data = _load()
    graph = _run(["make", "graph"])
    data["commands"]["graph"] = "make graph"
    check = _run(
        [
            "cargo",
            "check",
            "--manifest-path",
            "src-tauri/Cargo.toml",
            "--features",
            "desktop",
            "--bins",
        ]
    )
    data["commands"]["desktop_build"] = (
        "cargo check --manifest-path src-tauri/Cargo.toml --features desktop --bins"
    )
    data["automated"]["desktop_build"] = _outcome(check)
    if check.returncode != 0:
        print(check.stderr[-2000:], file=sys.stderr)

    ui = _run(["npm", "run", "check"])
    data["commands"]["ui_check"] = "npm run check"
    data["automated"]["ui_check"] = _outcome(ui)
    if ui.returncode != 0:
        print(ui.stderr[-2000:], file=sys.stderr)

    binary = APP_BUNDLE / "Contents/MacOS/sotto"
    data["commands"]["app_bundle"] = str(APP_BUNDLE.relative_to(ROOT))
    data["automated"]["app_bundle"] = "pass" if binary.is_file() else "not-run"
    data["commands"]["graph_status"] = _outcome(graph)
    _write(data)
    if data["automated"]["desktop_build"] != "pass" or data["automated"]["ui_check"] != "pass":
        raise SystemExit("automated desktop evidence is not all pass")


def apply_sets(pairs: list[str]) -> None:
    if sys.platform != "darwin":
        raise SystemExit("hardware evidence is macOS-only")
    data = _load()
    for raw in pairs:
        if "=" not in raw:
            raise SystemExit(f"expected name=status, got {raw}")
        name, status = raw.split("=", 1)
        if name not in HARDWARE:
            raise SystemExit(f"unknown hardware outcome {name}")
        if status not in ALLOWED:
            raise SystemExit(f"status must be one of {ALLOWED}")
        data["hardware"][name] = status
    _write(data)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--automated", action="store_true")
    parser.add_argument(
        "--set",
        dest="sets",
        nargs="+",
        default=[],
        help="hardware outcomes as name=pass|fail|not-run|needs-permission",
    )
    args = parser.parse_args()
    if not args.automated and not args.sets:
        parser.error("pass --automated and/or --set")
    if args.automated:
        run_automated()
    if args.sets:
        apply_sets(args.sets)


if __name__ == "__main__":
    main()
