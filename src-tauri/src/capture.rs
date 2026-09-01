use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::error::{Result, SottoError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureSource {
    System,
    Mic,
    Mixed,
}

impl CaptureSource {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "system" => Self::System,
            "mic" => Self::Mic,
            _ => Self::Mixed,
        }
    }
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

fn capture_unsupported(what: impl std::fmt::Display) -> SottoError {
    SottoError::app(
        "CAPTURE_UNSUPPORTED",
        format!("{what} capture backend is not available on this platform"),
        true,
        "Live hardware capture is not available. Grant microphone access for mic capture. System audio mix is not wired yet. make demo still uses the golden fixture.",
    )
}

/// Build a canonical 16-bit LE PCM, mono WAV from raw i16 samples.
fn wav_from_samples(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let block_align: u16 = channels * (bits_per_sample / 8);
    let byte_rate: u32 = sample_rate * u32::from(block_align);
    let data_len: u32 = (samples.len() * 2) as u32;
    let riff_len: u32 = 36 + data_len;

    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&riff_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");

    // fmt chunk
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    out.extend_from_slice(&1u16.to_le_bytes()); // audio format = PCM
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    out
}

fn duration_ms_for(sample_count: usize, sample_rate: u32) -> u64 {
    if sample_rate == 0 {
        return 0;
    }
    (sample_count as u64 * 1000) / u64::from(sample_rate)
}

/// Generate a mono 16 kHz sine tone WAV for the requested duration.
pub fn record_sine(duration_ms: u64, sample_rate: u32) -> Result<CaptureResult> {
    if duration_ms == 0 || sample_rate == 0 {
        return Err(SottoError::app(
            "CAPTURE_INVALID",
            "record_sine needs a positive duration and sample rate",
            true,
            "Pass duration_ms > 0 and sample_rate > 0.",
        ));
    }

    let sample_count = ((duration_ms * u64::from(sample_rate)) / 1000) as usize;
    let freq = 440.0_f64;
    let amplitude = 0.3_f64 * f64::from(i16::MAX);
    let mut samples = Vec::with_capacity(sample_count);
    for n in 0..sample_count {
        let t = n as f64 / f64::from(sample_rate);
        let value = (2.0 * std::f64::consts::PI * freq * t).sin() * amplitude;
        samples.push(value as i16);
    }

    let wav = wav_from_samples(&samples, sample_rate);
    Ok(CaptureResult {
        wav,
        duration_ms: duration_ms_for(samples.len(), sample_rate),
    })
}

#[derive(Debug)]
pub struct ChunkedRecorder {
    dir: PathBuf,
    sample_rate: u32,
    chunk_samples: usize,
    buffer: Vec<i16>,
    next_chunk: u32,
    paused: bool,
}

fn chunk_path(dir: &Path, index: u32) -> PathBuf {
    dir.join(format!("chunk-{index:04}.pcm"))
}

fn is_chunk_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with("chunk-") && n.ends_with(".pcm"))
        .unwrap_or(false)
}

fn rate_path(dir: &Path) -> PathBuf {
    dir.join("capture.rate")
}

fn write_sample_rate(dir: &Path, sample_rate: u32) -> Result<()> {
    fs::write(rate_path(dir), sample_rate.to_string())?;
    Ok(())
}

fn read_sample_rate(dir: &Path) -> u32 {
    fs::read_to_string(rate_path(dir))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .filter(|r| *r > 0)
        .unwrap_or_else(|| CaptureConfig::default().sample_rate)
}

fn read_chunks(dir: &Path) -> Result<Vec<i16>> {
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if is_chunk_file(&path) {
            files.push(path);
        }
    }
    files.sort();

    let mut samples = Vec::new();
    for path in files {
        let bytes = fs::read(&path)?;
        for pair in bytes.chunks_exact(2) {
            samples.push(i16::from_le_bytes([pair[0], pair[1]]));
        }
    }
    Ok(samples)
}

fn delete_chunks(dir: &Path) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if is_chunk_file(&path) {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

impl ChunkedRecorder {
    pub fn start(dir: &Path, cfg: CaptureConfig) -> Result<Self> {
        if cfg.sample_rate == 0 {
            return Err(SottoError::app(
                "CAPTURE_INVALID",
                "sample_rate must be positive",
                true,
                "Use CaptureConfig::default() or a positive sample rate.",
            ));
        }
        fs::create_dir_all(dir)?;
        write_sample_rate(dir, cfg.sample_rate)?;
        let chunk_ms = if cfg.chunk_ms == 0 {
            1_000
        } else {
            cfg.chunk_ms
        };
        let chunk_samples =
            ((u64::from(chunk_ms) * u64::from(cfg.sample_rate)) / 1000).max(1) as usize;
        Ok(Self {
            dir: dir.to_path_buf(),
            sample_rate: cfg.sample_rate,
            chunk_samples,
            buffer: Vec::new(),
            next_chunk: 0,
            paused: false,
        })
    }

    pub fn write_pcm(&mut self, pcm_i16: &[i16]) -> Result<()> {
        if self.paused {
            return Ok(());
        }
        self.buffer.extend_from_slice(pcm_i16);
        while self.buffer.len() >= self.chunk_samples {
            let rest = self.buffer.split_off(self.chunk_samples);
            let chunk = std::mem::replace(&mut self.buffer, rest);
            self.write_chunk(&chunk)?;
        }
        Ok(())
    }

    fn write_chunk(&mut self, samples: &[i16]) -> Result<()> {
        if samples.is_empty() {
            return Ok(());
        }
        let mut bytes = Vec::with_capacity(samples.len() * 2);
        for s in samples {
            bytes.extend_from_slice(&s.to_le_bytes());
        }
        fs::write(chunk_path(&self.dir, self.next_chunk), &bytes)?;
        self.next_chunk += 1;
        Ok(())
    }

    pub fn pause(&mut self) -> Result<()> {
        self.paused = true;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        self.paused = false;
        Ok(())
    }

    /// Flush the tail buffer as its own chunk file.
    pub fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let chunk = std::mem::take(&mut self.buffer);
        self.write_chunk(&chunk)?;
        Ok(())
    }

    /// Build a WAV from flushed chunks without consuming the recorder.
    pub fn finish(&mut self) -> Result<CaptureResult> {
        self.flush()?;
        let samples = read_chunks(&self.dir)?;
        let wav = wav_from_samples(&samples, self.sample_rate);
        let duration_ms = duration_ms_for(samples.len(), self.sample_rate);
        delete_chunks(&self.dir)?;
        Ok(CaptureResult { wav, duration_ms })
    }

    pub fn stop(mut self) -> Result<CaptureResult> {
        self.finish()
    }

    /// Recover a valid WAV from flushed chunks when `stop` never ran.
    pub fn recover(dir: &Path) -> Result<CaptureResult> {
        let samples = read_chunks(dir)?;
        if samples.is_empty() {
            return Err(SottoError::app(
                "CAPTURE_NO_CHUNKS",
                "no chunk-*.pcm files to recover",
                true,
                "Nothing was flushed before the crash.",
            ));
        }
        let sample_rate = read_sample_rate(dir);
        let wav = wav_from_samples(&samples, sample_rate);
        let duration_ms = duration_ms_for(samples.len(), sample_rate);
        Ok(CaptureResult { wav, duration_ms })
    }
}

impl Drop for ChunkedRecorder {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Live capture handle. The CPAL stream lives on a dedicated thread because
/// `cpal::Stream` is not `Send`. Tests inject PCM through `injected`.
pub struct LiveSession {
    rec: Arc<Mutex<ChunkedRecorder>>,
    stop: Option<mpsc::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

impl LiveSession {
    fn from_recorder(rec: ChunkedRecorder) -> Self {
        Self {
            rec: Arc::new(Mutex::new(rec)),
            stop: None,
            worker: None,
        }
    }

    /// Inject PCM without opening a microphone. Used by contract tests.
    /// Do not call `start_live(Mic)` from tests (it prompts for the device).
    pub fn injected(rec: ChunkedRecorder) -> Self {
        Self::from_recorder(rec)
    }

    pub fn pause(&self) -> Result<()> {
        lock_rec(&self.rec)?.pause()
    }

    pub fn resume(&self) -> Result<()> {
        lock_rec(&self.rec)?.resume()
    }

    pub fn finish(self) -> Result<CaptureResult> {
        if let Some(tx) = &self.stop {
            let _ = tx.send(());
        }
        if let Some(worker) = self.worker {
            let _ = worker.join();
        }
        lock_rec(&self.rec)?.finish()
    }
}

fn lock_rec(
    rec: &Arc<Mutex<ChunkedRecorder>>,
) -> Result<std::sync::MutexGuard<'_, ChunkedRecorder>> {
    rec.lock().map_err(|_| {
        SottoError::app(
            "CAPTURE_LOCK",
            "capture lock poisoned",
            true,
            "Stop and record again.",
        )
    })
}

fn start_mic(dir: &Path) -> Result<LiveSession> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = dir;
        return Err(capture_unsupported("Microphone"));
    }
    #[cfg(target_os = "macos")]
    {
        let cfg = CaptureConfig {
            source: CaptureSource::Mic,
            ..CaptureConfig::default()
        };
        let sample_rate = cfg.sample_rate;
        let rec = Arc::new(Mutex::new(ChunkedRecorder::start(dir, cfg)?));
        let rec_thread = Arc::clone(&rec);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sotto-mic".into())
            .spawn(
                move || match crate::capture_mic::start_input_stream(rec_thread, sample_rate) {
                    Ok(_stream) => {
                        let _ = ready_tx.send(Ok(()));
                        let _ = stop_rx.recv();
                    }
                    Err(err) => {
                        let _ = ready_tx.send(Err(err.to_string()));
                    }
                },
            )
            .map_err(|e| capture_unsupported(e))?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(LiveSession {
                rec,
                stop: Some(stop_tx),
                worker: Some(worker),
            }),
            Ok(Err(msg)) => {
                let _ = worker.join();
                Err(capture_unsupported(msg))
            }
            Err(_) => {
                let _ = worker.join();
                Err(capture_unsupported("Microphone"))
            }
        }
    }
}

/// Report whether a system-audio tap backend is available on this platform.
///
/// Possible return values:
/// - `"unsupported"` — off macOS, or macOS but no tap backend compiled in.
/// - `"needs-permission"` — macOS tap backend present, Screen Recording not granted.
/// - `"available"` — macOS tap backend present and permission granted.
///
/// This wave compiles no tap backend, so macOS also returns `"unsupported"`.
pub fn system_tap_status() -> &'static str {
    // No system-audio tap backend is compiled in this build.
    // Never claim "available" unless a backend is wired and the permission
    // check passes. `#[cfg(target_os = "macos")]` blocks below are kept as
    // placeholders so future waves only need to fill them in.
    #[cfg(not(target_os = "macos"))]
    {
        "unsupported"
    }
    #[cfg(target_os = "macos")]
    {
        // No tap backend compiled yet → honest "unsupported".
        // When a real tap is wired, replace this with a permission probe.
        "unsupported"
    }
}

/// Begin a live capture session. System-audio taps are not implemented and
/// return `CAPTURE_UNSUPPORTED` (recoverable). On macOS, `Mic`/`Mixed` open a
/// CPAL input stream. Elsewhere (and when no device is present) they return
/// the same recoverable error. Tests must not call `start_live(Mic)`.
pub fn start_live(source: CaptureSource, dir: &Path) -> Result<LiveSession> {
    match source {
        CaptureSource::System => Err(capture_unsupported("System audio")),
        CaptureSource::Mic | CaptureSource::Mixed => start_mic(dir),
    }
}
