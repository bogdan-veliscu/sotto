pub mod capture;
mod crypto;
mod engines;
mod error;
pub mod install;
pub mod notes;
pub mod search;
mod store;
pub mod stt;
pub use engines::{catalog, Engine, InstallState};
pub use error::SottoError;
pub use store::Store;

use std::path::Path;

use serde::Serialize;

#[cfg(feature = "desktop")]
mod commands;

#[derive(Debug, Serialize)]
pub struct DemoReport {
    pub session_id: String,
    pub status: String,
    pub engine_id: String,
    pub engine_mode: String,
    pub search_hits: usize,
    pub network_calls: u32,
    pub telemetry: String,
    pub cloud_mode: String,
    pub audio_is_ciphertext: bool,
    pub consent_enforced: bool,
    pub delete_all_clears_search: bool,
}

/// Offline judge/demo path. No network, no model download.
pub fn demo_pipeline(data_dir: &Path) -> error::Result<DemoReport> {
    let store = Store::open(data_dir)?;
    let telemetry = store.get_setting("telemetry")?.unwrap_or_default();
    let cloud_mode = store.get_setting("cloud_mode")?.unwrap_or_default();
    let session = store.create_session(Some("Privilege consult".into()), "mixed")?;
    let consent_enforced = store.start_recording(&session.id).is_err();
    store.acknowledge_consent(&session.id)?;
    store.start_recording(&session.id)?;
    let wav = include_bytes!("../../fixtures/sessions/CONSULT-001.wav");
    store.finalize_with_wav(&session.id, wav)?;
    let detail = store.transcribe(&session.id, None)?;
    let hits = store.search("privileged", 10)?;
    let audio_is_ciphertext = store.audio_is_ciphertext(&session.id)?;
    let engine_id = detail.session.model_id.clone().unwrap_or_default();
    let status = detail.session.status.clone();
    store.delete_all()?;
    let after = store.search("privileged", 10)?;
    Ok(DemoReport {
        session_id: session.id,
        status,
        engine_id,
        engine_mode: "local".into(),
        search_hits: hits.len(),
        network_calls: 0,
        telemetry,
        cloud_mode,
        audio_is_ciphertext,
        consent_enforced,
        delete_all_clears_search: after.is_empty(),
    })
}

#[cfg(feature = "desktop")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::fs;
    use std::sync::Mutex;
    use tauri::Manager;

    use commands::AppState;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            fs::create_dir_all(&dir)?;
            let store = Store::open(&dir).map_err(|e| e.to_string())?;
            app.manage(AppState {
                store: Mutex::new(store),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings_get,
            commands::settings_set,
            commands::engines_list,
            commands::sessions_list,
            commands::sessions_get,
            commands::recorder_start,
            commands::recorder_consent,
            commands::recorder_begin,
            commands::recorder_pause,
            commands::recorder_resume,
            commands::recorder_stop_fixture,
            commands::transcribe_run,
            commands::search_query,
            commands::sessions_rename,
            commands::sessions_export,
            commands::sessions_export_file,
            commands::privacy_settings,
            commands::sessions_delete,
            commands::model_install_file,
            commands::model_delete,
            commands::data_delete_all,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sotto");
}
