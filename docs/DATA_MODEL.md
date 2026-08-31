# Data model

SQLite first. Simple. FTS5 for search.

Tables match `src-tauri/src/store.rs`:

- `sessions` — id, timestamps, title, status, model_id, language, duration, consent_state, notes, source
- `audio_assets` — encrypted file path, checksum, size
- `transcripts` — raw_text, cleaned_text, status, language
- `transcript_segments` — start_ms, end_ms, text, confidence, speaker_label
- `summaries` — summary_text, action_items, key_points
- `settings` — key/value (telemetry, cloud_mode, default_model, …)
- `transcript_fts` — FTS5 on session_id, title, transcript_text, summary_text

Indexes: sessions.created_at, transcripts.session_id.

Audio bytes are not in SQLite. They are `.sotto` files under the app data `audio/` directory.
