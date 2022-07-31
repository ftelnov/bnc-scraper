use super::config::WsCfg;
use super::data::SymbolStateUpdate;
use crate::core::bnc::ws::worker::WsWorker;
use futures::future::ready;
use futures::Stream;
use futures_util::StreamExt;
use log::info;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

/// State that stores information used to balance across multiple ws listeners.
///
/// It will then be passed into scan method of the stream, so future updates will be filtered in a right way.
#[derive(Debug)]
struct BalancingState {
    last_update_id: Option<u64>,
}

impl Default for BalancingState {
    fn default() -> Self {
        Self {
            last_update_id: None,
        }
    }
}

/// A bridge between updates consumer and workers, makes consumer receive a really latest update, not the mess workers provide.
#[derive(Debug)]
pub struct WsMaster {
    workers_amount: u64,
    base_url: String,
}

impl WsMaster {
    pub fn new(base_url: String, workers_amount: u64) -> Self {
        Self {
            workers_amount,
            base_url,
        }
    }

    pub fn from_cfg(cfg: &WsCfg) -> Self {
        Self::new(cfg.baseurl.clone(), cfg.workers)
    }

    pub fn run(self, symbol: &str) -> impl Stream<Item = SymbolStateUpdate> {
        let (sender, receiver) = mpsc::channel(100);
        for i in 0..self.workers_amount {
            info!("Worker #{i} scheduled.");
            let worker = WsWorker::new(&self.base_url);
            worker.watch_price_updates(symbol, sender.clone());
        }
        let stream = ReceiverStream::new(receiver);

        // Filters the stream with balancing state;
        // On every iteration check whether current update_id is more then saved one.
        let mut balancer = BalancingState::default();
        stream.filter(move |event| {
            if let Some(ref mut last_update_id) = balancer.last_update_id {
                if event.update_id > *last_update_id {
                    *last_update_id = event.update_id;
                } else {
                    return ready(false);
                }
            } else {
                balancer.last_update_id = Some(event.update_id);
            }
            ready(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn it_ensures_that_updates_are_continuous() -> Result<()> {
        let master = WsMaster::from_cfg(&WsCfg {
            workers: 3,
            ..Default::default()
        });
        let symbol = "BTCUSDT";

        // Amount of validation steps before break;
        let break_at = 5;

        let mut stream = master.run(symbol);
        let mut latest = stream.next().await.unwrap();

        for i in 0..break_at {
            let current = stream.next().await.unwrap();
            assert!(latest.update_id < current.update_id);
            latest = current;
        }

        Ok(())
    }
}
