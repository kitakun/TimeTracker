//! Stub implementations for non-Windows platforms (compile-time only).
use super::types::{ActiveWindowInfo, HuddleInfo};

pub fn get_active_window() -> Option<ActiveWindowInfo> {
    None
}

pub fn get_idle_seconds() -> u64 {
    0
}

pub fn get_huddle_window() -> Option<HuddleInfo> {
    None
}

pub fn list_ide_windows() -> Vec<super::types::ActiveWindowInfo> {
    Vec::new()
}
