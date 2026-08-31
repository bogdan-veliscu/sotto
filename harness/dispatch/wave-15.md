# Wave 15–16 — search-filters

You implement **Sotto** in `/Users/bogdan/kiro/sotto`. Agent `scribe`. Requirements-First. Do not `/spec new`. Do not rewrite locked EARS.

Read first:

1. AGENTS.md
2. KIRO_BRIEF.md
3. .kiro/specs/search-filters/requirements.md
4. .kiro/specs/search-filters/design.md
5. .kiro/specs/search-filters/tasks.md
6. harness/graph/task-dag.yaml
7. src-tauri/src/search.rs
8. src-tauri/src/store.rs (`search_filtered`, `set_tags`, `list_tags`, `set_created_at` stubs)
9. src-tauri/tests/contract.rs (RED contract tests)

## Do this wave

Implement search filters so these tests pass:

- `ct_filter_date`
- `ct_tag_roundtrip`

Keep existing FTS `search("privileged")` working. No HTTP. No cloud index.

`created_at` is a unix-seconds string. Range is inclusive numeric compare.

Tags: trim, lowercase, drop empty, unique. `set_tags` replaces the set.

## Done gate

```
make graph
cd src-tauri && cargo test --no-default-features --test contract
make demo
```

## Do not

- Edit fixtures/ or harness/graph/fixture-lock.json
- Break CT-fts-search
- Commit or push
- Change the public names in search.rs / the Store methods listed above

When tests pass, check the boxes in `.kiro/specs/search-filters/tasks.md` and stop.
