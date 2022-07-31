use super::super::error::BncResult;
use super::config::WsCfg;
use super::data::SymbolBookTick;
use crate::core::bnc::ws::data::WsDataContainer;
use futures::Stream;
use futures_util::stream::{FilterMap, Map};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use log::{debug, warn};
use std::pin::Pin;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Error, Message};

pub struct WsWorker {
    base_url: String,
}

impl WsWorker {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }
    pub fn from_cfg(cfg: &WsCfg) -> Self {
        Self::new(cfg.baseurl().clone())
    }

    /// Connect to the BNC endpoint using given symbol, subscribe on the updates of this symbol's price.
    pub async fn symbol_price_ticks(
        &self,
        symbol: &str,
    ) -> BncResult<Pin<Box<impl Stream<Item = BncResult<SymbolBookTick>>>>> {
        let conn_url = format!(
            "{base_url}/stream?streams={symbol}@bookTicker",
            base_url = self.base_url,
            symbol = symbol.to_ascii_lowercase()
        );
        let (ws_stream, _) = connect_async(conn_url).await?;
        let stream = ws_stream
            .filter_map(|message| async {
                let message = message.ok()?;
                if message.is_text() {
                    Some(message)
                } else {
                    None
                }
            })
            .map(|message| {
                debug!(
                    "Received symbol price update event. Message: {:?}.",
                    message
                );
                let update: WsDataContainer<SymbolBookTick> =
                    serde_json::from_slice(&message.into_data())?;
                Ok(update.data)
            });
        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppCfg;
    use crate::core::logging::{setup_logger, LogCfg};
    use anyhow::Result;
    use log::info;
    use log::Level::Debug;

    #[tokio::test]
    async fn it_watches_for_first_symbol_update() -> Result<()> {
        let cfg = AppCfg::load()?;
        setup_logger(&LogCfg {
            enabled: true,
            level: Debug,
        })?;
        let symbol = "BTCUSDT";

        let worker = WsWorker::from_cfg(&cfg.core.bnc.ws);
        let mut events = worker.symbol_price_ticks(symbol).await?;
        let event = events.next().await.unwrap()?;

        info!("Successfully received event: {:?}", event);

        Ok(())
    }
}
