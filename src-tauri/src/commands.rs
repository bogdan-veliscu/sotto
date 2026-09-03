use std::collections::HashMap;
use std::sync::Mutex;

use serde::Deserialize;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::capture::{start_live, CaptureSource, LiveSession};
use crate::engines::Engine;
use crate::error::{ErrorBody, SottoError};
use crate::hotkey::{self, HotkeyView};
use crate::meeting::{self, DetectedMeeting};
use crate::presence::{hud_from_status, hud_view, login_item_backend, HudView};
use crate::store::{RecoverableCapture, SearchHit, Session, SessionDetail, Store};

pub struct AppState {
    pub store: Mutex<Store>,
    pub live: Mutex<HashMap<String, LiveSession>>,
}

fn map_err(err: crate::error::SottoError) -> ErrorBody {
    err.into()
}

fn push_hud(app: &AppHandle, status: &str, elapsed_ms: u64) {
    push_hud_view(app, hud_from_status(status, elapsed_ms));
}

fn push_hud_session(app: &AppHandle, session: &crate::store::Session, level: u8, elapsed_ms: u64) {
    push_hud_view(
        app,
        hud_view(
            &session.status,
            elapsed_ms,
            &session.source,
            &session.title,
            &session.id,
            level,
        ),
    );
}

fn push_hud_view(app: &AppHandle, view: HudView) {
    let _ = app.emit("sotto://hud", &view);
    if let Some(w) = app.get_webview_window("hud") {
        if view.led_on || view.paused {
            if let Ok(Some(monitor)) = w.primary_monitor() {
                let size = monitor.size();
                let scale = monitor.scale_factor();
                let width = 372.0 * scale;
                let x = (f64::from(size.width) - width) / 2.0;
                let y = 8.0 * scale;
                let _ = w.set_size(tauri::LogicalSize::new(372.0, 58.0));
                let _ = w.set_position(tauri::PhysicalPosition::new(
                    x.round() as i32,
                    y.round() as i32,
                ));
            }
            let _ = w.show();
        } else {
            let _ = w.hide();
        }
    }
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

fn recovery_blocked(state: &AppState, session_id: &str) -> Option<ErrorBody> {
    if state
        .live
        .lock()
        .ok()
        .map(|live| live.contains_key(session_id))
        .unwrap_or(false)
    {
        Some(
            SottoError::app(
                "RECOVERY_LIVE",
                "That recording is still running in this process.",
                true,
                "Stop it from the desk instead of recovering.",
            )
            .into(),
        )
    } else {
        None
    }
}

#[tauri::command]
pub fn recovery_list(state: State<AppState>) -> Result<Vec<RecoverableCapture>, ErrorBody> {
    let live_ids: Vec<String> = state.live.lock().unwrap().keys().cloned().collect();
    let mut found = state
        .store
        .lock()
        .unwrap()
        .list_recoverable()
        .map_err(map_err)?;
    found.retain(|item| !live_ids.iter().any(|id| id == &item.session_id));
    Ok(found)
}

#[tauri::command]
pub fn recovery_recover(state: State<AppState>, session_id: String) -> Result<Session, ErrorBody> {
    if let Some(err) = recovery_blocked(&state, &session_id) {
        return Err(err);
    }
    state
        .store
        .lock()
        .unwrap()
        .recover_live(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recovery_discard(state: State<AppState>, session_id: String) -> Result<Session, ErrorBody> {
    if let Some(err) = recovery_blocked(&state, &session_id) {
        return Err(err);
    }
    state
        .store
        .lock()
        .unwrap()
        .discard_live(&session_id)
        .map_err(map_err)
}

#[tauri::command]
pub fn recorder_start(state: State<AppState>, args: StartArgs) -> Result<Session, ErrorBody> {
    let store = state.store.lock().unwrap();
    let source = crate::capture::CaptureSource::try_parse(args.source.as_deref().unwrap_or("mic"))
        .map_err(map_err)?;
    let session = store
        .create_session(args.title, source.as_str())
        .map_err(map_err)?;
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
pub fn recorder_begin(
    app: AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<Session, ErrorBody> {
    let (source, dir) = {
        let store = state.store.lock().unwrap();
        let session = store.get_session(&session_id).map_err(map_err)?;
        (
            CaptureSource::try_parse(&session.source).map_err(map_err)?,
            store.live_dir(&session_id),
        )
    };
    let live = start_live(source, &dir).map_err(map_err)?;
    let session = state
        .store
        .lock()
        .unwrap()
        .start_recording(&session_id)
        .map_err(map_err)?;
    state.live.lock().unwrap().insert(session_id, live);
    push_hud_session(&app, &session, 0, 0);
    Ok(session)
}

#[tauri::command]
pub fn recorder_pause(
    app: AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<Session, ErrorBody> {
    if let Some(live) = state.live.lock().unwrap().get(&session_id) {
        live.pause().map_err(map_err)?;
    }
    let session = state
        .store
        .lock()
        .unwrap()
        .pause_recording(&session_id)
        .map_err(map_err)?;
    let level = state
        .live
        .lock()
        .unwrap()
        .get(&session_id)
        .map(|live| live.level())
        .unwrap_or(0);
    push_hud_session(&app, &session, level, 0);
    Ok(session)
}

#[tauri::command]
pub fn recorder_resume(
    app: AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<Session, ErrorBody> {
    if let Some(live) = state.live.lock().unwrap().get(&session_id) {
        live.resume().map_err(map_err)?;
    }
    let session = state
        .store
        .lock()
        .unwrap()
        .resume_recording(&session_id)
        .map_err(map_err)?;
    let level = state
        .live
        .lock()
        .unwrap()
        .get(&session_id)
        .map(|live| live.level())
        .unwrap_or(0);
    push_hud_session(&app, &session, level, 0);
    Ok(session)
}

#[tauri::command]
pub fn recorder_stop(
    app: AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<Session, ErrorBody> {
    let live = state
        .live
        .lock()
        .unwrap()
        .remove(&session_id)
        .ok_or_else(|| {
            map_err(SottoError::app(
                "CAPTURE_NOT_STARTED",
                "No live capture is running for this session.",
                true,
                "Start a recording from the desk. Stop never falls back to the golden fixture.",
            ))
        })?;
    let session = state
        .store
        .lock()
        .unwrap()
        .finalize_live(&session_id, live)
        .map_err(map_err)?;
    push_hud(&app, "idle", 0);
    Ok(session)
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
pub async fn transcribe_run(
    state: State<'_, AppState>,
    session_id: String,
    model_id: Option<String>,
) -> Result<SessionDetail, ErrorBody> {
    let job = state
        .store
        .lock()
        .unwrap()
        .prepare_transcribe(&session_id, model_id)
        .map_err(map_err)?;
    let result = tauri::async_runtime::spawn_blocking(move || crate::stt::transcribe_job(job))
        .await
        .map_err(|_| {
            map_err(SottoError::app(
                "ENGINE_WORKER_FAILED",
                "Transcription worker did not finish.",
                true,
                "Try again. Audio already captured stays on this Mac.",
            ))
        })?
        .map_err(map_err)?;
    state
        .store
        .lock()
        .unwrap()
        .commit_transcript(&session_id, &result)
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
pub fn model_import_local(
    state: State<AppState>,
    engine_id: String,
    path: String,
) -> Result<crate::install::InstallResult, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .import_model_path(&engine_id, std::path::Path::new(&path))
        .map_err(map_err)
}

#[tauri::command]
pub async fn model_download_parakeet(
    app: AppHandle,
    state: State<'_, AppState>,
    variant: String,
) -> Result<crate::install::InstallResult, ErrorBody> {
    let dir = state.store.lock().unwrap().data_dir();
    let emit_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::install::download_parakeet_http(&dir, &variant, &|progress| {
            let _ = emit_app.emit("sotto://model-download", progress);
        })
    })
    .await
    .map_err(|_| {
        map_err(SottoError::app(
            "DOWNLOAD_FAILED",
            "Parakeet download worker did not finish.",
            true,
            "Try again. The previous TDT folder was not changed.",
        ))
    })?
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

#[derive(Serialize)]
pub struct LoginItemReport {
    pub backend: String,
    pub requested: bool,
    pub applied: bool,
}

#[tauri::command]
pub fn presence_login_get(state: State<AppState>) -> Result<LoginItemReport, ErrorBody> {
    let requested = state
        .store
        .lock()
        .unwrap()
        .get_setting("launch_at_login")
        .map_err(map_err)?
        .as_deref()
        == Some("on");
    Ok(LoginItemReport {
        backend: login_item_backend().to_string(),
        requested,
        applied: requested && login_item_backend() == "smappservice",
    })
}

#[tauri::command]
pub fn presence_login_set(
    app: AppHandle,
    state: State<AppState>,
    enabled: bool,
) -> Result<LoginItemReport, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .set_setting("launch_at_login", if enabled { "on" } else { "off" })
        .map_err(map_err)?;
    let mut applied = false;
    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_autostart::ManagerExt;
        let auto = app.autolaunch();
        if enabled {
            auto.enable().map_err(|e| {
                map_err(SottoError::app(
                    "PRESENCE_UNSUPPORTED",
                    format!("Could not register the login item ({e})"),
                    true,
                    "Allow Sotto in Login Items & Extensions. Recording still starts only after consent.",
                ))
            })?;
            applied = true;
        } else {
            let _ = auto.disable();
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = &app;
        if enabled {
            return Err(map_err(SottoError::app(
                "PRESENCE_UNSUPPORTED",
                "Login item is macOS only.",
                true,
                "The preference is saved. On this Mac, Sotto can open at login.",
            )));
        }
    }
    Ok(LoginItemReport {
        backend: login_item_backend().to_string(),
        requested: enabled,
        applied,
    })
}

#[tauri::command]
pub fn presence_hud(status: String, elapsed_ms: u64) -> HudView {
    hud_from_status(&status, elapsed_ms)
}

#[tauri::command]
pub fn hud_tick(state: State<AppState>) -> HudView {
    let live_map = state.live.lock().unwrap();
    let Some((id, live)) = live_map.iter().next() else {
        return hud_from_status("idle", 0);
    };
    let level = live.level();
    let id = id.clone();
    drop(live_map);
    let session = match state.store.lock().unwrap().get_session(&id) {
        Ok(session) => session,
        Err(_) => return hud_from_status("idle", 0),
    };
    hud_view(
        &session.status,
        0,
        &session.source,
        &session.title,
        &session.id,
        level,
    )
}

/// Register the stored shortcut. Tests never call this (no OS grab).
pub(crate) fn apply_hotkey(app: &AppHandle) -> Result<(), ErrorBody> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

    let view = {
        let state = app.state::<AppState>();
        let store = state.store.lock().unwrap();
        hotkey::view(&store).map_err(map_err)?
    };
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    crate::fn_tap::arm(app);
    if hotkey::is_fn_shortcut(&view.shortcut) {
        return Ok(());
    }
    gs.on_shortcut(view.shortcut.as_str(), move |app, _shortcut, event| {
        let mode = {
            let app_state = app.state::<AppState>();
            let store = app_state.store.lock().unwrap();
            hotkey::stored_mode(&store).unwrap_or_else(|_| hotkey::DEFAULT_MODE.to_string())
        };
        let state = if event.state == ShortcutState::Pressed {
            "pressed"
        } else {
            "released"
        };
        let live_empty = {
            let app_state = app.state::<AppState>();
            app_state.live.lock().map(|m| m.is_empty()).unwrap_or(true)
        };
        if live_empty {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }
        let _ = app.emit(
            "sotto://hotkey",
            serde_json::json!({ "mode": mode, "state": state }),
        );
    })
    .map_err(|e| {
        map_err(SottoError::app(
            "HOTKEY_INVALID",
            format!("Could not register that shortcut ({e})"),
            true,
            "Pick a modifier plus a key that is not already used by the system.",
        ))
    })?;
    Ok(())
}

#[tauri::command]
pub fn hotkey_get(state: State<AppState>) -> Result<HotkeyView, ErrorBody> {
    let store = state.store.lock().unwrap();
    hotkey::view(&store).map_err(map_err)
}

#[tauri::command]
pub fn hotkey_set(
    app: AppHandle,
    state: State<AppState>,
    shortcut: String,
    mode: String,
) -> Result<HotkeyView, ErrorBody> {
    let shortcut = hotkey::parse_hotkey(&shortcut).map_err(map_err)?;
    let mode = hotkey::parse_hotkey_mode(&mode).map_err(map_err)?;
    let previous = {
        let store = state.store.lock().unwrap();
        hotkey::view(&store).map_err(map_err)?
    };
    {
        let store = state.store.lock().unwrap();
        store
            .set_setting("hotkey_toggle", &shortcut)
            .map_err(map_err)?;
        store.set_setting("hotkey_mode", &mode).map_err(map_err)?;
    }
    if let Err(err) = apply_hotkey(&app) {
        let store = state.store.lock().unwrap();
        let _ = store.set_setting("hotkey_toggle", &previous.shortcut);
        let _ = store.set_setting("hotkey_mode", &previous.mode);
        return Err(err);
    }
    Ok(HotkeyView { shortcut, mode })
}

#[derive(Serialize)]
pub struct MeetingDetectReport {
    pub enabled: bool,
    pub detected: Vec<DetectedMeeting>,
    pub should_prompt: bool,
    pub prompt: String,
}

fn meeting_report(state: &AppState) -> Result<MeetingDetectReport, ErrorBody> {
    let recording = !state.live.lock().unwrap().is_empty();
    let enabled = {
        let store = state.store.lock().unwrap();
        meeting::detect_enabled(&store).map_err(map_err)?
    };
    let detected = meeting::classify_processes(&meeting::list_process_names());
    let should_prompt = meeting::should_prompt(&detected, recording, enabled);
    let prompt = if should_prompt {
        meeting::prompt_copy(&detected)
    } else {
        String::new()
    };
    Ok(MeetingDetectReport {
        enabled,
        detected,
        should_prompt,
        prompt,
    })
}

#[tauri::command]
pub fn meeting_detect_get(state: State<AppState>) -> Result<MeetingDetectReport, ErrorBody> {
    meeting_report(&*state)
}

#[tauri::command]
pub fn meeting_detect_set(
    state: State<AppState>,
    enabled: bool,
) -> Result<MeetingDetectReport, ErrorBody> {
    state
        .store
        .lock()
        .unwrap()
        .set_setting("meeting_detect", if enabled { "on" } else { "off" })
        .map_err(map_err)?;
    meeting_report(&*state)
}

#[tauri::command]
pub fn system_tap_get() -> String {
    crate::system_tap_status().to_string()
}

#[tauri::command]
pub fn parakeet_runtime_get() -> String {
    crate::parakeet_runtime_status().to_string()
}

#[tauri::command]
pub fn capture_source_hint(source: String) -> Result<String, ErrorBody> {
    let src = crate::capture::CaptureSource::try_parse(&source).map_err(map_err)?;
    Ok(crate::capture::source_permission_hint(
        src,
        crate::system_tap_status(),
    ))
}
