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

/// Generalisation of price update.
///
/// All tickers' updates should be convertable to general representation.
#[derive(Debug, Clone)]
pub struct SymbolStateUpdate {
    pub update_id: u64,
    pub bid: InlineOrder,
    pub ask: InlineOrder,
}

impl From<SymbolBookTick> for SymbolStateUpdate {
    fn from(tick: SymbolBookTick) -> Self {
        Self {
            update_id: tick.update_id,
            bid: InlineOrder::new(tick.bid_price, tick.bid_qty),
            ask: InlineOrder::new(tick.ask_price, tick.ask_qty),
        }
    }
}
