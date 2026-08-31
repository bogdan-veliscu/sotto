use std::path::{Path, PathBuf};

use crate::engines::TranscriptResult;
use crate::error::{Result, SottoError};

pub const WHISPER_ENGINE_ID: &str = "whisper-large-v3-turbo";
pub const FIXTURE_FALLBACK_ENV: &str = "SOTTO_ALLOW_FIXTURE_FALLBACK";

pub fn whisper_weights_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("models").join("ggml-large-v3-turbo.bin")
}

/// Batch-transcribe WAV bytes with a local engine. Never downloads.
pub fn transcribe_local(
    engine_id: &str,
    _wav: &[u8],
    _cache_dir: &Path,
) -> Result<TranscriptResult> {
    let _ = engine_id;
    Err(SottoError::app(
        "NOT_IMPLEMENTED",
        "local STT is not implemented in this wave",
        true,
        "Wait for local-stt GREEN.",
    ))
}
