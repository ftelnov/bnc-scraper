use crate::core::bnc::data::InlineOrder;
use crate::core::bnc::ws::worker::depth::SymbolDepthUpdate;
use crate::core::bnc::ws::worker::price::SymbolPriceUpdate;
use tui::backend::Backend;
use tui::layout::Direction::Vertical;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem, Row, Table};
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

    fn orders_to_listitems(orders: Option<&[InlineOrder]>) -> Vec<ListItem> {
        if let Some(orders) = orders {
            orders
                .iter()
                .rev()
                .take(10)
                .map(|order| ListItem::new(order.to_string()))
                .collect()
        } else {
            vec![]
        }
    }

    fn draw_order_book<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let block = Block::default().title("Order book").borders(Borders::ALL);
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .direction(Direction::Horizontal)
            .margin(1)
            .split(area);

        let asks = List::new(Self::orders_to_listitems(
            self.depth_update.map(|update| update.asks.as_slice()),
        ))
        .block(Block::default().borders(Borders::ALL).title("Asks"));

        let bids = List::new(Self::orders_to_listitems(
            self.depth_update.map(|update| update.bids.as_slice()),
        ))
        .block(Block::default().borders(Borders::ALL).title("Bids"));

        frame.render_widget(block, area);
        frame.render_widget(asks, chunks[0]);
        frame.render_widget(bids, chunks[1]);
    }

    fn draw_best_price<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let block = Block::default().title("Best prices").borders(Borders::ALL);

        let default = "Not loaded";

        let best_ask = self
            .price_update
            .map(|price| price.ask.to_string())
            .unwrap_or_else(|| default.to_string());

        let best_bid = self
            .price_update
            .map(|price| price.bid.to_string())
            .unwrap_or_else(|| default.to_string());

        let table = Table::new(vec![Row::new(vec![best_ask, best_bid])])
            .header(
                Row::new(vec!["Best ask", "Best bid"])
                    .style(Style::default().add_modifier(Modifier::BOLD))
                    .bottom_margin(1),
            )
            .block(block)
            .widths(&[
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(10),
            ]);

        frame.render_widget(table, area);
    }

    pub fn draw<B: Backend>(&self, frame: &mut Frame<B>) {
        let size = frame.size();

        let block = Block::default()
            .title("Binance Scrapper")
            .borders(Borders::ALL);

        let chunks = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(0)])
            .margin(1)
            .split(size);

        frame.render_widget(block, size);
        self.draw_best_price(frame, chunks[0]);
        self.draw_order_book(frame, chunks[1]);
    }
}
