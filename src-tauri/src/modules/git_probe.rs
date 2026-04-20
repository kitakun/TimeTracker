use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Spawn `git` with the given arguments and return its output.
///
/// On Windows we set `CREATE_NO_WINDOW` (0x0800_0000) so the console window
/// never briefly flashes on screen. On other platforms this is a plain
/// `Command::output()` call.
fn run_git(args: &[&str]) -> std::io::Result<std::process::Output> {
    let mut cmd = std::process::Command::new("git");
    cmd.args(args);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW — prevents the console from appearing at all.
        cmd.creation_flags(0x0800_0000);
    }
    cmd.output()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub jira_key: Option<String>,
    pub repo_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraKeyPattern {
    pub pattern: String,
    pub description: String,
}

impl Default for JiraKeyPattern {
    fn default() -> Self {
        Self {
            pattern: r"([A-Z][A-Z0-9]+-\d+)".to_string(),
            description: "Standard Jira key (e.g. PROJ-123)".to_string(),
        }
    }
}

/// Probe a directory for git info and extract a Jira key from the branch name.
pub fn probe(dir: &Path, patterns: &[JiraKeyPattern]) -> GitInfo {
    let repo_root = find_repo_root(dir);
    let branch = repo_root.as_deref().and_then(|root| get_branch(Path::new(root)));

    let jira_key = branch.as_deref().and_then(|b| {
        extract_jira_key(b, patterns)
    });

    GitInfo {
        branch,
        jira_key,
        repo_root,
    }
}

fn find_repo_root(dir: &Path) -> Option<String> {
    let output = run_git(&["-C", dir.to_str()?, "rev-parse", "--show-toplevel"]).ok()?;
    if output.status.success() {
        let root = String::from_utf8(output.stdout).ok()?.trim().to_string();
        Some(root)
    } else {
        None
    }
}

fn get_branch(repo_root: &Path) -> Option<String> {
    let output =
        run_git(&["-C", repo_root.to_str()?, "symbolic-ref", "--short", "HEAD"]).ok()?;
    if output.status.success() {
        let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();
        if branch.is_empty() { None } else { Some(branch) }
    } else {
        // Detached HEAD — try to get commit hash
        let output =
            run_git(&["-C", repo_root.to_str()?, "rev-parse", "--short", "HEAD"]).ok()?;
        if output.status.success() {
            let hash = String::from_utf8(output.stdout).ok()?.trim().to_string();
            Some(format!("detached:{}", hash))
        } else {
            None
        }
    }
}

/// Extract a Jira issue key from a branch name using the provided patterns.
/// Tries each pattern in order; returns the first match.
pub fn extract_jira_key(branch: &str, patterns: &[JiraKeyPattern]) -> Option<String> {
    let patterns_to_use: Vec<&JiraKeyPattern> = if patterns.is_empty() {
        return extract_with_default(branch);
    } else {
        patterns.iter().collect()
    };

    let upper = branch.to_uppercase();
    for p in patterns_to_use {
        if let Ok(re) = Regex::new(&p.pattern) {
            if let Some(caps) = re.captures(&upper) {
                if let Some(m) = caps.get(1).or_else(|| caps.get(0)) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }
    None
}

fn extract_with_default(branch: &str) -> Option<String> {
    let re = Regex::new(r"([A-Z][A-Z0-9]+-\d+)").ok()?;
    // Branch might be lowercase – normalize first
    let upper = branch.to_uppercase();
    let caps = re.captures(&upper)?;
    Some(caps.get(1)?.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_patterns() -> Vec<JiraKeyPattern> {
        vec![JiraKeyPattern::default()]
    }

    #[test]
    fn extracts_simple_key() {
        assert_eq!(extract_jira_key("feature/PROJ-123-add-login", &default_patterns()), Some("PROJ-123".into()));
    }

    #[test]
    fn extracts_key_at_start() {
        assert_eq!(extract_jira_key("PROJ-456", &default_patterns()), Some("PROJ-456".into()));
    }

    #[test]
    fn case_insensitive_prefix() {
        assert_eq!(extract_jira_key("feature/proj-789-fix", &default_patterns()), Some("PROJ-789".into()));
    }

    #[test]
    fn no_key_returns_none() {
        assert_eq!(extract_jira_key("main", &default_patterns()), None);
        assert_eq!(extract_jira_key("develop", &default_patterns()), None);
        assert_eq!(extract_jira_key("feature/add-new-button", &default_patterns()), None);
    }

    #[test]
    fn custom_pattern() {
        let patterns = vec![JiraKeyPattern {
            pattern: r"(TT-\d+)".to_string(),
            description: "TimeTracker".to_string(),
        }];
        assert_eq!(extract_jira_key("TT-99/some-work", &patterns), Some("TT-99".into()));
        // Standard key should not match with only custom pattern
        assert_eq!(extract_jira_key("feature/PROJ-123", &patterns), None);
    }

    #[test]
    fn multi_digit_project() {
        assert_eq!(extract_jira_key("AB2C-10-something", &default_patterns()), Some("AB2C-10".into()));
    }
}
