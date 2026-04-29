import { useCallback, useMemo, useState } from "react";
import { IpReportCard } from "./components/IpReportCard";
import { SettingsPanel } from "./components/SettingsPanel";
import { ViolationBanner } from "./components/ViolationBanner";
import { useIpReport } from "./hooks/useIpReport";
import { useSettings } from "./hooks/useSettings";
import { useTauriEvent } from "./hooks/useTauriEvent";
import { flagOf } from "./lib/countries";
import type { IpReport, ViolationPayload } from "./types";
import "./App.css";

type Activation = { bundleId: string | null; name: string | null; pid: number; at: string };
type ViolationItem = ViolationPayload & { id: string };
type Tab = "report" | "settings" | "log";

const TABS: { key: Tab; label: string }[] = [
  { key: "report", label: "Report" },
  { key: "settings", label: "Settings" },
  { key: "log", label: "Log" },
];

export default function App() {
  const { report, refreshing, error, refresh, setReport } = useIpReport();
  const { settings, saving, save } = useSettings();
  const [recent, setRecent] = useState<Activation[]>([]);
  const [violations, setViolations] = useState<ViolationItem[]>([]);
  const [tab, setTab] = useState<Tab>("report");

  // 托盘菜单/外部触发的"重新检测"回写到前端
  useTauriEvent<IpReport>("ip-report-updated", (r) => setReport(r));

  const onActivation = useCallback((a: Omit<Activation, "at">) => {
    setRecent((prev) => [{ ...a, at: timeFmt(new Date()) }, ...prev].slice(0, 12));
  }, []);
  useTauriEvent<Omit<Activation, "at">>("app-activated", onActivation);

  const onViolation = useCallback((v: ViolationPayload) => {
    const id = `${v.bundleId}-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    setViolations((prev) => [{ ...v, id }, ...prev].slice(0, 3));
  }, []);
  useTauriEvent<ViolationPayload>("ip-violation", onViolation);

  const trustClass = useMemo(() => {
    if (!report) return "bad";
    return report.trustScore >= 80 ? "ok" : report.trustScore >= 50 ? "warn" : "bad";
  }, [report]);

  const trustOffset = useMemo(() => {
    const C = 2 * Math.PI * 14; // r=14
    const pct = report ? Math.max(0, Math.min(100, report.trustScore)) / 100 : 0;
    return { dasharray: `${C * pct} ${C}`, circumference: C };
  }, [report]);

  const fetchedAt = useMemo(() => {
    if (!report) return "—";
    return timeFmt(new Date(report.fetchedAt));
  }, [report]);

  return (
    <main className="app">
      {/* Drag handle for the overlay titlebar */}
      <div className="drag" />

      {/* Status bar */}
      <header className="status">
        <div className="status-left">
          <div className="ip">
            {report?.country && <span className="flag">{flagOf(report.country)}</span>}
            {report?.ip ? (
              <span>{report.ip}</span>
            ) : (
              <span className="placeholder">— · — · — · —</span>
            )}
          </div>
          <div className="loc">
            {report ? (
              <>
                <span>{report.countryName ?? report.country ?? "未知地区"}</span>
                {report.region && (
                  <>
                    <span className="sep">·</span>
                    {report.region}
                  </>
                )}
                {report.city && (
                  <>
                    <span className="sep">·</span>
                    {report.city}
                  </>
                )}
              </>
            ) : (
              <span>{refreshing ? "Probing network…" : "Awaiting probe"}</span>
            )}
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <div className="ring">
            <svg viewBox="0 0 32 32">
              <circle cx="16" cy="16" r="14" fill="none" strokeWidth="2.5" className="ring-track" />
              <circle
                cx="16"
                cy="16"
                r="14"
                fill="none"
                strokeWidth="2.5"
                strokeLinecap="round"
                className={`ring-fill ${trustClass}`}
                style={{
                  strokeDasharray: trustOffset.dasharray,
                  strokeDashoffset: 0,
                }}
              />
            </svg>
            <div className={`ring-num ${trustClass}`}>
              {report?.trustScore ?? "—"}
            </div>
          </div>
          <button
            className={`refresh-btn ${refreshing ? "spinning" : ""}`}
            onClick={refresh}
            disabled={refreshing}
            title="重新检测"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="23 4 23 10 17 10" />
              <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
            </svg>
          </button>
        </div>
      </header>

      {/* Tabs */}
      <nav className="tabs">
        {TABS.map((t) => (
          <button
            key={t.key}
            className={`tab ${tab === t.key ? "active" : ""}`}
            onClick={() => setTab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </nav>
      <div className="tabs-divider" />

      {/* Violations */}
      {violations.map((v) => (
        <ViolationBanner
          key={v.id}
          v={v}
          onDismiss={() =>
            setViolations((prev) => prev.filter((x) => x.id !== v.id))
          }
        />
      ))}

      {/* Content */}
      <section className="content" key={tab}>
        <div className="tab-pane">
          {error && (
            <div className="section" style={{ borderColor: "rgba(255,69,58,0.4)" }}>
              <div style={{ color: "var(--bad)", fontSize: 11 }}>{error}</div>
            </div>
          )}

          {tab === "report" && (
            <IpReportCard
              report={report}
              refreshing={refreshing}
              onRefresh={refresh}
            />
          )}

          {tab === "settings" && settings && (
            <SettingsPanel
              settings={settings}
              saving={saving}
              onSave={save}
            />
          )}

          {tab === "settings" && !settings && (
            <div className="empty">加载设置…</div>
          )}

          {tab === "log" && (
            <div className="section">
              <div className="section-title">最近激活</div>
              {recent.length === 0 ? (
                <div className="empty">切换到任何应用后此处会出现记录…</div>
              ) : (
                <ul className="recent">
                  {recent.map((r, i) => (
                    <li key={i}>
                      <span className="recent-time">{r.at}</span>
                      <div style={{ minWidth: 0 }}>
                        <div className="recent-name">{r.name ?? "(未知)"}</div>
                        <div className="recent-bid">{r.bundleId ?? ""}</div>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </div>
      </section>

      {/* Footer */}
      <footer className="footer">
        <span>last probe · {fetchedAt}</span>
        <span>v0.1.0</span>
      </footer>
    </main>
  );
}

function timeFmt(d: Date): string {
  const pad = (n: number) => n.toString().padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}
