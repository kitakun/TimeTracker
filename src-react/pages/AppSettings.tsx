import { useState, useEffect } from "react";
import { getSettings, saveSettings, AppSettings, getStorageInfo, eraseSessions, StorageInfo, checkForUpdate } from "../lib/tauri";
import { useI18n } from "../lib/i18n";
import { useToast } from "../lib/toast";
import { Check, Plus, Trash2, RefreshCw, Database, Download } from "lucide-react";
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

function testPattern(pattern: string, input: string): string | null {
  if (!pattern || !input) return null;
  try {
    const re = new RegExp(pattern);
    const m = input.match(re);
    return (m && m[1]) ? m[1] : null;
  } catch {
    return null;
  }
}

export default function AppSettingsPage() {
  const { t } = useI18n();
  const { toast } = useToast();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [testInput, setTestInput] = useState("");
  const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
  const [erasing, setErasing] = useState(false);
  const [currentVersion, setCurrentVersion] = useState<string | null>(null);
  const [latestVersion, setLatestVersion] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setSettings);
    getStorageInfo().then(setStorageInfo);
    getVersion().then(setCurrentVersion);
    checkForUpdate().then((tag) => { if (tag) setLatestVersion(tag); });
  }, []);

  async function handleErase() {
    if (!storageInfo) return;
    const msg = t("appSettings.eraseConfirm", { count: storageInfo.session_count });
    if (!window.confirm(msg)) return;
    setErasing(true);
    try {
      await eraseSessions();
      const info = await getStorageInfo();
      setStorageInfo(info);
      toast(t("appSettings.erased"), "success");
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setErasing(false);
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  }

  async function handleSave() {
    if (!settings) return;
    setSaving(true);
    try {
      await saveSettings(settings);
      toast(t("appSettings.saved"), "success");
      window.dispatchEvent(new CustomEvent("tt:settings-changed"));
    } catch (e) {
      toast(String(e), "error");
    } finally {
      setSaving(false);
    }
  }

  function addPattern() {
    if (!settings) return;
    setSettings((s) => s ? ({
      ...s,
      jira_patterns: [...s.jira_patterns, { pattern: "", description: "" }]
    }) : s);
  }

  function removePattern(i: number) {
    if (!settings) return;
    setSettings((s) => s ? ({
      ...s,
      jira_patterns: s.jira_patterns.filter((_, idx) => idx !== i)
    }) : s);
  }

  if (!settings) return <div className="page"><div className="empty">{t("appSettings.loading")}</div></div>;

  return (
    <div className="page">
      <div className="page-header">
        <div>
          <h1 className="page-title">{t("appSettings.title")}</h1>
          <p className="page-subtitle">{t("appSettings.subtitle")}</p>
        </div>
        {currentVersion && (
          <div className="version-info">
            <span className="version-tag">v{currentVersion}</span>
            {latestVersion && (
              <button
                className="btn btn-primary btn-sm version-update-btn"
                onClick={() => openUrl("https://github.com/kitakun/TimeTracker/releases")}
              >
                <Download size={12} /> {t("appSettings.downloadUpdate")}
              </button>
            )}
          </div>
        )}
      </div>

      {/* ── System ────────────────────────────────────────── */}
      <div className="card mb-4">
        <div className="card-title">{t("appSettings.system")}</div>
        <div className="form-row">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={settings.minimize_to_tray}
              onChange={(e) => setSettings((s) => s ? ({ ...s, minimize_to_tray: e.target.checked }) : s)}
            />
            {t("appSettings.minimizeToTray")}
          </label>
        </div>
        <div className="form-row">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={settings.auto_merge_enabled}
              onChange={(e) => setSettings((s) => s ? ({ ...s, auto_merge_enabled: e.target.checked }) : s)}
            />
            <span>
              {t("appSettings.autoMerge")}
              <span className="input-hint">{t("appSettings.autoMergeHint")}</span>
            </span>
          </label>
        </div>
      </div>

      {/* ── Tracking ──────────────────────────────────────── */}
      <div className="card mb-4">
        <div className="card-title">{t("appSettings.tracking")}</div>

        <div className="form-row">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={settings.idle_detection_enabled}
              onChange={(e) => setSettings((s) => s ? ({ ...s, idle_detection_enabled: e.target.checked }) : s)}
            />
            <span>
              {t("appSettings.idleDetection")}
              <span className="input-hint">{t("appSettings.idleDetectionHint")}</span>
            </span>
          </label>
        </div>

        <div className="form-grid" style={{ opacity: settings.idle_detection_enabled ? 1 : 0.4, pointerEvents: settings.idle_detection_enabled ? "auto" : "none" }}>
          <label className="form-label">
            {t("appSettings.idleThreshold")}
            <div className="input-hint">{t("appSettings.idleHint")}</div>
            <input
              className="form-input"
              type="number"
              min={30}
              max={3600}
              value={settings.idle_threshold_secs}
              onChange={(e) => setSettings((s) => s ? ({ ...s, idle_threshold_secs: Number(e.target.value) }) : s)}
            />
          </label>
          <label className="form-label">
            {t("appSettings.pollInterval")}
            <div className="input-hint">{t("appSettings.pollHint")}</div>
            <input
              className="form-input"
              type="number"
              min={2}
              max={60}
              value={settings.poll_interval_secs}
              onChange={(e) => setSettings((s) => s ? ({ ...s, poll_interval_secs: Number(e.target.value) }) : s)}
            />
          </label>
        </div>
      </div>

      {/* ── Integrations ──────────────────────────────────── */}
      <div className="card mb-4">
        <div className="card-title">{t("appSettings.integrations")}</div>
        <div className="form-row">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={settings.track_slack_huddles}
              onChange={(e) => setSettings((s) => s ? ({ ...s, track_slack_huddles: e.target.checked }) : s)}
            />
            <span>
              {t("appSettings.slackHuddle")}
              <span className="input-hint">{t("appSettings.slackHuddleHint")}</span>
            </span>
          </label>
        </div>
      </div>

      {/* ── Boards ────────────────────────────────────────── */}
      <div className="card mb-4">
        <div className="card-title">{t("appSettings.boards")}</div>
        <div className="form-row">
          <label className="toggle-label">
            <input
              type="checkbox"
              checked={settings.jira_enabled}
              onChange={(e) => setSettings((s) => s ? ({ ...s, jira_enabled: e.target.checked }) : s)}
            />
            <span>
              {t("appSettings.jiraEnable")}
              <span className="input-hint">{t("appSettings.jiraEnableHint")}</span>
            </span>
          </label>
        </div>

        {/* Jira Key Patterns — only visible when Jira is enabled */}
        {settings.jira_enabled && (
          <div className="mt-4">
            <div className="card-subtitle">{t("appSettings.jiraPatterns")}</div>
            <p className="text-muted mb-3">{t("appSettings.patternsHint")}</p>

            <div className="pattern-test-row mb-3">
              <label className="form-label" style={{ flex: 1 }}>
                {t("appSettings.patternTestLabel")}
                <input
                  className="form-input"
                  placeholder={t("appSettings.patternTestPlaceholder")}
                  value={testInput}
                  onChange={(e) => setTestInput(e.target.value)}
                />
              </label>
            </div>

            <div className="pattern-list">
              {settings.jira_patterns.map((p, i) => {
                const matchResult = testInput ? testPattern(p.pattern, testInput) : null;
                const hasTest = testInput.length > 0;
                return (
                  <div key={i} className="pattern-entry">
                    <div className="pattern-row">
                      <input
                        className="form-input"
                        placeholder="([A-Z][A-Z0-9]+-\d+)"
                        value={p.pattern}
                        onChange={(e) => setSettings((s) => s ? ({
                          ...s,
                          jira_patterns: s.jira_patterns.map((pp, idx) => idx === i ? { ...pp, pattern: e.target.value } : pp)
                        }) : s)}
                      />
                      <input
                        className="form-input"
                        placeholder={t("appSettings.patternDesc")}
                        value={p.description}
                        onChange={(e) => setSettings((s) => s ? ({
                          ...s,
                          jira_patterns: s.jira_patterns.map((pp, idx) => idx === i ? { ...pp, description: e.target.value } : pp)
                        }) : s)}
                      />
                      <button className="btn-icon text-red" onClick={() => removePattern(i)}><Trash2 size={13} /></button>
                    </div>
                    {hasTest && p.pattern && (
                      <div className={`pattern-result ${matchResult ? "pattern-result--match" : "pattern-result--no-match"}`}>
                        {matchResult
                          ? t("appSettings.patternMatches", { key: matchResult })
                          : t("appSettings.patternNoMatch")}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
            <button className="btn btn-ghost mt-2" onClick={addPattern}><Plus size={13} /> {t("appSettings.addPattern")}</button>
          </div>
        )}
      </div>

      {/* ── Storage ───────────────────────────────────────── */}
      <div className="card mb-4">
        <div className="card-title">{t("appSettings.storage")}</div>
        {storageInfo ? (
          <div className="storage-info">
            <div className="storage-row">
              <Database size={14} />
              <span className="storage-label">{t("appSettings.storageDb")}</span>
              <span className="storage-value">{formatBytes(storageInfo.db_size_bytes)}</span>
            </div>
            <div className="storage-row">
              <span className="storage-label">{t("appSettings.storageSessions")}</span>
              <span className="storage-value">{storageInfo.session_count}</span>
            </div>
            <div className="storage-actions mt-3">
              <button
                className="btn btn-danger"
                onClick={handleErase}
                disabled={erasing || storageInfo.session_count === 0}
              >
                {erasing ? <RefreshCw size={13} className="spinning" /> : <Trash2 size={13} />}
                {t("appSettings.eraseData")}
              </button>
            </div>
          </div>
        ) : (
          <div className="text-muted">{t("appSettings.loadingStorage")}</div>
        )}
      </div>

      <div className="form-actions">
        <button className="btn btn-primary" onClick={handleSave} disabled={saving}>
          {saving ? <RefreshCw size={14} className="spinning" /> : <Check size={14} />}
          {t("appSettings.saveSettings")}
        </button>
      </div>
    </div>
  );
}
