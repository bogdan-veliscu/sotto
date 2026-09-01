//! On-device Parakeet TDT decode via `parakeet-rs` / ONNX Runtime.
//!
//! Compiled only with `--features parakeet`. Never downloads weights. The
//! checksum blob `parakeet-tdt-0.6b-v3.bin` is not a model; callers must pass
//! a TDT directory (`encoder-model.onnx`, `decoder_joint-model.onnx`, `vocab.txt`).

use std::path::Path;

use parakeet_rs::{ParakeetTDT, TimestampMode, Transcriber};

use crate::engines::{TranscriptResult, TranscriptSegment};
use crate::error::{Result, SottoError};
use crate::install::PARAKEET_ENGINE_ID;
use crate::stt::pcm_f32_from_wav;

fn model_invalid(detail: impl std::fmt::Display) -> SottoError {
    SottoError::app(
        "ENGINE_MODEL_INVALID",
        format!("The Parakeet TDT weights could not be used ({detail})."),
        true,
        "Place encoder-model.onnx, decoder_joint-model.onnx, and vocab.txt in the local TDT folder. Sotto will not fetch them.",
    )
}

pub(crate) fn transcribe_tdt(wav: &[u8], model_dir: &Path) -> Result<TranscriptResult> {
    let samples = pcm_f32_from_wav(wav)?;
    let mut model = ParakeetTDT::from_pretrained(model_dir, None).map_err(model_invalid)?;
    let out = model
        .transcribe_samples(samples, 16_000, 1, Some(TimestampMode::Sentences))
        .map_err(model_invalid)?;

    let cleaned = out.text.trim().to_string();
    let mut segments: Vec<TranscriptSegment> = out
        .tokens
        .into_iter()
        .map(|token| TranscriptSegment {
            start_ms: (token.start * 1000.0) as i64,
            end_ms: (token.end * 1000.0) as i64,
            text: token.text,
            confidence: None,
        })
        .collect();
    if segments.is_empty() && !cleaned.is_empty() {
        segments.push(TranscriptSegment {
            start_ms: 0,
            end_ms: 0,
            text: cleaned.clone(),
            confidence: None,
        });
    }

    Ok(TranscriptResult {
        raw_text: cleaned.clone(),
        cleaned_text: cleaned,
        language: "auto".into(),
        segments,
        summary_text: String::new(),
        action_items: String::new(),
        key_points: String::new(),
        engine_id: PARAKEET_ENGINE_ID.to_string(),
    })
}
