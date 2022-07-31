use super::super::error::BncResult;
use super::config::WsCfg;
use super::data::SymbolBookTick;
use crate::core::bnc::error::BncError;
use crate::core::bnc::ws::data::{SymbolStateUpdate, WsDataContainer};
use futures::Stream;
use futures_util::stream::{FilterMap, Map};
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use log::{debug, warn};
use std::pin::Pin;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Error, Message};

/// WS worker handles realtime updates of the symbol's price.
///
/// It's purpose to schedule listening threads that will send the data to the provided sender.
///
/// It doesn't, however, provide load balancing across child processes - so worker's results may be repeated.
///
/// I'd better split WsWorker into different implementers, but that's something to consider later.
pub struct WsWorker<'a> {
    base_url: &'a str,
}

impl<'a> WsWorker<'a> {
    pub fn new(base_url: &'a str) -> Self {
        Self { base_url }
    }

    pub fn from_cfg(cfg: &'a WsCfg) -> Self {
        Self::new(cfg.baseurl())
    }

    /// Listen for price realtime updates, send them via provided sender.
    pub fn watch_price_updates(&self, symbol: &str, sender: Sender<SymbolStateUpdate>) {
        let book_ticker_endpoint = self.book_ticker_endpoint(symbol);
        tokio::task::spawn(async move {
            let mut stream = Self::symbol_book_ticks(&book_ticker_endpoint).await?;
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
        });
    }

    fn book_ticker_endpoint(&self, symbol: &str) -> String {
        format!(
            "{base_url}/stream?streams={symbol}@bookTicker",
            base_url = self.base_url,
            symbol = symbol.to_ascii_lowercase()
        )
    }

    /// Connect to the BNC book tick endpoint.
    async fn symbol_book_ticks(
        endpoint: &str,
    ) -> BncResult<Pin<Box<impl Stream<Item = BncResult<SymbolBookTick>>>>> {
        let (ws_stream, _) = connect_async(endpoint).await?;
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
        let mut events = WsWorker::symbol_book_ticks(&worker.book_ticker_endpoint(symbol)).await?;
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

        worker.watch_price_updates(symbol, sender);

        let update = receiver.recv().await.unwrap();

        info!("Successfully received update: {:?}", update);

        Ok(())
    }
}
