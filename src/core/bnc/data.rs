use serde::{de, Deserialize, Serialize};

/// UpdateID is supplied in most of the required binance API parts, so it's better to include it here.
pub type UpdateId = u64;

fn deserialize_amount<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    s.parse()
        .map_err(|_| de::Error::custom("Invalid amount format"))
}

/// It's fairly funny that amounts(e.g. in prices) in the states of binance API are presented as Strings.
/// So we provide kind of encapsulation here just to feel a little safer.
#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
pub struct Amount(#[serde(deserialize_with = "deserialize_amount")] f64);

/// Binance order representation - holds price and amount.
///
/// Again, due to strange binance implementation we are to use tuple syntax here
/// as they've provided arrays instead of json in some places.
#[derive(Deserialize, Clone, Debug)]
pub struct InlineOrder(Amount, Amount);

impl InlineOrder {
    pub fn new(price: Amount, amount: Amount) -> Self {
        Self(price, amount)
    }

    pub fn price(&self) -> &Amount {
        &self.0
    }

    pub fn amount(&self) -> &Amount {
        &self.1
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SymbolContainer<'a> {
    pub symbol: &'a str,
}
