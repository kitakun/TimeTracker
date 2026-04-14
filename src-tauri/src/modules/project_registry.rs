use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectInput {
    pub name: String,
    pub path: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectInput {
    pub name: Option<String>,
    pub color: Option<String>,
}

pub fn list_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, color, created_at, updated_at FROM projects ORDER BY name"
    )?;
    let projects = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            color: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?
    .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(projects)
}

pub fn get_project(conn: &Connection, id: &str) -> Result<Option<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, path, color, created_at, updated_at FROM projects WHERE id = ?1"
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            color: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn create_project(conn: &Connection, input: CreateProjectInput) -> Result<Project> {
    // Verify path exists
    if !std::path::Path::new(&input.path).exists() {
        anyhow::bail!("Path does not exist: {}", input.path);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let color = input.color.unwrap_or_else(|| "#4A90E2".to_string());

    conn.execute(
        "INSERT INTO projects (id, name, path, color, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, input.name, input.path, color, now, now],
    )?;

    Ok(Project { id, name: input.name, path: input.path, color, created_at: now.clone(), updated_at: now })
}

pub fn update_project(conn: &Connection, id: &str, input: UpdateProjectInput) -> Result<Project> {
    let now = Utc::now().to_rfc3339();
    if let Some(name) = &input.name {
        conn.execute("UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3", params![name, now, id])?;
    }
    if let Some(color) = &input.color {
        conn.execute("UPDATE projects SET color = ?1, updated_at = ?2 WHERE id = ?3", params![color, now, id])?;
    }
    get_project(conn, id)?.ok_or_else(|| anyhow::anyhow!("Project not found"))
}

pub fn delete_project(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

#[allow(dead_code)]
/// Find which registered project (if any) a file path belongs to.
pub fn find_project_for_path(conn: &Connection, file_path: &str) -> Result<Option<Project>> {
    let projects = list_projects(conn)?;
    // Longest-prefix match
    let best = projects.into_iter().filter(|p| {
        file_path.starts_with(&p.path)
    }).max_by_key(|p| p.path.len());
    Ok(best)
}
