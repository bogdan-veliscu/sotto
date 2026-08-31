# Search filters — Design

Module: `src-tauri/src/search.rs`. Public API is locked by the contract tests. Do not rename.

```rust
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    pub q: String,
    pub title: Option<String>,
    pub created_from: Option<String>,
    pub created_to: Option<String>,
    pub tag: Option<String>,
}

impl Store {
    pub fn search_filtered(&self, filter: &SearchFilter, limit: i64) -> Result<Vec<SearchHit>>;
    pub fn set_tags(&self, session_id: &str, tags: &[String]) -> Result<Vec<String>>;
    pub fn list_tags(&self, session_id: &str) -> Result<Vec<String>>;
    pub fn set_created_at(&self, session_id: &str, created_at: &str) -> Result<()>;
}
```

`created_at` in v1 is a unix-seconds string (see `now_rfc3339`). Range compare is numeric on that value, inclusive.

Tags live in `session_tags(session_id, tag)` with PK `(session_id, tag)`. Normalize: trim, lowercase, drop empty, unique, stable sort on return. `set_tags` replaces the set.

`search(q, limit)` remains a wrapper around `search_filtered` with only `q` set.

Empty `q` still applies title / date / tag filters against `sessions` (snippet may be empty). Text `q` uses FTS5 plus the same filters.

## Forbidden

- Remote search / HTTP
- Editing fixtures
- Breaking existing `search("privileged")` on the golden transcript
