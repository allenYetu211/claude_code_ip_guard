import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { InstalledApp, MonitoredApp, NotifyStyle, Settings } from "../types";
import { COUNTRY_LIST, flagOf, nameOf } from "../lib/countries";

export function SettingsPanel({
  settings,
  saving,
  onSave,
}: {
  settings: Settings;
  saving: boolean;
  onSave: (s: Settings) => void;
}) {
  const [draft, setDraft] = useState<Settings>(settings);
  const [running, setRunning] = useState<InstalledApp[]>([]);
  const [pickBundle, setPickBundle] = useState<string>("");
  const [pickCountry, setPickCountry] = useState<string>("");

  useEffect(() => setDraft(settings), [settings]);

  useEffect(() => {
    invoke<InstalledApp[]>("list_running_apps").then(setRunning).catch(() => {});
  }, []);

  const dirty = useMemo(
    () => JSON.stringify(draft) !== JSON.stringify(settings),
    [draft, settings],
  );

  function addCountry(raw: string) {
    const cc = raw.trim().toUpperCase();
    if (cc.length !== 2) return;
    if (draft.allowedCountries.includes(cc)) return;
    setDraft({ ...draft, allowedCountries: [...draft.allowedCountries, cc] });
    setPickCountry("");
  }

  function removeCountry(cc: string) {
    setDraft({
      ...draft,
      allowedCountries: draft.allowedCountries.filter((x) => x !== cc),
    });
  }

  const availableCountries = useMemo(
    () => COUNTRY_LIST.filter((c) => !draft.allowedCountries.includes(c.code)),
    [draft.allowedCountries],
  );

  function toggleApp(bundleId: string) {
    setDraft({
      ...draft,
      monitoredApps: draft.monitoredApps.map((a) =>
        a.bundleId === bundleId ? { ...a, enabled: !a.enabled } : a,
      ),
    });
  }

  function removeApp(bundleId: string) {
    setDraft({
      ...draft,
      monitoredApps: draft.monitoredApps.filter((a) => a.bundleId !== bundleId),
    });
  }

  function addApp() {
    const picked = running.find((r) => r.bundleId === pickBundle);
    if (!picked) return;
    if (draft.monitoredApps.some((a) => a.bundleId === picked.bundleId)) return;
    const next: MonitoredApp = {
      bundleId: picked.bundleId,
      name: picked.name,
      enabled: true,
    };
    setDraft({ ...draft, monitoredApps: [...draft.monitoredApps, next] });
    setPickBundle("");
  }

  function setStep(field: "cacheTtlSecs" | "alertCooldownSecs", delta: number) {
    const min = 30;
    const max = 3600;
    setDraft({
      ...draft,
      [field]: Math.max(min, Math.min(max, draft[field] + delta)),
    });
  }

  return (
    <>
      <div className="section">
        <div className="section-title">允许地区</div>
        {draft.allowedCountries.length > 0 && (
          <div className="chips" style={{ marginBottom: 6 }}>
            {draft.allowedCountries.map((cc) => (
              <span key={cc} className="chip">
                {flagOf(cc)} {nameOf(cc)}
                <span className="chip-x" onClick={() => removeCountry(cc)}>×</span>
              </span>
            ))}
          </div>
        )}
        <div style={{ display: "flex", gap: 6 }}>
          <select
            value={pickCountry}
            onChange={(e) => setPickCountry(e.currentTarget.value)}
            style={{ flex: 1 }}
          >
            <option value="">添加国家/地区…</option>
            {availableCountries.map((c) => (
              <option key={c.code} value={c.code}>
                {flagOf(c.code)} {c.name} ({c.code})
              </option>
            ))}
          </select>
          <button
            className="ghost"
            onClick={() => addCountry(pickCountry)}
            disabled={!pickCountry}
          >
            添加
          </button>
        </div>
      </div>

      <div className="section">
        <div className="section-title">提醒方式</div>
        <div className="segmented">
          {(["toast", "modal", "both"] as NotifyStyle[]).map((v) => (
            <button
              key={v}
              className={draft.notifyStyle === v ? "active" : ""}
              onClick={() => setDraft({ ...draft, notifyStyle: v })}
            >
              {v === "toast" && "通知"}
              {v === "modal" && "弹窗"}
              {v === "both" && "两者"}
            </button>
          ))}
        </div>
      </div>

      <div className="section">
        <div className="section-title">通用</div>
        <label className="toggle">
          <span>开机自动启动</span>
          <input
            type="checkbox"
            checked={draft.autoStart}
            onChange={(e) => setDraft({ ...draft, autoStart: e.currentTarget.checked })}
          />
          <span className="switch" />
        </label>

        <div className="field" style={{ marginTop: 10 }}>
          <span className="field-label">缓存时长 · {draft.cacheTtlSecs}s</span>
          <div className="stepper">
            <button onClick={() => setStep("cacheTtlSecs", -30)}>−</button>
            <input
              type="number"
              value={draft.cacheTtlSecs}
              min={30}
              max={3600}
              onChange={(e) =>
                setDraft({ ...draft, cacheTtlSecs: Number(e.currentTarget.value) || 300 })
              }
            />
            <button onClick={() => setStep("cacheTtlSecs", 30)}>+</button>
          </div>
        </div>

        <div className="field">
          <span className="field-label">提醒去抖 · {draft.alertCooldownSecs}s</span>
          <div className="stepper">
            <button onClick={() => setStep("alertCooldownSecs", -60)}>−</button>
            <input
              type="number"
              value={draft.alertCooldownSecs}
              min={30}
              max={3600}
              onChange={(e) =>
                setDraft({ ...draft, alertCooldownSecs: Number(e.currentTarget.value) || 600 })
              }
            />
            <button onClick={() => setStep("alertCooldownSecs", 60)}>+</button>
          </div>
        </div>
      </div>

      <div className="section">
        <div className="section-title">监控的应用</div>
        {draft.monitoredApps.length === 0 ? (
          <div className="empty">尚未监控任何应用</div>
        ) : (
          <ul className="apps">
            {draft.monitoredApps.map((a) => (
              <li key={a.bundleId} className="app-row">
                <div style={{ minWidth: 0 }}>
                  <div className="app-name">{a.name}</div>
                  <div className="app-bid">{a.bundleId}</div>
                </div>
                <label className="toggle" style={{ padding: 0 }}>
                  <input
                    type="checkbox"
                    checked={a.enabled}
                    onChange={() => toggleApp(a.bundleId)}
                  />
                  <span className="switch" />
                </label>
                <button
                  className="icon-btn"
                  title="移除"
                  onClick={() => removeApp(a.bundleId)}
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
        )}

        <div style={{ display: "flex", gap: 6, marginTop: 8 }}>
          <select
            value={pickBundle}
            onChange={(e) => setPickBundle(e.currentTarget.value)}
            style={{ flex: 1 }}
          >
            <option value="">添加运行中应用…</option>
            {running.map((r) => (
              <option key={r.bundleId} value={r.bundleId}>
                {r.name}
              </option>
            ))}
          </select>
          <button className="ghost" onClick={addApp} disabled={!pickBundle}>
            添加
          </button>
        </div>
      </div>

      <div style={{ display: "flex", gap: 6, marginTop: 4, marginBottom: 8 }}>
        <button
          className="ghost"
          style={{ flex: 1 }}
          onClick={() => setDraft(settings)}
          disabled={!dirty || saving}
        >
          重置
        </button>
        <button
          className="primary"
          style={{ flex: 1 }}
          onClick={() => onSave(draft)}
          disabled={!dirty || saving}
        >
          {saving ? "保存中…" : "保存设置"}
        </button>
      </div>
    </>
  );
}
