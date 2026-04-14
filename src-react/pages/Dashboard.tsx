import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  listMergedSessionsForDay, listSessionsForDay, listProjects, updateSession, startManualSession,
  deleteSession, MergedSession, Project, ActivitySnapshot, HuddleStatus, Session,
} from "../lib/tauri";
import { formatDurationHuman, formatTime, todayDate, totalDurationSecs } from "../lib/utils";
import { useI18n } from "../lib/i18n";
import { useTrackingState } from "../hooks/useTrackingState";
import { useToast } from "../lib/toast";
import { RefreshCw, Clock, Tag, GitBranch, Pause, Play, Phone, Edit2, Check, X, Loader2, Square, Timer, Trash2 } from "lucide-react";

function formatElapsed(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export default function Dashboard() {
  const { t, locale } = useI18n();
  const { toast } = useToast();
  const { state: trackingState, pause, resume } = useTrackingState();
  const [sessions, setSessions] = useState<MergedSession[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [huddle, setHuddle] = useState<HuddleStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [elapsed, setElapsed] = useState(0);
  const [editingIdx, setEditingIdx] = useState<number | null>(null);
  const [noteBuf, setNoteBuf] = useState("");
  const elapsedTimerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const today = todayDate();

  // ── Manual tracking ───────────────────────────────────────────────────────
  const [manualLabel, setManualLabel] = useState("");
  const [manualSessions, setManualSessions] = useState<Session[]>([]);
  const [manualElapsed, setManualElapsed] = useState<Record<string, number>>({});
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameBuf, setRenameBuf] = useState("");

  const reload = useCallback(async () => {
    try {
      const [merged, raw, p] = await Promise.all([
        listMergedSessionsForDay(today),
        listSessionsForDay(today),
        listProjects(),
      ]);
      setSessions(merged);
      setProjects(p);
      // Restore any open manual sessions — survives page navigation
      setManualSessions(raw.filter((s) => s.is_manual && !s.end_time));
    } finally {
      setLoading(false);
    }
  }, [today]);

  useEffect(() => {
    reload();
    // Poll every 10 s; also reload on session-relevant events below.
    const timer = setInterval(reload, 10_000);

    const unlistenActivity = listen<ActivitySnapshot>("activity-update", (e) => {
      // Reload immediately when the tracking state changes (session created/ended)
      if (e.payload.state === "Running") reload();
    });
    const unlistenHuddle = listen<HuddleStatus>("huddle-status", (e) => {
      setHuddle(e.payload.active ? e.payload : null);
      if (!e.payload.active) reload(); // refresh after huddle ends
    });

    return () => {
      clearInterval(timer);
      unlistenActivity.then((fn) => fn());
      unlistenHuddle.then((fn) => fn());
    };
  }, [reload]);

  // Live elapsed counter — based on the open session's start_time
  const liveSession = sessions.find((s) => !s.end_time && s.project_id);
  useEffect(() => {
    if (elapsedTimerRef.current) clearInterval(elapsedTimerRef.current);
    if (trackingState !== "running" || !liveSession) {
      setElapsed(0);
      return;
    }
    const startMs = new Date(liveSession.start_time).getTime();
    const tick = () => setElapsed(Math.max(0, Math.floor((Date.now() - startMs) / 1000)));
    tick();
    elapsedTimerRef.current = setInterval(tick, 1000);
    return () => { if (elapsedTimerRef.current) clearInterval(elapsedTimerRef.current); };
  }, [trackingState, liveSession?.start_time]);

  const projectById = (id: string | null) => projects.find((p) => p.id === id);

  // Live total: closed sessions + real-time elapsed for the open one
  const closedSessions = sessions.filter((s) => s.end_time !== null);
  const liveTotal = totalDurationSecs(closedSessions) +
    (trackingState === "running" && liveSession ? elapsed : 0);
  const total = totalDurationSecs(sessions); // for stats cards that don't need sub-second update

  const byKey: Record<string, number> = {};
  for (const s of sessions) {
    if (!s.jira_key) continue;
    byKey[s.jira_key] = (byKey[s.jira_key] ?? 0) + s.duration_secs;
  }

  const dateLabel = new Date().toLocaleDateString(locale === "ru" ? "ru-RU" : "en-GB", {
    weekday: "long", day: "numeric", month: "long", year: "numeric",
  });

  const isTracking = trackingState === "running";

  // ── Inline note editing ───────────────────────────────────────────────────
  function startEdit(idx: number, s: MergedSession) {
    setEditingIdx(idx);
    setNoteBuf(s.notes ?? "");
  }

  async function saveNote(s: MergedSession) {
    const id = s.session_ids[0];
    await updateSession(id, { notes: noteBuf || undefined });
    setEditingIdx(null);
    await reload();
  }

  function cancelEdit() {
    setEditingIdx(null);
    setNoteBuf("");
  }

  async function handleDeleteSession(s: MergedSession) {
    for (const id of s.session_ids) await deleteSession(id);
    await reload();
  }

  // ── Manual session handlers ───────────────────────────────────────────────
  async function handleAddManual() {
    const label = manualLabel.trim();
    if (!label) { toast(t("manual.labelEmpty"), "error"); return; }
    const session = await startManualSession(label);
    setManualSessions((prev) => [...prev, session]);
    setManualLabel("");
  }

  async function handleStopManual(s: Session) {
    const elapsed = Math.max(0, Math.floor((Date.now() - new Date(s.start_time).getTime()) / 1000));
    const now = new Date().toISOString();
    await updateSession(s.id, { end_time: now, duration_secs: elapsed });
    setManualSessions((prev) => prev.filter((m) => m.id !== s.id));
    setManualElapsed((prev) => { const next = { ...prev }; delete next[s.id]; return next; });
    await reload();
  }

  async function handleRenameManual(s: Session) {
    const label = renameBuf.trim();
    if (!label) return;
    await updateSession(s.id, { window_title: label });
    setManualSessions((prev) => prev.map((m) => m.id === s.id ? { ...m, window_title: label } : m));
    setRenamingId(null);
  }

  // Tick manual session elapsed counters every second
  useEffect(() => {
    if (manualSessions.length === 0) return;
    const id = setInterval(() => {
      const now = Date.now();
      setManualElapsed(
        Object.fromEntries(
          manualSessions.map((s) => [s.id, Math.max(0, Math.floor((now - new Date(s.start_time).getTime()) / 1000))])
        )
      );
    }, 1000);
    return () => clearInterval(id);
  }, [manualSessions]);

  // ── Live project context (what we're tracking right now) ──────────────────
  const liveProject = liveSession ? projectById(liveSession.project_id) : null;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("dashboard.title")}</h1>
          <p className="page-subtitle">{dateLabel}</p>
        </div>
        <div className="page-header-actions">
          <button
            className={`btn ${isTracking ? "btn-ghost" : "btn-primary"}`}
            onClick={isTracking ? pause : resume}
          >
            {isTracking ? <Pause size={14} /> : <Play size={14} />}
            {isTracking ? t("action.pauseTracking") : t("action.resumeTracking")}
          </button>
          <button className="btn btn-ghost" onClick={reload}>
            <RefreshCw size={14} className={loading ? "spinning" : ""} />
            {t("action.refresh")}
          </button>
        </div>
      </div>

      {/* Live project session card — only when tracking a registered project */}
      {isTracking && liveSession && liveProject && (
        <div className="live-session-card">
          <div className="live-session-left">
            <div className="live-pulse-wrap">
              <span className="live-pulse" />
              <span className="live-label">{t("dashboard.liveSession")}</span>
              <span
                className="proj-badge"
                style={{ backgroundColor: liveProject.color + "33", color: liveProject.color }}
              >
                {liveProject.name}
              </span>
            </div>
            <div className="live-badges">
              {liveSession.jira_key && (
                <span className="jira-badge"><Tag size={10} /> {liveSession.jira_key}</span>
              )}
              {liveSession.branch && (
                <span className="branch-badge"><GitBranch size={10} /> {liveSession.branch}</span>
              )}
            </div>
          </div>
          <div className="live-session-right">
            <div className="live-elapsed-label">{t("dashboard.elapsed")}</div>
            <div className="live-elapsed">{formatElapsed(elapsed)}</div>
            <button className="btn btn-ghost btn-sm" onClick={pause}>
              <Pause size={12} /> {t("action.pauseTracking")}
            </button>
          </div>
        </div>
      )}

      {/* Live Huddle card */}
      {huddle && (
        <div className="huddle-card">
          <div className="huddle-card-left">
            <div className="huddle-pulse-wrap">
              <span className="huddle-pulse" />
              <Phone size={14} className="huddle-icon" />
              <span className="huddle-label">{t("huddle.live")}</span>
            </div>
            {huddle.channel ? (
              <div className="huddle-channel">
                {huddle.window_title?.toLowerCase().includes(" with ")
                  ? t("huddle.with", { channel: huddle.channel })
                  : t("huddle.in", { channel: huddle.channel })}
              </div>
            ) : huddle.window_title && (
              <div className="huddle-channel">{huddle.window_title}</div>
            )}
          </div>
          <div className="huddle-card-right">
            <div className="huddle-elapsed-label">{t("huddle.elapsed")}</div>
            <div className="huddle-elapsed">{formatElapsed(huddle.elapsed_secs)}</div>
          </div>
        </div>
      )}

      {/* ── Manual task input ─────────────────────────────────────────────── */}
      <div className="manual-track-row">
        <Timer size={14} className="manual-track-icon" />
        <input
          className="form-input manual-track-input"
          placeholder={t("manual.placeholder")}
          value={manualLabel}
          onChange={(e) => setManualLabel(e.target.value)}
          onKeyDown={(e) => { if (e.key === "Enter") handleAddManual(); }}
        />
        <button className="btn btn-primary btn-sm" onClick={handleAddManual}>
          {t("manual.add")}
        </button>
      </div>

      {/* ── Active manual session cards ───────────────────────────────────── */}
      {manualSessions.map((s) => (
        <div key={s.id} className="manual-session-card">
          <div className="manual-session-left">
            <span className="live-pulse" />
            {renamingId === s.id ? (
              <div className="manual-rename-row">
                <input
                  className="form-input manual-rename-input"
                  value={renameBuf}
                  autoFocus
                  onChange={(e) => setRenameBuf(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") handleRenameManual(s);
                    if (e.key === "Escape") setRenamingId(null);
                  }}
                />
                <button className="btn-icon text-green" onClick={() => handleRenameManual(s)}><Check size={13} /></button>
                <button className="btn-icon text-muted" onClick={() => setRenamingId(null)}><X size={13} /></button>
              </div>
            ) : (
              <span
                className="manual-session-label"
                title={t("manual.rename")}
                onClick={() => { setRenamingId(s.id); setRenameBuf(s.window_title ?? ""); }}
              >
                {s.window_title ?? "–"}
                <Edit2 size={11} className="manual-rename-icon" />
              </span>
            )}
          </div>
          <div className="manual-session-right">
            <span className="manual-elapsed">{formatElapsed(manualElapsed[s.id] ?? 0)}</span>
            <button className="btn btn-ghost btn-sm" onClick={() => handleStopManual(s)}>
              <Square size={11} /> {t("manual.stop")}
            </button>
          </div>
        </div>
      ))}

      <div className="card-row">
        <div className="stat-card">
          <div className="stat-label">{t("dashboard.totalTracked")}</div>
          {/* liveTotal updates every second via the elapsed counter */}
          <div className="stat-value">{formatDurationHuman(liveTotal)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">{t("dashboard.sessions")}</div>
          <div className="stat-value">{sessions.length}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">{t("dashboard.issues")}</div>
          <div className="stat-value">{Object.keys(byKey).length}</div>
        </div>
      </div>

      {Object.keys(byKey).length > 0 && (
        <div className="card mb-4">
          <div className="card-title">{t("dashboard.byIssue")}</div>
          <div className="key-breakdown">
            {Object.entries(byKey).sort((a, b) => b[1] - a[1]).map(([key, secs]) => {
              const isActiveKey = isTracking && liveSession?.jira_key === key;
              return (
                <div key={key} className="key-row">
                  <span className={`jira-badge${isActiveKey ? " jira-badge--active" : ""}`}>
                    {isActiveKey && <Loader2 size={10} className="tracking-spinner" />}
                    {key}
                  </span>
                  <div className="key-bar-wrap">
                    <div className="key-bar" style={{ width: `${Math.round((secs / total) * 100)}%` }} />
                  </div>
                  <span className="key-dur">{formatDurationHuman(secs)}</span>
                </div>
              );
            })}
          </div>
        </div>
      )}

      <div className="card">
        <div className="card-title">{t("dashboard.sessionsTitle")}</div>
        {loading && sessions.length === 0 ? (
          <div className="empty">{t("dashboard.loading")}</div>
        ) : sessions.length === 0 ? (
          <div className="empty">{t("dashboard.noSessions")}</div>
        ) : (
          <div className="session-list">
            {sessions.map((s, i) => {
              const proj = projectById(s.project_id);
              const isLive = !s.end_time && isTracking;
              const isEditingNote = editingIdx === i;
              return (
                <div
                  key={i}
                  className={[
                    "session-row",
                    s.is_published ? "session-row--published" : "",
                    isLive ? "session-row--live" : "",
                    s.is_huddle ? "session-row--huddle" : "",
                  ].join(" ").trim()}
                >
                  <div className="session-time">
                    <Clock size={12} />
                    {formatTime(s.start_time)}–{s.end_time ? formatTime(s.end_time) : "…"}
                  </div>
                  <div className="session-meta">
                    {s.is_manual && (
                      <span className="manual-badge">
                        <Timer size={10} /> {s.window_title ?? "–"}
                      </span>
                    )}
                    {s.is_huddle && (
                      <span className="huddle-badge">
                        <Phone size={10} /> {s.huddle_channel ? `#${s.huddle_channel}` : t("huddle.badge")}
                      </span>
                    )}
                    {s.jira_key && (
                      <span className={`jira-badge${isLive ? " jira-badge--active" : ""}`}>
                        {isLive
                          ? <Loader2 size={10} className="tracking-spinner" />
                          : <Tag size={10} />
                        }
                        {s.jira_key}
                      </span>
                    )}
                    {s.branch && (
                      <span className="branch-badge"><GitBranch size={10} /> {s.branch}</span>
                    )}
                    {proj && (
                      <span className="proj-badge" style={{ backgroundColor: proj.color + "33", color: proj.color }}>
                        {proj.name}
                      </span>
                    )}
                  </div>

                  {/* Inline note area */}
                  <div className="session-note-area">
                    {isEditingNote ? (
                      <div className="session-note-edit">
                        <input
                          className="inline-input session-note-input"
                          value={noteBuf}
                          placeholder={t("dashboard.notePlaceholder")}
                          autoFocus
                          onChange={(e) => setNoteBuf(e.target.value)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") saveNote(s);
                            if (e.key === "Escape") cancelEdit();
                          }}
                        />
                        <button className="btn-icon text-green" onClick={() => saveNote(s)}><Check size={13} /></button>
                        <button className="btn-icon text-muted" onClick={cancelEdit}><X size={13} /></button>
                      </div>
                    ) : (
                      <div className="session-note-display" onClick={() => startEdit(i, s)}>
                        {s.notes
                          ? <span className="session-note-text">{s.notes}</span>
                          : <span className="session-note-empty">{t("dashboard.addNote")}</span>
                        }
                        <Edit2 size={11} className="session-note-edit-icon" />
                      </div>
                    )}
                  </div>

                  <div className="session-dur">
                    {s.is_manual && !s.end_time
                      ? formatElapsed(manualElapsed[s.session_ids[0]] ?? 0)
                      : formatDurationHuman(s.duration_secs)
                    }
                  </div>
                  {s.is_manual && !s.end_time && (
                    <button
                      className="btn btn-ghost btn-sm"
                      onClick={() => {
                        const ms = manualSessions.find((m) => m.id === s.session_ids[0]);
                        if (ms) handleStopManual(ms);
                      }}
                    >
                      <Square size={11} /> {t("manual.stop")}
                    </button>
                  )}
                  {s.is_published && <span className="published-badge">{t("dashboard.published")}</span>}
                  {isLive && !s.is_manual && <span className="live-badge">● live</span>}
                  {s.end_time && !s.is_published && (
                    <button
                      className="btn-icon text-red session-delete-btn"
                      title={t("dashboard.deleteSession")}
                      onClick={() => handleDeleteSession(s)}
                    >
                      <Trash2 size={12} />
                    </button>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
