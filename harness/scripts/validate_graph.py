#!/usr/bin/env python3
"""Validate the domain graph, task DAG, fixture lock."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
GRAPH = ROOT / "harness/graph/domain.graph.json"
DAG = ROOT / "harness/graph/task-dag.yaml"
LOCK = ROOT / "harness/graph/fixture-lock.json"
PROGRESS = ROOT / "harness/graph/progress.json"
FIXTURES = ROOT / "fixtures"


def load_json(path: Path) -> dict:
    return json.loads(path.read_text())


def parse_dag(text: str) -> list[dict]:
    waves: list[dict] = []
    current: dict | None = None
    for raw in text.splitlines():
        line = raw.rstrip()
        if line.startswith("  - id:"):
            current = {"id": int(line.split(":", 1)[1].strip()), "contract_tests": []}
            waves.append(current)
        elif current is not None and line.strip().startswith("spec:"):
            current["spec"] = line.split(":", 1)[1].strip()
        elif current is not None and "CT-" in line and line.strip().startswith("- CT-"):
            current.setdefault("contract_tests", []).append(line.strip().lstrip("- ").strip())
        elif current is not None and "contract_tests:" in line and "[" in line:
            inner = line.split("[", 1)[1].split("]", 1)[0]
            current["contract_tests"] = [x.strip() for x in inner.split(",") if x.strip()]
    return waves


def iter_fixture_files(root: Path) -> list[Path]:
    files: list[Path] = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != ".DS_Store":
            files.append(path)
    return files


def check_graph(graph: dict) -> list[str]:
    errors: list[str] = []
    ids = {n["id"] for n in graph["nodes"]}
    if len(ids) != len(graph["nodes"]):
        errors.append("duplicate node ids")
    for edge in graph["edges"]:
        if edge["from"] not in ids:
            errors.append(f"edge from unknown node {edge['from']}")
        if edge["to"] not in ids:
            errors.append(f"edge to unknown node {edge['to']}")
    invariants = [n for n in graph["nodes"] if n["type"] == "invariant"]
    for inv in invariants:
        enforced = [e for e in graph["edges"] if e["from"] == inv["id"] and e["rel"] == "ENFORCED_BY"]
        if not enforced:
            errors.append(f"invariant {inv['id']} has no ENFORCED_BY test")
        for e in enforced:
            node = next(n for n in graph["nodes"] if n["id"] == e["to"])
            path = ROOT / node["path"]
            if not path.is_file():
                errors.append(f"contract test missing: {node['path']}")
    return errors


def check_lock() -> list[str]:
    errors: list[str] = []
    lock = load_json(LOCK)
    for name, meta in lock["files"].items():
        path = FIXTURES / name
        if not path.is_file():
            errors.append(f"locked fixture missing: {name}")
            continue
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        if digest != meta["sha256"]:
            errors.append(f"FIXTURE DRIFT {name}: expected {meta['sha256'][:12]} got {digest[:12]}")
        if path.stat().st_size != meta["bytes"]:
            errors.append(f"FIXTURE SIZE DRIFT {name}")
    rels = {str(p.relative_to(FIXTURES)) for p in iter_fixture_files(FIXTURES)}
    for rel in sorted(rels):
        if rel not in lock["files"]:
            errors.append(f"unlocked fixture present: {rel}")
    return errors


def check_dag(graph: dict) -> list[str]:
    errors: list[str] = []
    waves = parse_dag(DAG.read_text())
    ids = [w["id"] for w in waves]
    if ids != sorted(ids) or len(ids) != len(set(ids)):
        errors.append("task-dag wave ids must be unique and ordered")
    test_ids = {n["id"] for n in graph["nodes"] if n["type"] == "contract_test"}
    for wave in waves:
        for ct in wave.get("contract_tests") or []:
            if ct not in test_ids:
                errors.append(f"wave {wave['id']} references unknown contract test {ct}")
    return errors


def status() -> int:
    graph = load_json(GRAPH)
    progress = load_json(PROGRESS) if PROGRESS.exists() else {}
    print(f"current_wave: {progress.get('current_wave')}")
    print(f"completed_nodes: {progress.get('completed_nodes')}")
    print(f"invariants: {sum(1 for n in graph['nodes'] if n['type'] == 'invariant')}")
    print(f"contract_tests: {sum(1 for n in graph['nodes'] if n['type'] == 'contract_test')}")
    return 0


def rewrite_lock() -> int:
    files = {}
    for path in iter_fixture_files(FIXTURES):
        rel = str(path.relative_to(FIXTURES))
        data = path.read_bytes()
        files[rel] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
    LOCK.write_text(json.dumps({"version": 1, "files": files}, indent=2) + "\n")
    print(f"wrote {LOCK} ({len(files)} files)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--status", action="store_true")
    parser.add_argument("--rewrite-lock", action="store_true")
    args = parser.parse_args()
    if args.status:
        return status()
    if args.rewrite_lock:
        return rewrite_lock()
    errors: list[str] = []
    graph = load_json(GRAPH)
    errors.extend(check_graph(graph))
    errors.extend(check_lock())
    errors.extend(check_dag(graph))
    if errors:
        print("GRAPH INVALID", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1
    print("graph ok: domain + fixture lock + task DAG")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
