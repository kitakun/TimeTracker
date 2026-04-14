use crate::modules::git_probe::{self, GitInfo, JiraKeyPattern};
use crate::modules::project_registry::Project;
use crate::platform::types::ActiveWindowInfo;
use serde::{Deserialize, Serialize};

/// A decision about which project/jira key to attribute a window to.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Attribution {
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub jira_key: Option<String>,
    pub branch: Option<String>,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum Confidence {
    /// Matched by git branch Jira key
    High,
    /// Matched by project path only (no Jira key on branch)
    Medium,
    /// No match
    #[default]
    Low,
}

pub struct AttributionEngine {
    jira_patterns: Vec<JiraKeyPattern>,
}

impl AttributionEngine {
    pub fn new(jira_patterns: Vec<JiraKeyPattern>) -> Self {
        Self { jira_patterns }
    }

    /// Attribute a window event to a project and Jira issue.
    /// `projects` should be all registered projects.
    pub fn attribute(
        &self,
        window: &ActiveWindowInfo,
        projects: &[Project],
    ) -> Attribution {
        // Try to find which registered project's folder is in the exe path
        let matching_project = window.exe_path.as_deref()
            .and_then(|exe| best_project_match(exe, projects))
            .or_else(|| find_project_by_title(&window.window_title, projects))
            // Last resort: check if the window title contains the project's folder name.
            // This covers IDEs (VS Code, Rider, etc.) that show the workspace folder in
            // their title bar even when the IDE itself is not inside the project directory.
            .or_else(|| find_project_by_folder_name(&window.window_title, projects));

        let project_id = matching_project.as_ref().map(|p| p.id.clone());
        let project_name = matching_project.as_ref().map(|p| p.name.clone());

        // If we have a matching project, probe its git info
        let git_info: GitInfo = if let Some(ref proj) = matching_project {
            git_probe::probe(std::path::Path::new(&proj.path), &self.jira_patterns)
        } else {
            GitInfo::default()
        };

        let confidence = match (&project_id, &git_info.jira_key) {
            (Some(_), Some(_)) => Confidence::High,
            (Some(_), None) => Confidence::Medium,
            _ => Confidence::Low,
        };

        Attribution {
            project_id,
            project_name,
            jira_key: git_info.jira_key,
            branch: git_info.branch,
            confidence,
        }
    }
}

fn best_project_match<'a>(exe_path: &str, projects: &'a [Project]) -> Option<&'a Project> {
    // Normalize separators
    let norm_exe = exe_path.replace('\\', "/").to_lowercase();
    projects.iter()
        .filter(|p| {
            let norm_proj = p.path.replace('\\', "/").to_lowercase();
            norm_exe.starts_with(&norm_proj)
        })
        .max_by_key(|p| p.path.len())
}

fn find_project_by_title<'a>(title: &str, projects: &'a [Project]) -> Option<&'a Project> {
    let lower_title = title.to_lowercase();
    projects.iter().find(|p| lower_title.contains(&p.name.to_lowercase()))
}

/// Match the window title against the last folder-name segment of each registered project path.
/// E.g. project path `D:/code/myapp` → folder `myapp`.
/// VS Code shows "file.ts - myapp - Visual Studio Code" → matches.
fn find_project_by_folder_name<'a>(title: &str, projects: &'a [Project]) -> Option<&'a Project> {
    let lower_title = title.to_lowercase();
    projects.iter()
        .filter_map(|p| {
            let folder = std::path::Path::new(&p.path)
                .file_name()
                .map(|n| n.to_string_lossy().to_lowercase())?;
            if folder.len() >= 3 && lower_title.contains(folder.as_str()) {
                Some((p, folder.len()))
            } else {
                None
            }
        })
        // Prefer the project whose folder name is longest (most specific match)
        .max_by_key(|(_, len)| *len)
        .map(|(p, _)| p)
}

/// Merge adjacent sessions with the same attribution into one.
/// This is used before displaying in the review UI.
pub fn merge_adjacent_sessions(sessions: &[crate::modules::session_store::Session])
    -> Vec<MergedSession>
{
    let mut result: Vec<MergedSession> = Vec::new();

    for s in sessions {
        if let Some(last) = result.last_mut() {
            let same_key = last.jira_key == s.jira_key;
            let same_project = last.project_id == s.project_id;
            let contiguous = is_contiguous(&last.end_time, &s.start_time, 300);

            if same_key && same_project && contiguous && !s.is_published && !last.is_published
                && !s.is_huddle && !last.is_huddle
            {
                last.end_time = s.end_time.clone();
                last.duration_secs += s.duration_secs;
                last.session_ids.push(s.id.clone());
                continue;
            }
        }
        result.push(MergedSession::from(s));
    }

    result
}

fn is_contiguous(end: &Option<String>, start: &str, gap_secs: i64) -> bool {
    let Some(e) = end else { return false };
    let Ok(et) = chrono::DateTime::parse_from_rfc3339(e) else { return false };
    let Ok(st) = chrono::DateTime::parse_from_rfc3339(start) else { return false };
    let gap = (st - et).num_seconds();
    gap >= 0 && gap <= gap_secs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedSession {
    pub session_ids: Vec<String>,
    pub project_id: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_secs: i64,
    pub jira_key: Option<String>,
    pub branch: Option<String>,
    pub is_published: bool,
    pub notes: Option<String>,
    pub is_huddle: bool,
    pub huddle_channel: Option<String>,
    pub is_manual: bool,
    pub window_title: Option<String>,
}

impl From<&crate::modules::session_store::Session> for MergedSession {
    fn from(s: &crate::modules::session_store::Session) -> Self {
        Self {
            session_ids: vec![s.id.clone()],
            project_id: s.project_id.clone(),
            start_time: s.start_time.clone(),
            end_time: s.end_time.clone(),
            duration_secs: s.duration_secs,
            jira_key: s.jira_key.clone(),
            branch: s.branch.clone(),
            is_published: s.is_published,
            notes: s.notes.clone(),
            is_huddle: s.is_huddle,
            huddle_channel: s.huddle_channel.clone(),
            is_manual: s.is_manual,
            window_title: s.window_title.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::session_store::Session;

    fn make_session(id: &str, start: &str, end: &str, dur: i64, key: Option<&str>, pub_: bool) -> Session {
        Session {
            id: id.into(),
            project_id: None,
            start_time: start.into(),
            end_time: Some(end.into()),
            duration_secs: dur,
            jira_key: key.map(Into::into),
            branch: None,
            window_title: None,
            process_name: None,
            is_idle: false,
            is_published: pub_,
            published_at: None,
            worklog_id: None,
            notes: None,
            created_at: start.into(),
            updated_at: start.into(),
            is_huddle: false,
            huddle_channel: None,
            is_manual: false,
        }
    }

    #[test]
    fn merges_adjacent_same_key() {
        let sessions = vec![
            make_session("1", "2024-01-15T09:00:00Z", "2024-01-15T09:30:00Z", 1800, Some("PROJ-1"), false),
            make_session("2", "2024-01-15T09:32:00Z", "2024-01-15T10:00:00Z", 1680, Some("PROJ-1"), false),
        ];
        let merged = merge_adjacent_sessions(&sessions);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].duration_secs, 3480);
        assert_eq!(merged[0].session_ids.len(), 2);
    }

    #[test]
    fn does_not_merge_different_keys() {
        let sessions = vec![
            make_session("1", "2024-01-15T09:00:00Z", "2024-01-15T09:30:00Z", 1800, Some("PROJ-1"), false),
            make_session("2", "2024-01-15T09:32:00Z", "2024-01-15T10:00:00Z", 1680, Some("PROJ-2"), false),
        ];
        let merged = merge_adjacent_sessions(&sessions);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn does_not_merge_large_gap() {
        let sessions = vec![
            make_session("1", "2024-01-15T09:00:00Z", "2024-01-15T09:30:00Z", 1800, Some("PROJ-1"), false),
            make_session("2", "2024-01-15T10:30:00Z", "2024-01-15T11:00:00Z", 1800, Some("PROJ-1"), false),
        ];
        let merged = merge_adjacent_sessions(&sessions);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn does_not_merge_published() {
        let sessions = vec![
            make_session("1", "2024-01-15T09:00:00Z", "2024-01-15T09:30:00Z", 1800, Some("PROJ-1"), true),
            make_session("2", "2024-01-15T09:32:00Z", "2024-01-15T10:00:00Z", 1680, Some("PROJ-1"), false),
        ];
        let merged = merge_adjacent_sessions(&sessions);
        assert_eq!(merged.len(), 2);
    }
}
