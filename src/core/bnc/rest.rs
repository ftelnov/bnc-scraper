use super::config::BncCfg;
use reqwest::Client;

#[derive(Debug, Clone)]
pub struct BncRestClient {
    base_url: String,
    client: Client,
}

impl BncRestClient {
    pub fn new(client: Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    pub fn from_cfg(cfg: &BncCfg) -> Self {
        Self::new(Client::new(), cfg.baseurl.clone())
    }
}
