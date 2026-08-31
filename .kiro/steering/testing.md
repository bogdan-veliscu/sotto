---
inclusion: fileMatch
fileMatchPattern: ["tests/**", "src-tauri/tests/**", "**/*test*.py"]
---

# Testing

- Graph invariants are encoded in `harness/graph/domain.graph.json`.
- Python tests under `tests/contract/` use `@pytest.mark.contract`.
- Rust pipeline: `src-tauri/tests/contract.rs`.
- Fixtures are the only seed data.
- After a wave, that wave's contract tests must pass before the next wave.
