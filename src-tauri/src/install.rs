use std::path::{Path, PathBuf};

use crate::engines::Engine;
use crate::error::{Result, SottoError};

pub const PARAKEET_ENGINE_ID: &str = "parakeet-tdt-0.6b-v3";

#[derive(Debug, Clone)]
pub struct InstallResult {
    pub engine_id: String,
    pub bytes_written: u64,
    pub sha256: String,
}

pub fn parakeet_weights_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("models").join("parakeet-tdt-0.6b-v3.bin")
}

pub fn install_bytes(
    engine_id: &str,
    _cache_dir: &Path,
    _bytes: &[u8],
    _expected_sha256: &str,
) -> Result<InstallResult> {
    let _ = engine_id;
    Err(SottoError::app(
        "NOT_IMPLEMENTED",
        "model install is not implemented in this wave",
        true,
        "Wait for model-install GREEN.",
    ))
}

pub fn delete_model(engine_id: &str, _cache_dir: &Path) -> Result<()> {
    let _ = engine_id;
    Err(SottoError::app(
        "NOT_IMPLEMENTED",
        "model delete is not implemented in this wave",
        true,
        "Wait for model-install GREEN.",
    ))
}

pub fn overlay_catalog(engines: Vec<Engine>, _cache_dir: &Path) -> Vec<Engine> {
    engines
}
