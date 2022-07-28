use serde::Deserialize;

/// Symbol represents some trading paper in BNC system.
pub type Symbol = String;

/// UpdateID is supplied in most of the required binance API parts, so it's better to include it here.
pub type UpdateId = u64;

/// It's fairly funny that amounts(e.g. in prices) in the states of binance API are presented as Strings.
/// So we provide kind of encapsulation here just to feel a little safer.
#[derive(Deserialize, Clone, Debug)]
pub struct Amount(String);

impl Amount {
    /// transforms amount to big float type. Should be enough for most cases.
    pub fn to_f64(&self) -> f64 {
        self.0.parse().expect(&format!(
            "Bnc provided incorrect price. Failed attempt at: {}",
            self.0
        ))
    }
}

/// Binance order representation - holds price and amount.
///
/// Again, due to strange binance implementation we are to use tuple syntax here
/// as they've provided arrays instead of json in some places.
#[derive(Deserialize, Clone, Debug)]
pub struct InlineOrder(Amount, Amount);

impl InlineOrder {
    pub fn price(&self) -> &Amount {
        &self.0
    }

    pub fn amount(&self) -> &Amount {
        &self.1
    }
}
