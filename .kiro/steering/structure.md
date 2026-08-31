---
inclusion: always
---

# Repository structure

```
src/                    # SvelteKit desk
src-tauri/src/          # store, crypto, engines, commands
src-tauri/tests/        # Rust contract pipeline
fixtures/               # GOLDEN
harness/
tests/contract/         # Python invariant tests
.kiro/specs/
docs/
```

Contract tests are `src-tauri/tests/contract.rs` plus `tests/contract/*.py`.
