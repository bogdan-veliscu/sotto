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

/// Local filesystem location for the Parakeet checksum blob (install tests).
///
/// Always under the caller's cache/data directory. Never a URL; callers must
/// never fetch this over the network.
pub fn parakeet_weights_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MODELS_DIR).join(PARAKEET_FILE)
}

/// Directory that holds a Parakeet TDT ONNX export
/// (`encoder-model.onnx`, `decoder_joint-model.onnx`, `vocab.txt`).
pub fn parakeet_model_dir(cache_dir: &Path) -> PathBuf {
    cache_dir.join(MODELS_DIR).join("parakeet-tdt-0.6b-v3")
}

/// True when `dir` looks like a Parakeet TDT 0.6B v3 ONNX layout.
pub fn parakeet_tdt_layout_ok(dir: &Path) -> bool {
    dir.is_dir()
        && dir.join("encoder-model.onnx").is_file()
        && dir.join("decoder_joint-model.onnx").is_file()
        && dir.join("vocab.txt").is_file()
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
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    if engine_id == PARAKEET_ENGINE_ID {
        let dir = parakeet_model_dir(cache_dir);
        match fs::remove_dir_all(&dir) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}

/// True when checksum-valid weights are present on disk for an engine.
pub fn is_installed(engine_id: &str, cache_dir: &Path) -> bool {
    if engine_id == PARAKEET_ENGINE_ID && parakeet_tdt_layout_ok(&parakeet_model_dir(cache_dir)) {
        return true;
    }
    weights_path_for(engine_id, cache_dir)
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// Live transcription requires a compiled decoder and a real model layout.
/// Fixture-replay is never live-ready. A Parakeet checksum `.bin` is not.
pub fn is_live_runnable(engine_id: &str, cache_dir: &Path) -> bool {
    if engine_id == WHISPER_ENGINE_ID {
        return cfg!(feature = "whisper")
            && crate::stt::whisper_layout_ok(&whisper_weights_path(cache_dir));
    }
    if engine_id == PARAKEET_ENGINE_ID {
        return cfg!(feature = "parakeet")
            && parakeet_tdt_layout_ok(&parakeet_model_dir(cache_dir));
    }
    false
}

const PARAKEET_TDT_FILES: [&str; 3] = [
    "encoder-model.onnx",
    "decoder_joint-model.onnx",
    "vocab.txt",
];

fn path_looks_remote(path: &Path) -> bool {
    crate::stt::looks_like_url(path)
}

fn import_rejected(message: impl Into<String>, hint: impl Into<String>) -> SottoError {
    SottoError::app("ENGINE_MODEL_INVALID", message, true, hint)
}

/// Copy a user-selected local Whisper file or Parakeet TDT directory into
/// `models/`. Validates layout on a staging path, then activates atomically.
/// Never fetches a URL. Failed imports delete staging and leave the previous
/// runnable model untouched.
pub fn import_local(engine_id: &str, cache_dir: &Path, source: &Path) -> Result<InstallResult> {
    if path_looks_remote(source) {
        return Err(import_rejected(
            "Model import is local files only.",
            "Choose a file or folder on this Mac. Sotto will not download weights.",
        ));
    }
    match engine_id {
        WHISPER_ENGINE_ID => import_whisper_file(cache_dir, source),
        PARAKEET_ENGINE_ID => import_parakeet_dir(cache_dir, source),
        _ => Err(SottoError::app(
            "ENGINE_UNKNOWN",
            format!("No importable engine with id {engine_id}."),
            true,
            "Import Whisper as a ggml file or Parakeet as a TDT directory.",
        )),
    }
}

fn import_whisper_file(cache_dir: &Path, source: &Path) -> Result<InstallResult> {
    if !source.is_file() {
        return Err(import_rejected(
            "Whisper import needs a local ggml/gguf file.",
            "Pick ggml-large-v3-turbo.bin on this Mac.",
        ));
    }
    let dest = whisper_weights_path(cache_dir);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = temp_sibling(&dest);
    let _ = fs::remove_file(&tmp);
    fs::copy(source, &tmp)?;
    if !crate::stt::whisper_layout_ok(&tmp) {
        let _ = fs::remove_file(&tmp);
        return Err(import_rejected(
            "That file is not a valid local Whisper model.",
            "Use a ggml or GGUF Whisper file. The previous model was not changed.",
        ));
    }
    if let Err(err) = fs::rename(&tmp, &dest) {
        let _ = fs::remove_file(&tmp);
        return Err(err.into());
    }
    let bytes = fs::metadata(&dest)?.len();
    let digest = sha256_file(&dest)?;
    Ok(InstallResult {
        engine_id: WHISPER_ENGINE_ID.to_string(),
        bytes_written: bytes,
        sha256: digest,
    })
}

fn import_parakeet_dir(cache_dir: &Path, source: &Path) -> Result<InstallResult> {
    if !parakeet_tdt_layout_ok(source) {
        return Err(import_rejected(
            "Parakeet import needs a TDT directory.",
            "Select the folder that contains encoder-model.onnx, decoder_joint-model.onnx, and vocab.txt.",
        ));
    }
    let dest = parakeet_model_dir(cache_dir);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let staging = sibling_with_suffix(&dest, ".importing");
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging)?;
    for name in PARAKEET_TDT_FILES {
        fs::copy(source.join(name), staging.join(name))?;
    }
    if !parakeet_tdt_layout_ok(&staging) {
        let _ = fs::remove_dir_all(&staging);
        return Err(import_rejected(
            "Parakeet staging layout was incomplete.",
            "The previous TDT directory was not changed.",
        ));
    }
    let backup = sibling_with_suffix(&dest, ".bak");
    let _ = fs::remove_dir_all(&backup);
    let had_dest = dest.exists();
    if had_dest {
        fs::rename(&dest, &backup)?;
    }
    if let Err(err) = fs::rename(&staging, &dest) {
        if had_dest {
            let _ = fs::rename(&backup, &dest);
        }
        let _ = fs::remove_dir_all(&staging);
        return Err(err.into());
    }
    let _ = fs::remove_dir_all(&backup);
    let mut bytes = 0u64;
    let mut hasher = Sha256::new();
    for name in PARAKEET_TDT_FILES {
        let data = fs::read(dest.join(name))?;
        bytes += data.len() as u64;
        hasher.update(&data);
    }
    Ok(InstallResult {
        engine_id: PARAKEET_ENGINE_ID.to_string(),
        bytes_written: bytes,
        sha256: lowercase_hex(&hasher.finalize()),
    })
}

fn sha256_file(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    Ok(lowercase_hex(&Sha256::digest(&data)))
}

fn sibling_with_suffix(dest: &Path, suffix: &str) -> PathBuf {
    let mut name = dest
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(suffix);
    match dest.parent() {
        Some(parent) => parent.join(name),
        None => PathBuf::from(name),
    }
}

/// Reconcile the frozen catalog with what is actually on disk.
///
/// When installable weights exist locally, mark that engine `ready`. When they
/// do not, keep the catalog's value. Settings must stop lying once a file is on
/// disk. No network I/O.
///
/// `live_ready` is stricter: decoder compiled + runnable layout. A checksum
/// blob can still be `install_state=ready` for the install contract without
/// being live-ready.
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
            engine.live_ready = is_live_runnable(&engine.id, cache_dir);
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
                live_ready: false,
                disk_size_mb: 1,
                notes: String::new(),
            }],
            dir.path(),
        );
        assert_eq!(engines[0].install_state, InstallState::Ready);
    }

    #[test]
    fn overlay_and_delete_tdt_dir() {
        let dir = tempdir().unwrap();
        let tdt = parakeet_model_dir(dir.path());
        fs::create_dir_all(&tdt).unwrap();
        fs::write(tdt.join("encoder-model.onnx"), b"x").unwrap();
        fs::write(tdt.join("decoder_joint-model.onnx"), b"x").unwrap();
        fs::write(tdt.join("vocab.txt"), b"x").unwrap();
        assert!(is_installed(PARAKEET_ENGINE_ID, dir.path()));

        let engines = overlay_catalog(
            vec![Engine {
                id: PARAKEET_ENGINE_ID.into(),
                vendor: "x".into(),
                name: "Parakeet".into(),
                version: "1".into(),
                mode: crate::engines::EngineMode::Local,
                supported_languages: vec!["en".into()],
                supports_timestamps: true,
                supports_streaming: false,
                requires_gpu: false,
                estimated_speed: "n/a".into(),
                estimated_accuracy: "n/a".into(),
                install_state: InstallState::NotInstalled,
                live_ready: false,
                disk_size_mb: 1,
                notes: String::new(),
            }],
            dir.path(),
        );
        assert_eq!(engines[0].install_state, InstallState::Ready);

        delete_model(PARAKEET_ENGINE_ID, dir.path()).unwrap();
        assert!(!is_installed(PARAKEET_ENGINE_ID, dir.path()));
        assert!(!tdt.exists());
    }
}
