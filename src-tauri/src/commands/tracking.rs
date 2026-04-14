use crate::modules::activity_monitor::TrackingState;
use crate::AppState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn pause_tracking(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.pause();
    drop(monitor);
    let _ = app.emit("tracking-state-changed", "paused");
    Ok(())
}

#[tauri::command]
pub fn resume_tracking(app: AppHandle, state: State<AppState>) -> Result<(), String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.resume();
    drop(monitor);
    let _ = app.emit("tracking-state-changed", "running");
    Ok(())
}

#[tauri::command]
pub fn get_tracking_state(state: State<AppState>) -> Result<String, String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    let s = match monitor.current_state() {
        TrackingState::Running => "running",
        TrackingState::Paused => "paused",
        TrackingState::Idle => "idle",
    };
    Ok(s.to_string())
}

#[tauri::command]
pub fn get_current_activity(state: State<AppState>) -> Result<Option<crate::modules::activity_monitor::ActivitySnapshot>, String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    Ok(monitor.last_snapshot())
}
