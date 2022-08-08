use super::super::data::WsDataContainer;
use super::WsWorker;
use crate::core::bnc::data::{InlineOrder, PriceLevel, Qty};
use crate::core::bnc::error::{BncError, BncResult};
use crate::core::bnc::snapshot::SymbolSnapshot;
use crate::core::bnc::ws::worker::{bnc_stream_connect, MessageSender};
use futures::Stream;
use futures_util::StreamExt;
use log::{debug, error, warn};
use serde::Deserialize;
use std::pin::Pin;
use tokio::task::JoinHandle;

/// Tick for an individual symbol's book update. Generally current best price for the provided symbol.
#[derive(Debug, Deserialize, Clone)]
struct SymbolBookTick {
    #[serde(rename = "u")]
    id: u64,

    #[serde(rename = "b")]
    bid_price: PriceLevel,

    #[serde(rename = "B")]
    bid_qty: Qty,

    #[serde(rename = "a")]
    ask_price: PriceLevel,

    #[serde(rename = "A")]
    ask_qty: Qty,
}

/// Generalisation of price update.
///
/// All tickers' updates should be convertable to general representation.
#[derive(Debug, Default, Clone)]
pub struct SymbolPriceUpdate {
    pub id: u64,
    pub bid: InlineOrder,
    pub ask: InlineOrder,
}

impl From<SymbolBookTick> for SymbolPriceUpdate {
    fn from(tick: SymbolBookTick) -> Self {
        Self {
            id: tick.id,
            bid: InlineOrder::new(tick.bid_price, tick.bid_qty),
            ask: InlineOrder::new(tick.ask_price, tick.ask_qty),
        }
    }
}

impl From<SymbolSnapshot> for SymbolPriceUpdate {
    fn from(snapshot: SymbolSnapshot) -> Self {
        let empty_snapshot_notification =
            "Snapshot returned by binance is empty. Are we corrupted?";
        Self {
            bid: snapshot
                .bids
                .into_iter()
                .last()
                .expect(empty_snapshot_notification),
            ask: snapshot
                .asks
                .into_iter()
                .last()
                .expect(empty_snapshot_notification),
            id: snapshot.last_update_id,
        }
    }
}

pub trait SymbolPriceWatcher {
    /// Listen for price realtime updates, send them via provided sender.
    ///
    /// Returns JoinHandle of the spawned task in order to store somewhere else.
    fn price_updates_watcher(
        &self,
        symbol: &str,
        sender: impl MessageSender<SymbolPriceUpdate> + 'static,
    ) -> JoinHandle<BncResult<()>>;
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
        debug!("Received symbol price update event.");
        let update: WsDataContainer<SymbolBookTick> = serde_json::from_slice(&message.into_data())?;
        Ok(update.data)
    });
    Ok(Box::pin(stream))
}

impl<'a> SymbolPriceWatcher for WsWorker<'a> {
    fn price_updates_watcher(
        &self,
        symbol: &str,
        sender: impl MessageSender<SymbolPriceUpdate> + 'static,
    ) -> JoinHandle<BncResult<()>> {
        let book_ticker_endpoint = book_ticker_endpoint(self.base_url, symbol);
        let future = async move {
            let mut stream = symbol_book_ticks(&book_ticker_endpoint).await?;
            while let Some(event) = stream.next().await {
                match event {
                    Ok(update) => {
                        debug!("Worker received symbol book tick. Tick: {:?}", update);
                        let send_result = sender.send(update.into());
                        let send_result = send_result.await;
                        match send_result {
                            Err(err) => match err {
                                BncError::DataTransmitError => {
                                    warn!("Sender could not process data. Error: {}", err)
                                }
                                BncError::DataRejected => {
                                    debug!("Data was rejected due to some predicate.")
                                }
                                err => {
                                    error!(
                                        "Data was rejected with unexpected error. Error: {}",
                                        err
                                    )
                                }
                            },
                            _ => {
                                debug!("Worker successfully sent data to consumer.")
                            }
                        }
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
        };
        tokio::task::spawn(future)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppCfg;
    use crate::core::logging::{setup_logger, LogCfg};
    use anyhow::Result;
    use log::{info, LevelFilter};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn it_watches_for_first_symbol_update_using_tick_book() -> Result<()> {
        let cfg = AppCfg::load()?;
        setup_logger(&LogCfg {
            level: LevelFilter::Debug,
            ..Default::default()
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
            level: LevelFilter::Debug,
            ..Default::default()
        })
        .ok();

        let symbol = "BTCUSDT";

        let worker = WsWorker::from_cfg(&cfg.core.bnc.ws);
        let (sender, mut receiver) = mpsc::channel(5);
        let handle = worker.price_updates_watcher(symbol, sender);

        let update = receiver.recv().await.unwrap();

        info!("Successfully received update: {:?}. Aborting task.", update);

        handle.abort();

        Ok(())
    }
}
