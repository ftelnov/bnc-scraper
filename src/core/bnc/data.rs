use serde::{de, Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// UpdateID is supplied in most of the required binance API parts, so it's better to include it here.
pub type UpdateId = u64;

pub type PriceLevel = String;
pub type Qty = String;

/// Binance order representation - holds price and amount.
///
/// Again, due to strange binance implementation we are to use tuple syntax here
/// as they've provided arrays instead of json in some places.
#[derive(Deserialize, Default, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct InlineOrder(pub PriceLevel, pub Qty);

impl InlineOrder {
    pub fn new(price_lvl: PriceLevel, qty: Qty) -> Self {
        Self(price_lvl, qty)
    }

    pub fn level(&self) -> &str {
        &self.0
    }

    pub fn qty(&self) -> &str {
        &self.1
    }
}

impl Display for InlineOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:<8}/{:<8}", self.level(), self.qty()))
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SymbolContainer<'a> {
    pub symbol: &'a str,
}
