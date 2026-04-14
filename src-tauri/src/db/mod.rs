use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::Path;

pub fn open(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        );"
    )?;

    let version: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |r| r.get(0),
    )?;

    if version < 1 {
        conn.execute_batch(MIGRATION_1)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![1i64],
        )?;
    }
    if version < 2 {
        conn.execute_batch(MIGRATION_2)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![2i64],
        )?;
    }
    if version < 3 {
        conn.execute_batch(MIGRATION_3)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![3i64],
        )?;
    }
    if version < 4 {
        conn.execute_batch(MIGRATION_4)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, datetime('now'))",
            params![4i64],
        )?;
    }

    Ok(())
}

const MIGRATION_1: &str = "
CREATE TABLE IF NOT EXISTS projects (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    path        TEXT NOT NULL UNIQUE,
    color       TEXT NOT NULL DEFAULT '#4A90E2',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id              TEXT PRIMARY KEY,
    project_id      TEXT REFERENCES projects(id),
    start_time      TEXT NOT NULL,
    end_time        TEXT,
    duration_secs   INTEGER NOT NULL DEFAULT 0,
    jira_key        TEXT,
    branch          TEXT,
    window_title    TEXT,
    process_name    TEXT,
    is_idle         INTEGER NOT NULL DEFAULT 0,
    is_published    INTEGER NOT NULL DEFAULT 0,
    published_at    TEXT,
    worklog_id      TEXT,
    notes           TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_sessions_start   ON sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_sessions_jira    ON sessions(jira_key);

CREATE TABLE IF NOT EXISTS settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
";

const MIGRATION_3: &str = "
ALTER TABLE sessions ADD COLUMN is_huddle INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN huddle_channel TEXT;
CREATE INDEX IF NOT EXISTS idx_sessions_huddle ON sessions(is_huddle);
";

const MIGRATION_4: &str = "
ALTER TABLE sessions ADD COLUMN is_manual INTEGER NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_sessions_manual ON sessions(is_manual);
";

const MIGRATION_2: &str = "
CREATE TABLE IF NOT EXISTS jira_connections (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    base_url        TEXT NOT NULL,
    email           TEXT NOT NULL,
    api_token       TEXT NOT NULL,
    is_active       INTEGER NOT NULL DEFAULT 1,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS publish_log (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL REFERENCES sessions(id),
    jira_key        TEXT NOT NULL,
    worklog_id      TEXT,
    duration_secs   INTEGER NOT NULL,
    published_at    TEXT NOT NULL,
    status          TEXT NOT NULL,
    error_msg       TEXT
);
";
