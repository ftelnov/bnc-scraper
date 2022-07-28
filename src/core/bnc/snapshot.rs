use super::data::UpdateId;
use super::data::{InlineOrder, Symbol};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct SymbolSnapshot {
    pub last_update_id: UpdateId,
    pub bids: Vec<InlineOrder>,
    pub asks: Vec<InlineOrder>,
}

/// Implementer of this trait are capable of fetching latest state of some symbol(in other words - snapshot).
#[async_trait]
pub trait SnapshotFetcher {
    // TODO: change return time to Result to provide non-panicing behaviour.
    async fn fetch_snapshot(&self, symbol: Symbol) -> SymbolSnapshot;
}
