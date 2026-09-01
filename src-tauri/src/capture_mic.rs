//! macOS microphone input via CPAL. Never used from `cargo test`.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};

use crate::capture::ChunkedRecorder;
use crate::error::{Result, SottoError};

fn unsupported(detail: impl std::fmt::Display) -> SottoError {
    SottoError::app(
        "CAPTURE_UNSUPPORTED",
        format!("Microphone capture is not available ({detail})"),
        true,
        "Grant microphone permission, or pick a device. System audio mix is not wired yet. make demo still uses the golden fixture.",
    )
}

pub fn start_input_stream(rec: Arc<Mutex<ChunkedRecorder>>, target_rate: u32) -> Result<Stream> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| unsupported("no default input device"))?;
    let supported = device.default_input_config().map_err(|e| unsupported(e))?;
    let sample_format = supported.sample_format();
    let config: StreamConfig = supported.into();
    let channels = config.channels.max(1) as usize;
    let in_rate = config.sample_rate.0;
    let rec_cb = Arc::clone(&rec);

    let stream = match sample_format {
        SampleFormat::F32 => build_stream::<f32>(
            &device,
            &config,
            rec_cb,
            channels,
            in_rate,
            target_rate,
            |s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16,
        )?,
        SampleFormat::I16 => build_stream::<i16>(
            &device,
            &config,
            rec_cb,
            channels,
            in_rate,
            target_rate,
            |s| s,
        )?,
        SampleFormat::U16 => build_stream::<u16>(
            &device,
            &config,
            rec_cb,
            channels,
            in_rate,
            target_rate,
            |s| (i32::from(s) - 32_768) as i16,
        )?,
        other => return Err(unsupported(format!("sample format {other:?}"))),
    };
    stream.play().map_err(|e| unsupported(e))?;
    Ok(stream)
}

fn build_stream<T: Copy + Send + cpal::SizedSample + 'static>(
    device: &cpal::Device,
    config: &StreamConfig,
    rec: Arc<Mutex<ChunkedRecorder>>,
    channels: usize,
    in_rate: u32,
    target_rate: u32,
    to_i16: impl Fn(T) -> i16 + Send + 'static,
) -> Result<Stream> {
    device
        .build_input_stream(
            config,
            move |data: &[T], _| {
                let mut mono = Vec::with_capacity(data.len() / channels + 1);
                for frame in data.chunks(channels) {
                    if let Some(first) = frame.first() {
                        mono.push(to_i16(*first));
                    }
                }
                let pcm = resample_to(&mono, in_rate, target_rate);
                if let Ok(mut rec) = rec.lock() {
                    let _ = rec.write_pcm(&pcm);
                }
            },
            move |err| eprintln!("sotto cpal: {err}"),
            None,
        )
        .map_err(|e| unsupported(e))
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
