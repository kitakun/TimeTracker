use tauri::{AppHandle, Emitter, Manager};

/// Called by the React app once its first render is complete.
/// Hides the splashscreen window, reveals the main window, and — crucially —
/// resumes the ActivityMonitor so session recording starts only after the UI
/// is fully ready.  This prevents sessions from accumulating time during the
/// loading phase and showing stale elapsed values on first open.
#[tauri::command]
pub fn close_splashscreen(app: AppHandle) {
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }

    // Resume the monitor now that the UI is ready.
    let state = app.state::<crate::AppState>();
    if let Ok(monitor) = state.monitor.lock() {
        monitor.resume();
    }
    let _ = app.emit("tracking-state-changed", "running");
}
