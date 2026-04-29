pub mod cloudflare;
pub mod ipapi_co;
pub mod ipinfo;
pub mod ipwho;

use crate::ip::{IpReport, ProviderResult};

pub type ProviderOutput = (IpReport, ProviderResult);

#[derive(Debug, Clone, Copy)]
pub enum Provider {
    IpWho,
    IpApiCo,
    IpInfo,
    Cloudflare,
}

impl Provider {
    pub fn name(self) -> &'static str {
        match self {
            Provider::IpWho => "ipwho.is",
            Provider::IpApiCo => "ipapi.co",
            Provider::IpInfo => "ipinfo.io",
            Provider::Cloudflare => "cloudflare",
        }
    }

    pub async fn fetch(self, client: &reqwest::Client) -> ProviderOutput {
        match self {
            Provider::IpWho => ipwho::fetch(client).await,
            Provider::IpApiCo => ipapi_co::fetch(client).await,
            Provider::IpInfo => ipinfo::fetch(client).await,
            Provider::Cloudflare => cloudflare::fetch(client).await,
        }
    }
}

pub const ALL: [Provider; 4] = [
    Provider::IpWho,
    Provider::IpApiCo,
    Provider::IpInfo,
    Provider::Cloudflare,
];
