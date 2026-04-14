use crate::modules::settings_manager::{self, AppSettings};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    settings_manager::load(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    settings_manager::save(&conn, &settings).map_err(|e| e.to_string())?;

    // Update monitor config live
    let mut monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    monitor.config.idle_threshold_secs = settings.idle_threshold_secs;
    monitor.config.poll_interval_secs = settings.poll_interval_secs;

    Ok(())
}
