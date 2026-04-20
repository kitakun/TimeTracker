mod commands;
mod db;
mod modules;
mod platform;
pub mod startup;

use modules::activity_monitor::{ActivityMonitor, TrackingState};
use modules::attribution::AttributionEngine;
use modules::project_registry;
use modules::session_store::{self, CreateSessionInput};
use modules::settings_manager;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItemBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

/// Sessions shorter than this (seconds) are discarded as accidental noise.
pub const MIN_SESSION_SECS: i64 = 30;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub monitor: Mutex<ActivityMonitor>,
    pub data_dir: std::path::PathBuf,
    /// Project+branch keys manually stopped by the user via the live-session
    /// pause button.  The tracking loop skips creating new sessions for these
    /// keys for 5 minutes so the session doesn't immediately reopen.
    /// Format: "{project_id}::{branch_or_empty}" → Instant when snoozed.
    pub snoozed_keys: Mutex<std::collections::HashMap<String, std::time::Instant>>,
}

/// Stable identifier for a (project, branch) pair used as the key in the
/// active-sessions map and the snoozed-keys set.
pub fn session_key(project_id: &str, branch: Option<&str>) -> String {
    format!("{}::{}", project_id, branch.unwrap_or(""))
}

/// Minimal bookkeeping for one currently-open tracked session.
struct ActiveSession {
    id: String,
    start_time: chrono::DateTime<chrono::Utc>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Show a visible dialog on panic instead of silently exiting.
    // Critical on Windows release builds where there is no console.
    install_panic_handler();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_single_instance::Builder::new()
                .callback(|app, _args, _cwd| {
                    // Second launch attempted – bring the existing window to front.
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.show();
                        let _ = win.unminimize();
                        let _ = win.set_focus();
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Cannot resolve app data dir: {e}"))?;

            let init = startup::try_init(&data_dir)
                .map_err(|e| format!("{e:#}"))?;

            let broadcast_tx = init.state.monitor.lock().unwrap().start();
            app.manage(init.state);

            // ── Background session recording loop ────────────────────────────
            // Use tauri::async_runtime::spawn — not tokio::spawn — so we are
            // guaranteed to be inside Tauri's managed tokio runtime.
            //
            // The loop tracks *multiple* simultaneous sessions — one per
            // (project_id, branch) pair that currently has an open IDE window.
            // This fixes two bugs from v3:
            //   • Branch change: the old key disappears → session finalised,
            //     new key appears → fresh session created automatically.
            //   • Two IDEs open: both keys are tracked independently.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = broadcast_tx.subscribe();

                // key = session_key(project_id, branch)  →  open session bookkeeping
                let mut active: std::collections::HashMap<String, ActiveSession> =
                    std::collections::HashMap::new();

                loop {
                    let snap = match rx.recv().await {
                        Ok(s) => s,
                        Err(_) => break,
                    };

                    let is_active = snap.state == TrackingState::Running;
                    let _ = app_handle.emit("activity-update", &snap);
                    let state = app_handle.state::<AppState>();

                    let snap_time = chrono::DateTime::parse_from_rfc3339(&snap.timestamp)
                        .ok()
                        .map(|t| t.with_timezone(&chrono::Utc))
                        .unwrap_or_else(chrono::Utc::now);

                    if is_active {
                        // ── Load projects + settings ──────────────────────────
                        let (projects, settings) = {
                            let db = state.db.lock().unwrap();
                            let p = project_registry::list_projects(&db).unwrap_or_default();
                            let s = settings_manager::load(&db).unwrap_or_default();
                            (p, s)
                        };
                        let engine = AttributionEngine::new(settings.jira_patterns.clone());

                        // ── Collect all current (project, branch) attributions ─
                        // We build a map keyed by session_key so that the
                        // foreground window takes priority but all background IDEs
                        // also get entries.
                        let mut current: std::collections::HashMap<
                            String,
                            (modules::attribution::Attribution,
                             Option<platform::types::ActiveWindowInfo>)
                        > = std::collections::HashMap::new();

                        // Foreground window
                        if let Some(w) = &snap.window {
                            let attr = engine.attribute(w, &projects);
                            if let Some(pid) = &attr.project_id {
                                let key = session_key(pid, attr.branch.as_deref());
                                current.insert(key, (attr, Some(w.clone())));
                            }
                        }

                        // All background IDE windows (VS Code, Rider, etc.)
                        for ide_w in platform::list_ide_windows() {
                            let attr = engine.attribute(&ide_w, &projects);
                            if let Some(pid) = &attr.project_id {
                                let key = session_key(pid, attr.branch.as_deref());
                                // Don't overwrite a foreground entry
                                current.entry(key).or_insert((attr, Some(ide_w)));
                            }
                        }

                        // ── Expire snoozed keys and filter them out ───────────
                        {
                            let mut snoozed = state.snoozed_keys.lock().unwrap();
                            snoozed.retain(|_, inst| {
                                inst.elapsed() < std::time::Duration::from_secs(300)
                            });
                            current.retain(|k, _| !snoozed.contains_key(k));
                        }

                        // ── Finalise sessions for keys that disappeared ────────
                        let disappeared: Vec<String> = active
                            .keys()
                            .filter(|k| !current.contains_key(*k))
                            .cloned()
                            .collect();

                        for key in disappeared {
                            if let Some(session) = active.remove(&key) {
                                let dur =
                                    (snap_time - session.start_time).num_seconds().max(0);
                                let db = state.db.lock().unwrap();
                                if dur >= MIN_SESSION_SECS {
                                    let update = session_store::UpdateSessionInput {
                                        end_time: Some(snap.timestamp.clone()),
                                        duration_secs: Some(dur),
                                        ..Default::default()
                                    };
                                    let _ = session_store::update_session(
                                        &db, &session.id, update,
                                    );
                                } else {
                                    let _ = session_store::delete_session(&db, &session.id);
                                }
                            }
                        }

                        // ── Heartbeat existing sessions ───────────────────────
                        // Also detect sessions that were closed externally (user
                        // deleted from UI) so we don't keep a stale active entry.
                        let mut externally_closed: Vec<String> = Vec::new();
                        for (key, session) in &active {
                            if !current.contains_key(key) {
                                continue; // will be handled as disappeared above
                            }
                            let dur = (snap_time - session.start_time).num_seconds().max(0);
                            let db = state.db.lock().unwrap();
                            let still_open: i64 = db
                                .query_row(
                                    "SELECT COUNT(*) FROM sessions \
                                     WHERE id=?1 AND end_time IS NULL",
                                    rusqlite::params![&session.id],
                                    |r| r.get(0),
                                )
                                .unwrap_or(0);
                            if still_open > 0 {
                                let update = session_store::UpdateSessionInput {
                                    duration_secs: Some(dur),
                                    ..Default::default()
                                };
                                let _ = session_store::update_session(
                                    &db, &session.id, update,
                                );
                            } else {
                                externally_closed.push(key.clone());
                            }
                        }
                        for key in externally_closed {
                            active.remove(&key);
                        }

                        // ── Create sessions for new keys ──────────────────────
                        for (key, (attr, tracking_window)) in &current {
                            if active.contains_key(key) {
                                continue; // already tracking
                            }

                            // Auto-merge: try to resume a leftover open session
                            // from the previous run (same project + same branch).
                            let maybe_resumed = if settings.auto_merge_enabled {
                                if let Some(pid) = attr.project_id.as_deref() {
                                    let db = state.db.lock().unwrap();
                                    session_store::find_resumable_session(&db, pid)
                                        .ok()
                                        .flatten()
                                        // Branch must match so we don't resume a
                                        // session from a different feature branch.
                                        .filter(|s| {
                                            s.branch.as_deref() == attr.branch.as_deref()
                                        })
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            if let Some(existing) = maybe_resumed {
                                let start =
                                    chrono::DateTime::parse_from_rfc3339(&existing.start_time)
                                        .ok()
                                        .map(|t| t.with_timezone(&chrono::Utc))
                                        .unwrap_or(snap_time);
                                active.insert(
                                    key.clone(),
                                    ActiveSession { id: existing.id, start_time: start },
                                );
                            } else {
                                let input = CreateSessionInput {
                                    project_id: attr.project_id.clone(),
                                    start_time: snap.timestamp.clone(),
                                    end_time: None,
                                    duration_secs: 0,
                                    jira_key: attr.jira_key.clone(),
                                    branch: attr.branch.clone(),
                                    window_title: tracking_window
                                        .as_ref()
                                        .map(|w| w.window_title.clone()),
                                    process_name: tracking_window
                                        .as_ref()
                                        .map(|w| w.process_name.clone()),
                                    is_idle: false,
                                    is_huddle: false,
                                    huddle_channel: None,
                                    is_manual: false,
                                };
                                let db = state.db.lock().unwrap();
                                if let Ok(session) = session_store::create_session(&db, input) {
                                    active.insert(
                                        key.clone(),
                                        ActiveSession {
                                            id: session.id,
                                            start_time: snap_time,
                                        },
                                    );
                                }
                            }
                        }
                    } else {
                        // Not active (paused/idle): finalise *all* open sessions.
                        let to_finalize: Vec<(String, ActiveSession)> =
                            active.drain().collect();
                        for (_, session) in to_finalize {
                            let dur =
                                (snap_time - session.start_time).num_seconds().max(0);
                            let db = state.db.lock().unwrap();
                            if dur >= MIN_SESSION_SECS {
                                let update = session_store::UpdateSessionInput {
                                    end_time: Some(snap.timestamp.clone()),
                                    duration_secs: Some(dur),
                                    ..Default::default()
                                };
                                let _ = session_store::update_session(
                                    &db, &session.id, update,
                                );
                            } else {
                                let _ = session_store::delete_session(&db, &session.id);
                            }
                        }
                    }
                }
            });

            // ── Slack Huddle monitoring loop ─────────────────────────────────
            modules::huddle_monitor::start(
                app.handle().clone(),
                init.monitor_config.poll_interval_secs,
            );

            // Intercept the window close button.
            // – No active sessions → hide to tray as usual.
            // – Active sessions    → emit "close-requested" so the UI can ask the user.
            let app_handle_close = app.handle().clone();
            if let Some(win) = app.get_webview_window("main") {
                win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let has_active = {
                            let state = app_handle_close.state::<AppState>();
                            let db = state.db.lock().unwrap();
                            db.query_row(
                                "SELECT COUNT(*) FROM sessions WHERE end_time IS NULL",
                                [],
                                |r| r.get::<_, i64>(0),
                            ).unwrap_or(0) > 0
                        };
                        if has_active {
                            let _ = app_handle_close.emit("close-requested", ());
                        } else if let Some(w) = app_handle_close.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                });
            }

            setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::projects::list_projects,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::delete_project,
            commands::sessions::list_sessions_for_day,
            commands::sessions::list_merged_sessions_for_day,
            commands::sessions::list_unpublished_for_day,
            commands::sessions::update_session,
            commands::sessions::delete_session,
            commands::sessions::list_sessions_for_range,
            commands::sessions::start_manual_session,
            commands::sessions::set_session_logged,
            commands::jira::save_jira_connection,
            commands::jira::get_jira_connection,
            commands::jira::test_jira_connection,
            commands::jira::publish_worklog,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::tracking::pause_tracking,
            commands::tracking::resume_tracking,
            commands::tracking::stop_live_session,
            commands::tracking::resume_tracked_project,
            commands::tracking::get_tracking_state,
            commands::tracking::get_current_activity,
            commands::storage::get_storage_info,
            commands::storage::erase_sessions,
            commands::update::check_for_update,
            commands::splashscreen::close_splashscreen,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn setup_tray(app: &tauri::App) -> anyhow::Result<()> {
    let open_item = MenuItemBuilder::new("Open TimeTracker").id("open").build(app)?;
    let quit_item  = MenuItemBuilder::new("Exit").id("quit").build(app)?;

    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;

    let mut builder = TrayIconBuilder::new()
        .menu(&menu)
        .show_menu_on_left_click(false)   // left-click opens window, not menu
        .tooltip("TimeTracker");

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

/// Install a panic hook that pops a Windows message-box so the user (and
/// developer) can actually read the crash message instead of the app just
/// silently vanishing.
pub fn install_panic_handler() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Build a human-readable message
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => s.to_string(),
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => s.clone(),
                None => "Unknown panic payload".to_string(),
            },
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let full = format!(
            "TimeTracker crashed unexpectedly.\n\n\
             Message  : {msg}\n\
             Location : {location}\n\n\
             Please report this at the project issue tracker."
        );

        // On Windows release builds, show a native dialog since there is no
        // console window.
        #[cfg(target_os = "windows")]
        show_error_dialog("TimeTracker - Fatal Error", &full);

        // Always also call the default hook (logs to stderr in debug builds).
        default_hook(info);
    }));
}

#[cfg(target_os = "windows")]
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    let title_w = to_wide(title);
    let msg_w = to_wide(message);

    unsafe {
        // MB_OK | MB_ICONERROR = 0x00000010
        windows::Win32::UI::WindowsAndMessaging::MessageBoxW(
            None,
            windows::core::PCWSTR(msg_w.as_ptr()),
            windows::core::PCWSTR(title_w.as_ptr()),
            windows::Win32::UI::WindowsAndMessaging::MB_OK
                | windows::Win32::UI::WindowsAndMessaging::MB_ICONERROR,
        );
    }
}
