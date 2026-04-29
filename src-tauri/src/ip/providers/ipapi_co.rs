use std::time::Instant;

use serde::Deserialize;

use crate::ip::{IpReport, ProviderResult};

const URL: &str = "https://ipapi.co/json/";
const NAME: &str = "ipapi.co";

#[derive(Debug, Deserialize)]
struct Resp {
    ip: Option<String>,
    country_code: Option<String>,
    country_name: Option<String>,
    region: Option<String>,
    city: Option<String>,
    asn: Option<String>,           // 形如 "AS13335"
    org: Option<String>,
    error: Option<bool>,
    reason: Option<String>,
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
        .send()
        .await;

    provider.latency_ms = Some(start.elapsed().as_millis() as u64);

    match res {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                provider.raw = Some(v.clone());
                match serde_json::from_value::<Resp>(v) {
                    Ok(parsed) => {
                        if parsed.error == Some(true) {
                            provider.error = parsed.reason;
                            return (report, provider);
                        }
                        report.ip = parsed.ip;
                        report.country = parsed.country_code;
                        report.country_name = parsed.country_name;
                        report.region = parsed.region;
                        report.city = parsed.city;
                        report.asn = parsed.asn.as_deref().and_then(parse_asn);
                        report.asn_org = parsed.org;
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

fn parse_asn(s: &str) -> Option<u32> {
    s.trim().trim_start_matches("AS").parse().ok()
}
