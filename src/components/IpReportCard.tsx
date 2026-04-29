import type { IpReport } from "../types";
import { flagOf } from "../lib/countries";

function ms(n: number | null): string {
  return n == null ? "—" : `${n}ms`;
}

function bool(v: boolean | null, label: string): string | null {
  return v === true ? label : null;
}

export function IpReportCard({ report, refreshing }: {
  report: IpReport | null;
  refreshing: boolean;
  onRefresh: () => void;
}) {
  if (!report) {
    return (
      <div className="empty">
        {refreshing ? "正在检测当前 IP …" : "尚未检测"}
      </div>
    );
  }

  const flags = [
    bool(report.isDatacenter, "Datacenter"),
    bool(report.isVpn, "VPN"),
    bool(report.isProxy, "Proxy"),
    bool(report.isTor, "Tor"),
  ].filter(Boolean) as string[];

  return (
    <>
      <div className="section">
        <div className="section-title">网络属性</div>
        <dl className="kv">
          <dt>ASN</dt>
          <dd>{report.asn ? `AS${report.asn}` : "—"}</dd>
          <dt>组织</dt>
          <dd title={report.asnOrg ?? ""}>{report.asnOrg ?? "—"}</dd>
          <dt>ISP</dt>
          <dd title={report.isp ?? ""}>{report.isp ?? "—"}</dd>
          <dt>风险</dt>
          <dd>
            {flags.length ? (
              <span className="badges">
                {flags.map((f) => (
                  <span key={f} className="badge">{f}</span>
                ))}
              </span>
            ) : (
              <span className="badge muted">无</span>
            )}
          </dd>
        </dl>
      </div>

      <div className="section">
        <div className="section-title">服务连通性</div>
        <div className="probe">
          <span
            className={`dot ${report.connectivity.claudeAi.reachable ? "ok" : "bad"} ${
              refreshing ? "pulse" : ""
            }`}
          />
          <span className="probe-name">claude.ai</span>
          <span className="probe-meta">
            {report.connectivity.claudeAi.status ?? "—"} · {ms(report.connectivity.claudeAi.latencyMs)}
          </span>
        </div>
        <div className="probe">
          <span
            className={`dot ${report.connectivity.anthropicCom.reachable ? "ok" : "bad"} ${
              refreshing ? "pulse" : ""
            }`}
          />
          <span className="probe-name">anthropic.com</span>
          <span className="probe-meta">
            {report.connectivity.anthropicCom.status ?? "—"} ·{" "}
            {ms(report.connectivity.anthropicCom.latencyMs)}
          </span>
        </div>
      </div>

      {report.country && (
        <div className="section">
          <div className="section-title">地理位置</div>
          <dl className="kv">
            <dt>国家</dt>
            <dd>
              {flagOf(report.country)} {report.countryName ?? report.country}
            </dd>
            {report.region && (
              <>
                <dt>地区</dt>
                <dd>{report.region}</dd>
              </>
            )}
            {report.city && (
              <>
                <dt>城市</dt>
                <dd>{report.city}</dd>
              </>
            )}
          </dl>
        </div>
      )}
    </>
  );
}
