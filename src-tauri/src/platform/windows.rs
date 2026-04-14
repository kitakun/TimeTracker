use super::types::{ActiveWindowInfo, HuddleInfo};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, MAX_PATH};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows::Win32::System::SystemInformation::GetTickCount64;

pub fn get_active_window() -> Option<ActiveWindowInfo> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        // Window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let window_title = String::from_utf16_lossy(&title_buf[..title_len as usize]);

        // Process ID
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return Some(ActiveWindowInfo {
                process_name: String::new(),
                window_title,
                exe_path: None,
            });
        }

        // Process name / exe path
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut path_buf = vec![0u16; MAX_PATH as usize];
        let mut size = MAX_PATH;
        let _ = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, windows::core::PWSTR(path_buf.as_mut_ptr()), &mut size);
        let exe_path = String::from_utf16_lossy(&path_buf[..size as usize]);

        let process_name = std::path::Path::new(&exe_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        Some(ActiveWindowInfo {
            process_name,
            window_title,
            exe_path: Some(exe_path),
        })
    }
}

/// Known IDE executable names (lower-case, no extension).
const IDE_PROCESSES: &[&str] = &[
    "code",           // VS Code / VSCodium
    "code - insiders",
    "cursor",         // Cursor
    "devenv",         // Visual Studio
    "rider64",        // JetBrains Rider
    "rider",
    "webstorm64",     // WebStorm
    "webstorm",
    "idea64",         // IntelliJ IDEA
    "intellijidea64",
    "clion64",        // CLion
    "clion",
    "phpstorm64",     // PhpStorm
    "goland64",       // GoLand
    "pycharm64",      // PyCharm
    "studio64",       // Android Studio
    "fleet",          // JetBrains Fleet
    "zed",            // Zed
    "sublime_text",   // Sublime Text
    "notepad++",      // Notepad++
    "eclipse",        // Eclipse
    "netbeans",       // NetBeans
];

/// Enumerate all visible windows owned by known IDE processes.
/// Used to detect work context even when the IDE is not the foreground window.
pub fn list_ide_windows() -> Vec<ActiveWindowInfo> {
    struct CollectState {
        results: Vec<ActiveWindowInfo>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam.0 as *mut CollectState);

        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        // Title must be non-empty
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        if title_len == 0 {
            return BOOL(1);
        }

        // Get process name
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return BOOL(1);
        }
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return BOOL(1),
        };
        let mut path_buf = vec![0u16; MAX_PATH as usize];
        let mut size = MAX_PATH;
        let _ = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(path_buf.as_mut_ptr()),
            &mut size,
        );
        let exe_path = String::from_utf16_lossy(&path_buf[..size as usize]);
        let process_name = std::path::Path::new(&exe_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if IDE_PROCESSES.contains(&process_name.as_str()) {
            let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
            state.results.push(ActiveWindowInfo {
                process_name: process_name.to_string(),
                window_title: title,
                exe_path: Some(exe_path),
            });
        }

        BOOL(1) // always continue to collect all IDE windows
    }

    let mut state = CollectState { results: Vec::new() };
    unsafe {
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut state as *mut _ as isize));
    }
    state.results
}

/// Enumerate all visible windows looking for a Slack Huddle.
/// Returns `Some(HuddleInfo)` when a huddle call window is detected.
pub fn get_huddle_window() -> Option<HuddleInfo> {
    struct SearchState {
        result: Option<HuddleInfo>,
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let state = &mut *(lparam.0 as *mut SearchState);

        // Skip invisible windows
        if !IsWindowVisible(hwnd).as_bool() {
            return BOOL(1);
        }

        // Get window title first (cheap) — skip if no "huddle" in it
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        if title_len == 0 {
            return BOOL(1);
        }
        let title = String::from_utf16_lossy(&title_buf[..title_len as usize]);
        if !title.to_lowercase().contains("huddle") {
            return BOOL(1);
        }

        // Confirm it belongs to slack.exe
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return BOOL(1);
        }
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return BOOL(1),
        };
        let mut path_buf = vec![0u16; MAX_PATH as usize];
        let mut size = MAX_PATH;
        let _ = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(path_buf.as_mut_ptr()),
            &mut size,
        );
        let exe_path = String::from_utf16_lossy(&path_buf[..size as usize]);
        let process_name = std::path::Path::new(&exe_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if process_name != "slack" {
            return BOOL(1);
        }

        state.result = Some(HuddleInfo {
            channel: parse_huddle_channel(&title),
            window_title: title,
        });
        BOOL(0) // stop enumeration — found what we need
    }

    let mut state = SearchState { result: None };
    unsafe {
        // Ignoring the error: EnumWindows returns Err when callback returns FALSE (early stop).
        let _ = EnumWindows(Some(enum_proc), LPARAM(&mut state as *mut _ as isize));
    }
    state.result
}

/// Parse a channel/recipient name from common Slack huddle window title patterns.
/// "Huddle in #general"  → Some("general")
/// "Huddle with @alice"  → Some("@alice")
/// "Huddle"              → None
fn parse_huddle_channel(title: &str) -> Option<String> {
    let lower = title.to_lowercase();
    if let Some(pos) = lower.find(" in #") {
        return Some(title[pos + 5..].trim().to_string());
    }
    if let Some(pos) = lower.find(" with ") {
        return Some(title[pos + 6..].trim().to_string());
    }
    None
}

pub fn get_idle_seconds() -> u64 {
    unsafe {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        if GetLastInputInfo(&mut lii).as_bool() {
            let tick_now = GetTickCount64();
            let elapsed_ms = tick_now.saturating_sub(lii.dwTime as u64);
            elapsed_ms / 1000
        } else {
            0
        }
    }
}
