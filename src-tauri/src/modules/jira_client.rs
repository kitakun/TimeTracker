use anyhow::{Context, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConnection {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub email: String,
    /// Stored as-is; in production you'd want OS keychain storage.
    pub api_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorklogPayload {
    pub issue_key: String,
    pub started: DateTime<Utc>,
    pub time_spent_seconds: i64,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorklogResult {
    pub worklog_id: String,
    pub issue_key: String,
    pub time_spent_seconds: i64,
    pub started: String,
}

#[derive(Debug, Deserialize)]
struct JiraWorklogResponse {
    id: String,
    #[serde(rename = "timeSpentSeconds")]
    time_spent_seconds: i64,
    started: String,
}

pub struct JiraClient {
    conn: JiraConnection,
    http: reqwest::Client,
}

impl JiraClient {
    pub fn new(conn: JiraConnection) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");
        Self { conn, http }
    }

    fn auth_header(&self) -> String {
        let raw = format!("{}:{}", self.conn.email, self.conn.api_token);
        let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
        format!("Basic {}", encoded)
    }

    fn api_url(&self, path: &str) -> String {
        let base = self.conn.base_url.trim_end_matches('/');
        format!("{}/rest/api/3{}", base, path)
    }

    /// Test the connection by fetching the current user.
    pub async fn test_connection(&self) -> Result<String> {
        let url = self.api_url("/myself");
        let resp = self.http.get(&url)
            .header("Authorization", self.auth_header())
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to connect to Jira")?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json().await?;
            let display_name = json["displayName"].as_str().unwrap_or("Unknown").to_string();
            Ok(display_name)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Jira auth failed ({}): {}", status, body);
        }
    }

    /// Add a worklog entry to a Jira issue.
    pub async fn add_worklog(&self, payload: WorklogPayload) -> Result<WorklogResult> {
        let url = self.api_url(&format!("/issue/{}/worklog", payload.issue_key));

        // Jira expects: "2021-01-17T12:34:00.000+0000"
        let started_str = payload.started.format("%Y-%m-%dT%H:%M:%S%.3f+0000").to_string();

        let body = serde_json::json!({
            "started": started_str,
            "timeSpentSeconds": payload.time_spent_seconds,
            "comment": payload.comment.as_deref().map(|c| serde_json::json!({
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{"type": "text", "text": c}]
                }]
            }))
        });

        let resp = self.http.post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send worklog to Jira")?;

        if resp.status().is_success() {
            let wl: JiraWorklogResponse = resp.json().await.context("Failed to parse Jira response")?;
            Ok(WorklogResult {
                worklog_id: wl.id,
                issue_key: payload.issue_key,
                time_spent_seconds: wl.time_spent_seconds,
                started: wl.started,
            })
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Jira worklog failed ({}): {}", status, body);
        }
    }

    #[allow(dead_code)]
    /// Delete a previously added worklog (for undo support).
    pub async fn delete_worklog(&self, issue_key: &str, worklog_id: &str) -> Result<()> {
        let url = self.api_url(&format!("/issue/{}/worklog/{}", issue_key, worklog_id));
        let resp = self.http.delete(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if resp.status().is_success() || resp.status().as_u16() == 204 {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Delete worklog failed ({}): {}", status, body);
        }
    }
}

/// Build the Jira worklog payload from a session.
/// Returns None if session has no jira_key or zero duration.
pub fn build_worklog_payload(
    session: &crate::modules::session_store::Session,
    comment: Option<String>,
) -> Option<WorklogPayload> {
    let key = session.jira_key.clone()?;
    if session.duration_secs <= 0 {
        return None;
    }
    let started = DateTime::parse_from_rfc3339(&session.start_time)
        .ok()?
        .with_timezone(&Utc);

    Some(WorklogPayload {
        issue_key: key,
        started,
        time_spent_seconds: session.duration_secs,
        comment,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::session_store::Session;

    fn dummy_session(dur: i64, key: Option<&str>) -> Session {
        Session {
            id: "test".into(),
            project_id: None,
            start_time: "2024-01-15T09:00:00Z".into(),
            end_time: Some("2024-01-15T09:30:00Z".into()),
            duration_secs: dur,
            jira_key: key.map(Into::into),
            branch: None,
            window_title: None,
            process_name: None,
            is_idle: false,
            is_published: false,
            published_at: None,
            worklog_id: None,
            notes: None,
            created_at: "2024-01-15T09:00:00Z".into(),
            updated_at: "2024-01-15T09:00:00Z".into(),
            is_huddle: false,
            huddle_channel: None,
        }
    }

    #[test]
    fn payload_built_correctly() {
        let s = dummy_session(3600, Some("PROJ-1"));
        let p = build_worklog_payload(&s, Some("test comment".into())).unwrap();
        assert_eq!(p.issue_key, "PROJ-1");
        assert_eq!(p.time_spent_seconds, 3600);
        assert!(p.comment.is_some());
    }

    #[test]
    fn no_payload_for_zero_duration() {
        let s = dummy_session(0, Some("PROJ-1"));
        assert!(build_worklog_payload(&s, None).is_none());
    }

    #[test]
    fn no_payload_without_key() {
        let s = dummy_session(1800, None);
        assert!(build_worklog_payload(&s, None).is_none());
    }
}
