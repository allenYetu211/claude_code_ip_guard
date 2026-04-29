export type ProbeResult = {
  reachable: boolean;
  status: number | null;
  latencyMs: number | null;
  error: string | null;
};

export type Connectivity = {
  claudeAi: ProbeResult;
  anthropicCom: ProbeResult;
  overallOk: boolean;
};

export type ProviderResult = {
  name: string;
  ok: boolean;
  error: string | null;
  latencyMs: number | null;
  raw: unknown | null;
};

export type MonitoredApp = {
  bundleId: string;
  name: string;
  enabled: boolean;
};

export type NotifyStyle = "toast" | "modal" | "both";

export type Settings = {
  monitoredApps: MonitoredApp[];
  allowedCountries: string[];
  cacheTtlSecs: number;
  alertCooldownSecs: number;
  notifyStyle: NotifyStyle;
  autoStart: boolean;
  language: string;
};

export type InstalledApp = {
  bundleId: string;
  name: string;
  path: string | null;
};

export type ViolationPayload = {
  bundleId: string;
  appName: string;
  country: string | null;
  countryName: string | null;
  ip: string | null;
  trustScore: number;
  allowedCountries: string[];
};

export type IpReport = {
  ip: string | null;
  country: string | null;
  countryName: string | null;
  region: string | null;
  city: string | null;
  asn: number | null;
  asnOrg: string | null;
  isp: string | null;
  isDatacenter: boolean | null;
  isVpn: boolean | null;
  isProxy: boolean | null;
  isTor: boolean | null;
  trustScore: number;
  connectivity: Connectivity;
  providerResults: ProviderResult[];
  fetchedAt: string;
};
