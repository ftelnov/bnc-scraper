use anyhow::Result;
use bnc_scraper::run::run_with_ui;

#[tokio::main]
async fn main() -> Result<()> {
    run_with_ui().await?;
    Ok(())
}
