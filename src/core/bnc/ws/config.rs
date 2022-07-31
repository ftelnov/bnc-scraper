use derive_getters::Getters;
use serde::Deserialize;

/// Configuration of websocket BNC part.
#[derive(Debug, Clone, Deserialize, Getters)]
pub struct WsCfg {
    pub baseurl: String,
}

impl Default for WsCfg {
    fn default() -> Self {
        Self {
            baseurl: String::from("wss://stream.binance.com:9443"),
        }
    }
}
