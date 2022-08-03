use super::ws::config::WsCfg;
use derive_getters::Getters;
use serde::Deserialize;

#[derive(Debug, Clone, Getters, Deserialize)]
pub struct BncCfg {
    pub baseurl: String,

    /// Amount of messages tokio's channels can store.
    pub chnlcapacity: usize,
    pub ws: WsCfg,
}

impl Default for BncCfg {
    fn default() -> Self {
        Self {
            baseurl: "https://api.binance.com".into(),
            ws: Default::default(),
            chnlcapacity: 64,
        }
    }
}
