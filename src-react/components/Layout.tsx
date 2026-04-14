import React, { useState, useEffect } from "react";
import { NavLink } from "react-router-dom";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import {
  Clock,
  FolderOpen,
  Settings,
  Link2,
  LayoutDashboard,
  Pause,
  Play,
  Wifi,
  WifiOff,
  PanelLeftClose,
  PanelLeftOpen,
} from "lucide-react";
import { useTrackingState } from "../hooks/useTrackingState";
import { useI18n, TranslationKey } from "../lib/i18n";
import { getSettings, updateSession, listSessionsForDay } from "../lib/tauri";
import { todayDate } from "../lib/utils";

const BASE_NAV: { to: string; labelKey: TranslationKey; icon: React.ElementType; requiresJira?: boolean }[] = [
  { to: "/", labelKey: "nav.today", icon: LayoutDashboard },
  { to: "/review", labelKey: "nav.review", icon: Clock },
  { to: "/projects", labelKey: "nav.projects", icon: FolderOpen },
  { to: "/jira", labelKey: "nav.jira", icon: Link2, requiresJira: true },
  { to: "/settings", labelKey: "nav.settings", icon: Settings },
];

export default function Layout({ children }: { children: React.ReactNode }) {
  const { state, pause, resume } = useTrackingState();
  const { t, locale, setLocale } = useI18n();
  const [collapsed, setCollapsed] = useState(false);
  const [jiraEnabled, setJiraEnabled] = useState(false);
  const [showCloseModal, setShowCloseModal] = useState(false);

  // Stop all open sessions and then hide the window
  async function stopAndMinimize() {
    const sessions = await listSessionsForDay(todayDate());
    const open = sessions.filter((s) => !s.end_time);
    const now = new Date().toISOString();
    await Promise.all(open.map((s) => {
      const dur = Math.max(0, Math.floor((Date.now() - new Date(s.start_time).getTime()) / 1000));
      return updateSession(s.id, { end_time: now, duration_secs: dur });
    }));
    setShowCloseModal(false);
    await getCurrentWindow().hide();
  }

  function refreshSettings() {
    getSettings()
      .then((s) => setJiraEnabled(s.jira_enabled))
      .catch(() => {});
  }

  useEffect(() => {
    refreshSettings();
    window.addEventListener("tt:settings-changed", refreshSettings);
    const unlistenClose = listen("close-requested", () => setShowCloseModal(true));
    return () => {
      window.removeEventListener("tt:settings-changed", refreshSettings);
      unlistenClose.then((fn) => fn());
    };
  }, []);

  const nav = BASE_NAV.filter((item) => !item.requiresJira || jiraEnabled);

  const stateColor = state === "running" ? "#22c55e" : "#94a3b8";
  const StateIcon = state === "running" ? Wifi : WifiOff;
  const stateKey: TranslationKey = state === "running" ? "state.running" : "state.paused";

  return (
    <div className={`app-shell${collapsed ? " sidebar-collapsed" : ""}`}>
      <aside className="sidebar">
        <div className="sidebar-brand">
          {!collapsed && (
            <>
              <Clock size={22} color="#4f86f7" />
              <span>TimeTracker</span>
            </>
          )}
          <button
            className="btn-icon sidebar-collapse-btn"
            title={collapsed ? t("layout.expandNav") : t("layout.collapseNav")}
            onClick={() => setCollapsed((v) => !v)}
          >
            {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
          </button>
        </div>

        <nav className="sidebar-nav">
          {nav.map(({ to, labelKey, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              end={to === "/"}
              className={({ isActive }) =>
                "sidebar-nav-item" + (isActive ? " active" : "")
              }
              title={collapsed ? t(labelKey) : undefined}
            >
              <Icon size={16} />
              {!collapsed && <span>{t(labelKey)}</span>}
            </NavLink>
          ))}
        </nav>

        <div className="sidebar-status">
          <div className="status-indicator" title={t(stateKey)}>
            <StateIcon size={13} color={stateColor} />
            {!collapsed && (
              <span style={{ color: stateColor }}>{t(stateKey)}</span>
            )}
          </div>
          <div className="sidebar-status-actions">
            <button
              className="btn-icon"
              title={state === "paused" ? t("action.resumeTracking") : t("action.pauseTracking")}
              onClick={state === "paused" ? resume : pause}
            >
              {state === "paused" ? <Play size={14} /> : <Pause size={14} />}
            </button>
            {!collapsed && (
              <button
                className="btn-icon lang-btn"
                title="Switch language"
                onClick={() => setLocale(locale === "en" ? "ru" : "en")}
              >
                {t("lang.switchTo")}
              </button>
            )}
          </div>
        </div>
      </aside>

      <main className="main-content">{children}</main>

      {/* ── Close confirmation modal ─────────────────────────────────────── */}
      {showCloseModal && (
        <div className="modal-overlay">
          <div className="modal">
            <div className="modal-title">{t("close.title")}</div>
            <p className="modal-body">{t("close.body")}</p>
            <div className="modal-actions">
              <button className="btn btn-ghost" onClick={() => setShowCloseModal(false)}>
                {t("close.cancel")}
              </button>
              <button className="btn btn-ghost" onClick={async () => {
                setShowCloseModal(false);
                await getCurrentWindow().hide();
              }}>
                {t("close.minimize")}
              </button>
              <button className="btn btn-primary" onClick={stopAndMinimize}>
                {t("close.stopAndMinimize")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
