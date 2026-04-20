use crate::modules::attribution::{merge_adjacent_sessions, MergedSession};
use crate::modules::session_store::{self, CreateSessionInput, Session, UpdateSessionInput};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_sessions_for_day(
    state: State<AppState>,
    date: String,
) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session_store::list_sessions_for_day(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_merged_sessions_for_day(
    state: State<AppState>,
    date: String,
) -> Result<Vec<MergedSession>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let sessions = session_store::list_sessions_for_day(&conn, &date)
        .map_err(|e| e.to_string())?;
    Ok(merge_adjacent_sessions(&sessions))
}

#[tauri::command]
pub fn list_unpublished_for_day(
    state: State<AppState>,
    date: String,
) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session_store::list_unpublished_for_day(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_session(
    state: State<AppState>,
    id: String,
    input: UpdateSessionInput,
) -> Result<Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session_store::update_session(&conn, &id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_session(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session_store::delete_session(&conn, &id).map_err(|e| e.to_string())
}

/// Create a manual tracking session with a user-supplied label.
/// The session is left open (end_time = NULL) until the user stops it.
#[tauri::command]
pub fn start_manual_session(state: State<AppState>, label: String) -> Result<Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let input = CreateSessionInput {
        project_id: None,
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: None,
        duration_secs: 0,
        jira_key: None,
        branch: None,
        window_title: Some(label),
        process_name: None,
        is_idle: false,
        is_huddle: false,
        huddle_channel: None,
        is_manual: true,
    };
    session_store::create_session(&conn, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_sessions_for_range(
    state: State<AppState>,
    from: String,
    to: String,
) -> Result<Vec<Session>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session_store::list_sessions_for_range(&conn, &from, &to).map_err(|e| e.to_string())
}


#[tauri::command]
pub fn set_session_logged(
    state: State<AppState>,
    id: String,
    logged: bool,
) -> Result<Session, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    session_store::set_session_logged(&conn, &id, logged).map_err(|e| e.to_string())
}
