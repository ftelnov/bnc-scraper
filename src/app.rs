use crate::config::AppCfg;
use crate::core::bnc::error::BncResult;
use crate::core::bnc::rest::BncRestClient;
use crate::core::bnc::snapshot::SnapshotFetcher;
use crate::core::bnc::state::{BncState, ControlledReceiver};
use crate::core::bnc::ws::worker::depth::SymbolDepthUpdate;
use crate::core::bnc::ws::worker::price::SymbolPriceUpdate;
use crate::ui::AppUI;
use tokio::sync::mpsc::Receiver;
use tui::backend::Backend;
use tui::Frame;

/// General application that controls both ui and data scraping.
pub struct App {
    depth_updates_receiver: Option<ControlledReceiver<SymbolDepthUpdate>>,
    price_updates_receiver: Option<ControlledReceiver<SymbolPriceUpdate>>,

    current_depth: Option<SymbolDepthUpdate>,
    current_price: Option<SymbolPriceUpdate>,

    cfg: AppCfg,
    symbol: String,

    should_quit: bool,
}

impl App {
    pub fn new(cfg: AppCfg, symbol: String) -> Self {
        Self {
            cfg,
            depth_updates_receiver: None,
            price_updates_receiver: None,
            current_depth: None,
            current_price: None,
            symbol,
            should_quit: false,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Initialise BNC app - it will fetch the snapshot, then schedules workers to infinitely update the current state.
    pub async fn init(&mut self) -> BncResult<()> {
        let rest_client = BncRestClient::from_cfg(&self.cfg.core.bnc);
        let snapshot = rest_client.fetch_snapshot(&self.symbol).await?;

        self.current_price = Some(SymbolPriceUpdate {
            id: 0,
            bid: snapshot.bids.last().unwrap().clone(),
            ask: snapshot.asks.last().unwrap().clone(),
        });

        self.current_depth = Some(snapshot.into());

        let state = BncState::from_cfg(&self.cfg.core.bnc);

        self.depth_updates_receiver = Some(state.symbol_depth_receiver(&self.symbol));
        self.price_updates_receiver = Some(state.symbol_price_receiver(&self.symbol));

        Ok(())
    }

    /// Actions that are to be performed on each tick of an application.
    ///
    /// For example, here should latest backend updates fetching goes.
    pub fn on_tick(&mut self) -> BncResult<()> {
        Ok(())
    }

    /// Draw current state of the application on the provided frame.
    pub fn draw<B: Backend>(&self, frame: &mut Frame<B>) {
        AppUI::new(self.current_price.as_ref(), self.current_depth.as_ref()).draw(frame)
    }

    /// Finalize application - abort tasks, clear the state. In other words, graceful shutdown.
    pub fn finalize(&mut self) -> BncResult<()> {
        if let Some(receiver) = &self.depth_updates_receiver {
            receiver.finalize();
        }
        if let Some(receiver) = &self.price_updates_receiver {
            receiver.finalize()
        }
        self.should_quit = true;
        Ok(())
    }
}
