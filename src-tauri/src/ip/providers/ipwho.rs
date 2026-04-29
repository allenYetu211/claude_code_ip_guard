use std::time::Instant;

use serde::Deserialize;

use crate::ip::{IpReport, ProviderResult};

const URL: &str = "https://ipwho.is/";
const NAME: &str = "ipwho.is";

#[derive(Debug, Deserialize)]
struct Resp {
    success: bool,
    message: Option<String>,
    ip: Option<String>,
    country_code: Option<String>,
    country: Option<String>,
    region: Option<String>,
    city: Option<String>,
    connection: Option<Connection>,
}

#[derive(Debug, Deserialize)]
struct Connection {
    asn: Option<u32>,
    org: Option<String>,
    isp: Option<String>,
    domain: Option<String>,
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

    let elapsed = start.elapsed().as_millis() as u64;
    provider.latency_ms = Some(elapsed);

    match res {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(v) => {
                provider.raw = Some(v.clone());
                match serde_json::from_value::<Resp>(v) {
                    Ok(parsed) => {
                        if !parsed.success {
                            provider.error = parsed.message;
                            return (report, provider);
                        }
                        report.ip = parsed.ip;
                        report.country = parsed.country_code;
                        report.country_name = parsed.country;
                        report.region = parsed.region;
                        report.city = parsed.city;
                        if let Some(c) = parsed.connection {
                            report.asn = c.asn;
                            report.asn_org = c.org.or(c.domain);
                            report.isp = c.isp;
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
