use std::path::Path;

use crate::error::{Result, SottoError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureSource {
    System,
    Mic,
    Mixed,
}

#[derive(Clone, Copy, Debug)]
pub struct CaptureConfig {
    pub source: CaptureSource,
    pub sample_rate: u32,
    pub chunk_ms: u32,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            source: CaptureSource::Mixed,
            sample_rate: 16_000,
            chunk_ms: 1_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CaptureResult {
    pub wav: Vec<u8>,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct ChunkedRecorder {
    _private: (),
}

fn not_implemented(what: &str) -> SottoError {
    SottoError::app(
        "NOT_IMPLEMENTED",
        format!("{what} is not implemented yet"),
        true,
        "Implement .kiro/specs/live-capture. Do not edit fixtures.",
    )
}

pub fn record_sine(_duration_ms: u64, _sample_rate: u32) -> Result<CaptureResult> {
    Err(not_implemented("record_sine"))
}

impl ChunkedRecorder {
    pub fn start(_dir: &Path, _cfg: CaptureConfig) -> Result<Self> {
        Err(not_implemented("ChunkedRecorder::start"))
    }

    pub fn write_pcm(&mut self, _pcm_i16: &[i16]) -> Result<()> {
        Err(not_implemented("write_pcm"))
    }

    pub fn pause(&mut self) -> Result<()> {
        Err(not_implemented("pause"))
    }

    pub fn resume(&mut self) -> Result<()> {
        Err(not_implemented("resume"))
    }

    pub fn flush(&mut self) -> Result<()> {
        Err(not_implemented("flush"))
    }

    pub fn stop(self) -> Result<CaptureResult> {
        Err(not_implemented("stop"))
    }

    pub fn recover(_dir: &Path) -> Result<CaptureResult> {
        Err(not_implemented("recover"))
    }
}

pub fn start_live(_source: CaptureSource, _dir: &Path) -> Result<ChunkedRecorder> {
    Err(not_implemented("start_live"))
}
