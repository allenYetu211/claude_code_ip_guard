use std::time::Instant;

use serde::Deserialize;

use crate::ip::{IpReport, ProviderResult};

const URL: &str = "https://ipinfo.io/json";
const NAME: &str = "ipinfo.io";

#[derive(Debug, Deserialize)]
struct Resp {
    ip: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    org: Option<String>,         // 形如 "AS13335 Cloudflare, Inc."
}

pub async fn fetch(client: &reqwest::Client) -> (IpReport, ProviderResult) {
    let start = Instant::now();
    let mut report = IpReport::default();
    let mut provider = ProviderResult {
        name: NAME.into(),
        ok: false,
        error: None,
        latency_ms: None,
        raw: None,
    };

    let res = client
        .get(URL)
        .header("User-Agent", "ip-guard/0.1 (macOS)")
        .header("Accept", "application/json")
        .send()
        .await;

    provider.latency_ms = Some(start.elapsed().as_millis() as u64);

    match res {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                provider.raw = Some(v.clone());
                match serde_json::from_value::<Resp>(v) {
                    Ok(parsed) => {
                        report.ip = parsed.ip;
                        report.country = parsed.country;
                        report.region = parsed.region;
                        report.city = parsed.city;
                        if let Some(org) = parsed.org {
                            let (asn, name) = split_org(&org);
                            report.asn = asn;
                            report.asn_org = name;
                        }
                        provider.ok = true;
                    }
                    Err(e) => provider.error = Some(format!("parse: {e}")),
                }
            }
            Err(e) => provider.error = Some(format!("json: {e}")),
        },
        Err(e) => provider.error = Some(format!("http: {e}")),
    }

    (report, provider)
}

// "AS13335 Cloudflare, Inc." -> (Some(13335), Some("Cloudflare, Inc."))
fn split_org(org: &str) -> (Option<u32>, Option<String>) {
    let trimmed = org.trim();
    if let Some(rest) = trimmed.strip_prefix("AS") {
        if let Some(idx) = rest.find(|c: char| c.is_whitespace()) {
            let asn = rest[..idx].parse().ok();
            let name = rest[idx..].trim().to_string();
            return (asn, if name.is_empty() { None } else { Some(name) });
        }
    }
    (None, Some(trimmed.to_string()))
}
