use std::fs;

use sotto_lib::capture::{
    record_sine, start_live, CaptureConfig, CaptureSource, ChunkedRecorder,
};
use sotto_lib::demo_pipeline;
use sotto_lib::stt::{transcribe_local, whisper_weights_path, WHISPER_ENGINE_ID};
use tempfile::tempdir;

const FIXTURE_WAV: &[u8] = include_bytes!("../../fixtures/sessions/CONSULT-001.wav");

fn is_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE"
}

#[test]
fn ct_capture_wav() {
    let result = record_sine(200, 16_000).expect("record_sine");
    assert!(is_wav(&result.wav), "sine output must be a WAV");
    assert!(result.duration_ms >= 150, "duration too short: {}", result.duration_ms);
    assert!(result.duration_ms <= 250, "duration too long: {}", result.duration_ms);
}

#[test]
fn ct_pause_resume() {
    let dir = tempdir().unwrap();
    let mut rec = ChunkedRecorder::start(dir.path(), CaptureConfig::default()).unwrap();
    let hundred_ms = vec![0i16; 1_600];
    rec.write_pcm(&hundred_ms).unwrap();
    rec.pause().unwrap();
    rec.write_pcm(&hundred_ms).unwrap();
    rec.resume().unwrap();
    rec.write_pcm(&hundred_ms).unwrap();
    rec.flush().unwrap();
    let result = rec.stop().unwrap();
    assert!(is_wav(&result.wav));
    assert!(
        result.duration_ms >= 150 && result.duration_ms <= 250,
        "paused PCM must not count, got {} ms",
        result.duration_ms
    );
}

#[test]
fn ct_crash_partial() {
    let dir = tempdir().unwrap();
    let mut rec = ChunkedRecorder::start(dir.path(), CaptureConfig::default()).unwrap();
    rec.write_pcm(&vec![0i16; 16_000]).unwrap();
    rec.flush().unwrap();
    drop(rec);
    let recovered = ChunkedRecorder::recover(dir.path()).expect("recover");
    assert!(is_wav(&recovered.wav));
    assert!(recovered.duration_ms >= 900, "recovered duration {}", recovered.duration_ms);
}

#[test]
fn ct_demo_still_offline() {
    let dir = tempdir().unwrap();
    let report = demo_pipeline(dir.path()).expect("demo");
    assert_eq!(report.network_calls, 0);
    assert_eq!(report.engine_id, "fixture-replay");
    assert_eq!(report.engine_mode, "local");
}

#[test]
fn ct_mic_unsupported_is_recoverable() {
    let dir = tempdir().unwrap();
    let err = start_live(CaptureSource::System, dir.path()).unwrap_err();
    assert_eq!(err.code(), "CAPTURE_UNSUPPORTED");
}

#[test]
fn ct_whisper_local_only() {
    let dir = tempdir().unwrap();
    let err = transcribe_local(WHISPER_ENGINE_ID, FIXTURE_WAV, dir.path()).unwrap_err();
    assert_eq!(err.code(), "ENGINE_NOT_INSTALLED");
}

#[test]
fn ct_demo_no_download() {
    let dir = tempdir().unwrap();
    let report = demo_pipeline(dir.path()).expect("demo");
    assert_eq!(report.network_calls, 0);
    assert_eq!(report.engine_id, "fixture-replay");
    assert!(
        !whisper_weights_path(dir.path()).exists(),
        "demo must not write Whisper weights"
    );
}

#[test]
fn ct_whisper_weights_are_local() {
    let dir = tempdir().unwrap();
    let path = whisper_weights_path(dir.path());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not-a-ggml-model").unwrap();
    let err = transcribe_local(WHISPER_ENGINE_ID, FIXTURE_WAV, dir.path()).unwrap_err();
    assert_eq!(err.code(), "ENGINE_MODEL_INVALID");
}

#[test]
fn demo_pipeline_holds_privacy_invariants() {
    let dir = tempdir().unwrap();
    let report = demo_pipeline(dir.path()).expect("demo pipeline");
    assert_eq!(report.telemetry, "off");
    assert_eq!(report.cloud_mode, "off");
    assert_eq!(report.network_calls, 0);
    assert_eq!(report.engine_mode, "local");
    assert_eq!(report.engine_id, "fixture-replay");
    assert!(report.consent_enforced);
    assert!(report.audio_is_ciphertext);
    assert!(report.search_hits >= 1);
    assert!(report.delete_all_clears_search);
    assert_eq!(report.status, "transcribed");
    let audio = dir.path().join("audio");
    if audio.exists() {
        for entry in fs::read_dir(audio).unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                let bytes = fs::read(&path).unwrap();
                assert!(
                    bytes.is_empty() || !bytes.starts_with(b"RIFF"),
                    "plaintext WAV leftover at {}",
                    path.display()
                );
            }
        }
    }
}
