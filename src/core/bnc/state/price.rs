use crate::core::bnc::error::BncResult;
use crate::core::bnc::state::balancer::MessageBalancer;
use crate::core::bnc::ws::config::WsCfg;

use crate::core::bnc::ws::worker::price::{SymbolPriceUpdate, SymbolPriceWatcher};
use crate::core::bnc::ws::worker::WsWorker;
use log::debug;
use std::sync::Arc;
use tokio::sync::watch::{channel, Receiver};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub type PriceReceiver = Receiver<SymbolPriceUpdate>;

struct PriceManagerCfg<'a> {
    ws_base_url: &'a str,
    workers: u64,
}

impl<'a> PriceManagerCfg<'a> {
    fn from_cfg(cfg: &'a WsCfg) -> Self {
        Self {
            ws_base_url: &cfg.baseurl,
            workers: cfg.workers,
        }
    }
}

pub struct PriceStateManager<'a> {
    cfg: PriceManagerCfg<'a>,
    tasks: Vec<JoinHandle<BncResult<()>>>,
}

impl<'a> PriceStateManager<'a> {
    pub fn from_cfg(cfg: &'a WsCfg) -> Self {
        Self {
            cfg: PriceManagerCfg::from_cfg(cfg),
            tasks: vec![],
        }
    }

    pub fn init(&mut self, symbol: &str) -> PriceReceiver {
        let (sender, receiver) = channel(SymbolPriceUpdate::default());

        let balancer = Arc::new(Mutex::new(MessageBalancer::new(sender)));

        let worker = WsWorker::new(self.cfg.ws_base_url);
        let mut tasks = vec![];

        for i in 0..self.cfg.workers {
            debug!("Initialised #{} worker of symbol price receiver.", i);
            tasks.push(worker.price_updates_watcher(symbol, balancer.clone()));
        }

        self.tasks = tasks;

        receiver
    }

    pub fn stop(&self) {
        self.tasks.iter().for_each(|task| task.abort());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppCfg;
    use crate::core::bnc::state::balancer::BalancedEntity;

    use crate::core::logging::tests::setup_test_logger;
    use anyhow::Result;
    use std::ops::Deref;

    #[tokio::test]
    async fn it_watches_for_price_updates() -> Result<()> {
        let cfg = AppCfg::load()?;
        setup_test_logger();
        let mut state = PriceStateManager::from_cfg(&cfg.core.bnc.ws);
        let symbol = "BTCUSDT";

        // Amount of validation steps before break;
        let break_at = 5;

        let mut receiver = state.init(symbol);

        let mut latest = {
            receiver.changed().await.unwrap();
            receiver.borrow().deref().clone()
        };

        for _ in 0..break_at {
            let current = {
                receiver.changed().await.unwrap();
                receiver.borrow().deref().clone()
            };
            assert!(current.update_id() > latest.update_id());
            latest = current;
        }

        state.stop();

        Ok(())
    }
}
