use super::WsWorker;
use crate::core::bnc::data::InlineOrder;
use crate::core::bnc::error::{BncError, BncResult};
use crate::core::bnc::ws::data::WsDataContainer;
use crate::core::bnc::ws::worker::bnc_stream_connect;
use futures::Stream;
use futures_util::StreamExt;
use log::{debug, warn};
use serde::Deserialize;
use std::pin::Pin;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SymbolDepthUpdate {
    #[serde(rename = "lastUpdateId")]
    pub id: u64,

    pub bids: Vec<InlineOrder>,
    pub asks: Vec<InlineOrder>,
}

pub trait SymbolDepthWatcher {
    /// Listen for depth realtime updates, send them via provided sender.
    ///
    /// Returns JoinHandle of the spawned task in order to store somewhere else.
    fn depth_updates_watcher(
        &self,
        symbol: &str,
        sender: Sender<SymbolDepthUpdate>,
    ) -> JoinHandle<BncResult<()>>;
}

fn depth_updates_endpoint(base_endpoint: &str, symbol: &str) -> String {
    format!(
        "{base_url}/stream?streams={symbol}@depth5",
        base_url = base_endpoint,
        symbol = symbol.to_ascii_lowercase()
    )
}

/// Connect to the BNC depth tick endpoint.
async fn symbol_depth_ticks(
    endpoint: &str,
) -> BncResult<Pin<Box<impl Stream<Item = BncResult<SymbolDepthUpdate>>>>> {
    let stream = bnc_stream_connect(endpoint).await?;
    let stream = stream.map(|message| {
        debug!(
            "Received symbol price update event. Message: {:?}.",
            message
        );
        let update: WsDataContainer<SymbolDepthUpdate> =
            serde_json::from_slice(&message.into_data())?;
        Ok(update.data)
    });
    Ok(Box::pin(stream))
}

impl<'a> SymbolDepthWatcher for WsWorker<'a> {
    fn depth_updates_watcher(
        &self,
        symbol: &str,
        sender: Sender<SymbolDepthUpdate>,
    ) -> JoinHandle<BncResult<()>> {
        let depth_endpoint = depth_updates_endpoint(self.base_url, symbol);
        tokio::task::spawn(async move {
            let mut stream = symbol_depth_ticks(&depth_endpoint).await?;
            while let Some(event) = stream.next().await {
                match event {
                    Ok(update) => {
                        debug!("Worker received update tick. Tick: {:?}", update);
                        sender
                            .send(update)
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

    struct TestCtx {
        cfg: AppCfg,
    }

    impl TestCtx {
        fn new() -> Self {
            let cfg = AppCfg::load().unwrap();
            setup_logger(&LogCfg {
                enabled: true,
                level: Debug,
            })
            .ok();
            Self { cfg }
        }
    }

    #[tokio::test]
    async fn it_watches_for_first_depth_update() -> Result<()> {
        let ctx = TestCtx::new();
        let symbol = "BTCUSDT";

        let worker = WsWorker::from_cfg(&ctx.cfg.core.bnc.ws);
        let mut events =
            symbol_depth_ticks(&depth_updates_endpoint(worker.base_url, symbol)).await?;
        let event = events.next().await.unwrap()?;

        info!("Successfully received event: {:?}", event);

        Ok(())
    }

    #[tokio::test]
    async fn it_watches_for_first_depth_update_using_pub_method() -> Result<()> {
        let ctx = TestCtx::new();
        let symbol = "BTCUSDT";

        let worker = WsWorker::from_cfg(&ctx.cfg.core.bnc.ws);
        let (sender, mut receiver) = mpsc::channel(10);
        let handle = worker.depth_updates_watcher(symbol, sender);

        let update = receiver.recv().await.unwrap();

        info!("Successfully received update: {:?}. Aborting task.", update);

        handle.abort();

        Ok(())
    }

    #[tokio::test]
    async fn it_ensures_that_updates_are_continious() -> Result<()> {
        let ctx = TestCtx::new();
        let symbol = "BTCUSDT";
        let breaks_at = 5;

        let worker = WsWorker::from_cfg(&ctx.cfg.core.bnc.ws);
        let (sender, mut receiver) = mpsc::channel(10);
        let handle = worker.depth_updates_watcher(symbol, sender);
        let mut latest = receiver.recv().await.unwrap();

        for _ in 0..breaks_at {
            let update = receiver.recv().await.unwrap();
            assert!(latest.id < update.id);
            latest = update;
        }

        info!("Latest received update: {:?}. Aborting task.", latest);

        handle.abort();

        Ok(())
    }
}
