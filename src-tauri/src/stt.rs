use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::engines::{self, EngineMode, TranscriptResult, FIXTURE_ENGINE_ID};
use crate::error::{Result, SottoError};

pub const WHISPER_ENGINE_ID: &str = "whisper-large-v3-turbo";
pub const FIXTURE_FALLBACK_ENV: &str = "SOTTO_ALLOW_FIXTURE_FALLBACK";
/// Magic-only stubs are not live-ready. Real ggml-large-v3-turbo is ~800 MiB.
pub const WHISPER_MIN_LIVE_BYTES: u64 = 1_048_576;
const GOLDEN_WAV: &[u8] = include_bytes!("../../fixtures/sessions/CONSULT-001.wav");

/// Work for on-device inference with no Store attached.
///
/// Callers must drop any `Mutex<Store>` guard before running
/// [`transcribe_job`]. The job is `Send` so the desktop command can
/// `spawn_blocking`.
#[derive(Debug, Clone)]
pub struct TranscribeJob {
    pub engine_id: String,
    pub wav: Vec<u8>,
    pub cache_dir: PathBuf,
}

/// Run [`transcribe_local`] without holding the Store mutex.
pub fn transcribe_job(job: TranscribeJob) -> Result<TranscriptResult> {
    transcribe_local(&job.engine_id, &job.wav, &job.cache_dir)
}

/// Report whether the on-device Parakeet TDT decoder is compiled into this
/// binary.
///
/// - `"not-built"` — no decoder that can produce a transcript is compiled in
///   (Linux CI, `--no-default-features`).
/// - `"ready"` — `parakeet-rs` inference is compiled in. Weights may still be
///   absent (`ENGINE_NOT_INSTALLED`) or a dummy blob (`ENGINE_MODEL_INVALID`).
pub fn parakeet_runtime_status() -> &'static str {
    if cfg!(feature = "parakeet") {
        "ready"
    } else {
        "not-built"
    }
}

/// Local filesystem location for the Whisper ggml weights.
///
/// Always under the caller's cache/data directory. This function never
/// resolves a URL and callers must never fetch it over the network.
pub fn whisper_weights_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("models").join("ggml-large-v3-turbo.bin")
}

/// Returns true when fixture-replay fallback is explicitly opted into via env.
pub fn fixture_fallback_allowed() -> bool {
    std::env::var(FIXTURE_FALLBACK_ENV).ok().as_deref() == Some("1")
}

/// True when `wav` is the locked CONSULT-001 fixture capture.
pub fn is_golden_wav(wav: &[u8]) -> bool {
    wav == GOLDEN_WAV
}

/// True if the given path string points at a remote resource. Weights are
/// local files only; a URL-shaped path is always rejected.
pub(crate) fn looks_like_url(path: &Path) -> bool {
    let s = path.to_string_lossy().to_ascii_lowercase();
    s.starts_with("http://") || s.starts_with("https://")
}

/// Minimal, offline validity check for a ggml/gguf Whisper model file.
///
/// We only consult bytes already on disk — never the network. A real weights
/// file begins with a known magic value ("ggml", "ggjt", "ggla", "ggmf" or
/// the newer "GGUF"). Anything else is treated as an invalid local model.
fn is_valid_ggml(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    matches!(&bytes[..4], b"ggml" | b"ggjt" | b"ggla" | b"ggmf" | b"GGUF")
}

fn file_is_valid_ggml(path: &Path) -> Result<bool> {
    let mut file = File::open(path)?;
    let mut magic = [0u8; 4];
    let n = file.read(&mut magic)?;
    Ok(is_valid_ggml(&magic[..n]))
}

/// True when `path` is a local ggml/gguf Whisper file. Never fetches a URL.
pub fn whisper_layout_ok(path: &Path) -> bool {
    if looks_like_url(path) || !path.is_file() {
        return false;
    }
    file_is_valid_ggml(path).unwrap_or(false)
}

/// Live-ready Whisper needs a valid magic **and** more than a truncated stub.
pub fn whisper_live_layout_ok(path: &Path) -> bool {
    if !whisper_layout_ok(path) {
        return false;
    }
    std::fs::metadata(path)
        .map(|meta| meta.len() >= WHISPER_MIN_LIVE_BYTES)
        .unwrap_or(false)
}

fn not_installed() -> SottoError {
    SottoError::app(
        "ENGINE_NOT_INSTALLED",
        "Whisper Large-v3 Turbo weights are not installed on this Mac.",
        true,
        "Install the local model file. Sotto will not download it or use the cloud.",
    )
}

fn model_invalid() -> SottoError {
    SottoError::app(
        "ENGINE_MODEL_INVALID",
        "The Whisper weights file on disk is not a valid local model.",
        true,
        "Replace it with a valid local ggml Whisper model. Sotto will not fetch it.",
    )
}

/// Batch-transcribe WAV bytes with a local engine. Never downloads, never
/// selects a cloud engine silently.
///
/// - `fixture-replay` → the golden fixture transcript.
/// - `whisper-large-v3-turbo` → local weights only. Missing →
///   `ENGINE_NOT_INSTALLED`. Present but invalid → `ENGINE_MODEL_INVALID`.
///   Present and valid → on-device transcription when the `whisper` feature
///   is compiled in.
/// - any cloud/api engine → `CLOUD_DISABLED` (cloud policy lives in
///   `resolve_engine`; this is a defensive guard).
pub fn transcribe_local(engine_id: &str, wav: &[u8], cache_dir: &Path) -> Result<TranscriptResult> {
    if engine_id == FIXTURE_ENGINE_ID {
        if !is_golden_wav(wav) {
            return Err(SottoError::app(
                "FIXTURE_AUDIO_MISMATCH",
                "fixture-replay only transcribes the golden CONSULT-001 capture.",
                true,
                "Install a local Whisper or Parakeet model to transcribe this recording. Audio stays encrypted on this Mac.",
            ));
        }
        return Ok(engines::fixture_transcript());
    }

    if engine_id == WHISPER_ENGINE_ID {
        return transcribe_whisper(wav, cache_dir);
    }

    if engine_id == crate::install::PARAKEET_ENGINE_ID {
        return transcribe_parakeet(wav, cache_dir);
    }

    if engine_id == crate::stt_apple::APPLE_SPEECH_ENGINE_ID {
        return crate::stt_apple::transcribe(wav, cache_dir);
    }

    // Non-fixture, non-whisper: consult the catalog. Cloud/api engines are
    // never run locally without explicit cloud mode (enforced upstream).
    let catalog = engines::catalog()?;
    match catalog.iter().find(|e| e.id == engine_id) {
        Some(engine) if engine.mode != EngineMode::Local => Err(SottoError::app(
            "CLOUD_DISABLED",
            format!("Engine {} is not local and cloud mode is off.", engine.id),
            true,
            "Pick a local engine, or explicitly enable cloud mode in Settings.",
        )),
        Some(_) => Err(SottoError::app(
            "ENGINE_NOT_INSTALLED",
            format!("Engine {engine_id} is not installed on this Mac."),
            true,
            "Install a local model. Sotto will not send audio to the cloud.",
        )),
        None => Err(SottoError::app(
            "ENGINE_UNKNOWN",
            format!("No transcription engine with id {engine_id}."),
            true,
            "Choose an engine from the catalog.",
        )),
    }
}

fn transcribe_whisper(wav: &[u8], cache_dir: &Path) -> Result<TranscriptResult> {
    let weights = whisper_weights_path(cache_dir);

    // Local files only. A URL-shaped path is never fetched.
    if looks_like_url(&weights) {
        return Err(model_invalid());
    }

    if !weights.exists() {
        return Err(not_installed());
    }

    if !file_is_valid_ggml(&weights)? {
        return Err(model_invalid());
    }

    if !whisper_live_layout_ok(&weights) {
        return Err(model_invalid());
    }

    // Valid local weights present.
    run_whisper_inference(WHISPER_ENGINE_ID, wav, &weights)
}

fn parakeet_not_installed() -> SottoError {
    SottoError::app(
        "ENGINE_NOT_INSTALLED",
        "Parakeet TDT weights are not installed on this Mac.",
        true,
        "Download the pinned TDT pack from Models, or import a local folder. Sotto will not use the cloud.",
    )
}

fn parakeet_not_a_model() -> SottoError {
    SottoError::app(
        "ENGINE_MODEL_INVALID",
        "The Parakeet file on disk is not a TDT model directory.",
        true,
        "Place encoder-model.onnx, decoder_joint-model.onnx, and vocab.txt in models/parakeet-tdt-0.6b-v3/. A checksum blob is not a model.",
    )
}

/// Transcribe with the local Parakeet weights. Local files only; never
/// downloads and never silently selects cloud.
///
/// - checksum blob and TDT directory both absent → `ENGINE_NOT_INSTALLED`.
/// - present, `parakeet` feature off → `ENGINE_NOT_BUILT` recoverable.
/// - present, decoder compiled, dummy `.bin` / incomplete dir → `ENGINE_MODEL_INVALID`.
/// - TDT directory present, decoder compiled → on-device transcript.
/// - never copies CONSULT-001 fixture text; never returns `CLOUD_DISABLED`.
fn transcribe_parakeet(wav: &[u8], cache_dir: &Path) -> Result<TranscriptResult> {
    let blob = crate::install::parakeet_weights_path(cache_dir);
    let tdt = crate::install::parakeet_model_dir(cache_dir);

    // Local files only. A URL-shaped path is never fetched.
    if looks_like_url(&blob) || looks_like_url(&tdt) {
        return Err(parakeet_not_a_model());
    }

    let blob_present = blob.exists();
    let tdt_ok = crate::install::parakeet_tdt_layout_ok(&tdt);
    if !blob_present && !tdt_ok {
        return Err(parakeet_not_installed());
    }

    #[cfg(not(feature = "parakeet"))]
    {
        let _ = wav;
        return Err(SottoError::app(
            "ENGINE_NOT_BUILT",
            "Local Parakeet inference is not compiled into this build.",
            true,
            "Build with the `parakeet` feature to transcribe with local weights.",
        ));
    }

    #[cfg(feature = "parakeet")]
    {
        if !tdt_ok {
            return Err(parakeet_not_a_model());
        }
        crate::stt_parakeet::transcribe_tdt(wav, &tdt)
    }
}

#[cfg(feature = "whisper")]
fn run_whisper_inference(
    engine_id: &'static str,
    wav: &[u8],
    weights: &Path,
) -> Result<TranscriptResult> {
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

    let samples = pcm_f32_from_wav(wav)?;

    let ctx = WhisperContext::new_with_params(
        weights.to_string_lossy().as_ref(),
        WhisperContextParameters::default(),
    )
    .map_err(|_| model_invalid())?;
    let mut state = ctx.create_state().map_err(|_| model_invalid())?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state.full(params, &samples).map_err(|_| model_invalid())?;

    let n = state.full_n_segments().map_err(|_| model_invalid())?;
    let mut raw_text = String::new();
    let mut segments = Vec::new();
    for i in 0..n {
        let text = state.full_get_segment_text(i).unwrap_or_default();
        let start_ms = state.full_get_segment_t0(i).unwrap_or(0) * 10;
        let end_ms = state.full_get_segment_t1(i).unwrap_or(0) * 10;
        raw_text.push_str(&text);
        segments.push(engines::TranscriptSegment {
            start_ms,
            end_ms,
            text: text.trim().to_string(),
            confidence: None,
        });
    }

    let cleaned = raw_text.trim().to_string();
    Ok(TranscriptResult {
        raw_text: cleaned.clone(),
        cleaned_text: cleaned,
        language: "auto".into(),
        segments,
        summary_text: String::new(),
        action_items: String::new(),
        key_points: String::new(),
        engine_id: engine_id.to_string(),
    })
}

#[cfg(not(feature = "whisper"))]
fn run_whisper_inference(
    _engine_id: &'static str,
    _wav: &[u8],
    _weights: &Path,
) -> Result<TranscriptResult> {
    // Weights validated locally, but on-device inference is not compiled in
    // this build (the optional `whisper` Cargo feature is off, e.g. Linux CI).
    // Never fall back to cloud; never download.
    Err(SottoError::app(
        "ENGINE_NOT_BUILT",
        "Local Whisper inference is not compiled into this build.",
        true,
        "Build with the `whisper` feature to transcribe with local weights.",
    ))
}

/// Decode a mono 16-bit PCM WAV into f32 samples for local STT.
#[cfg(any(feature = "whisper", feature = "parakeet"))]
pub(crate) fn pcm_f32_from_wav(wav: &[u8]) -> Result<Vec<f32>> {
    if wav.len() < 44 || !wav.starts_with(b"RIFF") || &wav[8..12] != b"WAVE" {
        return Err(SottoError::app(
            "AUDIO_INVALID",
            "Input is not a PCM WAV.",
            true,
            "Provide a mono 16-bit PCM WAV.",
        ));
    }
    // Locate the `data` chunk.
    let mut pos = 12;
    let mut data: &[u8] = &[];
    while pos + 8 <= wav.len() {
        let id = &wav[pos..pos + 4];
        let size =
            u32::from_le_bytes([wav[pos + 4], wav[pos + 5], wav[pos + 6], wav[pos + 7]]) as usize;
        let body_start = pos + 8;
        let body_end = (body_start + size).min(wav.len());
        if id == b"data" {
            data = &wav[body_start..body_end];
            break;
        }
        pos = body_end + (size & 1);
    }
    let samples = data
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0)
        .collect();
    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn fixture_engine_returns_golden() {
        let dir = tempdir().unwrap();
        const GOLDEN: &[u8] = include_bytes!("../../fixtures/sessions/CONSULT-001.wav");
        let result = transcribe_local(FIXTURE_ENGINE_ID, GOLDEN, dir.path()).unwrap();
        assert_eq!(result.engine_id, FIXTURE_ENGINE_ID);
    }

    #[test]
    fn missing_weights_not_installed() {
        let dir = tempdir().unwrap();
        let err = transcribe_local(WHISPER_ENGINE_ID, b"", dir.path()).unwrap_err();
        assert_eq!(err.code(), "ENGINE_NOT_INSTALLED");
    }

    #[test]
    fn garbage_weights_model_invalid() {
        let dir = tempdir().unwrap();
        let path = whisper_weights_path(dir.path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"not-a-ggml-model").unwrap();
        let err = transcribe_local(WHISPER_ENGINE_ID, b"", dir.path()).unwrap_err();
        assert_eq!(err.code(), "ENGINE_MODEL_INVALID");
    }

    #[test]
    fn ggml_magic_is_valid() {
        assert!(is_valid_ggml(b"ggml....."));
        assert!(is_valid_ggml(b"GGUF....."));
        assert!(!is_valid_ggml(b"nope"));
        assert!(!is_valid_ggml(b"no"));
    }
}
