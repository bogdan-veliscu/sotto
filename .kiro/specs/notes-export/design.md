# Notes export — Design

Module: `src-tauri/src/notes.rs`. Public API is locked by the contract tests. Do not rename.

```rust
pub struct Notes {
    pub summary: String,
    pub action_items: String,
    pub key_points: String,
}

pub fn extract_notes(transcript: &str) -> Result<Notes>;
```

## Extractive, local

No LLM, no HTTP. Derive notes from the transcript text:

- **summary**: first 1–2 sentences (or first ~240 chars), must preserve a distinctive term from the input.
- **action_items**: lines / sentences that look like follow-ups (`follow up`, `todo`, `next:`, leading `- `).
- **key_points**: remaining short claims, or the same sentences if nothing else exists.

Empty transcript → empty strings, not an error.

When `TranscriptResult.summary_text` is empty after STT, `Store::transcribe` / `persist_transcript` MUST fill notes via `extract_notes` so Whisper output still gets a summary. Fixture-replay may keep its golden notes; they must still match extractive constraints (they already do).

## File export

```rust
impl Store {
    pub fn export_markdown_file(&self, session_id: &str, dest: &Path) -> Result<()>;
    pub fn privacy_settings(&self) -> Result<PrivacySettings>;
}

pub struct PrivacySettings {
    pub telemetry: String,
    pub cloud_mode: String,
    pub retention_days: String,
}
```

`export_markdown_file` writes `export_markdown()` bytes to `dest`. Create parent dirs. Reject `http://` / `https://` destinations. Desktop may wrap this with a save dialog later; tests call the path API.

## Settings

`privacy_settings` reads the existing settings keys. Defaults stay off. The desk Settings/Models pane should show telemetry and cloud_mode as off unless the user turned them on.

## Forbidden

- Cloud LLM / OpenAI / any HTTP from `notes.rs`
- Editing fixtures
- Turning telemetry on in `ensure_defaults`
