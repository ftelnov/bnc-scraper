/// Module that holds core app's functionality - binance interaction, base models, etc
mod core;

/// General application's configuration;
///
/// This module doesn't include some specific configuration for the subparts of the application,  
/// but the summary of these configuration files.
pub mod config;

pub mod ui;

pub mod app;
/// Utilities that absorbs ui, core and provide full-featured application.
pub mod run;

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn it_compiles() {
        assert_ne!("Scraper", "Not finished yet!");
    }
}
