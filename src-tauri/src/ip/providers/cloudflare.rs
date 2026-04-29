use std::time::Instant;

use crate::ip::{IpReport, ProviderResult};

// Cloudflare 公共 trace 接口，返回 plain text key=value 行；
// 使用 1.1.1.1 而非 cloudflare.com 以避免 host 解析依赖。
const URL: &str = "https://1.1.1.1/cdn-cgi/trace";
const NAME: &str = "cloudflare";

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

    let text = match res {
        Ok(r) => r.text().await,
        Err(e) => {
            provider.error = Some(format!("http: {e}"));
            return (report, provider);
        }
    };

    let body = match text {
        Ok(t) => t,
        Err(e) => {
            provider.error = Some(format!("body: {e}"));
            return (report, provider);
        }
    };

    let mut map = serde_json::Map::new();
    for line in body.lines() {
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_string(), serde_json::Value::String(v.trim().to_string()));
            match k.trim() {
                "ip" => report.ip = Some(v.trim().to_string()),
                "loc" => report.country = Some(v.trim().to_uppercase()),
                _ => {}
            }
        }
    }
    provider.raw = Some(serde_json::Value::Object(map));
    provider.ok = report.ip.is_some();
    if !provider.ok {
        provider.error = Some("no ip in trace body".into());
    }

    (report, provider)
}
