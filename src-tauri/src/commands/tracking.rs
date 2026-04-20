use crate::modules::activity_monitor::TrackingState;
use crate::modules::session_store;
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

/// Stops the live auto-tracked session for one specific (project, branch) pair
/// without pausing the global ActivityMonitor.  Other simultaneously-tracked
/// sessions (different projects or different branches) keep running.
///
/// The caller passes the `session_id` directly (taken from `MergedSession.session_ids[0]`)
/// so the close is a definitive lookup by primary key — no ambiguous search.
/// `duration_secs` is the frontend-computed effective duration (real elapsed minus
/// any accumulated pause time), so paused intervals are excluded from the record.
///
/// After closing the session the key is snoozed for 5 minutes so the tracking
/// loop won't immediately create a replacement session while the IDE is still
/// open.
#[tauri::command]
pub fn stop_live_session(
    app: AppHandle,
    state: State<AppState>,
    session_id: String,
    project_id: String,
    branch: Option<String>,
    duration_secs: i64,
) -> Result<(), String> {
    let key = crate::session_key(&project_id, branch.as_deref());
    let now_str = chrono::Utc::now().to_rfc3339();

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if duration_secs >= crate::MIN_SESSION_SECS {
            let update = session_store::UpdateSessionInput {
                end_time: Some(now_str),
                duration_secs: Some(duration_secs),
                ..Default::default()
            };
            session_store::update_session(&db, &session_id, update)
                .map_err(|e| e.to_string())?;
        } else {
            session_store::delete_session(&db, &session_id)
                .map_err(|e| e.to_string())?;
        }
    }

    // Snooze so the tracking loop doesn't immediately recreate the session.
    {
        let mut snoozed = state.snoozed_keys.lock().map_err(|e| e.to_string())?;
        snoozed.insert(key, std::time::Instant::now());
    }

    // Notify the frontend so it can refresh the session list.
    let _ = app.emit("tracking-state-changed", "running");
    Ok(())
}

/// Removes the snooze for a (project, branch) pair so the tracking loop can
/// resume creating sessions for it.  Call this when the user clicks
/// "Start new session" on a manually-stopped project card.
#[tauri::command]
pub fn resume_tracked_project(
    state: State<AppState>,
    project_id: String,
    branch: Option<String>,
) -> Result<(), String> {
    let key = crate::session_key(&project_id, branch.as_deref());
    let mut snoozed = state.snoozed_keys.lock().map_err(|e| e.to_string())?;
    snoozed.remove(&key);
    Ok(())
}

#[tauri::command]
pub fn get_current_activity(state: State<AppState>) -> Result<Option<crate::modules::activity_monitor::ActivitySnapshot>, String> {
    let monitor = state.monitor.lock().map_err(|e| e.to_string())?;
    Ok(monitor.last_snapshot())
}
