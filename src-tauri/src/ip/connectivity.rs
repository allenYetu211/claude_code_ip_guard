use std::time::{Duration, Instant};

use crate::ip::{Connectivity, ProbeResult};

pub async fn probe(client: &reqwest::Client, url: &str) -> ProbeResult {
    let start = Instant::now();
    let res = client
        .head(url)
        .timeout(Duration::from_secs(5))
        .header("User-Agent", "ip-guard/0.1 (macOS)")
        .send()
        .await;
    let latency_ms = Some(start.elapsed().as_millis() as u64);
    match res {
        Ok(r) => {
            let status = r.status().as_u16();
            // 200 / 403 / 503 都视为 TCP+TLS 通了；只要 DNS/连接没失败就算可达。
            let reachable = matches!(status, 200..=599);
            ProbeResult {
                reachable,
                status: Some(status),
                latency_ms,
                error: None,
            }
        }
        Err(e) => ProbeResult {
            reachable: false,
            status: None,
            latency_ms,
            error: Some(e.to_string()),
        },
    }
}

pub async fn probe_all(client: &reqwest::Client) -> Connectivity {
    let (claude_ai, anthropic_com) = tokio::join!(
        probe(client, "https://claude.ai/"),
        probe(client, "https://www.anthropic.com/"),
    );
    let overall_ok = claude_ai.reachable || anthropic_com.reachable;
    Connectivity { claude_ai, anthropic_com, overall_ok }
}
