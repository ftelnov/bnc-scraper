use crate::app::App;
use crate::config::AppCfg;
use crate::core::bnc::ws::worker::depth::SymbolDepthUpdate;
use crate::core::bnc::ws::worker::price::SymbolPriceUpdate;
use crate::ui::runner::{UiController, UiRunner};
use crate::ui::AppUI;
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::io::Stdout;
use std::time::{Duration, Instant};
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

pub fn read_symbol() -> Result<String> {
    println!("Write symbol you are going to scrap(empty for BTCUSDT): ");
    let symbol = std::io::stdin()
        .lines()
        .next()
        .expect("You have not provided symbol.")?;
    Ok(if symbol.is_empty() {
        "BTCUSDT".to_string()
    } else {
        symbol
    })
}

/// Run application with UI. Use it from binaries directly.
pub fn run_with_ui() -> Result<()> {
    let symbol = read_symbol()?;

    let mut runner: UiRunner<CrosstermBackend<Stdout>> = UiRunner::new()?;
    let cfg = AppCfg::load()?;

    let tick_rate = Duration::from_millis(cfg.ui.tick_rate);

    let app = App::new(cfg, symbol);

    run_app(&mut runner.terminal, app, tick_rate)?;

    runner.finalize()?;

    println!("Thx for using that garbage! Cya!");

    Ok(())
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| app.draw(f))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Finalize an application if CTRL + C/c is pressed.
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('c'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('C')) => app.finalize()?,
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick()?;
            last_tick = Instant::now();
        }

        if app.should_quit() {
            return Ok(());
        }
    }
}
