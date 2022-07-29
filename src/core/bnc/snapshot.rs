use super::data::InlineOrder;
use super::data::UpdateId;
use super::error::BncResult;
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct SymbolSnapshot {
    pub last_update_id: UpdateId,
    pub bids: Vec<InlineOrder>,
    pub asks: Vec<InlineOrder>,
}

/// Implementer of this trait are capable of fetching latest state of some symbol(in other words - snapshot).
#[async_trait]
pub trait SnapshotFetcher {
    /// Fetch current snapshot of the symbol. Depth should be set to 1 here - we ain't gonna need any further.
    async fn fetch_snapshot(&self, symbol: &str) -> BncResult<SymbolSnapshot>;
}
