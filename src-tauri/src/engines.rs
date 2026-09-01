use serde::{Deserialize, Serialize};

use crate::error::{Result, SottoError};

pub const CATALOG_JSON: &str = include_str!("../../fixtures/models.json");
pub const FIXTURE_ENGINE_ID: &str = "fixture-replay";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineMode {
    Local,
    Cloud,
    Api,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallState {
    NotInstalled,
    Installing,
    Ready,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    pub id: String,
    pub vendor: String,
    pub name: String,
    pub version: String,
    pub mode: EngineMode,
    pub supported_languages: Vec<String>,
    pub supports_timestamps: bool,
    pub supports_streaming: bool,
    pub requires_gpu: bool,
    pub estimated_speed: String,
    pub estimated_accuracy: String,
    pub install_state: InstallState,
    /// True only when a compiled decoder and a runnable on-disk layout exist.
    /// Fixture-replay is never live-ready. A Parakeet checksum `.bin` is not live-ready.
    #[serde(default)]
    pub live_ready: bool,
    pub disk_size_mb: u32,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    engines: Vec<Engine>,
}

pub fn catalog() -> Result<Vec<Engine>> {
    let file: CatalogFile = serde_json::from_str(CATALOG_JSON)?;
    Ok(file.engines)
}

/// Resolve a transcription engine. Never silently switches to cloud/api,
/// and never silently substitutes fixture-replay for a not-ready local engine
/// unless `SOTTO_ALLOW_FIXTURE_FALLBACK=1` is set.
#[allow(dead_code)]
pub fn resolve_engine<'a>(
    requested: &str,
    cloud_enabled: bool,
    engines: &'a [Engine],
) -> Result<&'a Engine> {
    if let Some(chosen) = engines.iter().find(|e| e.id == requested) {
        if chosen.mode != EngineMode::Local && !cloud_enabled {
            return Err(SottoError::app(
                "CLOUD_DISABLED",
                format!(
                    "Engine {} is {} but cloud mode is off.",
                    chosen.id,
                    match chosen.mode {
                        EngineMode::Cloud => "cloud",
                        EngineMode::Api => "api",
                        EngineMode::Local => "local",
                    }
                ),
                true,
                "Pick a local engine, or explicitly enable cloud mode in Settings.",
            ));
        }
        if chosen.install_state == InstallState::Ready {
            return Ok(chosen);
        }
        // Catalog install_state is not the source of truth for Whisper weights
        // already on disk. Hand the engine to transcribe_local unless the
        // caller opted into fixture-replay fallback.
        if chosen.mode == EngineMode::Local && fixture_fallback_allowed() {
            // fall through to the ready local fixture
        } else if chosen.mode == EngineMode::Local {
            return Ok(chosen);
        }
    }

    engines
        .iter()
        .find(|e| e.mode == EngineMode::Local && e.install_state == InstallState::Ready)
        .ok_or_else(|| {
            SottoError::app(
                "NO_LOCAL_ENGINE",
                "No local transcription engine is ready.",
                true,
                "Install a local model in Settings. Sotto will not send audio to the cloud.",
            )
        })
}

#[allow(dead_code)]
fn fixture_fallback_allowed() -> bool {
    std::env::var(crate::stt::FIXTURE_FALLBACK_ENV)
        .ok()
        .as_deref()
        == Some("1")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptResult {
    pub raw_text: String,
    pub cleaned_text: String,
    pub language: String,
    pub segments: Vec<TranscriptSegment>,
    pub summary_text: String,
    pub action_items: String,
    pub key_points: String,
    pub engine_id: String,
}

pub fn fixture_transcript() -> TranscriptResult {
    let file = include_str!("../../fixtures/sessions/CONSULT-001.transcript.json");
    serde_json::from_str(file).expect("golden transcript must parse")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_no_cloud_default() {
        let engines = catalog().unwrap();
        assert!(engines.iter().any(|e| e.id == FIXTURE_ENGINE_ID));
        assert!(engines.iter().all(|e| e.mode == EngineMode::Local));
    }

    #[test]
    fn never_silent_cloud_fallback() {
        let engines = vec![
            Engine {
                id: "broken-local".into(),
                vendor: "x".into(),
                name: "Broken".into(),
                version: "1".into(),
                mode: EngineMode::Local,
                supported_languages: vec!["en".into()],
                supports_timestamps: true,
                supports_streaming: false,
                requires_gpu: false,
                estimated_speed: "slow".into(),
                estimated_accuracy: "n/a".into(),
                install_state: InstallState::Error,
                live_ready: false,
                disk_size_mb: 1,
                notes: String::new(),
            },
            Engine {
                id: "cloud-stt".into(),
                vendor: "x".into(),
                name: "Cloud".into(),
                version: "1".into(),
                mode: EngineMode::Cloud,
                supported_languages: vec!["en".into()],
                supports_timestamps: true,
                supports_streaming: true,
                requires_gpu: false,
                estimated_speed: "fast".into(),
                estimated_accuracy: "high".into(),
                install_state: InstallState::Ready,
                live_ready: false,
                disk_size_mb: 0,
                notes: String::new(),
            },
            Engine {
                id: FIXTURE_ENGINE_ID.into(),
                vendor: "sotto".into(),
                name: "Fixture".into(),
                version: "1".into(),
                mode: EngineMode::Local,
                supported_languages: vec!["en".into()],
                supports_timestamps: true,
                supports_streaming: false,
                requires_gpu: false,
                estimated_speed: "instant".into(),
                estimated_accuracy: "fixture".into(),
                install_state: InstallState::Ready,
                live_ready: false,
                disk_size_mb: 0,
                notes: String::new(),
            },
        ];
        let resolved = resolve_engine("cloud-stt", false, &engines).unwrap_err();
        assert_eq!(resolved.code(), "CLOUD_DISABLED");
        // Catalog not-installed is not a silent fixture fallback. The engine
        // is returned so transcribe_local can inspect local weights.
        let not_ready = resolve_engine("broken-local", false, &engines).unwrap();
        assert_eq!(not_ready.id, "broken-local");
        assert_eq!(not_ready.mode, EngineMode::Local);
        let ready = resolve_engine(FIXTURE_ENGINE_ID, false, &engines).unwrap();
        assert_eq!(ready.id, FIXTURE_ENGINE_ID);
        assert_eq!(ready.mode, EngineMode::Local);
    }
}
