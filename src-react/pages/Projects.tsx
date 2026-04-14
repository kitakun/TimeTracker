import { useState, useEffect, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { listProjects, createProject, updateProject, deleteProject, Project } from "../lib/tauri";
import { useI18n } from "../lib/i18n";
import { Plus, Trash2, Edit2, Check, X, FolderOpen } from "lucide-react";

const COLORS = ["#4A90E2", "#27ae60", "#e67e22", "#9b59b6", "#e74c3c", "#1abc9c", "#f39c12"];

export default function Projects() {
  const { t } = useI18n();
  const [projects, setProjects] = useState<Project[]>([]);
  const [adding, setAdding] = useState(false);
  const [editing, setEditing] = useState<string | null>(null);
  const [form, setForm] = useState({ name: "", path: "", color: COLORS[0] });
  const [editBuf, setEditBuf] = useState({ name: "", color: COLORS[0] });
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    const p = await listProjects();
    setProjects(p);
  }, []);

  async function pickFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      const dirName = selected.replace(/\\/g, "/").split("/").filter(Boolean).pop() ?? "";
      setForm((f) => ({
        ...f,
        path: selected,
        name: f.name.trim() === "" ? dirName : f.name,
      }));
    }
  }

  useEffect(() => { reload(); }, [reload]);

  async function handleAdd() {
    setError(null);
    try {
      await createProject({ name: form.name.trim(), path: form.path.trim(), color: form.color });
      setAdding(false);
      setForm({ name: "", path: "", color: COLORS[0] });
      await reload();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleUpdate(id: string) {
    await updateProject(id, { name: editBuf.name, color: editBuf.color });
    setEditing(null);
    await reload();
  }

  async function handleDelete(id: string, name: string) {
    if (!confirm(t("projects.deleteConfirm", { name }))) return;
    await deleteProject(id);
    await reload();
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("projects.title")}</h1>
          <p className="page-subtitle">{t("projects.subtitle")}</p>
        </div>
        <button className="btn btn-primary" onClick={() => setAdding(true)}>
          <Plus size={14} /> {t("projects.addProject")}
        </button>
      </div>

      {error && (
        <div className="alert alert-error">
          <X size={14} onClick={() => setError(null)} style={{ cursor: "pointer" }} />
          {error}
        </div>
      )}

      {adding && (
        <div className="card mb-4">
          <div className="card-title">{t("projects.newProject")}</div>
          <div className="form-grid">
            <label className="form-label">{t("projects.fieldName")}
              <input
                className="form-input"
                placeholder="My Project"
                value={form.name}
                onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
              />
            </label>
            <label className="form-label">{t("projects.fieldPath")}
              <div className="path-row">
                <input
                  className="form-input"
                  placeholder="C:\Users\me\repos\my-project"
                  value={form.path}
                  onChange={(e) => setForm((f) => ({ ...f, path: e.target.value }))}
                />
                <button
                  type="button"
                  className="btn btn-ghost btn-icon-square"
                  title={t("projects.browsePath")}
                  onClick={pickFolder}
                >
                  <FolderOpen size={15} />
                </button>
              </div>
            </label>
            <label className="form-label">{t("projects.fieldColor")}
              <div className="color-row">
                {COLORS.map((c) => (
                  <button
                    key={c}
                    className={`color-swatch ${form.color === c ? "selected" : ""}`}
                    style={{ backgroundColor: c }}
                    onClick={() => setForm((f) => ({ ...f, color: c }))}
                  />
                ))}
              </div>
            </label>
          </div>
          <div className="form-actions">
            <button className="btn btn-primary" onClick={handleAdd}><Check size={14} /> {t("action.save")}</button>
            <button className="btn btn-ghost" onClick={() => setAdding(false)}><X size={14} /> {t("action.cancel")}</button>
          </div>
        </div>
      )}

      {projects.length === 0 && !adding ? (
        <div className="card">
          <div className="empty">
            <FolderOpen size={32} color="#4f86f7" />
            <p>{t("projects.noProjects")}</p>
            <p className="text-muted">{t("projects.noProjectsHint")}</p>
          </div>
        </div>
      ) : (
        <div className="project-list">
          {projects.map((p) => {
            const isEditing = editing === p.id;
            return (
              <div key={p.id} className="project-card">
                <div className="project-color-bar" style={{ backgroundColor: p.color }} />
                <div className="project-body">
                  {isEditing ? (
                    <div className="form-grid">
                      <label className="form-label">{t("projects.fieldName")}
                        <input
                          className="form-input"
                          value={editBuf.name}
                          onChange={(e) => setEditBuf((b) => ({ ...b, name: e.target.value }))}
                        />
                      </label>
                      <label className="form-label">{t("projects.fieldColor")}
                        <div className="color-row">
                          {COLORS.map((c) => (
                            <button
                              key={c}
                              className={`color-swatch ${editBuf.color === c ? "selected" : ""}`}
                              style={{ backgroundColor: c }}
                              onClick={() => setEditBuf((b) => ({ ...b, color: c }))}
                            />
                          ))}
                        </div>
                      </label>
                    </div>
                  ) : (
                    <>
                      <div className="project-name">{p.name}</div>
                      <div className="project-path text-muted">{p.path}</div>
                    </>
                  )}
                </div>
                <div className="project-actions">
                  {isEditing ? (
                    <>
                      <button className="btn-icon text-green" onClick={() => handleUpdate(p.id)}><Check size={14} /></button>
                      <button className="btn-icon text-muted" onClick={() => setEditing(null)}><X size={14} /></button>
                    </>
                  ) : (
                    <>
                      <button className="btn-icon" onClick={() => { setEditing(p.id); setEditBuf({ name: p.name, color: p.color }); }}><Edit2 size={14} /></button>
                      <button className="btn-icon text-red" onClick={() => handleDelete(p.id, p.name)}><Trash2 size={14} /></button>
                    </>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
