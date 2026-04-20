use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::modules::git_probe::JiraKeyPattern;

fn default_track_slack_huddles() -> bool { true }
fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub idle_threshold_secs: u64,
    pub poll_interval_secs: u64,
    pub jira_patterns: Vec<JiraKeyPattern>,
    pub start_on_login: bool,
    pub minimize_to_tray: bool,
    /// Whether Slack Huddle calls should be tracked as sessions (default: true).
    #[serde(default = "default_track_slack_huddles")]
    pub track_slack_huddles: bool,
    /// Whether the Jira integration is enabled (default: false).
    #[serde(default)]
    pub jira_enabled: bool,
    /// Whether idle detection is active (default: true). When false, tracking
    /// never enters the Idle state automatically — only a manual pause stops it.
    #[serde(default = "default_true")]
    pub idle_detection_enabled: bool,
    /// When true, the app will resume the previous open session after a short
    /// restart or wake-from-sleep (gap < 1 hour) instead of creating a new one.
    #[serde(default)]
    pub auto_merge_enabled: bool,
    /// When true, unhandled JavaScript errors are shown as toast notifications.
    #[serde(default)]
    pub show_unexpected_errors: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            idle_threshold_secs: 300,
            poll_interval_secs: 5,
            jira_patterns: vec![JiraKeyPattern::default()],
            start_on_login: false,
            minimize_to_tray: true,
            track_slack_huddles: true,
            jira_enabled: false,
            idle_detection_enabled: true,
            auto_merge_enabled: false,
            show_unexpected_errors: false,
        }
    }
}

const KEY: &str = "app_settings";

pub fn load(conn: &Connection) -> Result<AppSettings> {
    let result: rusqlite::Result<String> = conn.query_row(
        "SELECT value FROM settings WHERE key=?1",
        params![KEY],
        |row| row.get(0),
    );
    match result {
        Ok(json) => {
            let settings: AppSettings = serde_json::from_str(&json)
                .unwrap_or_default();
            Ok(settings)
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AppSettings::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save(conn: &Connection, settings: &AppSettings) -> Result<()> {
    let json = serde_json::to_string(settings)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1,?2,?3)
         ON CONFLICT(key) DO UPDATE SET value=?2, updated_at=?3",
        params![KEY, json, now],
    )?;
    Ok(())
}
