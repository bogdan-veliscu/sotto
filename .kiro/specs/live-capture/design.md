# Live capture — Design

Module: `src-tauri/src/capture.rs`. Public API is locked by the contract tests. Do not rename.

```rust
pub enum CaptureSource { System, Mic, Mixed }
pub struct CaptureConfig { pub source: CaptureSource, pub sample_rate: u32, pub chunk_ms: u32 }
pub struct CaptureResult { pub wav: Vec<u8>, pub duration_ms: u64 }

pub fn record_sine(duration_ms: u64, sample_rate: u32) -> Result<CaptureResult>;

pub struct ChunkedRecorder { /* private */ }
impl ChunkedRecorder {
    pub fn start(dir: &Path, cfg: CaptureConfig) -> Result<Self>;
    pub fn write_pcm(&mut self, pcm_i16: &[i16]) -> Result<()>;
    pub fn pause(&mut self) -> Result<()>;
    pub fn resume(&mut self) -> Result<()>;
    pub fn flush(&mut self) -> Result<()>;
    pub fn stop(self) -> Result<CaptureResult>;
    pub fn recover(dir: &Path) -> Result<CaptureResult>;
}
```

## WAV

- 16-bit little-endian PCM, mono, `sample_rate` (default 16000).
- Header must pass `crate::crypto::looks_like_wav`.
- Build the header yourself or use a small crate. Keep `make demo` free of network after crates are cached.

## ChunkedRecorder

- Create `dir` if missing.
- Buffer PCM. Every `chunk_ms` of *unpaused* audio, flush a file `chunk-NNNN.pcm` (raw i16le, no header).
- `pause`: set a flag; `write_pcm` becomes a no-op.
- `resume`: clear the flag.
- `stop`: concatenate chunks + tail buffer, wrap as WAV, **delete** chunk files and any temp WAV after the caller encrypts (the recorder returns bytes; it must still delete `chunk-*.pcm` on successful stop).
- `recover`: read remaining `chunk-*.pcm` in order, wrap WAV. Used when `stop` never ran.

## Demo

`demo_pipeline` keeps ingesting `CONSULT-001.wav`. Do not switch demo to sine or mic.

## Wave 8 (same PR)

`Mic` source: CPAL input if it compiles on macOS. If the device is missing in tests, tests must not require a hardware mic. Contract test for wave 8 only checks that requesting `System` without a tap returns `CAPTURE_UNSUPPORTED` with `recoverable == true` via `capture::open_backend(CaptureSource::System)`.

```rust
pub fn open_backend(source: CaptureSource) -> Result<Box<dyn LiveBackend>>;
```

`LiveBackend` can stay crate-private. Public test helper:

```rust
pub fn system_backend_error() -> SottoError; // or a function that attempts System and maps errors
```

Simplest: `pub fn start_live(source: CaptureSource, dir: &Path) -> Result<ChunkedRecorder>` which for `System` returns `CAPTURE_UNSUPPORTED` until taps exist, for `Mic` tries CPAL and the same error if no input device.

## Forbidden

- Network in capture.
- Keeping plaintext WAV next to the encrypted `.sotto` after finalize.
- Editing `fixtures/`.
