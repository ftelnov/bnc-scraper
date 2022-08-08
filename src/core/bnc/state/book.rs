use crate::core::bnc::config::BncCfg;
use crate::core::bnc::data::{InlineOrder, PriceLevel, Qty};
use crate::core::bnc::error::BncError::DataTransmitError;
use crate::core::bnc::error::{BncError, BncResult};
use crate::core::bnc::rest::BncRestClient;
use crate::core::bnc::snapshot::{SnapshotFetcher, SymbolSnapshot};
use crate::core::bnc::ws::worker::depth::{SymbolDepthUpdate, SymbolDepthWatcher};
use crate::core::bnc::ws::worker::{MessageSender, WsWorker};
use log::debug;
use reqwest::Client;
use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::watch::{channel, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Mode of current Order Book.
///
/// Snapshot is for just initialised order book.
///
/// Update is for order book that was updated with incremental changes.
#[derive(Debug)]
pub enum OrderBookMode {
    Snapshot {
        last_update_id: u64,
    },
    Update {
        first_update_id: u64,
        final_update_id: u64,
    },
}

pub type TableDisplay = Vec<(PriceLevel, Qty)>;
pub type OrderBookReceiver = Receiver<OrderBookDisplay>;
pub type OrderBookSender = Sender<OrderBookDisplay>;

/// Structure that provides easy access to price levels.
struct OrderTable(BTreeMap<String, Qty>);

impl OrderTable {
    pub fn from_orders(data: Vec<InlineOrder>) -> Self {
        Self(data.into_iter().map(|order| (order.0, order.1)).collect())
    }

    /// Update this table so level will satisfy provided order.
    pub fn update_level(&mut self, order: InlineOrder) {
        let qty_not_empty = order.1.chars().any(|char| char != '0' && char != '.');
        if !qty_not_empty {
            self.0.remove(&order.0);
            return;
        }
        let entry = self.0.entry(order.0);
        match entry {
            Entry::Vacant(vc) => {
                vc.insert(order.1);
            }
            Entry::Occupied(mut oc) => {
                *oc.get_mut() = order.1;
            }
        }
    }

    /// Get owned version of table's top, copying levels' identities.
    ///
    /// It is limited to top N as it is bad for the performance.
    pub fn owned_top(&self) -> TableDisplay {
        self.0
            .keys()
            .take(10)
            .rev()
            .map(|key| (key.clone(), self.0.get(key).unwrap().clone()))
            .collect()
    }
}

/// Holds current mode of order book and its tables.
pub struct OrderBook {
    mode: OrderBookMode,
    bids: OrderTable,
    asks: OrderTable,
}

#[derive(Clone)]
pub struct OrderBookDisplay {
    pub bids: TableDisplay,
    pub asks: TableDisplay,
}

impl From<SymbolSnapshot> for OrderBook {
    fn from(snapshot: SymbolSnapshot) -> Self {
        Self {
            mode: OrderBookMode::Snapshot {
                last_update_id: snapshot.last_update_id,
            },
            bids: OrderTable::from_orders(snapshot.bids),
            asks: OrderTable::from_orders(snapshot.asks),
        }
    }
}

impl From<SymbolDepthUpdate> for OrderBook {
    fn from(update: SymbolDepthUpdate) -> Self {
        Self {
            mode: OrderBookMode::Update {
                first_update_id: update.first_update_id,
                final_update_id: update.final_update_id,
            },
            bids: OrderTable::from_orders(update.bids),
            asks: OrderTable::from_orders(update.asks),
        }
    }
}

impl OrderBook {
    fn process_depth_update(&mut self, update: SymbolDepthUpdate) {
        self.mode = OrderBookMode::Update {
            first_update_id: update.first_update_id,
            final_update_id: update.final_update_id,
        };
        for order in update.bids {
            self.bids.update_level(order);
        }
        for order in update.asks {
            self.asks.update_level(order);
        }
    }

    fn is_update_satisfying(&self, update: &SymbolDepthUpdate) -> bool {
        match self.mode {
            OrderBookMode::Snapshot { last_update_id } => {
                // There should be also compare with the initial value, but it's omitted due to task preferences.
                // More info about REAL order book management is here:
                // https://binance-docs.github.io/apidocs/spot/en/#how-to-manage-a-local-order-book-correctly
                // Basically it means that snapshot should go AFTER you started ur ws workers.
                // So here is mostly incorrect logic.
                if update.final_update_id > last_update_id {
                    return true;
                }
                debug!(
                    "Depth update would not be merged into current order book's snapshot.\
                    Snapshot last_update_id: {last_update_id};\
                    Update: first_update_id = {}, final_update_id = {}\
                ",
                    update.first_update_id, update.final_update_id
                )
            }
            OrderBookMode::Update {
                final_update_id, ..
            } => {
                if update.first_update_id - 1 == final_update_id {
                    return true;
                }
                debug!(
                    "Depth update would not be merged into current book incrementing state.\
                    Current book mode: {:?}; \
                    Current update first_id: {}; Current update final_id: {}\
                ",
                    self.mode, update.first_update_id, update.final_update_id
                )
            }
        }
        false
    }

    /// To be called when you want to sum received depth update with current book state.
    ///
    /// Returns true if update was accepted, false otherwise.
    pub fn add_depth_update(&mut self, update: SymbolDepthUpdate) -> bool {
        let is_satisfying = self.is_update_satisfying(&update);
        if is_satisfying {
            self.process_depth_update(update)
        }
        is_satisfying
    }

    pub fn top(&self) -> OrderBookDisplay {
        OrderBookDisplay {
            asks: self.asks.owned_top(),
            bids: self.bids.owned_top(),
        }
    }
}

/// Balances updates that are passed to order book.
struct OrderBookBalancer {
    sender: OrderBookSender,
    book: OrderBook,
}

#[async_trait::async_trait]
impl MessageSender<SymbolDepthUpdate> for Arc<Mutex<OrderBookBalancer>> {
    async fn send(&self, data: SymbolDepthUpdate) -> BncResult<()> {
        let mut lock = self.lock().await;

        let is_updated = lock.book.add_depth_update(data);
        if !is_updated {
            return Err(BncError::DataRejected);
        }

        lock.sender
            .send(lock.book.top())
            .map_err(|_| DataTransmitError)?;

        Ok(())
    }
}

/// Settings for order book manager.
///
/// Just an encapsulation over ordinary app's configuration.
struct ManagerCfg<'a> {
    workers: u64,
    ws_conn_url: &'a str,
    rest_conn_url: &'a str,
}

impl<'a> ManagerCfg<'a> {
    fn from_cfg(cfg: &'a BncCfg) -> Self {
        Self {
            workers: cfg.ws.workers,
            ws_conn_url: &cfg.ws.baseurl,
            rest_conn_url: &cfg.baseurl,
        }
    }
}

/// Schedules workers to update order book in realtime, provide notifications of its updates.
pub struct OrderBookManager<'a> {
    cfg: ManagerCfg<'a>,
    tasks: Vec<JoinHandle<BncResult<()>>>,
}

impl<'a> OrderBookManager<'a> {
    /// Schedule workers, get receiver of current book's top.
    pub async fn init(&mut self, symbol: &str) -> BncResult<OrderBookReceiver> {
        let client = BncRestClient::new(Client::new(), self.cfg.rest_conn_url.to_string());
        let snapshot = client.fetch_snapshot(symbol).await?;
        let book = OrderBook::from(snapshot);

        let (sender, receiver) = channel(book.top());

        let balancer = Arc::new(Mutex::new(OrderBookBalancer { sender, book }));

        let worker = WsWorker::new(self.cfg.ws_conn_url);
        let mut tasks = vec![];

        for i in 0..self.cfg.workers {
            debug!("Initialised #{} worker of symbol depth receiver.", i);
            tasks.push(worker.depth_updates_watcher(symbol, balancer.clone()));
        }

        self.tasks = tasks;

        Ok(receiver)
    }

    /// Terminate scheduled tasks.
    pub fn stop(&self) {
        self.tasks.iter().for_each(|task| task.abort());
    }

    pub fn from_cfg(cfg: &'a BncCfg) -> Self {
        Self {
            cfg: ManagerCfg::from_cfg(cfg),
            tasks: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppCfg;
    use crate::core::logging::tests::setup_test_logger;
    use anyhow::Result;
    use std::ops::Deref;

    #[tokio::test]
    async fn it_watches_for_book_updates() -> Result<()> {
        let cfg = AppCfg::load()?;
        setup_test_logger();
        let mut state = OrderBookManager::from_cfg(&cfg.core.bnc);
        let symbol = "BTCUSDT";

        // Amount of validation steps before break;
        let break_at = 3;

        let mut receiver = state.init(symbol).await?;

        let mut latest = {
            receiver.changed().await.unwrap();
            receiver.borrow_and_update().deref().clone()
        };

        for _ in 0..break_at {
            let current = {
                receiver.changed().await.unwrap();
                receiver.borrow_and_update().deref().clone()
            };
            assert!(current.bids != latest.bids || current.asks != latest.asks);
            latest = current;
        }

        state.stop();

        Ok(())
    }
}
