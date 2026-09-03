//! macOS Fn / Globe key. Tap toggles, hold is press-to-talk.
//! Tests never install an event monitor.

#![cfg(feature = "desktop")]

use tauri::AppHandle;

#[cfg(target_os = "macos")]
mod macos {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    use tauri::{AppHandle, Emitter, Manager};

    use crate::hotkey::FN_HOLD_MS;

    static ARMED: AtomicBool = AtomicBool::new(false);
    static FN_DOWN: AtomicBool = AtomicBool::new(false);
    static PTT_ARMED: AtomicBool = AtomicBool::new(false);
    static APP: OnceLock<AppHandle> = OnceLock::new();

    extern "C" {
        fn sotto_fn_tap_start(cb: extern "C" fn(i32)) -> i32;
    }

    extern "C" fn on_fn_bit(pressed: i32) {
        on_fn_edge(pressed != 0);
    }

    fn emit_hotkey(mode: &str, state: &str) {
        let Some(app) = APP.get() else {
            return;
        };
        let live_empty = app
            .state::<crate::commands::AppState>()
            .live
            .lock()
            .map(|m| m.is_empty())
            .unwrap_or(true);
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
    }

    fn on_fn_edge(pressed: bool) {
        if pressed {
            FN_DOWN.store(true, Ordering::SeqCst);
            PTT_ARMED.store(false, Ordering::SeqCst);
            thread::spawn(|| {
                thread::sleep(Duration::from_millis(FN_HOLD_MS));
                if FN_DOWN.load(Ordering::SeqCst) {
                    PTT_ARMED.store(true, Ordering::SeqCst);
                    emit_hotkey("ptt", "pressed");
                }
            });
            return;
        }
        FN_DOWN.store(false, Ordering::SeqCst);
        if PTT_ARMED.swap(false, Ordering::SeqCst) {
            emit_hotkey("ptt", "released");
        } else {
            emit_hotkey("toggle", "pressed");
        }
    }

    pub fn arm(app: &AppHandle) {
        let _ = APP.set(app.clone());
        if ARMED.swap(true, Ordering::SeqCst) {
            return;
        }
        let rc = unsafe { sotto_fn_tap_start(on_fn_bit) };
        if rc != 1 {
            ARMED.store(false, Ordering::SeqCst);
            eprintln!("sotto: Fn key tap was not armed");
        }
    }
}

#[cfg(target_os = "macos")]
pub fn arm(app: &AppHandle) {
    macos::arm(app);
}

#[cfg(not(target_os = "macos"))]
pub fn arm(app: &AppHandle) {
    let _ = app;
}
