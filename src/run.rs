use crate::app::App;
use crate::config::AppCfg;
use crate::core::logging::setup_logger;
use crate::ui::runner::{UiController, UiRunner};
use anyhow::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyModifiers};

use log::info;
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
pub async fn run_with_ui() -> Result<()> {
    let cfg = AppCfg::load()?;
    setup_logger(&cfg.logging)?;

    let symbol = read_symbol()?;
    info!("User chose symbol: {}.", symbol);
    let tick_rate = Duration::from_millis(cfg.ui.tick_rate);
    let mut app = App::new(&cfg, symbol);

    app.init().await?;

    //.. And only after that we initialise UI.
    let mut runner: UiRunner<CrosstermBackend<Stdout>> = UiRunner::new()?;

    run_app(&mut runner.terminal, app, tick_rate).await?;

    runner.finalize()?;

    println!("Thx for using that garbage! Cya!");

    Ok(())
}

pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'_>,
    tick_rate: Duration,
) -> Result<()> {
    let last_tick = Instant::now();

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

        if app.should_quit() {
            return Ok(());
        }
    }
}
