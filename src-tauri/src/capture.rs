use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::error::{Result, SottoError};

pub use crate::capture_mix::mix_pcm;

#[cfg(target_os = "macos")]
use std::collections::VecDeque;

#[cfg(target_os = "macos")]
struct MixBus {
    mic: VecDeque<i16>,
    sys: VecDeque<i16>,
    rec: Arc<Mutex<ChunkedRecorder>>,
}

#[cfg(target_os = "macos")]
impl MixBus {
    fn new(rec: Arc<Mutex<ChunkedRecorder>>) -> Self {
        Self {
            mic: VecDeque::new(),
            sys: VecDeque::new(),
            rec,
        }
    }

    fn push_mic(&mut self, pcm: &[i16]) {
        self.mic.extend(pcm.iter().copied());
        self.drain();
    }

    fn push_sys(&mut self, pcm: &[i16]) {
        self.sys.extend(pcm.iter().copied());
        self.drain();
    }

    fn drain(&mut self) {
        const MAX_SKEW: usize = 16_000;
        align_with_silence(&mut self.mic, &mut self.sys, MAX_SKEW);
        align_with_silence(&mut self.sys, &mut self.mic, MAX_SKEW);
        let n = self.mic.len().min(self.sys.len());
        if n == 0 {
            return;
        }
        let mic: Vec<i16> = self.mic.drain(..n).collect();
        let sys: Vec<i16> = self.sys.drain(..n).collect();
        let mixed = mix_pcm(&mic, &sys);
        if let Ok(mut rec) = self.rec.lock() {
            let _ = rec.write_pcm(&mixed);
        }
    }
}

#[cfg(target_os = "macos")]
fn align_with_silence(ahead: &mut VecDeque<i16>, behind: &mut VecDeque<i16>, max_skew: usize) {
    if ahead.len() > max_skew && behind.len() + max_skew < ahead.len() {
        let pad = (ahead.len() - behind.len()).min(max_skew);
        behind.extend(std::iter::repeat_n(0, pad));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureSource {
    System,
    Mic,
    Mixed,
}

impl CaptureSource {
    pub fn try_parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "system" => Ok(Self::System),
            "mic" => Ok(Self::Mic),
            "mixed" => Ok(Self::Mixed),
            _ => Err(SottoError::app(
                "SOURCE_UNKNOWN",
                format!("Capture source {raw:?} is not mic, system, or mixed."),
                true,
                "Pick microphone, what you hear, or mixed. Consent is still required.",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Mic => "mic",
            Self::Mixed => "mixed",
        }
    }
}

/// Desk copy for the chosen capture lane. Always requires consent.
/// Mixed never claims a microphone-only fallback.
pub fn source_permission_hint(source: CaptureSource, tap_status: &str) -> String {
    match source {
        CaptureSource::Mic => {
            "Microphone access is required. Consent is still required.".into()
        }
        CaptureSource::System => match tap_status {
            "available" => {
                "Screen Recording is granted. Consent is still required.".into()
            }
            "needs-permission" => {
                "Grant Screen Recording in System Settings. Consent is still required."
                    .into()
            }
            _ => "System audio is not available here. Consent is still required.".into(),
        },
        CaptureSource::Mixed => match tap_status {
            "available" => {
                "Needs Screen Recording and the microphone. Mixed will not fall back to mic-only. Consent is still required.".into()
            }
            "needs-permission" => {
                "Grant Screen Recording. Mixed also needs the microphone and will not fall back to mic-only. Consent is still required.".into()
            }
            _ => {
                "Mixed needs Screen Recording and the microphone. It will not record microphone only. Consent is still required.".into()
            }
        },
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

fn mixed_unavailable(detail: impl std::fmt::Display) -> SottoError {
    SottoError::app(
        "MIXED_UNAVAILABLE",
        format!("Mixed capture is not available ({detail})"),
        true,
        "Grant Screen Recording and microphone access. Mixed will not record microphone only. make demo still uses the golden fixture.",
    )
}

fn capture_unsupported(what: impl std::fmt::Display) -> SottoError {
    SottoError::app(
        "CAPTURE_UNSUPPORTED",
        format!("{what} capture backend is not available on this platform"),
        true,
        "Grant microphone access for mic capture, or Screen Recording for system audio. Mixed capture needs both and will not fall back to mic-only. make demo still uses the golden fixture.",
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

fn pcm_peak_pct(pcm: &[i16]) -> u32 {
    let max = pcm
        .iter()
        .map(|s| s.unsigned_abs() as u32)
        .max()
        .unwrap_or(0);
    ((max * 100) / i16::MAX as u32).min(100)
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
    peak: AtomicU32,
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
            peak: AtomicU32::new(0),
        })
    }

    pub fn write_pcm(&mut self, pcm_i16: &[i16]) -> Result<()> {
        if self.paused {
            return Ok(());
        }
        let peak = pcm_peak_pct(pcm_i16);
        self.peak.fetch_max(peak, Ordering::Relaxed);
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

    /// Instantaneous input peak 0–100, then decay so a quiet room falls back.
    pub fn take_level(&self) -> u8 {
        let v = self.peak.load(Ordering::Relaxed);
        self.peak.store(v.saturating_mul(5) / 8, Ordering::Relaxed);
        v.min(100) as u8
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

    pub fn level(&self) -> u8 {
        lock_rec(&self.rec).map(|rec| rec.take_level()).unwrap_or(0)
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
/// - `"unsupported"` — off macOS (no ScreenCaptureKit).
/// - `"needs-permission"` — macOS tap compiled, Screen Recording not granted.
/// - `"available"` — macOS tap compiled and `CGPreflightScreenCaptureAccess` is true.
pub fn system_tap_status() -> &'static str {
    #[cfg(not(target_os = "macos"))]
    {
        "unsupported"
    }
    #[cfg(target_os = "macos")]
    {
        crate::capture_system::tap_status()
    }
}

fn start_system(dir: &Path) -> Result<LiveSession> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = dir;
        return Err(capture_unsupported("System audio"));
    }
    #[cfg(target_os = "macos")]
    {
        let cfg = CaptureConfig {
            source: CaptureSource::System,
            ..CaptureConfig::default()
        };
        let sample_rate = cfg.sample_rate;
        let rec = Arc::new(Mutex::new(ChunkedRecorder::start(dir, cfg)?));
        let rec_thread = Arc::clone(&rec);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sotto-system".into())
            .spawn(move || {
                match crate::capture_system::start_system_stream(rec_thread, sample_rate) {
                    Ok(_tap) => {
                        let _ = ready_tx.send(Ok(()));
                        let _ = stop_rx.recv();
                    }
                    Err(err) => {
                        let _ = ready_tx.send(Err(err.to_string()));
                    }
                }
            })
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
                Err(capture_unsupported("System audio"))
            }
        }
    }
}

fn start_mixed(dir: &Path) -> Result<LiveSession> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = dir;
        return Err(mixed_unavailable("not macOS"));
    }
    #[cfg(target_os = "macos")]
    {
        if !crate::capture_system::screen_recording_granted() {
            return Err(mixed_unavailable("Screen Recording is off"));
        }
        let cfg = CaptureConfig {
            source: CaptureSource::Mixed,
            ..CaptureConfig::default()
        };
        let sample_rate = cfg.sample_rate;
        let rec = Arc::new(Mutex::new(ChunkedRecorder::start(dir, cfg)?));
        let rec_thread = Arc::clone(&rec);
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("sotto-mixed".into())
            .spawn(move || {
                let bus = Arc::new(Mutex::new(MixBus::new(rec_thread)));
                let bus_sys = Arc::clone(&bus);
                let tap = match crate::capture_system::start_system_sink(sample_rate, move |pcm| {
                    if let Ok(mut bus) = bus_sys.lock() {
                        bus.push_sys(pcm);
                    }
                }) {
                    Ok(tap) => tap,
                    Err(err) => {
                        let _ = ready_tx.send(Err(err.to_string()));
                        return;
                    }
                };
                let bus_mic = Arc::clone(&bus);
                match crate::capture_mic::start_input_sink(sample_rate, move |pcm| {
                    if let Ok(mut bus) = bus_mic.lock() {
                        bus.push_mic(pcm);
                    }
                }) {
                    Ok(_stream) => {
                        let _ = ready_tx.send(Ok(()));
                        let _ = stop_rx.recv();
                        drop(tap);
                    }
                    Err(err) => {
                        drop(tap);
                        let _ = ready_tx.send(Err(err.to_string()));
                    }
                }
            })
            .map_err(|e| mixed_unavailable(e))?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(LiveSession {
                rec,
                stop: Some(stop_tx),
                worker: Some(worker),
            }),
            Ok(Err(msg)) => {
                let _ = worker.join();
                Err(mixed_unavailable(msg))
            }
            Err(_) => {
                let _ = worker.join();
                Err(mixed_unavailable("mixed backends"))
            }
        }
    }
}

/// Begin a live capture session. System-audio uses ScreenCaptureKit on macOS
/// when Screen Recording is already granted; otherwise `CAPTURE_UNSUPPORTED`.
/// Mixed requires both the tap and the microphone; it never falls back to
/// mic-only. Tests never prompt. Tests must not call `start_live(Mic)`.
pub fn start_live(source: CaptureSource, dir: &Path) -> Result<LiveSession> {
    match source {
        CaptureSource::System => start_system(dir),
        CaptureSource::Mic => start_mic(dir),
        CaptureSource::Mixed => start_mixed(dir),
    }
}
