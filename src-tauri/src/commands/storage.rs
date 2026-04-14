use crate::AppState;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct StorageInfo {
    pub db_size_bytes: u64,
    pub session_count: i64,
}

/// Returns the on-disk DB file size and total number of recorded sessions.
#[tauri::command]
pub fn get_storage_info(state: State<AppState>) -> Result<StorageInfo, String> {
    let db_path = state.data_dir.join("timetracker.db");
    let db_size_bytes = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let session_count: i64 = db
        .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
        .unwrap_or(0);

    Ok(StorageInfo { db_size_bytes, session_count })
}

/// Deletes all session rows (settings and projects are preserved).
#[tauri::command]
pub fn erase_sessions(state: State<AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute("DELETE FROM sessions", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
