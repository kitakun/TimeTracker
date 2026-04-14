//! Linux platform implementation.
//!
//! Window tracking and idle detection are not yet implemented.
//! All functions return safe no-op values so the app compiles and runs;
//! session recording simply won't activate until a real implementation is added.
use crate::platform::types::{ActiveWindowInfo, HuddleInfo};

pub fn get_active_window() -> Option<ActiveWindowInfo> {
    None
}

pub fn get_idle_seconds() -> u64 {
    0
}

pub fn list_ide_windows() -> Vec<ActiveWindowInfo> {
    Vec::new()
}

pub fn get_huddle_window() -> Option<HuddleInfo> {
    None
}
