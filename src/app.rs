use crate::config::AppCfg;
use crate::core::bnc::error::BncError::DataTransmitError;
use crate::core::bnc::error::BncResult;
use crate::core::bnc::rest::BncRestClient;
use crate::core::bnc::snapshot::SnapshotFetcher;
use crate::core::bnc::state::book::{OrderBookManager, OrderBookReceiver};
use crate::core::bnc::state::price::{PriceReceiver, PriceStateManager};
use crate::core::bnc::ws::worker::depth::SymbolDepthUpdate;
use crate::core::bnc::ws::worker::price::SymbolPriceUpdate;
use crate::ui::{draw_background, draw_best_price, draw_order_book, get_global_layout};
use anyhow::Result;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tokio::spawn;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::Mutex;
use tokio::task::{spawn_local, JoinHandle};
use tui::backend::Backend;
use tui::{Frame, Terminal};

pub type SharedTerminal<B> = Arc<Mutex<Terminal<B>>>;

/// General application that controls both ui and data scraping.
pub struct App<'a> {
    symbol: String,

    should_quit: bool,

    price_manager: PriceStateManager<'a>,
    order_book_manager: OrderBookManager<'a>,

    price_state_watcher: Option<PriceReceiver>,
    book_state_watcher: Option<OrderBookReceiver>,
}

impl<'a> App<'a> {
    pub fn new(cfg: &'a AppCfg, symbol: String) -> Self {
        Self {
            price_manager: PriceStateManager::from_cfg(&cfg.core.bnc.ws),
            order_book_manager: OrderBookManager::from_cfg(&cfg.core.bnc),
            symbol,
            should_quit: false,
            price_state_watcher: None,
            book_state_watcher: None,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Initialise BNC app - it will fetch the snapshot, then schedules workers to infinitely update the current state.
    pub async fn init(&mut self) -> BncResult<()> {
        let price_state_receiver = self.price_manager.init(&self.symbol);
        let order_book_receiver = self.order_book_manager.init(&self.symbol).await?;
        self.price_state_watcher = Some(price_state_receiver);
        self.book_state_watcher = Some(order_book_receiver);

        Ok(())
    }

    /// Draw current state of the application on the provided frame.
    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        draw_background(frame);
        let layout = get_global_layout(frame);
        if let Some(order_book_rx) = self.book_state_watcher.as_mut() {
            draw_order_book(
                frame,
                layout.order_book,
                order_book_rx.borrow_and_update().deref(),
            );
        }

        if let Some(price_rx) = self.price_state_watcher.as_mut() {
            draw_best_price(
                frame,
                layout.best_prices,
                price_rx.borrow_and_update().deref(),
            );
        }
    }

    /// Finalize application - abort tasks, clear the state. In other words, graceful shutdown.
    pub fn finalize(&mut self) -> BncResult<()> {
        self.order_book_manager.stop();
        self.price_manager.stop();

        self.should_quit = true;
        Ok(())
    }
}
