import { useState, useEffect, useCallback } from "react";
import { format, subDays } from "date-fns";
import {
  listMergedSessionsForDay, listProjects, publishWorklog,
  updateSession, deleteSession, MergedSession, Project
} from "../lib/tauri";
import { formatDurationHuman, formatTime, totalDurationSecs } from "../lib/utils";
import { useI18n } from "../lib/i18n";
import {
  ChevronLeft, ChevronRight, Send, Trash2, Edit2, Check, X, Phone
} from "lucide-react";

export default function Review() {
  const { t } = useI18n();
  const [date, setDate] = useState(() => format(new Date(), "yyyy-MM-dd"));
  const [sessions, setSessions] = useState<MergedSession[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [publishing, setPublishing] = useState<Record<number, boolean>>({});
  const [editing, setEditing] = useState<number | null>(null);
  const [editBuf, setEditBuf] = useState<{ jira_key: string; notes: string; duration_secs: number }>({
    jira_key: "", notes: "", duration_secs: 0
  });
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const reload = useCallback(async () => {
    const [s, p] = await Promise.all([listMergedSessionsForDay(date), listProjects()]);
    setSessions(s);
    setProjects(p);
  }, [date]);

  useEffect(() => { reload(); }, [reload]);

  const prevDay = () => setDate(format(subDays(new Date(date), 1), "yyyy-MM-dd"));
  const nextDay = () => setDate(format(subDays(new Date(date), -1), "yyyy-MM-dd"));

  const projById = (id: string | null) => projects.find((p) => p.id === id);

  async function handlePublish(idx: number, s: MergedSession) {
    if (!s.jira_key) { setError(t("review.noJiraKey")); return; }
    setPublishing((p) => ({ ...p, [idx]: true }));
    try {
      const sessionId = s.session_ids[0];
      await publishWorklog({ session_id: sessionId, comment: s.notes ?? undefined });
      setSuccess(t("review.publishedSuccess", { duration: formatDurationHuman(s.duration_secs), key: s.jira_key }));
      await reload();
    } catch (e) {
      setError(String(e));
    } finally {
      setPublishing((p) => ({ ...p, [idx]: false }));
    }
  }

  async function handleDelete(s: MergedSession) {
    if (!confirm(t("review.deleteConfirm"))) return;
    for (const id of s.session_ids) await deleteSession(id);
    await reload();
  }

  function startEdit(idx: number, s: MergedSession) {
    setEditing(idx);
    setEditBuf({ jira_key: s.jira_key ?? "", notes: s.notes ?? "", duration_secs: s.duration_secs });
  }

  async function saveEdit(s: MergedSession) {
    const id = s.session_ids[0];
    await updateSession(id, {
      jira_key: editBuf.jira_key || undefined,
      notes: editBuf.notes || undefined,
      duration_secs: editBuf.duration_secs,
    });
    setEditing(null);
    await reload();
  }

  const unpublished = sessions.filter((s) => !s.is_published);
  const total = totalDurationSecs(sessions);
  const unpublishedTotal = totalDurationSecs(unpublished);

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("review.title")}</h1>
          <p className="page-subtitle">{t("review.subtitle")}</p>
        </div>
        <div className="date-nav">
          <button className="btn btn-ghost btn-icon" onClick={prevDay}><ChevronLeft size={16} /></button>
          <input
            type="date"
            className="date-input"
            value={date}
            onChange={(e) => setDate(e.target.value)}
          />
          <button className="btn btn-ghost btn-icon" onClick={nextDay}><ChevronRight size={16} /></button>
        </div>
      </div>

      {error && (
        <div className="alert alert-error">
          <X size={14} onClick={() => setError(null)} style={{ cursor: "pointer" }} />
          {error}
        </div>
      )}
      {success && (
        <div className="alert alert-success">
          <Check size={14} onClick={() => setSuccess(null)} style={{ cursor: "pointer" }} />
          {success}
        </div>
      )}

      <div className="card-row">
        <div className="stat-card">
          <div className="stat-label">{t("review.total")}</div>
          <div className="stat-value">{formatDurationHuman(total)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">{t("review.unpublished")}</div>
          <div className="stat-value">{formatDurationHuman(unpublishedTotal)}</div>
        </div>
        <div className="stat-card">
          <div className="stat-label">{t("review.publishable")}</div>
          <div className="stat-value">{t("review.publishableSessions", { count: unpublished.filter((s) => s.jira_key).length })}</div>
        </div>
      </div>

      {sessions.length === 0 ? (
        <div className="card"><div className="empty">{t("review.noSessions")}</div></div>
      ) : (
        <div className="card review-card">
          <div className="review-table-wrap">
            <table className="review-table">
              <thead>
                <tr>
                  <th>{t("review.colTime")}</th>
                  <th>{t("review.colDuration")}</th>
                  <th>{t("review.colIssue")}</th>
                  <th>{t("review.colBranch")}</th>
                  <th>{t("review.colProject")}</th>
                  <th>{t("review.colNotes")}</th>
                  <th>{t("review.colStatus")}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((s, i) => {
                  const proj = projById(s.project_id);
                  const isEditing = editing === i;
                  return (
                    <tr key={i} className={`${s.is_published ? "row-published" : ""} ${s.is_huddle ? "row-huddle" : ""}`}>
                      <td className="cell-time">
                        {formatTime(s.start_time)}–{s.end_time ? formatTime(s.end_time) : "…"}
                      </td>
                      <td>
                        {isEditing ? (
                          <input
                            type="number"
                            className="inline-input"
                            style={{ width: 70 }}
                            value={Math.round(editBuf.duration_secs / 60)}
                            onChange={(e) => setEditBuf((b) => ({ ...b, duration_secs: Number(e.target.value) * 60 }))}
                          />
                        ) : (
                          formatDurationHuman(s.duration_secs)
                        )}
                      </td>
                      <td>
                        {isEditing ? (
                          <input
                            className="inline-input"
                            value={editBuf.jira_key}
                            placeholder="PROJ-123"
                            onChange={(e) => setEditBuf((b) => ({ ...b, jira_key: e.target.value.toUpperCase() }))}
                          />
                        ) : s.is_huddle ? (
                          <span className="huddle-badge">
                            <Phone size={10} /> {s.huddle_channel ? `#${s.huddle_channel}` : "Huddle"}
                          </span>
                        ) : s.jira_key ? (
                          <span className="jira-badge">{s.jira_key}</span>
                        ) : (
                          <span className="text-muted">–</span>
                        )}
                      </td>
                      <td className="text-muted cell-branch">{s.branch?.slice(0, 30) ?? "–"}</td>
                      <td>
                        {proj && (
                          <span className="proj-badge" style={{ backgroundColor: proj.color + "22", color: proj.color }}>
                            {proj.name}
                          </span>
                        )}
                      </td>
                      <td>
                        {isEditing ? (
                          <input
                            className="inline-input"
                            value={editBuf.notes}
                            placeholder="notes…"
                            onChange={(e) => setEditBuf((b) => ({ ...b, notes: e.target.value }))}
                          />
                        ) : (
                          <span className="text-muted">{s.notes ?? ""}</span>
                        )}
                      </td>
                      <td>
                        {s.is_published ? (
                          <span className="published-badge">{t("review.statusPublished")}</span>
                        ) : (
                          <span className="unpublished-badge">{t("review.statusPending")}</span>
                        )}
                      </td>
                      <td>
                        <div className="row-actions">
                          {isEditing ? (
                            <>
                              <button className="btn-icon text-green" title={t("review.titleSave")} onClick={() => saveEdit(s)}><Check size={14} /></button>
                              <button className="btn-icon text-muted" title={t("review.titleCancel")} onClick={() => setEditing(null)}><X size={14} /></button>
                            </>
                          ) : (
                            <>
                              {!s.is_published && (
                                <>
                                  <button className="btn-icon" title={t("review.titleEdit")} onClick={() => startEdit(i, s)}><Edit2 size={13} /></button>
                                  <button
                                    className="btn-icon text-blue"
                                    title={t("review.titlePublish")}
                                    disabled={!s.jira_key || publishing[i]}
                                    onClick={() => handlePublish(i, s)}
                                  >
                                    <Send size={13} />
                                  </button>
                                  <button className="btn-icon text-red" title={t("review.titleDelete")} onClick={() => handleDelete(s)}><Trash2 size={13} /></button>
                                </>
                              )}
                            </>
                          )}
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
