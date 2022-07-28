use crate::core::bnc::config::BncCfg;
use derive_getters::Getters;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, Getters, Default)]
pub struct CoreCfg {
    #[serde(default)]
    pub bnc: BncCfg,
}
