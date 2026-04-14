use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub project_id: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_secs: i64,
    pub jira_key: Option<String>,
    pub branch: Option<String>,
    pub window_title: Option<String>,
    pub process_name: Option<String>,
    pub is_idle: bool,
    pub is_published: bool,
    pub published_at: Option<String>,
    pub worklog_id: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// True when this session was recorded from a Slack Huddle window.
    pub is_huddle: bool,
    /// Channel or recipient name parsed from the Slack Huddle window title.
    pub huddle_channel: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionInput {
    pub project_id: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_secs: i64,
    pub jira_key: Option<String>,
    pub branch: Option<String>,
    pub window_title: Option<String>,
    pub process_name: Option<String>,
    pub is_idle: bool,
    pub is_huddle: bool,
    pub huddle_channel: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateSessionInput {
    pub end_time: Option<String>,
    pub duration_secs: Option<i64>,
    pub jira_key: Option<String>,
    pub notes: Option<String>,
    pub project_id: Option<Option<String>>,
}

pub fn create_session(conn: &Connection, input: CreateSessionInput) -> Result<Session> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sessions (id, project_id, start_time, end_time, duration_secs, jira_key, branch,
         window_title, process_name, is_idle, is_published, is_huddle, huddle_channel, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,0,?11,?12,?13,?13)",
        params![
            id, input.project_id, input.start_time, input.end_time, input.duration_secs,
            input.jira_key, input.branch, input.window_title, input.process_name,
            input.is_idle as i64, input.is_huddle as i64, input.huddle_channel, now
        ],
    )?;
    get_session(conn, &id)?.ok_or_else(|| anyhow::anyhow!("Failed to retrieve created session"))
}

pub fn update_session(conn: &Connection, id: &str, input: UpdateSessionInput) -> Result<Session> {
    let now = Utc::now().to_rfc3339();
    if let Some(end) = &input.end_time {
        conn.execute("UPDATE sessions SET end_time=?1, updated_at=?2 WHERE id=?3", params![end, now, id])?;
    }
    if let Some(dur) = input.duration_secs {
        conn.execute("UPDATE sessions SET duration_secs=?1, updated_at=?2 WHERE id=?3", params![dur, now, id])?;
    }
    if let Some(key) = &input.jira_key {
        conn.execute("UPDATE sessions SET jira_key=?1, updated_at=?2 WHERE id=?3", params![key, now, id])?;
    }
    if let Some(notes) = &input.notes {
        conn.execute("UPDATE sessions SET notes=?1, updated_at=?2 WHERE id=?3", params![notes, now, id])?;
    }
    if let Some(pid) = input.project_id {
        conn.execute("UPDATE sessions SET project_id=?1, updated_at=?2 WHERE id=?3", params![pid, now, id])?;
    }
    get_session(conn, id)?.ok_or_else(|| anyhow::anyhow!("Session not found"))
}

pub fn get_session(conn: &Connection, id: &str) -> Result<Option<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,start_time,end_time,duration_secs,jira_key,branch,
         window_title,process_name,is_idle,is_published,published_at,worklog_id,notes,created_at,updated_at,
         is_huddle,huddle_channel
         FROM sessions WHERE id=?1"
    )?;
    let mut rows = stmt.query_map(params![id], map_row)?;
    Ok(rows.next().transpose()?)
}

pub fn list_sessions_for_day(conn: &Connection, date: &str) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,start_time,end_time,duration_secs,jira_key,branch,
         window_title,process_name,is_idle,is_published,published_at,worklog_id,notes,created_at,updated_at,
         is_huddle,huddle_channel
         FROM sessions
         WHERE date(start_time) = ?1 AND is_idle = 0
         ORDER BY start_time"
    )?;
    let rows = stmt.query_map(params![date], map_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn list_sessions_for_range(conn: &Connection, from: &str, to: &str) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,start_time,end_time,duration_secs,jira_key,branch,
         window_title,process_name,is_idle,is_published,published_at,worklog_id,notes,created_at,updated_at,
         is_huddle,huddle_channel
         FROM sessions
         WHERE start_time >= ?1 AND start_time <= ?2 AND is_idle = 0
         ORDER BY start_time"
    )?;
    let rows = stmt.query_map(params![from, to], map_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn list_unpublished_for_day(conn: &Connection, date: &str) -> Result<Vec<Session>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,start_time,end_time,duration_secs,jira_key,branch,
         window_title,process_name,is_idle,is_published,published_at,worklog_id,notes,created_at,updated_at,
         is_huddle,huddle_channel
         FROM sessions
         WHERE date(start_time) = ?1 AND is_idle = 0 AND is_published = 0 AND jira_key IS NOT NULL
         ORDER BY start_time"
    )?;
    let rows = stmt.query_map(params![date], map_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

pub fn mark_published(conn: &Connection, id: &str, worklog_id: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE sessions SET is_published=1, published_at=?1, worklog_id=?2, updated_at=?1 WHERE id=?3",
        params![now, worklog_id, id],
    )?;
    Ok(())
}

pub fn delete_session(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE id=?1", params![id])?;
    Ok(())
}

/// Called once on startup to clean up sessions that were left open by a
/// previous run (crash, forced kill, or clean exit that skipped finalisation).
///
/// Sessions with enough duration get a synthetic end_time computed as
/// `start_time + duration_secs`.  Stub rows (below the minimum) are deleted.
pub fn close_orphaned_sessions(conn: &Connection, min_secs: i64) -> Result<()> {
    // Assign a synthetic end_time for sessions that accumulated enough time.
    conn.execute(
        "UPDATE sessions
         SET end_time = strftime('%Y-%m-%dT%H:%M:%SZ',
                         datetime(start_time, '+' || duration_secs || ' seconds'))
         WHERE end_time IS NULL AND duration_secs >= ?1",
        params![min_secs],
    )?;
    // Delete orphan stubs that are too short to be meaningful.
    conn.execute("DELETE FROM sessions WHERE end_time IS NULL", [])?;
    Ok(())
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        project_id: row.get(1)?,
        start_time: row.get(2)?,
        end_time: row.get(3)?,
        duration_secs: row.get(4)?,
        jira_key: row.get(5)?,
        branch: row.get(6)?,
        window_title: row.get(7)?,
        process_name: row.get(8)?,
        is_idle: row.get::<_, i64>(9)? != 0,
        is_published: row.get::<_, i64>(10)? != 0,
        published_at: row.get(11)?,
        worklog_id: row.get(12)?,
        notes: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
        is_huddle: row.get::<_, i64>(16)? != 0,
        huddle_channel: row.get(17)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::NamedTempFile;

    fn test_conn() -> (NamedTempFile, Connection) {
        let f = NamedTempFile::new().unwrap();
        let conn = db::open(f.path()).unwrap();
        (f, conn)
    }

    fn make_input(start: &str, dur: i64) -> CreateSessionInput {
        CreateSessionInput {
            project_id: None,
            start_time: start.into(),
            end_time: None,
            duration_secs: dur,
            jira_key: None,
            branch: None,
            window_title: None,
            process_name: None,
            is_idle: false,
            is_huddle: false,
            huddle_channel: None,
        }
    }

    // ── Basic CRUD ────────────────────────────────────────────────────────────

    #[test]
    fn create_and_retrieve() {
        let (_f, conn) = test_conn();
        let input = CreateSessionInput {
            jira_key: Some("PROJ-1".into()),
            branch: Some("feature/PROJ-1-login".into()),
            end_time: Some("2024-01-15T09:30:00Z".into()),
            duration_secs: 1800,
            ..make_input("2024-01-15T09:00:00Z", 1800)
        };
        let s = create_session(&conn, input).unwrap();
        assert_eq!(s.jira_key.as_deref(), Some("PROJ-1"));
        let fetched = get_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(fetched.duration_secs, 1800);
    }

    #[test]
    fn idle_sessions_excluded_from_daily_list() {
        let (_f, conn) = test_conn();
        let idle = CreateSessionInput { is_idle: true, ..make_input("2024-01-15T09:00:00Z", 300) };
        create_session(&conn, idle).unwrap();
        let sessions = list_sessions_for_day(&conn, "2024-01-15").unwrap();
        assert!(sessions.is_empty(), "Idle sessions must not appear in the daily list");
    }

    // ── Duration accumulation (mirrors the lib.rs recording loop) ────────────
    //
    // The loop now stores total elapsed = snap_time - session_start_time, NOT
    // just the last poll interval.  These tests validate that contract.

    #[test]
    fn duration_is_total_elapsed_not_one_interval() {
        let (_f, conn) = test_conn();

        // Session starts at T=0
        let s = create_session(&conn, make_input("2024-01-15T10:00:00Z", 0)).unwrap();
        assert_eq!(s.duration_secs, 0);

        // Poll at T+5s — duration should be 5
        update_session(&conn, &s.id, UpdateSessionInput {
            end_time: Some("2024-01-15T10:00:05Z".into()),
            duration_secs: Some(5),
            ..Default::default()
        }).unwrap();

        // Poll at T+900s (15 min later) — duration should be 900
        update_session(&conn, &s.id, UpdateSessionInput {
            end_time: Some("2024-01-15T10:15:00Z".into()),
            duration_secs: Some(900),
            ..Default::default()
        }).unwrap();

        let fetched = get_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(fetched.duration_secs, 900,
            "After 15 minutes the stored duration must be 900 s, not 5 s (one poll interval)");
        assert_eq!(fetched.end_time.as_deref(), Some("2024-01-15T10:15:00Z"));
    }

    #[test]
    fn short_session_can_be_deleted() {
        let (_f, conn) = test_conn();
        let s = create_session(&conn, make_input("2024-01-15T10:00:00Z", 0)).unwrap();

        // Simulate the stub-cleanup: session lasted only 5 s — below MIN threshold
        const MIN_SESSION_SECS: i64 = 30;
        let dur = 5_i64;
        if dur < MIN_SESSION_SECS {
            delete_session(&conn, &s.id).unwrap();
        }

        let fetched = get_session(&conn, &s.id).unwrap();
        assert!(fetched.is_none(), "Stub session under 30 s must be deleted");
    }

    #[test]
    fn long_session_survives_min_threshold() {
        let (_f, conn) = test_conn();
        let s = create_session(&conn, make_input("2024-01-15T10:00:00Z", 0)).unwrap();

        const MIN_SESSION_SECS: i64 = 30;
        let dur = 900_i64;
        if dur < MIN_SESSION_SECS {
            delete_session(&conn, &s.id).unwrap();
        } else {
            update_session(&conn, &s.id, UpdateSessionInput {
                end_time: Some("2024-01-15T10:15:00Z".into()),
                duration_secs: Some(dur),
                ..Default::default()
            }).unwrap();
        }

        let fetched = get_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(fetched.duration_secs, 900);
    }

    // ── Filtering / listing ───────────────────────────────────────────────────

    #[test]
    fn list_sessions_for_day_returns_correct_date_only() {
        let (_f, conn) = test_conn();
        let today = CreateSessionInput {
            end_time: Some("2024-01-15T10:30:00Z".into()),
            duration_secs: 1800,
            ..make_input("2024-01-15T10:00:00Z", 1800)
        };
        let yesterday = CreateSessionInput {
            end_time: Some("2024-01-14T10:30:00Z".into()),
            duration_secs: 1800,
            ..make_input("2024-01-14T10:00:00Z", 1800)
        };
        create_session(&conn, today).unwrap();
        create_session(&conn, yesterday).unwrap();

        let sessions = list_sessions_for_day(&conn, "2024-01-15").unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].start_time.starts_with("2024-01-15"));
    }

    #[test]
    fn delete_removes_session() {
        let (_f, conn) = test_conn();
        let s = create_session(&conn, make_input("2024-01-15T10:00:00Z", 60)).unwrap();
        delete_session(&conn, &s.id).unwrap();
        assert!(get_session(&conn, &s.id).unwrap().is_none());
    }

    #[test]
    fn update_jira_key_and_notes() {
        let (_f, conn) = test_conn();
        let s = create_session(&conn, make_input("2024-01-15T10:00:00Z", 60)).unwrap();
        assert!(s.jira_key.is_none());

        update_session(&conn, &s.id, UpdateSessionInput {
            jira_key: Some("PROJ-42".into()),
            notes: Some("reviewed login flow".into()),
            ..Default::default()
        }).unwrap();

        let fetched = get_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(fetched.jira_key.as_deref(), Some("PROJ-42"));
        assert_eq!(fetched.notes.as_deref(), Some("reviewed login flow"));
    }
}
