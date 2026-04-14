pub mod types;

// ── OS-specific implementations ───────────────────────────────────────────────
// Each sub-module exposes the same four functions; only one is compiled.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
use windows as imp;
#[cfg(target_os = "macos")]
use macos as imp;
#[cfg(target_os = "linux")]
use linux as imp;

use types::{ActiveWindowInfo, HuddleInfo};

// ── Public API ────────────────────────────────────────────────────────────────

/// Get the currently focused window.
pub fn get_active_window() -> Option<ActiveWindowInfo> {
    imp::get_active_window()
}

/// Seconds since the last keyboard or mouse input.
pub fn get_idle_seconds() -> u64 {
    imp::get_idle_seconds()
}

/// All visible windows owned by known IDE processes (VS Code, Rider, Cursor, …).
/// Used to detect the active project even when the IDE is not focused.
pub fn list_ide_windows() -> Vec<ActiveWindowInfo> {
    imp::list_ide_windows()
}

/// The active Slack Huddle window, if any.
pub fn get_huddle_window() -> Option<HuddleInfo> {
    imp::get_huddle_window()
}
