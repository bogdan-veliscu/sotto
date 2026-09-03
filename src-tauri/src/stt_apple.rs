//! Apple on-device speech. Never Apple servers.
//!
//! macOS 26+ uses SpeechAnalyzer / SpeechTranscriber. Other targets return
//! `ENGINE_NOT_BUILT`. Contract tests and `demo_pipeline` must not call
//! [`transcribe`].

use std::fs;
use std::path::Path;

#[cfg(target_os = "macos")]
use std::ffi::c_char;

use crate::engines::{TranscriptResult, TranscriptSegment};
use crate::error::{Result, SottoError};

pub const APPLE_SPEECH_ENGINE_ID: &str = "apple-speech-ondevice";

#[cfg(target_os = "macos")]
extern "C" {
    fn sotto_apple_speech_available() -> i32;
    fn sotto_apple_speech_transcribe(
        wav_path: *const c_char,
        out_buf: *mut c_char,
        out_cap: i32,
        err_buf: *mut c_char,
        err_cap: i32,
    ) -> i32;
}

/// True when the on-device Apple recognizer exists. Does not download assets
/// and does not prompt for Speech Recognition permission.
pub fn available() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { sotto_apple_speech_available() == 1 }
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn transcribe(wav: &[u8], cache_dir: &Path) -> Result<TranscriptResult> {
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (wav, cache_dir);
        return Err(not_built());
    }

    #[cfg(target_os = "macos")]
    {
        if !available() {
            return Err(SottoError::app(
                "ENGINE_SETUP_REQUIRED",
                "Apple on-device speech is not available on this Mac.",
                true,
                "Use Parakeet or Whisper, or update macOS. Sotto will not send audio to Apple servers.",
            ));
        }
        let tmp_dir = cache_dir.join("tmp");
        fs::create_dir_all(&tmp_dir)?;
        let wav_path = tmp_dir.join("apple-speech.wav");
        fs::write(&wav_path, wav)?;
        let result = transcribe_path(&wav_path);
        let _ = fs::remove_file(&wav_path);
        result
    }
}

#[cfg(target_os = "macos")]
fn transcribe_path(wav_path: &Path) -> Result<TranscriptResult> {
    use std::ffi::{CStr, CString};

    let path = CString::new(wav_path.to_string_lossy().as_bytes()).map_err(|_| {
        SottoError::app(
            "ENGINE_MODEL_INVALID",
            "Apple Speech could not read the local capture file.",
            true,
            "Retry transcription. Audio stays on this Mac.",
        )
    })?;
    let mut out = vec![0u8; 1024 * 1024];
    let mut err = vec![0u8; 4096];
    let rc = unsafe {
        sotto_apple_speech_transcribe(
            path.as_ptr(),
            out.as_mut_ptr() as *mut c_char,
            out.len() as i32,
            err.as_mut_ptr() as *mut c_char,
            err.len() as i32,
        )
    };
    if rc != 0 {
        let msg = unsafe { CStr::from_ptr(err.as_ptr() as *const c_char) }
            .to_string_lossy()
            .into_owned();
        return Err(SottoError::app(
            "ENGINE_SETUP_REQUIRED",
            if msg.is_empty() {
                "Apple on-device speech did not finish.".into()
            } else {
                msg
            },
            true,
            "Grant Speech Recognition, or use Parakeet / Whisper. Audio is not sent to Apple servers.",
        ));
    }
    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_string_lossy()
        .trim()
        .to_string();
    let mut segments = Vec::new();
    if !text.is_empty() {
        segments.push(TranscriptSegment {
            start_ms: 0,
            end_ms: 0,
            text: text.clone(),
            confidence: None,
        });
    }
    Ok(TranscriptResult {
        raw_text: text.clone(),
        cleaned_text: text,
        language: "auto".into(),
        segments,
        summary_text: String::new(),
        action_items: String::new(),
        key_points: String::new(),
        engine_id: APPLE_SPEECH_ENGINE_ID.to_string(),
    })
}

#[cfg(not(target_os = "macos"))]
fn not_built() -> SottoError {
    SottoError::app(
        "ENGINE_NOT_BUILT",
        "Apple Speech is only compiled for macOS.",
        true,
        "Use this Mac build, or import Parakeet / Whisper weights.",
    )
}
