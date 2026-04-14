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
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut rx = broadcast_tx.subscribe();
                let mut current_session_id: Option<String> = None;
                // Tracks the wall-clock moment the current session started so we
                // can always compute total duration = now - session_start, not just
                // one poll interval.
                let mut session_start_time: Option<chrono::DateTime<chrono::Utc>> = None;

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
                        .map(|t| t.with_timezone(&chrono::Utc));

                    if is_active {
                        if current_session_id.is_none() {
                            // ── Resolve attribution ───────────────────────────
                            // Load projects/settings once for both foreground and
                            // background IDE attribution attempts.
                            let db = state.db.lock().unwrap();
                            let projects =
                                project_registry::list_projects(&db).unwrap_or_default();
                            let settings =
                                settings_manager::load(&db).unwrap_or_default();
                            drop(db);

                            let engine =
                                AttributionEngine::new(settings.jira_patterns);

                            // 1. Try the foreground window first.
                            let fg_window = snap.window.clone();
                            let fg_attr = fg_window
                                .as_ref()
                                .map(|w| engine.attribute(w, &projects));

                            // 2. If the foreground gave no registered-project match,
                            //    scan background IDE windows (VS Code, Rider, etc.).
                            //    This tracks work even when the IDE is not focused.
                            let (attribution, tracking_window) =
                                if fg_attr.as_ref().map_or(false, |a| a.project_id.is_some()) {
                                    (fg_attr.unwrap(), fg_window)
                                } else {
                                    let bg = platform::list_ide_windows()
                                        .into_iter()
                                        .find_map(|ide_w| {
                                            let a = engine.attribute(&ide_w, &projects);
                                            if a.project_id.is_some() {
                                                Some((a, ide_w))
                                            } else {
                                                None
                                            }
                                        });
                                    if let Some((bg_attr, bg_w)) = bg {
                                        (bg_attr, Some(bg_w))
                                    } else {
                                        // No project match at all — skip this poll.
                                        // We only record sessions for registered projects.
                                        continue;
                                    }
                                };

                            let input = CreateSessionInput {
                                project_id: attribution.project_id,
                                start_time: snap.timestamp.clone(),
                                end_time: None,
                                duration_secs: 0,
                                jira_key: attribution.jira_key,
                                branch: attribution.branch,
                                window_title: tracking_window
                                    .as_ref()
                                    .map(|w| w.window_title.clone()),
                                process_name: tracking_window
                                    .as_ref()
                                    .map(|w| w.process_name.clone()),
                                is_idle: false,
                                is_huddle: false,
                                huddle_channel: None,
                            };

                            let db = state.db.lock().unwrap();
                            if let Ok(session) = session_store::create_session(&db, input) {
                                current_session_id = Some(session.id);
                                session_start_time = snap_time;
                            }
                        } else if let Some(ref id) = current_session_id {
                            // Heartbeat update: keep duration current but leave end_time = NULL
                            // so the frontend can identify the still-open session.
                            // end_time is written only when the session is finalised below.
                            if let (Some(start), Some(end)) = (session_start_time, snap_time) {
                                let dur = (end - start).num_seconds().max(0);
                                let update = session_store::UpdateSessionInput {
                                    end_time: None,
                                    duration_secs: Some(dur),
                                    ..Default::default()
                                };
                                let db = state.db.lock().unwrap();
                                let _ = session_store::update_session(&db, id, update);
                            }
                        }
                    } else if let Some(id) = current_session_id.take() {
                        // Finalise: compute total elapsed and either save or discard
                        let dur = match (session_start_time.take(), snap_time) {
                            (Some(start), Some(end)) => (end - start).num_seconds().max(0),
                            _ => 0,
                        };

                        let db = state.db.lock().unwrap();
                        if dur >= MIN_SESSION_SECS {
                            let update = session_store::UpdateSessionInput {
                                end_time: Some(snap.timestamp.clone()),
                                duration_secs: Some(dur),
                                ..Default::default()
                            };
                            let _ = session_store::update_session(&db, &id, update);
                        } else {
                            // Too short to be meaningful — remove the stub row
                            let _ = session_store::delete_session(&db, &id);
                        }
                    }

                    let _ = snap; // snap consumed; last_snapshot no longer needed
                }
            });

            // ── Slack Huddle monitoring loop ─────────────────────────────────
            modules::huddle_monitor::start(
                app.handle().clone(),
                init.monitor_config.poll_interval_secs,
            );

            // Intercept the window close button: hide to tray instead of quitting.
            let app_handle_close = app.handle().clone();
            if let Some(win) = app.get_webview_window("main") {
                win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(w) = app_handle_close.get_webview_window("main") {
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
            commands::jira::save_jira_connection,
            commands::jira::get_jira_connection,
            commands::jira::test_jira_connection,
            commands::jira::publish_worklog,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::tracking::pause_tracking,
            commands::tracking::resume_tracking,
            commands::tracking::get_tracking_state,
            commands::tracking::get_current_activity,
            commands::storage::get_storage_info,
            commands::storage::erase_sessions,
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
