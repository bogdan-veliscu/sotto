use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

use crate::crypto::{self, KEY_LEN};
use crate::engines::{self, Engine, TranscriptResult, FIXTURE_ENGINE_ID};
use crate::error::{Result, SottoError};

const KEY_FILE: &str = "master.key";
const DB_FILE: &str = "sotto.sqlite";
const AUDIO_DIR: &str = "audio";

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub title: String,
    pub status: String,
    pub model_id: Option<String>,
    pub language: Option<String>,
    pub duration_seconds: Option<i64>,
    pub consent_state: String,
    pub notes: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub session_id: String,
    pub title: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionDetail {
    pub session: Session,
    pub transcript: Option<String>,
    pub summary: Option<String>,
    pub action_items: Option<String>,
    pub key_points: Option<String>,
    pub segments: Vec<engines::TranscriptSegment>,
    pub audio_encrypted: bool,
    pub audio_path: Option<String>,
}

pub struct Store {
    conn: Connection,
    data_dir: PathBuf,
    master_key: [u8; KEY_LEN],
}

impl Store {
    pub fn open(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir)?;
        fs::create_dir_all(data_dir.join(AUDIO_DIR))?;
        let key_path = data_dir.join(KEY_FILE);
        let master_key = if key_path.exists() {
            let bytes = fs::read(&key_path)?;
            if bytes.len() != KEY_LEN {
                return Err(SottoError::app(
                    "KEY_INVALID",
                    "Local master key is the wrong length.",
                    false,
                    "Do not replace master.key. Restore it with the audio files.",
                ));
            }
            let mut key = [0u8; KEY_LEN];
            key.copy_from_slice(&bytes);
            key
        } else {
            let key = crypto::new_master_key();
            fs::write(&key_path, key)?;
            key
        };

        let conn = Connection::open(data_dir.join(DB_FILE))?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let store = Self {
            conn,
            data_dir: data_dir.to_path_buf(),
            master_key,
        };
        store.init_schema()?;
        store.ensure_defaults()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                started_at TEXT,
                ended_at TEXT,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                model_id TEXT,
                language TEXT,
                duration_seconds INTEGER,
                consent_state TEXT NOT NULL,
                notes TEXT,
                source TEXT NOT NULL DEFAULT 'mixed'
            );
            CREATE TABLE IF NOT EXISTS audio_assets (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                encrypted INTEGER NOT NULL DEFAULT 1,
                file_size INTEGER,
                checksum TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS transcripts (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                raw_text TEXT NOT NULL,
                cleaned_text TEXT,
                status TEXT NOT NULL,
                language TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS transcript_segments (
                id TEXT PRIMARY KEY,
                transcript_id TEXT NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER NOT NULL,
                text TEXT NOT NULL,
                confidence REAL,
                speaker_label TEXT,
                FOREIGN KEY(transcript_id) REFERENCES transcripts(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS summaries (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                summary_text TEXT NOT NULL,
                action_items TEXT,
                key_points TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at);
            CREATE INDEX IF NOT EXISTS idx_transcripts_session ON transcripts(session_id);
            CREATE VIRTUAL TABLE IF NOT EXISTS transcript_fts USING fts5(
                session_id,
                title,
                transcript_text,
                summary_text
            );
            "#,
        )?;
        Ok(())
    }

    fn ensure_defaults(&self) -> Result<()> {
        let defaults = [
            ("telemetry", "off"),
            ("cloud_mode", "off"),
            ("default_model", FIXTURE_ENGINE_ID),
            ("onboarding_complete", "false"),
            ("retention_days", "0"),
            ("export_format", "markdown"),
        ];
        for (key, value) in defaults {
            self.conn.execute(
                "INSERT OR IGNORE INTO settings(key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn cloud_enabled(&self) -> Result<bool> {
        Ok(self.get_setting("cloud_mode")?.as_deref() == Some("on"))
    }

    pub fn list_engines(&self) -> Result<Vec<Engine>> {
        engines::catalog()
    }

    pub fn create_session(&self, title: Option<String>, source: &str) -> Result<Session> {
        let id = format!("SES-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
        let now = now_rfc3339();
        let title = title
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| format!("Untitled {id}"));
        self.conn.execute(
            "INSERT INTO sessions(id, created_at, title, status, consent_state, source)
             VALUES (?1, ?2, ?3, 'idle', 'pending', ?4)",
            params![id, now, title, source],
        )?;
        self.get_session(&id)
    }

    pub fn acknowledge_consent(&self, session_id: &str) -> Result<Session> {
        let n = self.conn.execute(
            "UPDATE sessions SET consent_state = 'acknowledged' WHERE id = ?1",
            params![session_id],
        )?;
        if n == 0 {
            return Err(SottoError::app(
                "SESSION_MISSING",
                "That session does not exist.",
                false,
                "Refresh the list.",
            ));
        }
        self.get_session(session_id)
    }

    pub fn start_recording(&self, session_id: &str) -> Result<Session> {
        let session = self.get_session(session_id)?;
        if session.consent_state != "acknowledged" {
            return Err(SottoError::app(
                "CONSENT_REQUIRED",
                "Recording starts only after the disclosure is acknowledged.",
                true,
                "Read the consent card and confirm you may record this conversation.",
            ));
        }
        if session.status == "recording" {
            return Ok(session);
        }
        let now = now_rfc3339();
        self.conn.execute(
            "UPDATE sessions SET status = 'recording', started_at = COALESCE(started_at, ?2) WHERE id = ?1",
            params![session_id, now],
        )?;
        self.get_session(session_id)
    }

    pub fn pause_recording(&self, session_id: &str) -> Result<Session> {
        self.conn.execute(
            "UPDATE sessions SET status = 'paused' WHERE id = ?1 AND status = 'recording'",
            params![session_id],
        )?;
        self.get_session(session_id)
    }

    pub fn resume_recording(&self, session_id: &str) -> Result<Session> {
        self.start_recording(session_id)
    }

    pub fn finalize_with_wav(&self, session_id: &str, wav: &[u8]) -> Result<Session> {
        let session = self.get_session(session_id)?;
        if session.consent_state != "acknowledged" {
            return Err(SottoError::app(
                "CONSENT_REQUIRED",
                "Cannot save audio without consent.",
                true,
                "Acknowledge the disclosure, then record again.",
            ));
        }
        let packed = crypto::encrypt(&self.master_key, wav)?;
        let roundtrip = crypto::decrypt(&self.master_key, &packed)?;
        if roundtrip != wav {
            return Err(SottoError::app(
                "ENCRYPT_INVARIANT",
                "Encrypted audio did not round-trip.",
                false,
                "This is a defect. Do not ship this build.",
            ));
        }
        if crypto::looks_like_wav(&packed) {
            return Err(SottoError::app(
                "ENCRYPT_INVARIANT",
                "Encrypted audio still looks like a WAV file.",
                false,
                "This is a defect. Do not ship this build.",
            ));
        }
        let asset_id = format!("AUD-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
        let rel = PathBuf::from(AUDIO_DIR).join(format!("{asset_id}.sotto"));
        let abs = self.data_dir.join(&rel);
        fs::write(&abs, &packed)?;
        let checksum = crypto::sha256_hex(&packed);
        let now = now_rfc3339();
        self.conn.execute(
            "INSERT INTO audio_assets(id, session_id, kind, file_path, encrypted, file_size, checksum, created_at)
             VALUES (?1, ?2, 'mixed', ?3, 1, ?4, ?5, ?6)",
            params![
                asset_id,
                session_id,
                rel.to_string_lossy(),
                packed.len() as i64,
                checksum,
                now
            ],
        )?;
        self.conn.execute(
            "UPDATE sessions SET status = 'recorded', ended_at = ?2, duration_seconds = 1 WHERE id = ?1",
            params![session_id, now],
        )?;
        self.get_session(session_id)
    }

    pub fn transcribe(&self, session_id: &str, requested_model: Option<String>) -> Result<SessionDetail> {
        let cloud = self.cloud_enabled()?;
        let catalog = engines::catalog()?;
        let requested = requested_model
            .or(self.get_setting("default_model")?)
            .unwrap_or_else(|| FIXTURE_ENGINE_ID.to_string());
        let engine = engines::resolve_engine(&requested, cloud, &catalog)?;
        if engine.id != FIXTURE_ENGINE_ID {
            return Err(SottoError::app(
                "ENGINE_NOT_WIRED",
                format!("{} is catalogued but not wired in this wave.", engine.name),
                true,
                "Use Fixture replay until Parakeet / Whisper land in a later wave.",
            ));
        }
        let result = engines::fixture_transcript();
        self.persist_transcript(session_id, engine.id.as_str(), &result)?;
        self.get_detail(session_id)
    }

    fn persist_transcript(
        &self,
        session_id: &str,
        engine_id: &str,
        result: &TranscriptResult,
    ) -> Result<()> {
        let transcript_id = format!("TR-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
        let now = now_rfc3339();
        self.conn.execute(
            "INSERT INTO transcripts(id, session_id, raw_text, cleaned_text, status, language, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, 'complete', ?5, ?6, ?6)",
            params![
                transcript_id,
                session_id,
                result.raw_text,
                result.cleaned_text,
                result.language,
                now
            ],
        )?;
        for seg in &result.segments {
            let sid = format!("SEG-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
            self.conn.execute(
                "INSERT INTO transcript_segments(id, transcript_id, start_ms, end_ms, text, confidence)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![sid, transcript_id, seg.start_ms, seg.end_ms, seg.text, seg.confidence],
            )?;
        }
        let summary_id = format!("SUM-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
        self.conn.execute(
            "INSERT INTO summaries(id, session_id, summary_text, action_items, key_points, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                summary_id,
                session_id,
                result.summary_text,
                result.action_items,
                result.key_points,
                now
            ],
        )?;
        let title: String = self.conn.query_row(
            "SELECT title FROM sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO transcript_fts(session_id, title, transcript_text, summary_text)
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, title, result.raw_text, result.summary_text],
        )?;
        self.conn.execute(
            "UPDATE sessions SET status = 'transcribed', model_id = ?2, language = ?3 WHERE id = ?1",
            params![session_id, engine_id, result.language],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self, limit: i64) -> Result<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, started_at, ended_at, title, status, model_id, language,
                    duration_seconds, consent_state, notes, source
             FROM sessions ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], row_to_session)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Session> {
        self.conn
            .query_row(
                "SELECT id, created_at, started_at, ended_at, title, status, model_id, language,
                        duration_seconds, consent_state, notes, source
                 FROM sessions WHERE id = ?1",
                params![session_id],
                row_to_session,
            )
            .map_err(|_| {
                SottoError::app(
                    "SESSION_MISSING",
                    "That session does not exist.",
                    false,
                    "Refresh the list.",
                )
            })
    }

    pub fn get_detail(&self, session_id: &str) -> Result<SessionDetail> {
        let session = self.get_session(session_id)?;
        let transcript: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT id, raw_text FROM transcripts WHERE session_id = ?1 ORDER BY created_at DESC LIMIT 1",
                params![session_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let mut segments = Vec::new();
        if let Some((tid, _)) = &transcript {
            let mut stmt = self.conn.prepare(
                "SELECT start_ms, end_ms, text, confidence FROM transcript_segments
                 WHERE transcript_id = ?1 ORDER BY start_ms",
            )?;
            let mapped = stmt.query_map(params![tid], |row| {
                Ok(engines::TranscriptSegment {
                    start_ms: row.get(0)?,
                    end_ms: row.get(1)?,
                    text: row.get(2)?,
                    confidence: row.get(3)?,
                })
            })?;
            segments = mapped.collect::<rusqlite::Result<Vec<_>>>()?;
        }
        let summary = self
            .conn
            .query_row(
                "SELECT summary_text, action_items, key_points FROM summaries WHERE session_id = ?1 LIMIT 1",
                params![session_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()?;
        let audio = self
            .conn
            .query_row(
                "SELECT file_path, encrypted FROM audio_assets WHERE session_id = ?1 LIMIT 1",
                params![session_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
            )
            .optional()?;
        Ok(SessionDetail {
            session,
            transcript: transcript.map(|(_, text)| text),
            summary: summary.as_ref().map(|s| s.0.clone()),
            action_items: summary.as_ref().and_then(|s| s.1.clone()),
            key_points: summary.as_ref().and_then(|s| s.2.clone()),
            segments,
            audio_encrypted: audio.as_ref().map(|a| a.1 == 1).unwrap_or(false),
            audio_path: audio.map(|a| a.0),
        })
    }

    pub fn search(&self, q: &str, limit: i64) -> Result<Vec<SearchHit>> {
        let trimmed = q.trim();
        if trimmed.is_empty() {
            return Ok(vec![]);
        }
        let mut stmt = self.conn.prepare(
            "SELECT session_id, title, snippet(transcript_fts, 2, '«', '»', '…', 16)
             FROM transcript_fts WHERE transcript_fts MATCH ?1 LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![trimmed, limit], |row| {
            Ok(SearchHit {
                session_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn rename_session(&self, session_id: &str, title: &str) -> Result<Session> {
        self.conn.execute(
            "UPDATE sessions SET title = ?2 WHERE id = ?1",
            params![session_id, title],
        )?;
        self.get_session(session_id)
    }

    pub fn export_markdown(&self, session_id: &str) -> Result<String> {
        let detail = self.get_detail(session_id)?;
        let mut out = String::new();
        out.push_str(&format!("# {}\n\n", detail.session.title));
        out.push_str(&format!("Session: {}\n", detail.session.id));
        out.push_str(&format!("Status: {}\n", detail.session.status));
        if let Some(model) = &detail.session.model_id {
            out.push_str(&format!("Model: {model}\n"));
        }
        out.push_str("\n## Transcript\n\n");
        out.push_str(detail.transcript.as_deref().unwrap_or("_None._"));
        out.push_str("\n\n## Summary\n\n");
        out.push_str(detail.summary.as_deref().unwrap_or("_None._"));
        out.push_str("\n\n## Action items\n\n");
        out.push_str(detail.action_items.as_deref().unwrap_or("_None._"));
        out.push_str("\n");
        Ok(out)
    }

    pub fn audio_is_ciphertext(&self, session_id: &str) -> Result<bool> {
        let path: String = self.conn.query_row(
            "SELECT file_path FROM audio_assets WHERE session_id = ?1 LIMIT 1",
            params![session_id],
            |row| row.get(0),
        )?;
        let bytes = fs::read(self.data_dir.join(path))?;
        Ok(!crypto::looks_like_wav(&bytes))
    }

    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let paths: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT file_path FROM audio_assets WHERE session_id = ?1")?;
            let rows = stmt.query_map(params![session_id], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for rel in paths {
            let _ = fs::remove_file(self.data_dir.join(rel));
        }
        self.conn
            .execute("DELETE FROM transcript_fts WHERE session_id = ?1", params![session_id])?;
        self.conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
        Ok(())
    }

    pub fn delete_all(&self) -> Result<()> {
        let audio = self.data_dir.join(AUDIO_DIR);
        if audio.exists() {
            fs::remove_dir_all(&audio)?;
            fs::create_dir_all(&audio)?;
        }
        self.conn.execute_batch(
            "DELETE FROM transcript_fts;
             DELETE FROM transcript_segments;
             DELETE FROM transcripts;
             DELETE FROM summaries;
             DELETE FROM audio_assets;
             DELETE FROM sessions;",
        )?;
        Ok(())
    }
}

fn now_rfc3339() -> String {
    // Keep the core std-only. Precision is fine for v1.
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        created_at: row.get(1)?,
        started_at: row.get(2)?,
        ended_at: row.get(3)?,
        title: row.get(4)?,
        status: row.get(5)?,
        model_id: row.get(6)?,
        language: row.get(7)?,
        duration_seconds: row.get(8)?,
        consent_state: row.get(9)?,
        notes: row.get(10)?,
        source: row.get(11)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn defaults_are_private() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        assert_eq!(store.get_setting("telemetry").unwrap().unwrap(), "off");
        assert_eq!(store.get_setting("cloud_mode").unwrap().unwrap(), "off");
    }

    #[test]
    fn consent_blocks_record() {
        let dir = tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        let session = store.create_session(Some("Test".into()), "mixed").unwrap();
        let err = store.start_recording(&session.id).unwrap_err();
        assert_eq!(err.code(), "CONSENT_REQUIRED");
    }
}
