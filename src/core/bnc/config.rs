use derive_getters::Getters;
use serde::Deserialize;

#[derive(Debug, Clone, Getters, Deserialize)]
pub struct BncCfg {
    pub baseurl: String,
}

impl Default for BncCfg {
    fn default() -> Self {
        Self {
            baseurl: "https://api.binance.com".into(),
        }
    }
}
