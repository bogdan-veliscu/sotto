---
inclusion: always
---

# Technology stack (locked)

- Tauri 2, Rust, SvelteKit, SQLite, FTS5
- AES-GCM for audio files
- Node 22 + npm for the UI
- Python 3.12 only for harness/graph and contract tests
- `make demo` is the judge path and must run offline after crate fetch
- Forbidden: required cloud APIs, LLM keys, telemetry on by default, Inter/Roboto/purple UI
