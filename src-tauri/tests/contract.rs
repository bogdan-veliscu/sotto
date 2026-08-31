use std::fs;

use sotto_lib::demo_pipeline;

#[test]
fn demo_pipeline_holds_privacy_invariants() {
    let dir = tempfile::tempdir().unwrap();
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
    // After delete_all the data dir audio folder has no leftover WAV.
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
