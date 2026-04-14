import { useState, useEffect } from "react";
import {
  saveJiraConnection, getJiraConnection, testJiraConnection, JiraConnectionInfo
} from "../lib/tauri";
import { useI18n } from "../lib/i18n";
import { Check, X, Link2, RefreshCw } from "lucide-react";

export default function JiraSettings() {
  const { t } = useI18n();
  const [current, setCurrent] = useState<JiraConnectionInfo | null>(null);
  const [form, setForm] = useState({ name: "Jira", base_url: "", email: "", api_token: "" });
  const [testing, setTesting] = useState(false);
  const [saving, setSaving] = useState(false);
  const [testResult, setTestResult] = useState<{ ok: boolean; msg: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getJiraConnection().then((c) => {
      setCurrent(c);
      if (c) setForm({ name: c.name, base_url: c.base_url, email: c.email, api_token: "" });
    });
  }, []);

  async function handleTest() {
    setTesting(true);
    setTestResult(null);
    try {
      const user = await testJiraConnection();
      setTestResult({ ok: true, msg: t("jira.connectedAs", { user }) });
    } catch (e) {
      setTestResult({ ok: false, msg: String(e) });
    } finally {
      setTesting(false);
    }
  }

  async function handleSave() {
    setError(null);
    setSaving(true);
    try {
      const c = await saveJiraConnection(form);
      setCurrent(c);
      setTestResult(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("jira.title")}</h1>
          <p className="page-subtitle">{t("jira.subtitle")}</p>
        </div>
      </div>

      {error && (
        <div className="alert alert-error">
          <X size={14} onClick={() => setError(null)} style={{ cursor: "pointer" }} />
          {error}
        </div>
      )}

      {current && !form.api_token && (
        <div className="alert alert-info">
          <Link2 size={14} />
          {t("jira.connectedInfo", { url: current.base_url, email: current.email })}
        </div>
      )}

      <div className="card">
        <div className="card-title">{t("jira.connectionDetails")}</div>
        <div className="form-grid">
          <label className="form-label">
            {t("jira.displayName")}
            <input
              className="form-input"
              value={form.name}
              onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
            />
          </label>
          <label className="form-label">
            {t("jira.baseUrl")}
            <input
              className="form-input"
              placeholder="https://yourcompany.atlassian.net"
              value={form.base_url}
              onChange={(e) => setForm((f) => ({ ...f, base_url: e.target.value }))}
            />
          </label>
          <label className="form-label">
            {t("jira.email")}
            <input
              className="form-input"
              type="email"
              placeholder="you@company.com"
              value={form.email}
              onChange={(e) => setForm((f) => ({ ...f, email: e.target.value }))}
            />
          </label>
          <label className="form-label">
            {t("jira.apiToken")}
            <input
              className="form-input"
              type="password"
              placeholder={current ? t("jira.apiTokenUpdatePlaceholder") : t("jira.apiTokenPlaceholder")}
              value={form.api_token}
              onChange={(e) => setForm((f) => ({ ...f, api_token: e.target.value }))}
            />
            <span className="form-hint">{t("jira.tokenHint")}</span>
          </label>
        </div>

        {testResult && (
          <div className={`alert ${testResult.ok ? "alert-success" : "alert-error"}`}>
            {testResult.ok ? <Check size={14} /> : <X size={14} />}
            {testResult.msg}
          </div>
        )}

        <div className="form-actions">
          <button className="btn btn-primary" onClick={handleSave} disabled={saving || !form.api_token}>
            {saving ? <RefreshCw size={14} className="spinning" /> : <Check size={14} />}
            {t("jira.save")}
          </button>
          <button className="btn btn-ghost" onClick={handleTest} disabled={testing || !current}>
            {testing ? <RefreshCw size={14} className="spinning" /> : <Link2 size={14} />}
            {t("jira.testConnection")}
          </button>
        </div>
      </div>

      <div className="card mt-4">
        <div className="card-title">{t("jira.howItWorks")}</div>
        <ul className="help-list">
          <li>{t("jira.step1")}</li>
          <li>{t("jira.step2")}</li>
          <li>{t("jira.step3")}</li>
          <li>{t("jira.step4")}</li>
        </ul>
      </div>
    </div>
  );
}
