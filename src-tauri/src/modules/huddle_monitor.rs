//! Background task that detects active Slack Huddle windows and records
//! call-duration sessions independently of the foreground activity monitor.
//!
//! Design:
//! - Polls `platform::get_huddle_window()` every `poll_secs`.
//! - On huddle start → creates a session with `is_huddle = true`.
//! - While in huddle → updates the session duration (total elapsed, not delta).
//! - On huddle end → finalises the session; discards it if < MIN_SESSION_SECS.
//! - Emits `"huddle-status"` events so the UI can display a live call card.

use crate::modules::session_store::{self, CreateSessionInput, UpdateSessionInput};
use crate::AppState;
use crate::MIN_SESSION_SECS;
use chrono::Utc;
use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

/// Payload emitted on the `"huddle-status"` event.
#[derive(Debug, Clone, Serialize)]
pub struct HuddleStatus {
    pub active: bool,
    pub channel: Option<String>,
    pub window_title: Option<String>,
    pub elapsed_secs: i64,
}

/// Spawn the huddle monitoring loop inside Tauri's async runtime.
/// `poll_secs` controls how often windows are scanned (minimum 2 s).
pub fn start(app_handle: AppHandle, poll_secs: u64) {
    tauri::async_runtime::spawn(async move {
        let poll = Duration::from_secs(poll_secs.max(2));
        let mut current_session_id: Option<String> = None;
        let mut session_start_time: Option<chrono::DateTime<Utc>> = None;

        loop {
            tokio::time::sleep(poll).await;

            let now = Utc::now();
            let state = app_handle.state::<AppState>();

            // Respect the user's toggle — if disabled, finalize any open session and idle.
            let tracking_enabled = {
                let db = state.db.lock().unwrap();
                crate::modules::settings_manager::load(&db)
                    .map(|s| s.track_slack_huddles)
                    .unwrap_or(true)
            };
            if !tracking_enabled {
                if let Some(id) = current_session_id.take() {
                    let dur = session_start_time
                        .take()
                        .map(|start| (now - start).num_seconds().max(0))
                        .unwrap_or(0);
                    let db = state.db.lock().unwrap();
                    if dur >= MIN_SESSION_SECS {
                        let update = crate::modules::session_store::UpdateSessionInput {
                            end_time: Some(now.to_rfc3339()),
                            duration_secs: Some(dur),
                            ..Default::default()
                        };
                        let _ = session_store::update_session(&db, &id, update);
                    } else {
                        let _ = session_store::delete_session(&db, &id);
                    }
                }
                continue;
            }

            let huddle = crate::platform::get_huddle_window();

            if let Some(ref info) = huddle {
                // ── Huddle is active ──────────────────────────────────────────
                let elapsed = session_start_time
                    .map(|start| (now - start).num_seconds().max(0))
                    .unwrap_or(0);

                let _ = app_handle.emit(
                    "huddle-status",
                    HuddleStatus {
                        active: true,
                        channel: info.channel.clone(),
                        window_title: Some(info.window_title.clone()),
                        elapsed_secs: elapsed,
                    },
                );

                if current_session_id.is_none() {
                    // Start a new huddle session
                    let input = CreateSessionInput {
                        project_id: None,
                        start_time: now.to_rfc3339(),
                        end_time: None,
                        duration_secs: 0,
                        jira_key: None,
                        branch: None,
                        window_title: Some(info.window_title.clone()),
                        process_name: Some("slack".to_string()),
                        is_idle: false,
                        is_huddle: true,
                        huddle_channel: info.channel.clone(),
                    };
                    let db = state.db.lock().unwrap();
                    if let Ok(session) = session_store::create_session(&db, input) {
                        current_session_id = Some(session.id);
                        session_start_time = Some(now);
                    }
                } else if let Some(ref id) = current_session_id {
                    // Update ongoing huddle session with total elapsed time
                    if let Some(start) = session_start_time {
                        let dur = (now - start).num_seconds().max(0);
                        let update = UpdateSessionInput {
                            end_time: Some(now.to_rfc3339()),
                            duration_secs: Some(dur),
                            ..Default::default()
                        };
                        let db = state.db.lock().unwrap();
                        let _ = session_store::update_session(&db, id, update);
                    }
                }
            } else {
                // ── Huddle ended (or never started) ──────────────────────────
                let _ = app_handle.emit(
                    "huddle-status",
                    HuddleStatus {
                        active: false,
                        channel: None,
                        window_title: None,
                        elapsed_secs: 0,
                    },
                );

                if let Some(id) = current_session_id.take() {
                    let dur = session_start_time
                        .take()
                        .map(|start| (now - start).num_seconds().max(0))
                        .unwrap_or(0);

                    let db = state.db.lock().unwrap();
                    if dur >= MIN_SESSION_SECS {
                        let update = UpdateSessionInput {
                            end_time: Some(now.to_rfc3339()),
                            duration_secs: Some(dur),
                            ..Default::default()
                        };
                        let _ = session_store::update_session(&db, &id, update);
                    } else {
                        // Stub row below minimum — remove it
                        let _ = session_store::delete_session(&db, &id);
                    }
                }
            }
        }
    });
}
