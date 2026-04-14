use crate::modules::project_registry::{
    self, CreateProjectInput, Project, UpdateProjectInput,
};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_projects(state: State<AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    project_registry::list_projects(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    input: CreateProjectInput,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    project_registry::create_project(&conn, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_project(
    state: State<AppState>,
    id: String,
    input: UpdateProjectInput,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    project_registry::update_project(&conn, &id, input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    project_registry::delete_project(&conn, &id).map_err(|e| e.to_string())
}
