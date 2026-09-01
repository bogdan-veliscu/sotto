use std::fs;

use sotto_lib::capture::{
    record_sine, start_live, CaptureConfig, CaptureSource, ChunkedRecorder, LiveSession,
};
use sotto_lib::demo_pipeline;
use sotto_lib::install::{
    delete_model, install_bytes, overlay_catalog, parakeet_weights_path, PARAKEET_ENGINE_ID,
};
use sotto_lib::notes::extract_notes;
use sotto_lib::search::SearchFilter;
use sotto_lib::stt::{transcribe_local, whisper_weights_path, WHISPER_ENGINE_ID};
use sotto_lib::{catalog, InstallState, Store};
use tempfile::tempdir;

const FIXTURE_WAV: &[u8] = include_bytes!("../../fixtures/sessions/CONSULT-001.wav");

fn is_wav(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WAVE"
}

#[test]
fn ct_capture_wav() {
    let result = record_sine(200, 16_000).expect("record_sine");
    assert!(is_wav(&result.wav), "sine output must be a WAV");
    assert!(
        result.duration_ms >= 150,
        "duration too short: {}",
        result.duration_ms
    );
    assert!(
        result.duration_ms <= 250,
        "duration too long: {}",
        result.duration_ms
    );
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
    assert!(
        recovered.duration_ms >= 900,
        "recovered duration {}",
        recovered.duration_ms
    );
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
    // On non-macOS, System capture is always unsupported (no tap backend).
    // On macOS with a compiled tap, start_live(System) may succeed — that is
    // also acceptable as long as it is NOT the fixture. Linux CI still proves
    // the unsupported path because no tap backend is compiled there.
    match start_live(CaptureSource::System, dir.path()) {
        Err(err) => {
            assert_eq!(err.code(), "CAPTURE_UNSUPPORTED");
            assert!(err.recoverable());
        }
        Ok(_session) => {
            // A real tap opened on macOS. The test accepts this; CT-system-not-fixture
            // separately enforces that the bytes ≠ CONSULT-001.
            #[cfg(not(target_os = "macos"))]
            panic!("system capture must be unsupported on this platform");
        }
    }
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

const PARAKEET_BLOB: &[u8] = b"parakeet-test-blob";
const PARAKEET_SHA256: &str = "0b73fc4fa437d2d3c146f9aa3dbf7f3b538e130ba3d0aa69668a0cc8995729b9";

#[test]
fn ct_checksum() {
    let dir = tempdir().unwrap();
    let err = install_bytes(PARAKEET_ENGINE_ID, dir.path(), PARAKEET_BLOB, "deadbeef").unwrap_err();
    assert_eq!(err.code(), "CHECKSUM_MISMATCH");
    assert!(
        !parakeet_weights_path(dir.path()).exists(),
        "failed checksum must not leave weights on disk"
    );

    let result = install_bytes(
        PARAKEET_ENGINE_ID,
        dir.path(),
        PARAKEET_BLOB,
        PARAKEET_SHA256,
    )
    .expect("matching checksum");
    assert_eq!(result.sha256, PARAKEET_SHA256);
    assert!(parakeet_weights_path(dir.path()).exists());
}

#[test]
fn ct_parakeet_local() {
    let dir = tempdir().unwrap();
    let missing = transcribe_local(PARAKEET_ENGINE_ID, FIXTURE_WAV, dir.path()).unwrap_err();
    assert_eq!(missing.code(), "ENGINE_NOT_INSTALLED");

    install_bytes(
        PARAKEET_ENGINE_ID,
        dir.path(),
        PARAKEET_BLOB,
        PARAKEET_SHA256,
    )
    .expect("install");

    let engines = overlay_catalog(catalog().expect("catalog"), dir.path());
    let parakeet = engines
        .iter()
        .find(|e| e.id == PARAKEET_ENGINE_ID)
        .expect("parakeet in catalog");
    assert_eq!(parakeet.install_state, InstallState::Ready);

    let after = transcribe_local(PARAKEET_ENGINE_ID, FIXTURE_WAV, dir.path());
    match after {
        Ok(result) => assert_eq!(result.engine_id, PARAKEET_ENGINE_ID),
        Err(err) => {
            assert_ne!(err.code(), "ENGINE_NOT_INSTALLED");
            assert_ne!(err.code(), "CLOUD_DISABLED");
        }
    }

    delete_model(PARAKEET_ENGINE_ID, dir.path()).expect("delete");
    assert!(!parakeet_weights_path(dir.path()).exists());
    let engines = overlay_catalog(catalog().expect("catalog"), dir.path());
    let parakeet = engines
        .iter()
        .find(|e| e.id == PARAKEET_ENGINE_ID)
        .expect("parakeet in catalog");
    assert_eq!(parakeet.install_state, InstallState::NotInstalled);
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

const EXTRACT_SRC: &str = "This is a privileged consult. We will not send the recording to a third-party note taker. Follow up with the client on the engagement letter.";

#[test]
fn ct_summary_from_transcript() {
    let notes = extract_notes(EXTRACT_SRC).expect("extract_notes");
    let blob = format!(
        "{} {} {}",
        notes.summary.to_lowercase(),
        notes.action_items.to_lowercase(),
        notes.key_points.to_lowercase()
    );
    assert!(
        blob.contains("privileged"),
        "summary must keep the distinctive claim"
    );
    assert!(
        blob.contains("follow up") || blob.contains("engagement"),
        "action items must keep the follow-up"
    );
}

#[test]
fn ct_export_file() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let session = store
        .create_session(Some("Privilege consult".into()), "mixed")
        .unwrap();
    store.acknowledge_consent(&session.id).unwrap();
    store.start_recording(&session.id).unwrap();
    store.finalize_with_wav(&session.id, FIXTURE_WAV).unwrap();
    store.transcribe(&session.id, None).unwrap();
    let dest = dir.path().join("export.md");
    store
        .export_markdown_file(&session.id, &dest)
        .expect("export file");
    let body = fs::read_to_string(&dest).expect("read export");
    assert!(body.to_lowercase().contains("privileged"));
}

#[test]
fn ct_settings_privacy() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let privacy = store.privacy_settings().expect("privacy");
    assert_eq!(privacy.telemetry, "off");
    assert_eq!(privacy.cloud_mode, "off");
}

#[test]
fn ct_filter_date() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let old = store
        .create_session(Some("Old consult".into()), "mixed")
        .unwrap();
    let new = store
        .create_session(Some("New consult".into()), "mixed")
        .unwrap();
    store.set_created_at(&old.id, "100").expect("backdate old");
    store.set_created_at(&new.id, "500").expect("backdate new");
    let hits = store
        .search_filtered(
            &SearchFilter {
                created_from: Some("400".into()),
                created_to: Some("600".into()),
                ..Default::default()
            },
            20,
        )
        .expect("date filter");
    assert_eq!(hits.len(), 1, "expected only the in-range session");
    assert_eq!(hits[0].session_id, new.id);
}

#[test]
fn ct_tag_roundtrip() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let session = store
        .create_session(Some("Tagged consult".into()), "mixed")
        .unwrap();
    let saved = store
        .set_tags(
            &session.id,
            &["Privilege".into(), "consult".into(), " privilege ".into()],
        )
        .expect("set tags");
    assert_eq!(saved, vec!["consult".to_string(), "privilege".to_string()]);
    assert_eq!(store.list_tags(&session.id).expect("list tags"), saved);
    let hits = store
        .search_filtered(
            &SearchFilter {
                tag: Some("privilege".into()),
                ..Default::default()
            },
            20,
        )
        .expect("tag filter");
    assert!(
        hits.iter().any(|h| h.session_id == session.id),
        "tagged session must be found"
    );
}

#[test]
fn ct_keychain() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let report = store.key_report().expect("key_report");
    assert_eq!(report.key_len, 32);
    #[cfg(target_os = "macos")]
    assert_eq!(report.backend, "keychain");
    #[cfg(not(target_os = "macos"))]
    {
        assert_eq!(report.backend, "file");
        let meta = fs::metadata(dir.path().join("master.key")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(meta.permissions().mode() & 0o777, 0o600);
    }
    let first = report.fingerprint.clone();
    drop(store);
    let again = Store::open(dir.path())
        .unwrap()
        .key_report()
        .expect("reopen");
    assert_eq!(again.fingerprint, first);
    let leak = dir.path().join("audio").join("leak.wav");
    fs::create_dir_all(leak.parent().unwrap()).unwrap();
    fs::write(&leak, b"RIFF\x24\x00\x00\x00WAVEfmt leftover").unwrap();
    let n = Store::open(dir.path())
        .unwrap()
        .scrub_plaintext_temps()
        .expect("scrub");
    assert!(n >= 1);
    assert!(!leak.exists(), "plaintext WAV leftover");
}

#[test]
fn ct_retention() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let old = store
        .create_session(Some("Old meeting".into()), "mixed")
        .unwrap();
    let keep = store
        .create_session(Some("Keep meeting".into()), "mixed")
        .unwrap();
    store.acknowledge_consent(&old.id).unwrap();
    store.acknowledge_consent(&keep.id).unwrap();
    store.start_recording(&old.id).unwrap();
    store.start_recording(&keep.id).unwrap();
    store.finalize_with_wav(&old.id, FIXTURE_WAV).unwrap();
    store.finalize_with_wav(&keep.id, FIXTURE_WAV).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    store
        .set_created_at(&old.id, &(now.saturating_sub(10 * 86_400)).to_string())
        .unwrap();
    store.set_created_at(&keep.id, &now.to_string()).unwrap();
    store.set_setting("retention_days", "7").unwrap();
    let deleted = store.apply_retention().expect("retention");
    assert!(deleted >= 1);
    assert!(store.get_session(&old.id).is_err());
    assert!(store.get_session(&keep.id).is_ok());
}

#[test]
fn ct_filter_title() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let hit = store
        .create_session(Some("Privilege consult".into()), "mixed")
        .unwrap();
    let _miss = store
        .create_session(Some("Standup notes".into()), "mixed")
        .unwrap();
    let hits = store
        .search_filtered(
            &SearchFilter {
                title: Some("privilege".into()),
                ..Default::default()
            },
            20,
        )
        .expect("title filter");
    assert_eq!(hits.len(), 1, "expected only the title match");
    assert_eq!(hits[0].session_id, hit.id);
}

#[test]
fn ct_live_stop_not_fixture() {
    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let session = store
        .create_session(Some("Live take".into()), "mic")
        .unwrap();
    store.acknowledge_consent(&session.id).unwrap();
    store.start_recording(&session.id).unwrap();
    let rec_dir = store.live_dir(&session.id);
    let mut rec = ChunkedRecorder::start(&rec_dir, CaptureConfig::default()).unwrap();
    rec.write_pcm(&vec![120i16; 3_200]).unwrap();
    let live = LiveSession::injected(rec);
    store
        .finalize_live(&session.id, live)
        .expect("finalize_live");
    assert!(
        store.audio_is_ciphertext(&session.id).unwrap(),
        "live audio must be encrypted"
    );
    let mut rec =
        ChunkedRecorder::start(&dir.path().join("compare"), CaptureConfig::default()).unwrap();
    rec.write_pcm(&vec![120i16; 3_200]).unwrap();
    let captured = rec.stop().unwrap();
    assert!(is_wav(&captured.wav));
    assert_ne!(
        captured.wav.as_slice(),
        FIXTURE_WAV,
        "chunk PCM must not be CONSULT-001"
    );
}

#[test]
fn ct_fixture_audio_mismatch() {
    let dir = tempdir().unwrap();
    let sine = record_sine(200, 16_000).unwrap();
    let err = transcribe_local("fixture-replay", &sine.wav, dir.path()).unwrap_err();
    assert_eq!(err.code(), "FIXTURE_AUDIO_MISMATCH");
    assert!(err.recoverable());
}

#[test]
fn ct_hud_recording() {
    let live = sotto_lib::presence::hud_from_status("recording", 65_000);
    assert!(live.led_on);
    assert!(!live.paused);
    assert_eq!(live.clock, "01:05");
    assert_eq!(live.caption, "on this Mac");
    let paused = sotto_lib::presence::hud_from_status("paused", 1_000);
    assert!(paused.paused);
    assert!(!paused.led_on);
    assert_eq!(paused.clock, "00:01");
}

#[test]
fn ct_login_item_backend() {
    let backend = sotto_lib::presence::login_item_backend();
    assert_eq!(backend == "smappservice", cfg!(target_os = "macos"));
    assert_eq!(backend == "unsupported", !cfg!(target_os = "macos"));
}

#[test]
fn ct_hotkey_parse() {
    assert_eq!(
        sotto_lib::hotkey::parse_hotkey(" Command+Shift+Space ").unwrap(),
        "Command+Shift+Space"
    );
    let err = sotto_lib::hotkey::parse_hotkey("   ").unwrap_err();
    assert_eq!(err.code(), "HOTKEY_INVALID");
    assert_eq!(
        sotto_lib::hotkey::DEFAULT_TOGGLE,
        "CommandOrControl+Shift+Space"
    );
}

#[test]
fn ct_hotkey_mode() {
    assert_eq!(
        sotto_lib::hotkey::parse_hotkey_mode("toggle").unwrap(),
        "toggle"
    );
    assert_eq!(sotto_lib::hotkey::parse_hotkey_mode("ptt").unwrap(), "ptt");
    assert_eq!(sotto_lib::hotkey::parse_hotkey_mode("").unwrap(), "toggle");
    let err = sotto_lib::hotkey::parse_hotkey_mode("silent").unwrap_err();
    assert_eq!(err.code(), "HOTKEY_INVALID");
}

#[test]
fn ct_meeting_detect_apps() {
    let zoom = sotto_lib::meeting::classify_processes(&["Finder", "zoom.us", "Safari"]);
    assert_eq!(zoom.len(), 1);
    assert_eq!(zoom[0].kind, "zoom");
    let mixed = sotto_lib::meeting::classify_processes(&["Slack", "Microsoft Teams", "Chrome"]);
    assert_eq!(mixed.len(), 2);
    assert!(mixed.iter().any(|d| d.kind == "slack"));
    assert!(mixed.iter().any(|d| d.kind == "teams"));
    assert!(
        sotto_lib::meeting::classify_processes(&["Google Chrome", "Finder", "steam"]).is_empty()
    );
}

// ── Wave 29-30: system-audio ──────────────────────────────────────────────────

/// CT-system-tap-status
/// Off macOS: must be "unsupported".
/// On macOS: must be one of "unsupported", "needs-permission", or "available".
/// Either way it must not claim "available" without a compiled tap backend.
#[test]
fn ct_system_tap_status() {
    let status = sotto_lib::system_tap_status();
    let valid = ["unsupported", "needs-permission", "available"];
    assert!(
        valid.contains(&status),
        "system_tap_status returned unknown value: {status:?}"
    );
    #[cfg(not(target_os = "macos"))]
    assert_eq!(
        status, "unsupported",
        "off macOS must always be unsupported, got {status:?}"
    );
    #[cfg(target_os = "macos")]
    {
        assert!(
            status == "needs-permission" || status == "available",
            "macOS with a compiled tap must be needs-permission or available, got {status:?}"
        );
    }
}

/// CT-system-not-fixture
/// start_live(System) must return CAPTURE_UNSUPPORTED (recoverable) when no
/// tap is available, AND must never write or return CONSULT-001 bytes.
/// On macOS with a real tap this test accepts Ok(_) — the live session bytes
/// will differ from the fixture by construction.
#[test]
fn ct_system_not_fixture() {
    let dir = tempdir().unwrap();
    match start_live(CaptureSource::System, dir.path()) {
        Err(err) => {
            // No tap available: must be the canonical recoverable error.
            assert_eq!(
                err.code(),
                "CAPTURE_UNSUPPORTED",
                "System error must be CAPTURE_UNSUPPORTED, got {}",
                err.code()
            );
            assert!(err.recoverable(), "CAPTURE_UNSUPPORTED must be recoverable");
        }
        Ok(session) => {
            // A real tap opened. Consume it and prove the bytes ≠ CONSULT-001.
            let result = session.finish().expect("finish live session");
            assert_ne!(
                result.wav.as_slice(),
                FIXTURE_WAV,
                "system tap must not return CONSULT-001 fixture bytes"
            );
        }
    }
}

#[test]
fn ct_meeting_never_silent() {
    let apps = sotto_lib::meeting::classify_processes(&["zoom.us"]);
    assert!(sotto_lib::meeting::should_prompt(&apps, false, true));
    assert!(!sotto_lib::meeting::should_prompt(&apps, true, true));
    assert!(!sotto_lib::meeting::should_prompt(&apps, false, false));
    assert!(!sotto_lib::meeting::should_prompt(&[], false, true));
    let copy = sotto_lib::meeting::prompt_copy(&apps);
    assert!(copy.contains("Zoom"));
    assert!(copy.contains("consent"));

    let dir = tempdir().unwrap();
    let store = Store::open(dir.path()).unwrap();
    let session = store
        .create_session(Some("Detected meeting".into()), "mic")
        .unwrap();
    let err = store.start_recording(&session.id).unwrap_err();
    assert_eq!(err.code(), "CONSENT_REQUIRED");
    assert_eq!(store.get_setting("meeting_detect").unwrap(), None);
}
