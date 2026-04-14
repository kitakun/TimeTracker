use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveWindowInfo {
    pub process_name: String,
    pub window_title: String,
    /// Executable path, if available
    pub exe_path: Option<String>,
}

/// Information about an active Slack Huddle detected via window enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuddleInfo {
    /// Parsed channel/recipient, e.g. "general" from "Huddle in #general"
    pub channel: Option<String>,
    /// Full window title as reported by the OS
    pub window_title: String,
}
