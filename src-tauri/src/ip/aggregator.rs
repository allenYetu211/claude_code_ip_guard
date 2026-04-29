use std::collections::HashMap;

use chrono::Utc;
use futures_util::future::join_all;

use crate::ip::{
    compute_trust_score, connectivity, providers, Connectivity, IpReport,
    ProviderResult,
};

// 已知数据中心 / 主流云厂商 ASN（不完全列表，后续可扩）
const DATACENTER_ASNS: &[u32] = &[
    16509, // AWS
    14618, // AWS
    15169, // Google
    396982,// Google Cloud
    8075,  // Microsoft Azure
    14061, // DigitalOcean
    63949, // Linode/Akamai
    20473, // Choopa/Vultr
    16276, // OVH
    24940, // Hetzner
    13335, // Cloudflare
    36352, // ColoCrossing
];

pub async fn fetch_report(
    client: &reqwest::Client,
    allowed_countries: &[String],
) -> IpReport {
    // 并发拉取所有 provider + 同时跑连通性探测
    let providers_fut = join_all(providers::ALL.iter().map(|p| p.fetch(client)));
    let (provider_outputs, connectivity) = tokio::join!(
        providers_fut,
        connectivity::probe_all(client),
    );

    let mut partials: Vec<(IpReport, ProviderResult)> = provider_outputs.into_iter().collect();
    let provider_results: Vec<ProviderResult> =
        partials.iter().map(|(_, p)| p.clone()).collect();

    let merged = merge(partials.drain(..).map(|(r, _)| r).collect(), connectivity);
    apply_trust(merged, allowed_countries, provider_results)
}

fn merge(reports: Vec<IpReport>, connectivity: Connectivity) -> IpReport {
    let mut out = IpReport::default();
    out.connectivity = connectivity;

    // 国家投票：≥2 票一致优先；否则取首个非空
    let mut tally: HashMap<String, usize> = HashMap::new();
    for r in &reports {
        if let Some(c) = &r.country {
            *tally.entry(c.to_uppercase()).or_default() += 1;
        }
    }
    out.country = tally
        .iter()
        .filter(|(_, &n)| n >= 2)
        .max_by_key(|(_, &n)| n)
        .map(|(c, _)| c.clone())
        .or_else(|| reports.iter().find_map(|r| r.country.clone()));

    // 其它字段：第一个非空
    out.ip = first_non_empty(&reports, |r| r.ip.clone());
    out.country_name = first_non_empty(&reports, |r| r.country_name.clone());
    out.region = first_non_empty(&reports, |r| r.region.clone());
    out.city = first_non_empty(&reports, |r| r.city.clone());
    out.asn = reports.iter().find_map(|r| r.asn);
    out.asn_org = first_non_empty(&reports, |r| r.asn_org.clone());
    out.isp = first_non_empty(&reports, |r| r.isp.clone());

    // datacenter：ASN 命中已知列表则标 true
    out.is_datacenter = match out.asn {
        Some(asn) if DATACENTER_ASNS.contains(&asn) => Some(true),
        Some(_) => Some(false),
        None => None,
    };

    out
}

fn first_non_empty<T: Clone>(reports: &[IpReport], f: impl Fn(&IpReport) -> Option<T>) -> Option<T> {
    reports.iter().find_map(f)
}

fn apply_trust(
    mut report: IpReport,
    allowed_countries: &[String],
    provider_results: Vec<ProviderResult>,
) -> IpReport {
    report.provider_results = provider_results;
    report.trust_score = compute_trust_score(&report, allowed_countries);
    report.fetched_at = Utc::now();
    report
}

// 仅供测试可见
#[cfg(test)]
pub fn merge_for_test(reports: Vec<IpReport>) -> IpReport {
    let conn = Connectivity {
        claude_ai: ProbeResult::unknown(),
        anthropic_com: ProbeResult::unknown(),
        overall_ok: false,
    };
    merge(reports, conn)
}
