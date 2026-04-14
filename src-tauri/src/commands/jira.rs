use crate::modules::jira_client::{self, JiraClient, JiraConnection};
use crate::modules::session_store;
use crate::AppState;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SaveJiraConnectionInput {
    pub name: String,
    pub base_url: String,
    pub email: String,
    pub api_token: String,
}

#[derive(Debug, Serialize)]
pub struct JiraConnectionInfo {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub email: String,
    pub is_active: bool,
}

#[tauri::command]
pub async fn save_jira_connection(
    state: State<'_, AppState>,
    input: SaveJiraConnectionInput,
) -> Result<JiraConnectionInfo, String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Deactivate existing connections
    conn.execute("UPDATE jira_connections SET is_active=0", [])
        .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO jira_connections (id, name, base_url, email, api_token, is_active, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,1,?6,?6)",
        params![id, input.name, input.base_url, input.email, input.api_token, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(JiraConnectionInfo {
        id,
        name: input.name,
        base_url: input.base_url,
        email: input.email,
        is_active: true,
    })
}

#[tauri::command]
pub fn get_jira_connection(state: State<AppState>) -> Result<Option<JiraConnectionInfo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let result = conn.query_row(
        "SELECT id,name,base_url,email,is_active FROM jira_connections WHERE is_active=1 LIMIT 1",
        [],
        |row| Ok(JiraConnectionInfo {
            id: row.get(0)?,
            name: row.get(1)?,
            base_url: row.get(2)?,
            email: row.get(3)?,
            is_active: row.get::<_, i64>(4)? != 0,
        }),
    );
    match result {
        Ok(info) => Ok(Some(info)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn test_jira_connection(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let jira_conn = get_active_connection(&state)?;
    let client = JiraClient::new(jira_conn);
    client.test_connection().await.map_err(|e| e.to_string())
}

#[derive(Debug, Deserialize)]
pub struct PublishWorklogInput {
    pub session_id: String,
    pub comment: Option<String>,
}

#[tauri::command]
pub async fn publish_worklog(
    state: State<'_, AppState>,
    input: PublishWorklogInput,
) -> Result<String, String> {
    let jira_conn = get_active_connection(&state)?;

    let session = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        session_store::get_session(&db, &input.session_id)
            .map_err(|e| e.to_string())?
            .ok_or("Session not found")?
    };

    if session.is_published {
        return Err("Session already published".to_string());
    }

    let payload = jira_client::build_worklog_payload(&session, input.comment)
        .ok_or("Session has no Jira key or zero duration")?;

    let client = JiraClient::new(jira_conn);
    let result = client.add_worklog(payload).await.map_err(|e| e.to_string())?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        session_store::mark_published(&db, &input.session_id, &result.worklog_id)
            .map_err(|e| e.to_string())?;

        // Record in publish_log
        let log_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        db.execute(
            "INSERT INTO publish_log (id, session_id, jira_key, worklog_id, duration_secs, published_at, status)
             VALUES (?1,?2,?3,?4,?5,?6,'success')",
            params![log_id, input.session_id, result.issue_key, result.worklog_id, result.time_spent_seconds, now],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(result.worklog_id)
}

fn get_active_connection(state: &State<AppState>) -> Result<JiraConnection, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.query_row(
        "SELECT id,name,base_url,email,api_token FROM jira_connections WHERE is_active=1 LIMIT 1",
        [],
        |row| Ok(JiraConnection {
            id: row.get(0)?,
            name: row.get(1)?,
            base_url: row.get(2)?,
            email: row.get(3)?,
            api_token: row.get(4)?,
        }),
    )
    .map_err(|_| "No active Jira connection configured".to_string())
}
