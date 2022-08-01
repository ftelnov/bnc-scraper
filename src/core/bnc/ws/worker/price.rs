use super::super::data::WsDataContainer;
use super::WsWorker;
use crate::core::bnc::data::{Amount, InlineOrder};
use crate::core::bnc::error::{BncError, BncResult};
use crate::core::bnc::ws::worker::bnc_stream_connect;
use futures::Stream;
use futures_util::StreamExt;
use log::{debug, warn};
use serde::Deserialize;
use std::pin::Pin;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;

/// Tick for an individual symbol's book update. Generally current best price for the provided symbol.
#[derive(Debug, Deserialize, Clone)]
struct SymbolBookTick {
    #[serde(rename = "u")]
    update_id: u64,

    #[serde(rename = "b")]
    bid_price: Amount,

    #[serde(rename = "B")]
    bid_qty: Amount,

    #[serde(rename = "a")]
    ask_price: Amount,

    #[serde(rename = "A")]
    ask_qty: Amount,
}

/// Generalisation of price update.
///
/// All tickers' updates should be convertable to general representation.
#[derive(Debug, Clone)]
pub struct SymbolPriceUpdate {
    pub update_id: u64,
    pub bid: InlineOrder,
    pub ask: InlineOrder,
}

impl From<SymbolBookTick> for SymbolPriceUpdate {
    fn from(tick: SymbolBookTick) -> Self {
        Self {
            update_id: tick.update_id,
            bid: InlineOrder::new(tick.bid_price, tick.bid_qty),
            ask: InlineOrder::new(tick.ask_price, tick.ask_qty),
        }
    }
}

fn book_ticker_endpoint(base_endpoint: &str, symbol: &str) -> String {
    format!(
        "{base_url}/stream?streams={symbol}@bookTicker",
        base_url = base_endpoint,
        symbol = symbol.to_ascii_lowercase()
    )
}

/// Connect to the BNC book tick endpoint.
async fn symbol_book_ticks(
    endpoint: &str,
) -> BncResult<Pin<Box<impl Stream<Item = BncResult<SymbolBookTick>>>>> {
    let stream = bnc_stream_connect(endpoint).await?;
    let stream = stream.map(|message| {
        debug!(
            "Received symbol price update event. Message: {:?}.",
            message
        );
        let update: WsDataContainer<SymbolBookTick> = serde_json::from_slice(&message.into_data())?;
        Ok(update.data)
    });
    Ok(Box::pin(stream))
}

pub trait SymbolPriceWatcher {
    /// Listen for price realtime updates, send them via provided sender.
    ///
    /// Returns JoinHandle of the spawned task in order to store somewhere else.
    fn price_updates_watcher(
        &self,
        symbol: &str,
        sender: Sender<SymbolPriceUpdate>,
    ) -> JoinHandle<BncResult<()>>;
}

impl<'a> SymbolPriceWatcher for WsWorker<'a> {
    fn price_updates_watcher(
        &self,
        symbol: &str,
        sender: Sender<SymbolPriceUpdate>,
    ) -> JoinHandle<BncResult<()>> {
        let book_ticker_endpoint = book_ticker_endpoint(self.base_url, symbol);
        tokio::task::spawn(async move {
            let mut stream = symbol_book_ticks(&book_ticker_endpoint).await?;
            while let Some(event) = stream.next().await {
                match event {
                    Ok(update) => {
                        debug!("Worker received symbol book tick. Tick: {:?}", update);
                        sender
                            .send(update.into())
                            .await
                            .map_err(|_| BncError::DataTransmitError)?;
                    }
                    Err(err) => {
                        warn!(
                            "Error occurred during worker processing the message. Err: {}",
                            err
                        );
                    }
                }
            }
            BncResult::Ok(())
        })
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
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn it_watches_for_first_symbol_update_using_tick_book() -> Result<()> {
        let cfg = AppCfg::load()?;
        setup_logger(&LogCfg {
            enabled: true,
            level: Debug,
        })
        .ok();
        let symbol = "BTCUSDT";

        let worker = WsWorker::from_cfg(&cfg.core.bnc.ws);
        let mut events = symbol_book_ticks(&book_ticker_endpoint(worker.base_url, symbol)).await?;
        let event = events.next().await.unwrap()?;

        info!("Successfully received event: {:?}", event);

        Ok(())
    }

    #[tokio::test]
    async fn it_watches_for_first_symbol_update_using_pub_method() -> Result<()> {
        let cfg = AppCfg::load()?;

        setup_logger(&LogCfg {
            enabled: true,
            level: Debug,
        })
        .ok();

        let symbol = "BTCUSDT";

        let worker = WsWorker::from_cfg(&cfg.core.bnc.ws);

        let (sender, mut receiver) = mpsc::channel(10);

        let handle = worker.price_updates_watcher(symbol, sender);

        let update = receiver.recv().await.unwrap();

        info!("Successfully received update: {:?}. Aborting task.", update);

        handle.abort();

        Ok(())
    }
}
