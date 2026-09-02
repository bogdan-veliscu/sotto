use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use uuid::Uuid;

use crate::crypto::{self, KEY_LEN};
use crate::engines::{self, Engine, TranscriptResult, FIXTURE_ENGINE_ID};
use crate::error::{Result, SottoError};

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
    pub tags: Vec<String>,
}

pub struct Store {
    conn: Connection,
    data_dir: PathBuf,
    master_key: [u8; KEY_LEN],
    key_backend: String,
}

#[cfg_attr(not(feature = "desktop"), allow(dead_code))]
impl Store {
    pub fn open(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir)?;
        fs::create_dir_all(data_dir.join(AUDIO_DIR))?;
        let (master_key, key_backend) = crate::keys::load_or_create(data_dir)?;

        let conn = Connection::open(data_dir.join(DB_FILE))?;
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        let store = Self {
            conn,
            data_dir: data_dir.to_path_buf(),
            master_key,
            key_backend: key_backend.to_string(),
        };
        store.init_schema()?;
        store.ensure_defaults()?;
        let _ = store.apply_retention();
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
            CREATE TABLE IF NOT EXISTS session_tags (
                session_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (session_id, tag),
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
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
        Ok(crate::install::overlay_catalog(
            engines::catalog()?,
            &self.data_dir,
        ))
    }

    pub fn install_model_bytes(
        &self,
        engine_id: &str,
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<crate::install::InstallResult> {
        crate::install::install_bytes(engine_id, &self.data_dir, bytes, expected_sha256)
    }

    pub fn delete_installed_model(&self, engine_id: &str) -> Result<()> {
        crate::install::delete_model(engine_id, &self.data_dir)
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

    pub fn live_dir(&self, session_id: &str) -> PathBuf {
        self.data_dir.join("live").join(session_id)
    }

    pub fn finalize_with_wav(&self, session_id: &str, wav: &[u8]) -> Result<Session> {
        self.finalize_capture(session_id, wav, 1)
    }

    pub fn finalize_live(
        &self,
        session_id: &str,
        live: crate::capture::LiveSession,
    ) -> Result<Session> {
        let captured = live.finish()?;
        let secs = i64::try_from((captured.duration_ms + 500) / 1000).unwrap_or(0);
        let session = self.finalize_capture(session_id, &captured.wav, secs)?;
        let _ = fs::remove_dir_all(self.live_dir(session_id));
        Ok(session)
    }

    fn finalize_capture(
        &self,
        session_id: &str,
        wav: &[u8],
        duration_seconds: i64,
    ) -> Result<Session> {
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
            "UPDATE sessions SET status = 'recorded', ended_at = ?2, duration_seconds = ?3 WHERE id = ?1",
            params![session_id, now, duration_seconds],
        )?;
        self.get_session(session_id)
    }

    pub fn import_model_path(
        &self,
        engine_id: &str,
        source: &Path,
    ) -> Result<crate::install::InstallResult> {
        crate::install::import_local(engine_id, &self.data_dir, source)
    }

    #[allow(dead_code)]
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    pub fn transcribe(
        &self,
        session_id: &str,
        requested_model: Option<String>,
    ) -> Result<SessionDetail> {
        let job = self.prepare_transcribe(session_id, requested_model)?;
        let result = crate::stt::transcribe_job(job)?;
        self.commit_transcript(session_id, &result)
    }

    /// Resolve engine and decrypt WAV. Does not run inference. Callers that
    /// hold `Mutex<Store>` must drop the guard before `transcribe_job`.
    pub fn prepare_transcribe(
        &self,
        session_id: &str,
        requested_model: Option<String>,
    ) -> Result<crate::stt::TranscribeJob> {
        let wav = self.read_session_wav(session_id)?;
        let requested = requested_model
            .or(self.get_setting("default_model")?)
            .unwrap_or_else(|| FIXTURE_ENGINE_ID.to_string());

        if requested == FIXTURE_ENGINE_ID {
            if crate::stt::is_golden_wav(&wav) {
                return Ok(crate::stt::TranscribeJob {
                    engine_id: FIXTURE_ENGINE_ID.to_string(),
                    wav,
                    cache_dir: self.data_dir.clone(),
                });
            }
            return Err(engine_setup_required());
        }

        let cloud = self.cloud_enabled()?;
        let catalog = self.list_engines()?;
        let engine = catalog.iter().find(|e| e.id == requested).ok_or_else(|| {
            SottoError::app(
                "ENGINE_UNKNOWN",
                format!("No transcription engine with id {requested}."),
                true,
                "Choose an engine from the catalog.",
            )
        })?;
        if engine.mode != engines::EngineMode::Local && !cloud {
            return Err(SottoError::app(
                "CLOUD_DISABLED",
                format!("Engine {} is not local and cloud mode is off.", engine.id),
                true,
                "Pick a local engine, or explicitly enable cloud mode in Settings.",
            ));
        }
        if !engine.live_ready {
            return Err(engine_setup_required());
        }
        Ok(crate::stt::TranscribeJob {
            engine_id: engine.id.clone(),
            wav,
            cache_dir: self.data_dir.clone(),
        })
    }

    /// Persist a worker transcript and return session detail.
    pub fn commit_transcript(
        &self,
        session_id: &str,
        result: &TranscriptResult,
    ) -> Result<SessionDetail> {
        self.persist_transcript(session_id, result.engine_id.as_str(), result)?;
        self.get_detail(session_id)
    }

    /// Decrypt the session's stored audio to plaintext WAV bytes for
    /// on-device transcription. Returns empty bytes when no audio asset
    /// exists. fixture-replay requires the golden WAV.
    fn read_session_wav(&self, session_id: &str) -> Result<Vec<u8>> {
        let rel: Option<String> = self
            .conn
            .query_row(
                "SELECT file_path FROM audio_assets WHERE session_id = ?1 LIMIT 1",
                params![session_id],
                |row| row.get(0),
            )
            .optional()?;
        match rel {
            Some(rel) => {
                let packed = fs::read(self.data_dir.join(rel))?;
                crypto::decrypt(&self.master_key, &packed)
            }
            None => Ok(Vec::new()),
        }
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
        let notes = if result.summary_text.trim().is_empty() {
            let src = if result.cleaned_text.trim().is_empty() {
                result.raw_text.as_str()
            } else {
                result.cleaned_text.as_str()
            };
            crate::notes::extract_notes(src)?
        } else {
            crate::notes::Notes {
                summary: result.summary_text.clone(),
                action_items: result.action_items.clone(),
                key_points: result.key_points.clone(),
            }
        };
        let summary_id = format!("SUM-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
        self.conn.execute(
            "INSERT INTO summaries(id, session_id, summary_text, action_items, key_points, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                summary_id,
                session_id,
                &notes.summary,
                &notes.action_items,
                &notes.key_points,
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
            params![session_id, title, result.raw_text, &notes.summary],
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
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
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
            tags: self.list_tags(session_id)?,
        })
    }

    pub fn search(&self, q: &str, limit: i64) -> Result<Vec<SearchHit>> {
        self.search_filtered(
            &crate::search::SearchFilter {
                q: q.to_string(),
                ..Default::default()
            },
            limit,
        )
    }

    pub fn search_filtered(
        &self,
        filter: &crate::search::SearchFilter,
        limit: i64,
    ) -> Result<Vec<SearchHit>> {
        let q = filter.q.trim().to_string();
        let title = filter
            .title
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let from = filter
            .created_from
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let to = filter
            .created_to
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let tag = filter.tag.as_deref().and_then(crate::search::normalize_tag);
        let has_filter = title.is_some() || from.is_some() || to.is_some() || tag.is_some();
        if q.is_empty() && !has_filter {
            return Ok(vec![]);
        }

        let mut sql = if q.is_empty() {
            String::from("SELECT s.id, s.title, '' FROM sessions s WHERE 1=1")
        } else {
            String::from(
                "SELECT f.session_id, f.title, snippet(transcript_fts, 2, '«', '»', '…', 16)
                 FROM transcript_fts f
                 JOIN sessions s ON s.id = f.session_id
                 WHERE transcript_fts MATCH ?1",
            )
        };
        let mut binds: Vec<String> = Vec::new();
        if !q.is_empty() {
            binds.push(q);
        }
        if let Some(title) = title {
            sql.push_str(" AND LOWER(s.title) LIKE '%' || LOWER(?) || '%'");
            binds.push(title);
        }
        if let Some(from) = from {
            sql.push_str(" AND CAST(s.created_at AS INTEGER) >= CAST(? AS INTEGER)");
            binds.push(from);
        }
        if let Some(to) = to {
            sql.push_str(" AND CAST(s.created_at AS INTEGER) <= CAST(? AS INTEGER)");
            binds.push(to);
        }
        if let Some(tag) = tag {
            sql.push_str(
                " AND EXISTS (SELECT 1 FROM session_tags t WHERE t.session_id = s.id AND t.tag = ?)",
            );
            binds.push(tag);
        }
        sql.push_str(" LIMIT ?");
        binds.push(limit.to_string());

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(binds.iter()), |row| {
            Ok(SearchHit {
                session_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn set_tags(&self, session_id: &str, tags: &[String]) -> Result<Vec<String>> {
        let _ = self.get_session(session_id)?;
        let normalized = crate::search::normalize_tags(tags);
        self.conn.execute(
            "DELETE FROM session_tags WHERE session_id = ?1",
            params![session_id],
        )?;
        for tag in &normalized {
            self.conn.execute(
                "INSERT INTO session_tags(session_id, tag) VALUES (?1, ?2)",
                params![session_id, tag],
            )?;
        }
        Ok(normalized)
    }

    pub fn list_tags(&self, session_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM session_tags WHERE session_id = ?1 ORDER BY tag")?;
        let rows = stmt.query_map(params![session_id], |row| row.get(0))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    pub fn set_created_at(&self, session_id: &str, created_at: &str) -> Result<()> {
        let n = self.conn.execute(
            "UPDATE sessions SET created_at = ?2 WHERE id = ?1",
            params![session_id, created_at],
        )?;
        if n == 0 {
            return Err(SottoError::app(
                "SESSION_MISSING",
                "That session does not exist.",
                false,
                "Refresh the list.",
            ));
        }
        Ok(())
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

    pub fn export_markdown_file(&self, session_id: &str, dest: &Path) -> Result<()> {
        crate::notes::reject_remote_dest(dest)?;
        let body = self.export_markdown(session_id)?;
        if let Some(parent) = dest.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(dest, body)?;
        Ok(())
    }

    pub fn privacy_settings(&self) -> Result<crate::notes::PrivacySettings> {
        Ok(crate::notes::PrivacySettings {
            telemetry: self
                .get_setting("telemetry")?
                .unwrap_or_else(|| "off".into()),
            cloud_mode: self
                .get_setting("cloud_mode")?
                .unwrap_or_else(|| "off".into()),
            retention_days: self
                .get_setting("retention_days")?
                .unwrap_or_else(|| "0".into()),
        })
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
        self.conn.execute(
            "DELETE FROM transcript_fts WHERE session_id = ?1",
            params![session_id],
        )?;
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
             DELETE FROM session_tags;
             DELETE FROM audio_assets;
             DELETE FROM sessions;",
        )?;
        Ok(())
    }

    pub fn key_report(&self) -> Result<crate::keys::KeyReport> {
        Ok(crate::keys::KeyReport {
            backend: self.key_backend.clone(),
            key_len: KEY_LEN,
            fingerprint: crate::keys::fingerprint(&self.master_key),
        })
    }

    pub fn apply_retention(&self) -> Result<u32> {
        let days = self
            .get_setting("retention_days")?
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);
        if days == 0 {
            return Ok(0);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let cutoff = now.saturating_sub(days.saturating_mul(86_400));
        let ids: Vec<String> = {
            let mut stmt = self
                .conn
                .prepare("SELECT id FROM sessions WHERE CAST(created_at AS INTEGER) < ?1")?;
            let rows = stmt.query_map(params![cutoff as i64], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let n = ids.len() as u32;
        for id in ids {
            self.delete_session(&id)?;
        }
        Ok(n)
    }

    pub fn scrub_plaintext_temps(&self) -> Result<u32> {
        let audio = self.data_dir.join(AUDIO_DIR);
        if !audio.exists() {
            return Ok(0);
        }
        let mut removed = 0u32;
        for entry in fs::read_dir(&audio)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let looks_temp = name.ends_with(".wav") || name.ends_with(".tmp");
            let bytes = fs::read(&path).unwrap_or_default();
            if looks_temp || crypto::looks_like_wav(&bytes) {
                fs::remove_file(&path)?;
                removed += 1;
            }
        }
        Ok(removed)
    }
}

fn engine_setup_required() -> SottoError {
    SottoError::app(
        "ENGINE_SETUP_REQUIRED",
        "This recording is encrypted on this Mac. Use Apple on-device Speech, or install Parakeet / Whisper, then transcribe it.",
        true,
        "Fixture replay is only for make demo. Open Models: Apple Speech needs no download; Parakeet can be downloaded or imported.",
    )
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
