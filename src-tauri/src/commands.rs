use std::sync::Mutex;

use serde::Deserialize;
use tauri::State;

use crate::engines::Engine;
use crate::error::ErrorBody;
use crate::store::{SearchHit, Session, SessionDetail, Store};

pub struct AppState {
    pub store: Mutex<Store>,
}

fn map_err(err: crate::error::SottoError) -> ErrorBody {
    err.into()
}

#[derive(Deserialize)]
pub struct StartArgs {
    pub source: Option<String>,
    pub title: Option<String>,
}

#[tauri::command]
pub fn settings_get(state: State<AppState>, key: String) -> Result<Option<String>, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .get_setting(&key)
        .map_err(map_err)
}

#[tauri::command]
pub fn settings_set(state: State<AppState>, key: String, value: String) -> Result<(), ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .set_setting(&key, &value)
        .map_err(map_err)
}

#[tauri::command]
pub fn engines_list(state: State<AppState>) -> Result<Vec<Engine>, ErrorBody> {
    state.store.lock().unwrap().list_engines().map_err(map_err)
}

#[tauri::command]
pub fn sessions_list(
    state: State<AppState>,
    limit: Option<i64>,
) -> Result<Vec<Session>, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .list_sessions(limit.unwrap_or(50))
        .map_err(map_err)
}

#[tauri::command]
pub fn sessions_get(
    state: State<AppState>,
    session_id: String,
) -> Result<SessionDetail, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .get_detail(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recorder_start(state: State<AppState>, args: StartArgs) -> Result<Session, ErrorBody> {
    let store = state.store.lock().unwrap();
    let source = args.source.unwrap_or_else(|| "mixed".into());
    let session = store.create_session(args.title, &source).map_err(map_err)?;
    Ok(session)
}

#[tauri::command]
pub fn recorder_consent(state: State<AppState>, session_id: String) -> Result<Session, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .acknowledge_consent(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recorder_begin(state: State<AppState>, session_id: String) -> Result<Session, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .start_recording(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recorder_pause(state: State<AppState>, session_id: String) -> Result<Session, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .pause_recording(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recorder_resume(state: State<AppState>, session_id: String) -> Result<Session, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .resume_recording(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recorder_stop_fixture(
    state: State<AppState>,
    session_id: String,
) -> Result<Session, ErrorBody> {
    let wav = include_bytes!("../../fixtures/sessions/CONSULT-001.wav");
    state
        .store
        .lock()
        .unwrap()
        .finalize_with_wav(&session_id, wav)
        .map_err(map_err)
}

#[tauri::command]
pub fn transcribe_run(
    state: State<AppState>,
    session_id: String,
    model_id: Option<String>,
) -> Result<SessionDetail, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .transcribe(&session_id, model_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn search_query(
    state: State<AppState>,
    q: String,
    limit: Option<i64>,
    title: Option<String>,
    created_from: Option<String>,
    created_to: Option<String>,
    tag: Option<String>,
) -> Result<Vec<SearchHit>, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .search_filtered(
            &crate::search::SearchFilter {
                q,
                title,
                created_from,
                created_to,
                tag,
            },
            limit.unwrap_or(20),
        )
        .map_err(map_err)
}

#[tauri::command]
pub fn sessions_set_tags(
    state: State<AppState>,
    session_id: String,
    tags: Vec<String>,
) -> Result<Vec<String>, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .set_tags(&session_id, &tags)
        .map_err(map_err)
}

#[tauri::command]
pub fn sessions_rename(
    state: State<AppState>,
    session_id: String,
    title: String,
) -> Result<Session, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .rename_session(&session_id, &title)
        .map_err(map_err)
}

#[tauri::command]
pub fn sessions_export(state: State<AppState>, session_id: String) -> Result<String, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .export_markdown(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn sessions_export_file(
    state: State<AppState>,
    session_id: String,
    dest: String,
) -> Result<(), ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .export_markdown_file(&session_id, std::path::Path::new(&dest))
        .map_err(map_err)
}

#[tauri::command]
pub fn privacy_settings(
    state: State<AppState>,
) -> Result<crate::notes::PrivacySettings, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .privacy_settings()
        .map_err(map_err)
}

#[tauri::command]
pub fn sessions_delete(state: State<AppState>, session_id: String) -> Result<(), ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .delete_session(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn model_install_file(
    state: State<AppState>,
    engine_id: String,
    path: String,
    expected_sha256: String,
) -> Result<crate::install::InstallResult, ErrorBody> {
    let bytes = std::fs::read(&path)
        .map_err(crate::error::SottoError::from)
        .map_err(map_err)?;
    state
        .store
        .lock()
        .unwrap()
        .install_model_bytes(&engine_id, &bytes, &expected_sha256)
        .map_err(map_err)
}

#[tauri::command]
pub fn model_delete(state: State<AppState>, engine_id: String) -> Result<(), ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .delete_installed_model(&engine_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn data_delete_all(state: State<AppState>) -> Result<(), ErrorBody> {
    state.store.lock().unwrap().delete_all().map_err(map_err)
}

#[tauri::command]
pub fn key_report(state: State<AppState>) -> Result<crate::keys::KeyReport, ErrorBody> {
    state.store.lock().unwrap().key_report().map_err(map_err)
}

#[tauri::command]
pub fn retention_apply(state: State<AppState>) -> Result<u32, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .apply_retention()
        .map_err(map_err)
}
