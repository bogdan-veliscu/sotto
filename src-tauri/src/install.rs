use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::engines::{Engine, InstallState};
use crate::error::{Result, SottoError};
use crate::stt::{whisper_weights_path, WHISPER_ENGINE_ID};

pub const PARAKEET_ENGINE_ID: &str = "parakeet-tdt-0.6b-v3";

const MODELS_DIR: &str = "models";
const PARAKEET_FILE: &str = "parakeet-tdt-0.6b-v3.bin";

#[derive(Debug, Clone, Serialize)]
pub struct InstallResult {
    pub engine_id: String,
    pub bytes_written: u64,
    pub sha256: String,
}

/// Local filesystem location for the Parakeet weights.
///
/// Always under the caller's cache/data directory. Never a URL; callers must
/// never fetch this over the network.
pub fn parakeet_weights_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MODELS_DIR).join(PARAKEET_FILE)
}

/// Resolve the on-disk weights path for an installable engine id.
fn weights_path_for(engine_id: &str, cache_dir: &Path) -> Option<PathBuf> {
    match engine_id {
        PARAKEET_ENGINE_ID => Some(parakeet_weights_path(cache_dir)),
        WHISPER_ENGINE_ID => Some(whisper_weights_path(cache_dir)),
        _ => None,
    }
}

fn lowercase_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Install model weights from bytes already in memory. Verifies the SHA-256
/// before anything lands at the engine path.
///
/// This function performs **no** network I/O. Bytes are provided by the
/// caller (a user-initiated action in the desk). On a checksum mismatch it
/// leaves no weights file on disk. Writes go to a sibling temp file and are
/// renamed into place only after the digest matches, so a crash never leaves
/// a truncated "ready" model.
pub fn install_bytes(
    engine_id: &str,
    cache_dir: &Path,
    bytes: &[u8],
    expected_sha256: &str,
) -> Result<InstallResult> {
    let dest = weights_path_for(engine_id, cache_dir).ok_or_else(|| {
        SottoError::app(
            "ENGINE_UNKNOWN",
            format!("No installable engine with id {engine_id}."),
            true,
            "Choose an installable engine from the catalog.",
        )
    })?;

    let actual = lowercase_hex(&Sha256::digest(bytes));
    let expected = expected_sha256.trim().to_ascii_lowercase();

    if actual != expected {
        // Never keep a file that failed checksum. Nothing was written yet,
        // but be defensive about any stale temp sibling.
        let tmp = temp_sibling(&dest);
        let _ = fs::remove_file(&tmp);
        return Err(SottoError::app(
            "CHECKSUM_MISMATCH",
            "Model bytes do not match the expected SHA-256.",
            true,
            "Provide the pinned weights file. Nothing was installed.",
        ));
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write via a sibling temp file then rename so a crash mid-write does not
    // leave a truncated model at the ready path.
    let tmp = temp_sibling(&dest);
    fs::write(&tmp, bytes)?;
    if let Err(err) = fs::rename(&tmp, &dest) {
        let _ = fs::remove_file(&tmp);
        return Err(err.into());
    }

    Ok(InstallResult {
        engine_id: engine_id.to_string(),
        bytes_written: bytes.len() as u64,
        sha256: actual,
    })
}

fn temp_sibling(dest: &Path) -> PathBuf {
    let mut file_name = dest
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    file_name.push(".tmp");
    match dest.parent() {
        Some(parent) => parent.join(file_name),
        None => PathBuf::from(file_name),
    }
}

/// Remove installed weights for an engine. Overlay then reports the engine as
/// not-installed. Does not touch other engines and never downloads anything.
pub fn delete_model(engine_id: &str, cache_dir: &Path) -> Result<()> {
    let dest = weights_path_for(engine_id, cache_dir).ok_or_else(|| {
        SottoError::app(
            "ENGINE_UNKNOWN",
            format!("No installable engine with id {engine_id}."),
            true,
            "Choose an installable engine from the catalog.",
        )
    })?;
    match fs::remove_file(&dest) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// True when checksum-valid weights are present on disk for an engine.
pub fn is_installed(engine_id: &str, cache_dir: &Path) -> bool {
    weights_path_for(engine_id, cache_dir)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Reconcile the frozen catalog with what is actually on disk.
///
/// When installable weights exist locally, mark that engine `ready`. When they
/// do not, keep the catalog's value. Settings must stop lying once a file is on
/// disk. No network I/O.
pub fn overlay_catalog(engines: Vec<Engine>, cache_dir: &Path) -> Vec<Engine> {
    engines
        .into_iter()
        .map(|mut engine| {
            if weights_path_for(&engine.id, cache_dir).is_some() {
                engine.install_state = if is_installed(&engine.id, cache_dir) {
                    InstallState::Ready
                } else {
                    engine.install_state
                };
            }
            engine
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const BLOB: &[u8] = b"parakeet-test-blob";
    const BLOB_SHA256: &str = "0b73fc4fa437d2d3c146f9aa3dbf7f3b538e130ba3d0aa69668a0cc8995729b9";

    #[test]
    fn mismatch_leaves_no_file() {
        let dir = tempdir().unwrap();
        let err = install_bytes(PARAKEET_ENGINE_ID, dir.path(), BLOB, "deadbeef").unwrap_err();
        assert_eq!(err.code(), "CHECKSUM_MISMATCH");
        assert!(!parakeet_weights_path(dir.path()).exists());
    }

    #[test]
    fn match_installs_and_deletes() {
        let dir = tempdir().unwrap();
        let result = install_bytes(PARAKEET_ENGINE_ID, dir.path(), BLOB, BLOB_SHA256).unwrap();
        assert_eq!(result.sha256, BLOB_SHA256);
        assert_eq!(result.bytes_written, BLOB.len() as u64);
        assert!(parakeet_weights_path(dir.path()).exists());

        delete_model(PARAKEET_ENGINE_ID, dir.path()).unwrap();
        assert!(!parakeet_weights_path(dir.path()).exists());
    }

    #[test]
    fn delete_missing_is_ok() {
        let dir = tempdir().unwrap();
        delete_model(PARAKEET_ENGINE_ID, dir.path()).unwrap();
    }

    #[test]
    fn overlay_marks_whisper_ready_from_disk() {
        let dir = tempdir().unwrap();
        let path = whisper_weights_path(dir.path());
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"ggml").unwrap();
        let engines = overlay_catalog(
            vec![Engine {
                id: WHISPER_ENGINE_ID.into(),
                vendor: "x".into(),
                name: "Whisper".into(),
                version: "1".into(),
                mode: crate::engines::EngineMode::Local,
                supported_languages: vec!["en".into()],
                supports_timestamps: true,
                supports_streaming: false,
                requires_gpu: false,
                estimated_speed: "n/a".into(),
                estimated_accuracy: "n/a".into(),
                install_state: InstallState::NotInstalled,
                disk_size_mb: 1,
                notes: String::new(),
            }],
            dir.path(),
        );
        assert_eq!(engines[0].install_state, InstallState::Ready);
    }
}
