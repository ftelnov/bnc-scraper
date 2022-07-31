use crate::core::bnc::data::{Amount, InlineOrder};
use serde::Deserialize;

/// Data container simply holds some serde value.
#[derive(Debug, Deserialize, Clone)]
pub struct WsDataContainer<T> {
    pub data: T,
}

/// Tick for an individual symbol's book update. Generally current best price for the provided symbol.
#[derive(Debug, Deserialize, Clone)]
pub struct SymbolBookTick {
    #[serde(rename = "u")]
    pub update_id: u64,

    #[serde(rename = "b")]
    pub bid_price: Amount,

    #[serde(rename = "B")]
    pub bid_qty: Amount,

    #[serde(rename = "a")]
    pub ask_price: Amount,

    #[serde(rename = "A")]
    pub ask_qty: Amount,
}
