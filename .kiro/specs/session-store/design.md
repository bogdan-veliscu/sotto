# Session store — Design

`Store::open` creates the data directory, master key, SQLite schema, and default settings.

`engines::resolve_engine` is pure. Catalog is `fixtures/models.json`, compiled in with `include_str!`.
