use crate::core::bnc::data::InlineOrder;
use crate::core::bnc::state::book::{OrderBookDisplay, TableDisplay};
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

fn orders_to_listitems(orders: &TableDisplay) -> Vec<ListItem> {
    orders
        .iter()
        .take(10)
        .map(|order| ListItem::new(format!("{}/{}", order.0, order.1)))
        .collect()
}

pub fn draw_order_book<B: Backend>(frame: &mut Frame<B>, area: Rect, book: &OrderBookDisplay) {
    let block = Block::default().title("Order book").borders(Borders::ALL);
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .margin(1)
        .split(area);

    let asks = List::new(orders_to_listitems(&book.asks))
        .block(Block::default().borders(Borders::ALL).title("Asks"));

    let bids = List::new(orders_to_listitems(&book.bids))
        .block(Block::default().borders(Borders::ALL).title("Bids"));

    frame.render_widget(block, area);
    frame.render_widget(asks, chunks[0]);
    frame.render_widget(bids, chunks[1]);
}

pub fn draw_best_price<B: Backend>(frame: &mut Frame<B>, area: Rect, update: &SymbolPriceUpdate) {
    let block = Block::default().title("Best prices").borders(Borders::ALL);

    let best_ask = update.ask.to_string();

    let best_bid = update.bid.to_string();

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

pub struct AppUiLayout {
    pub best_prices: Rect,
    pub order_book: Rect,
}

pub fn get_global_layout<B: Backend>(frame: &Frame<B>) -> AppUiLayout {
    let chunks = Layout::default()
        .direction(Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)])
        .margin(1)
        .split(frame.size());

    AppUiLayout {
        best_prices: chunks[0],
        order_book: chunks[1],
    }
}

pub fn draw_background<B: Backend>(frame: &mut Frame<B>) {
    let size = frame.size();

    let block = Block::default()
        .title("Binance Scrapper")
        .borders(Borders::ALL);

    frame.render_widget(block, size);
}
