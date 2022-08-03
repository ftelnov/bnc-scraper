use super::data::InlineOrder;
use crate::core::bnc::config::BncCfg;
use crate::core::bnc::error::BncError::DataTransmitError;
use crate::core::bnc::error::{BncError, BncResult};
use crate::core::bnc::ws::worker::depth::{SymbolDepthUpdate, SymbolDepthWatcher};
use crate::core::bnc::ws::worker::price::{SymbolPriceUpdate, SymbolPriceWatcher};
use crate::core::bnc::ws::worker::{MessageSender, WsWorker};
use futures::Stream;
use futures_util::future::ready;
use log::debug;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Stream of the implementors can be balanced using specific function.
pub trait Balanced {
    /// Get current update id of the entity to balance it across others.
    fn update_id(&self) -> u64;
}

impl Balanced for SymbolPriceUpdate {
    fn update_id(&self) -> u64 {
        self.id
    }
}

impl Balanced for SymbolDepthUpdate {
    fn update_id(&self) -> u64 {
        self.id
    }
}

/// State to hold balance data. It will be moved into needed clojure to compare its data with new entries.
///
/// MessageSender is implemented for the shared state of message balancer.
#[derive(Debug)]
struct MessageBalancer<T> {
    last_update_id: Option<u64>,
    sender: Sender<T>,
}

impl<T> MessageBalancer<T> {
    fn new(sender: Sender<T>) -> Self {
        Self {
            last_update_id: None,
            sender,
        }
    }
}

/// We implement sending messages that could be balanced(e.g. implements Balanced trait) for shared MessageBalancer state.
#[async_trait::async_trait]
impl<B: Balanced + Send> MessageSender<B> for Arc<Mutex<MessageBalancer<B>>> {
    async fn send(&self, data: B) -> BncResult<()> {
        debug!("Send is called now");
        let mut balancer = self.lock().await;
        if let Some(ref mut last_update_id) = balancer.last_update_id {
            if data.update_id() > *last_update_id {
                *last_update_id = data.update_id();
            } else {
                return Err(BncError::DataRejected);
            }
        } else {
            balancer.last_update_id = Some(data.update_id());
        }

        balancer
            .sender
            .send(data)
            .await
            .map_err(|_| DataTransmitError)?;

        Ok(())
    }
}

/// Return type of state watchers.
///
/// It holds receiver and vector of tasks' handlers that were scheduled for it.
///
/// It provides finalize method that will abort all the given tasks.
pub struct ControlledReceiver<T> {
    receiver: Receiver<T>,
    tasks: Vec<JoinHandle<BncResult<()>>>,
}

impl<T> ControlledReceiver<T> {
    /// Finalize receiver, abort it's child tasks.
    pub fn finalize(&self) {
        self.tasks.iter().for_each(|task| task.abort())
    }
}

pub struct BncState<'a> {
    ws_base_url: &'a str,
    channel_capacity: usize,
    workers: u64,
}

impl<'a> BncState<'a> {
    pub fn from_cfg(cfg: &'a BncCfg) -> Self {
        Self {
            ws_base_url: &cfg.ws.baseurl,
            workers: cfg.ws.workers,
            channel_capacity: cfg.chnlcapacity,
        }
    }

    /// Schedules N amount of workers to provide data for the new receiver.
    ///
    /// Returns receiver of symbol price updates.
    pub fn symbol_price_receiver(&self, symbol: &str) -> ControlledReceiver<SymbolPriceUpdate> {
        let (sender, receiver) = channel(self.channel_capacity);
        let balancer = Arc::new(Mutex::new(MessageBalancer::new(sender)));
        let worker = WsWorker::new(self.ws_base_url);

        let mut tasks = vec![];

        for i in 0..self.workers {
            debug!("Initialised #{} worker of symbol price receiver.", i);
            tasks.push(worker.price_updates_watcher(symbol, balancer.clone()));
        }

        ControlledReceiver { receiver, tasks }
    }

    /// Schedules N amount of workers to provide data for the new receiver.
    ///
    /// Returns receiver of depth updates.
    pub fn symbol_depth_receiver(&self, symbol: &str) -> ControlledReceiver<SymbolDepthUpdate> {
        let (sender, receiver) = channel(self.channel_capacity);
        let balancer = Arc::new(Mutex::new(MessageBalancer::new(sender)));
        let worker = WsWorker::new(self.ws_base_url);
        let mut tasks = vec![];

        for i in 0..self.workers {
            debug!("Initialised #{} worker of symbol price receiver.", i);
            tasks.push(worker.depth_updates_watcher(symbol, balancer.clone()));
        }

        ControlledReceiver { receiver, tasks }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bnc::ws::config::WsCfg;
    use crate::core::logging::{setup_logger, LogCfg};
    use anyhow::Result;
    use log::Level;
    use tokio_stream::wrappers::ReceiverStream;

    struct TestCtx {
        cfg: BncCfg,
    }

    impl TestCtx {
        fn new() -> Self {
            setup_logger(&LogCfg {
                level: Level::Debug,
                ..Default::default()
            })
            .ok();
            Self {
                cfg: BncCfg {
                    ws: WsCfg {
                        workers: 3,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }
        }
    }

    #[tokio::test]
    async fn it_ensures_that_price_updates_are_continuous() -> Result<()> {
        let ctx = TestCtx::new();
        let state = BncState::from_cfg(&ctx.cfg);
        let symbol = "BTCUSDT";

        // Amount of validation steps before break;
        let break_at = 5;

        let mut receiver = state.symbol_price_receiver(symbol).receiver;

        let mut latest = receiver.recv().await.unwrap();

        for _ in 0..break_at {
            let current = receiver.recv().await.unwrap();
            assert!(latest.id < current.id);
            latest = current;
        }

        Ok(())
    }

    #[tokio::test]
    async fn it_ensures_that_depth_updates_are_continuous() -> Result<()> {
        let ctx = TestCtx::new();
        let state = BncState::from_cfg(&ctx.cfg);
        let symbol = "BTCUSDT";

        // Amount of validation steps before break;
        let break_at = 5;

        let mut receiver = state.symbol_depth_receiver(symbol).receiver;

        let mut latest = receiver.recv().await.unwrap();

        for _ in 0..break_at {
            let current = receiver.recv().await.unwrap();
            assert!(latest.id < current.id);
            latest = current;
        }

        Ok(())
    }
}
