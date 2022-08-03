use crate::core::bnc::ws::worker::depth::SymbolDepthUpdate;
use crate::core::bnc::ws::worker::price::SymbolPriceUpdate;
use tui::backend::Backend;
use tui::widgets::{Block, Borders};
use tui::Frame;

pub mod config;
pub mod runner;

#[derive(Debug)]
pub struct AppUI<'a> {
    price_update: Option<&'a SymbolPriceUpdate>,
    depth_update: Option<&'a SymbolDepthUpdate>,
}

impl<'a> AppUI<'a> {
    pub fn new(
        price_update: Option<&'a SymbolPriceUpdate>,
        depth_update: Option<&'a SymbolDepthUpdate>,
    ) -> Self {
        Self {
            price_update,
            depth_update,
        }
    }

    pub fn draw<B: Backend>(&self, frame: &mut Frame<B>) {
        let size = frame.size();
        let block = Block::default()
            .title("Binance Scrapper")
            .borders(Borders::ALL);
        frame.render_widget(block, size);
    }
}
