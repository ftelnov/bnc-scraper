use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::io::Stdout;
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

pub trait UiController: Sized {
    /// Setups terminal and makes it output to the stdout.
    fn new() -> Result<Self>;

    /// Restore terminal.
    fn finalize(&mut self) -> Result<()>;
}

/// Terminal initialising and restoring entity.
pub struct UiRunner<B: Backend> {
    pub terminal: Terminal<B>,
}

impl UiController for UiRunner<CrosstermBackend<Stdout>> {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    fn finalize(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}
