//! macOS system-audio tap via ScreenCaptureKit.
//!
//! Tests never prompt: we only call `CGPreflightScreenCaptureAccess`.
//! If Screen Recording is off, return `CAPTURE_UNSUPPORTED` with
//! `needs-permission`. A real tap is opened only when TCC already allows it.

use std::sync::{Arc, Mutex};

use screencapturekit::prelude::*;

use crate::capture::ChunkedRecorder;
use crate::error::{Result, SottoError};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGPreflightScreenCaptureAccess() -> bool;
}

fn unsupported(detail: impl std::fmt::Display) -> SottoError {
    SottoError::app(
        "CAPTURE_UNSUPPORTED",
        format!("System audio capture is not available ({detail})"),
        true,
        "Grant Screen Recording to Sotto in System Settings → Privacy & Security. Consent is still required before any capture. make demo still uses the golden fixture.",
    )
}

pub fn screen_recording_granted() -> bool {
    unsafe { CGPreflightScreenCaptureAccess() }
}

pub fn tap_status() -> &'static str {
    if screen_recording_granted() {
        "available"
    } else {
        "needs-permission"
    }
}

/// Live ScreenCaptureKit stream. Dropping it stops capture.
pub struct SystemTap {
    stream: SCStream,
}

pub fn start_system_stream(
    rec: Arc<Mutex<ChunkedRecorder>>,
    target_rate: u32,
) -> Result<SystemTap> {
    start_system_sink(target_rate, move |pcm| {
        if let Ok(mut rec) = rec.lock() {
            let _ = rec.write_pcm(pcm);
        }
    })
}

pub fn start_system_sink<F>(target_rate: u32, on_pcm: F) -> Result<SystemTap>
where
    F: Fn(&[i16]) + Send + Sync + 'static,
{
    if !screen_recording_granted() {
        return Err(unsupported("Screen Recording is off"));
    }
    let content = SCShareableContent::get().map_err(|e| unsupported(e))?;
    let displays = content.displays();
    let display = displays
        .first()
        .ok_or_else(|| unsupported("no display to attach an audio tap"))?;
    let filter = SCContentFilter::create()
        .with_display(display)
        .with_excluding_windows(&[])
        .build();
    let in_rate = sck_sample_rate(target_rate);
    let config = SCStreamConfiguration::new()
        .with_width(16)
        .with_height(16)
        .with_captures_audio(true)
        .with_sample_rate(in_rate as i32)
        .with_channel_count(1)
        .with_excludes_current_process_audio(true);
    let mut stream = SCStream::new(&filter, &config);
    stream.add_output_handler(
        move |sample, _| {
            let pcm = pcm_from_sample(&sample, in_rate, target_rate);
            if pcm.is_empty() {
                return;
            }
            on_pcm(&pcm);
        },
        SCStreamOutputType::Audio,
    );
    stream.start_capture().map_err(|e| unsupported(e))?;
    Ok(SystemTap { stream })
}

impl Drop for SystemTap {
    fn drop(&mut self) {
        let _ = self.stream.stop_capture();
    }
}

/// ScreenCaptureKit only accepts 8/16/24/48 kHz. Prefer the recorder rate.
fn sck_sample_rate(target: u32) -> u32 {
    match target {
        8_000 | 16_000 | 24_000 | 48_000 => target,
        _ => 48_000,
    }
}

fn pcm_from_sample(sample: &CMSampleBuffer, in_rate: u32, target_rate: u32) -> Vec<i16> {
    let Some(list) = sample.audio_buffer_list() else {
        return Vec::new();
    };
    let Some(buffer) = list.get(0) else {
        return Vec::new();
    };
    let bytes = buffer.data();
    if bytes.is_empty() {
        return Vec::new();
    }
    let channels = buffer.number_channels().max(1) as usize;
    let fmt = sample.format_description();
    let is_float = fmt.as_ref().is_none_or(|f| f.audio_is_float());
    let actual_rate = fmt
        .as_ref()
        .and_then(|f| f.audio_sample_rate())
        .map(|r| r.round() as u32)
        .filter(|r| *r > 0)
        .unwrap_or(in_rate);
    let mono = if is_float {
        f32_interleaved_to_mono(bytes, channels)
    } else {
        i16_interleaved_to_mono(bytes, channels)
    };
    resample_to(&mono, actual_rate, target_rate)
}

fn i16_interleaved_to_mono(bytes: &[u8], channels: usize) -> Vec<i16> {
    let step = 2 * channels.max(1);
    bytes
        .chunks_exact(step)
        .map(|frame| i16::from_le_bytes([frame[0], frame[1]]))
        .collect()
}

fn f32_interleaved_to_mono(bytes: &[u8], channels: usize) -> Vec<i16> {
    let step = 4 * channels.max(1);
    bytes
        .chunks_exact(step)
        .map(|frame| {
            let f = f32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]);
            (f.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
        })
        .collect()
}

fn resample_to(samples: &[i16], from: u32, to: u32) -> Vec<i16> {
    if from == 0 || to == 0 || from == to {
        return samples.to_vec();
    }
    let ratio = f64::from(from) / f64::from(to);
    let n = ((samples.len() as f64) / ratio).floor() as usize;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let src = (i as f64 * ratio) as usize;
        if src < samples.len() {
            out.push(samples[src]);
        }
    }
    out
}
