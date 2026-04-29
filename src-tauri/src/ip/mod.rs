use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod aggregator;
pub mod cache;
pub mod connectivity;
pub mod providers;

pub use aggregator::fetch_report;
pub use cache::ReportCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpReport {
    pub ip: Option<String>,
    pub country: Option<String>,
    pub country_name: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub asn: Option<u32>,
    pub asn_org: Option<String>,
    pub isp: Option<String>,
    pub is_datacenter: Option<bool>,
    pub is_vpn: Option<bool>,
    pub is_proxy: Option<bool>,
    pub is_tor: Option<bool>,
    pub trust_score: u8,
    pub connectivity: Connectivity,
    pub provider_results: Vec<ProviderResult>,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connectivity {
    pub claude_ai: ProbeResult,
    pub anthropic_com: ProbeResult,
    pub overall_ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub reachable: bool,
    pub status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderResult {
    pub name: String,
    pub ok: bool,
    pub error: Option<String>,
    pub latency_ms: Option<u64>,
    pub raw: Option<serde_json::Value>,
}

pub fn compute_trust_score(report: &IpReport, allowed_countries: &[String]) -> u8 {
    let mut score: i32 = 100;
    if report.is_datacenter == Some(true) { score -= 25; }
    if report.is_vpn == Some(true)        { score -= 30; }
    if report.is_proxy == Some(true)      { score -= 20; }
    if report.is_tor == Some(true)        { score -= 50; }
    if !allowed_countries.is_empty() {
        match &report.country {
            Some(c) if allowed_countries.iter().any(|a| a.eq_ignore_ascii_case(c)) => {}
            _ => score -= 20,
        }
    }
    if !report.connectivity.overall_ok { score -= 15; }
    score.clamp(0, 100) as u8
}

impl Default for IpReport {
    fn default() -> Self {
        Self {
            ip: None,
            country: None,
            country_name: None,
            region: None,
            city: None,
            asn: None,
            asn_org: None,
            isp: None,
            is_datacenter: None,
            is_vpn: None,
            is_proxy: None,
            is_tor: None,
            trust_score: 0,
            connectivity: Connectivity {
                claude_ai: ProbeResult::unknown(),
                anthropic_com: ProbeResult::unknown(),
                overall_ok: false,
            },
            provider_results: vec![],
            fetched_at: Utc::now(),
        }
    }
}

impl ProbeResult {
    pub fn unknown() -> Self {
        Self { reachable: false, status: None, latency_ms: None, error: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(country: Option<&str>, asn: Option<u32>, ip: Option<&str>) -> IpReport {
        IpReport {
            country: country.map(|s| s.to_string()),
            asn,
            ip: ip.map(|s| s.to_string()),
            ..IpReport::default()
        }
    }

    #[test]
    fn merge_country_majority_wins() {
        let merged = aggregator::merge_for_test(vec![
            r(Some("US"), Some(13335), Some("1.1.1.1")),
            r(Some("US"), None, None),
            r(Some("JP"), None, None),
            r(None, None, None),
        ]);
        assert_eq!(merged.country.as_deref(), Some("US"));
        assert_eq!(merged.is_datacenter, Some(true)); // 13335 = Cloudflare
    }

    #[test]
    fn merge_no_majority_falls_back_to_first() {
        let merged = aggregator::merge_for_test(vec![
            r(Some("JP"), None, None),
            r(Some("US"), None, None),
            r(Some("DE"), None, None),
        ]);
        assert_eq!(merged.country.as_deref(), Some("JP"));
    }

    #[test]
    fn trust_score_clamps_and_penalizes() {
        let mut rep = r(Some("US"), Some(16509), Some("1.2.3.4"));
        rep.is_datacenter = Some(true);
        rep.connectivity.overall_ok = false;
        let s = compute_trust_score(&rep, &["JP".into()]);
        // 100 - 25(dc) - 20(country) - 15(conn) = 40
        assert_eq!(s, 40);
    }
}
