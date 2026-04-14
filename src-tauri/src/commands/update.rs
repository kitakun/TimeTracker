use semver::Version;
use tauri::command;

const RELEASES_API: &str =
    "https://api.github.com/repos/kitakun/TimeTracker/releases/latest";

/// Fetches the latest release tag from GitHub and returns it when the remote
/// version is strictly *newer* than the currently running build.
/// Returns `None` on network error, rate-limit, parse failure, or when the
/// running version is already up to date (or ahead of the latest release).
#[command]
pub async fn check_for_update() -> Result<Option<String>, String> {
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            "TimeTracker/",
            env!("CARGO_PKG_VERSION"),
            " (update-check)"
        ))
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = match client.get(RELEASES_API).send().await {
        Ok(r) => r,
        // Network unavailable or DNS failure — silently return nothing.
        Err(_) => return Ok(None),
    };

    if !resp.status().is_success() {
        return Ok(None);
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let tag = match json["tag_name"].as_str() {
        Some(t) => t.to_string(),
        None => return Ok(None),
    };

    // Strip a leading "v" so both "v1.2.3" and "1.2.3" parse correctly.
    let tag_bare = tag.trim_start_matches('v');
    let current_bare = env!("CARGO_PKG_VERSION").trim_start_matches('v');

    // Parse both as semver.  If either fails, fall back to string equality so
    // non-standard tags (e.g. "nightly-abc") at least detect an exact match.
    let is_newer = match (Version::parse(tag_bare), Version::parse(current_bare)) {
        (Ok(remote), Ok(local)) => remote > local,
        _ => tag_bare != current_bare,
    };

    if is_newer { Ok(Some(tag)) } else { Ok(None) }
}
