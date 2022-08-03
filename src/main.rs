use anyhow::Result;
use bnc_scraper::run::run_with_ui;
use bnc_scraper::ui::runner::UiRunner;

#[tokio::main]
async fn main() -> Result<()> {
    run_with_ui()?;
    Ok(())
}
