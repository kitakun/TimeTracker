pub mod types;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(not(target_os = "windows"))]
pub mod stub;

use types::{ActiveWindowInfo, HuddleInfo};

/// Get the currently focused window.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    #[cfg(target_os = "windows")]
    return windows::get_active_window();

    #[cfg(not(target_os = "windows"))]
    return stub::get_active_window();
}

/// Get seconds since last user input (keyboard/mouse).
pub fn get_idle_seconds() -> u64 {
    #[cfg(target_os = "windows")]
    return windows::get_idle_seconds();

    #[cfg(not(target_os = "windows"))]
    return stub::get_idle_seconds();
}

/// Return all visible windows owned by known IDE processes (VS Code, Rider, etc.).
/// Used to detect active project context even when the IDE is not focused.
pub fn list_ide_windows() -> Vec<ActiveWindowInfo> {
    #[cfg(target_os = "windows")]
    return windows::list_ide_windows();

    #[cfg(not(target_os = "windows"))]
    return stub::list_ide_windows();
}

/// Scan all visible windows for an active Slack Huddle.
pub fn get_huddle_window() -> Option<HuddleInfo> {
    #[cfg(target_os = "windows")]
    return windows::get_huddle_window();

    #[cfg(not(target_os = "windows"))]
    return stub::get_huddle_window();
}
