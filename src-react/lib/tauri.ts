import { invoke } from "@tauri-apps/api/core";

// ── Types ────────────────────────────────────────────────────────────────────

export interface Project {
  id: string;
  name: string;
  path: string;
  color: string;
  created_at: string;
  updated_at: string;
}

export interface Session {
  id: string;
  project_id: string | null;
  start_time: string;
  end_time: string | null;
  duration_secs: number;
  jira_key: string | null;
  branch: string | null;
  window_title: string | null;
  process_name: string | null;
  is_idle: boolean;
  is_published: boolean;
  published_at: string | null;
  worklog_id: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
  is_huddle: boolean;
  huddle_channel: string | null;
  is_manual: boolean;
}

export interface MergedSession {
  session_ids: string[];
  project_id: string | null;
  start_time: string;
  end_time: string | null;
  duration_secs: number;
  jira_key: string | null;
  branch: string | null;
  is_published: boolean;
  notes: string | null;
  is_huddle: boolean;
  huddle_channel: string | null;
  is_manual: boolean;
  window_title: string | null;
}

export interface HuddleStatus {
  active: boolean;
  channel: string | null;
  window_title: string | null;
  elapsed_secs: number;
}

export interface JiraConnectionInfo {
  id: string;
  name: string;
  base_url: string;
  email: string;
  is_active: boolean;
}

export interface JiraKeyPattern {
  pattern: string;
  description: string;
}

export interface AppSettings {
  idle_threshold_secs: number;
  poll_interval_secs: number;
  jira_patterns: JiraKeyPattern[];
  start_on_login: boolean;
  minimize_to_tray: boolean;
  track_slack_huddles: boolean;
  jira_enabled: boolean;
}

export interface StorageInfo {
  db_size_bytes: number;
  session_count: number;
}

export interface ActivitySnapshot {
  timestamp: string;
  state: "Running" | "Paused" | "Idle";
  window: { process_name: string; window_title: string; exe_path: string | null } | null;
  idle_secs: number;
}

// ── Project commands ─────────────────────────────────────────────────────────

export const listProjects = () => invoke<Project[]>("list_projects");
export const createProject = (input: { name: string; path: string; color?: string }) =>
  invoke<Project>("create_project", { input });
export const updateProject = (id: string, input: { name?: string; color?: string }) =>
  invoke<Project>("update_project", { id, input });
export const deleteProject = (id: string) => invoke<void>("delete_project", { id });

// ── Session commands ──────────────────────────────────────────────────────────

export const listSessionsForDay = (date: string) =>
  invoke<Session[]>("list_sessions_for_day", { date });
export const listMergedSessionsForDay = (date: string) =>
  invoke<MergedSession[]>("list_merged_sessions_for_day", { date });
export const listUnpublishedForDay = (date: string) =>
  invoke<Session[]>("list_unpublished_for_day", { date });
export const updateSession = (
  id: string,
  input: { end_time?: string; duration_secs?: number; jira_key?: string; notes?: string; project_id?: string | null; window_title?: string }
) => invoke<Session>("update_session", { id, input });
export const startManualSession = (label: string) =>
  invoke<Session>("start_manual_session", { label });
export const deleteSession = (id: string) => invoke<void>("delete_session", { id });
export const listSessionsForRange = (from: string, to: string) =>
  invoke<Session[]>("list_sessions_for_range", { from, to });

// ── Jira commands ─────────────────────────────────────────────────────────────

export const saveJiraConnection = (input: {
  name: string;
  base_url: string;
  email: string;
  api_token: string;
}) => invoke<JiraConnectionInfo>("save_jira_connection", { input });
export const getJiraConnection = () =>
  invoke<JiraConnectionInfo | null>("get_jira_connection");
export const testJiraConnection = () => invoke<string>("test_jira_connection");
export const publishWorklog = (input: { session_id: string; comment?: string }) =>
  invoke<string>("publish_worklog", { input });

// ── Settings commands ─────────────────────────────────────────────────────────

export const getSettings = () => invoke<AppSettings>("get_settings");
export const saveSettings = (settings: AppSettings) =>
  invoke<void>("save_settings", { settings });

// ── Storage commands ──────────────────────────────────────────────────────────

export const getStorageInfo = () => invoke<StorageInfo>("get_storage_info");
export const eraseSessions = () => invoke<void>("erase_sessions");

// ── Tracking commands ─────────────────────────────────────────────────────────

export const pauseTracking = () => invoke<void>("pause_tracking");
export const resumeTracking = () => invoke<void>("resume_tracking");
export const getTrackingState = () => invoke<string>("get_tracking_state");
export const getCurrentActivity = () => invoke<ActivitySnapshot | null>("get_current_activity");
