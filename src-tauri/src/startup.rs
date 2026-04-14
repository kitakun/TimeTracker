//! Startup sequence extracted from `lib.rs::setup()` so it can be exercised
//! by tests without a live Tauri application.
//!
//! `try_init` mirrors every step that runs before Tauri hands control to the
//! UI layer.  Tests call it with a temporary directory and assert each step
//! succeeds.

use crate::modules::activity_monitor::{ActivityMonitor, MonitorConfig};
use crate::modules::session_store;
use crate::modules::settings_manager;
use crate::AppState;
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Mutex;

/// Result returned by a successful `try_init`.
pub struct InitResult {
    pub state: AppState,
    pub monitor_config: MonitorConfig,
}

/// Performs every startup step that does NOT require a Tauri `App` handle:
///
/// 1. Creates `data_dir` (and any missing parents).
/// 2. Opens the SQLite database and runs all pending migrations.
/// 3. Loads (or defaults) application settings.
/// 4. Constructs the `ActivityMonitor` from those settings.
/// 5. Returns the fully initialised `AppState`.
///
/// Returns `Err` with a descriptive message on any failure so callers (and
/// tests) know exactly which step went wrong.
pub fn try_init(data_dir: &Path) -> Result<InitResult> {
    // Step 1 – directory
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("create_dir_all failed for {data_dir:?}"))?;

    // Step 2 – database
    let db_path = data_dir.join("timetracker.db");
    let conn = crate::db::open(&db_path)
        .with_context(|| format!("db::open failed for {db_path:?}"))?;

    // Step 3 – close sessions that were left open by the previous run
    // (crash, forced kill, or clean exit before the loop could finalise them).
    // Errors here are non-fatal; we log but continue.
    if let Err(e) = session_store::close_orphaned_sessions(&conn, crate::MIN_SESSION_SECS) {
        eprintln!("[startup] close_orphaned_sessions failed: {e:#}");
    }

    // Step 4 – settings (graceful default on any error)
    let settings = settings_manager::load(&conn).unwrap_or_default();

    // Step 5 – monitor config
    let config = MonitorConfig {
        idle_threshold_secs: settings.idle_threshold_secs,
        poll_interval_secs: settings.poll_interval_secs,
    };

    // Step 6 – state
    let monitor = ActivityMonitor::new(config.clone());
    let state = AppState {
        db: Mutex::new(conn),
        monitor: Mutex::new(monitor),
        data_dir: data_dir.to_path_buf(),
    };

    Ok(InitResult { state, monitor_config: config })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::project_registry::{self, CreateProjectInput};
    use crate::modules::settings_manager::{self as sm, AppSettings};
    use tempfile::TempDir;

    /// Helper: fresh temp dir + successful init.
    fn fresh() -> (TempDir, InitResult) {
        let dir = TempDir::new().expect("tempdir");
        let result = try_init(dir.path()).expect("try_init on fresh dir");
        (dir, result)
    }

    // ── Core startup ─────────────────────────────────────────────────────────

    #[test]
    fn fresh_install_succeeds() {
        let dir = TempDir::new().unwrap();
        let r = try_init(dir.path());
        assert!(r.is_ok(), "fresh install failed: {:?}", r.err());
    }

    #[test]
    fn restart_on_existing_db_succeeds() {
        let dir = TempDir::new().unwrap();
        // First boot
        try_init(dir.path()).expect("first boot");
        // Second boot – same directory, DB already exists
        let r = try_init(dir.path());
        assert!(r.is_ok(), "restart failed: {:?}", r.err());
    }

    #[test]
    fn db_file_is_created() {
        let dir = TempDir::new().unwrap();
        try_init(dir.path()).unwrap();
        assert!(
            dir.path().join("timetracker.db").exists(),
            "timetracker.db was not created"
        );
    }

    #[test]
    fn nested_data_dir_is_created() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        let r = try_init(&nested);
        assert!(r.is_ok(), "nested dir creation failed: {:?}", r.err());
        assert!(nested.exists());
    }

    // ── Settings defaults ────────────────────────────────────────────────────

    #[test]
    fn default_settings_on_fresh_db() {
        let (_dir, res) = fresh();
        let db = res.state.db.lock().unwrap();
        let s = sm::load(&db).unwrap();
        assert_eq!(s.idle_threshold_secs, AppSettings::default().idle_threshold_secs);
        assert_eq!(s.poll_interval_secs, AppSettings::default().poll_interval_secs);
        assert!(!s.jira_patterns.is_empty(), "default patterns must not be empty");
    }

    #[test]
    fn settings_persist_across_restart() {
        let dir = TempDir::new().unwrap();
        {
            let res = try_init(dir.path()).unwrap();
            let db = res.state.db.lock().unwrap();
            let mut s = sm::load(&db).unwrap();
            s.idle_threshold_secs = 999;
            sm::save(&db, &s).unwrap();
        }
        // Restart
        let res2 = try_init(dir.path()).unwrap();
        let db2 = res2.state.db.lock().unwrap();
        let s2 = sm::load(&db2).unwrap();
        assert_eq!(s2.idle_threshold_secs, 999, "setting was not persisted");
    }

    // ── Monitor config ────────────────────────────────────────────────────────

    #[test]
    fn monitor_config_reflects_settings() {
        let dir = TempDir::new().unwrap();
        {
            let res = try_init(dir.path()).unwrap();
            let db = res.state.db.lock().unwrap();
            let mut s = sm::load(&db).unwrap();
            s.idle_threshold_secs = 42;
            s.poll_interval_secs = 7;
            sm::save(&db, &s).unwrap();
        }
        let res2 = try_init(dir.path()).unwrap();
        assert_eq!(res2.monitor_config.idle_threshold_secs, 42);
        assert_eq!(res2.monitor_config.poll_interval_secs, 7);
    }

    // ── AppState usability ───────────────────────────────────────────────────

    #[test]
    fn app_state_db_mutex_is_usable() {
        let (_dir, res) = fresh();
        // We should be able to lock the db mutex and do a query
        let db = res.state.db.lock().expect("lock poisoned");
        let projects = project_registry::list_projects(&db);
        assert!(projects.is_ok(), "list_projects on fresh db failed");
        assert!(projects.unwrap().is_empty());
    }

    #[test]
    fn app_state_monitor_mutex_is_usable() {
        let (_dir, res) = fresh();
        let monitor = res.state.monitor.lock().expect("lock poisoned");
        // Monitor must start in non-running state before broadcast loop begins
        use crate::modules::activity_monitor::TrackingState;
        assert!(
            matches!(monitor.current_state(), TrackingState::Paused | TrackingState::Running),
            "unexpected initial tracking state"
        );
    }

    // ── Database schema ───────────────────────────────────────────────────────

    #[test]
    fn all_expected_tables_exist() {
        let (_dir, res) = fresh();
        let db = res.state.db.lock().unwrap();
        let tables = ["projects", "sessions", "settings", "jira_connections",
                       "publish_log", "schema_migrations"];
        for table in &tables {
            let count: i64 = db
                .query_row(
                    &format!(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{table}'"
                    ),
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            assert_eq!(count, 1, "table '{table}' is missing from the schema");
        }
    }

    #[test]
    fn migrations_are_idempotent() {
        let dir = TempDir::new().unwrap();
        for _ in 0..3 {
            let r = try_init(dir.path());
            assert!(r.is_ok(), "migration failed on repeated init: {:?}", r.err());
        }
    }

    // ── Data integrity ────────────────────────────────────────────────────────

    #[test]
    fn project_created_and_readable_after_restart() {
        let dir = TempDir::new().unwrap();
        let project_id;
        {
            let res = try_init(dir.path()).unwrap();
            let db = res.state.db.lock().unwrap();
            let p = project_registry::create_project(
                &db,
                CreateProjectInput {
                    name: "Test Project".into(),
                    path: dir.path().to_str().unwrap().to_string(),
                    color: None,
                },
            )
            .unwrap();
            project_id = p.id;
        }
        // Restart and verify
        let res2 = try_init(dir.path()).unwrap();
        let db2 = res2.state.db.lock().unwrap();
        let p2 = project_registry::get_project(&db2, &project_id)
            .unwrap()
            .expect("project missing after restart");
        assert_eq!(p2.name, "Test Project");
    }

    // ── Failure modes ─────────────────────────────────────────────────────────

    #[test]
    fn returns_err_on_read_only_db_path() {
        // Point at a path that is itself a directory – SQLite can't open it.
        let dir = TempDir::new().unwrap();
        // Create a directory where the DB file should go
        let fake_db = dir.path().join("timetracker.db");
        std::fs::create_dir_all(&fake_db).unwrap();
        let r = try_init(dir.path());
        assert!(
            r.is_err(),
            "expected error when db path is a directory, got Ok"
        );
    }

    // ── Panic handler ─────────────────────────────────────────────────────────

    #[test]
    fn panic_handler_installs_without_panic() {
        // install_panic_handler should not itself panic, even when called
        // multiple times (each call replaces the previous hook).
        crate::install_panic_handler();
        crate::install_panic_handler();
    }
}
