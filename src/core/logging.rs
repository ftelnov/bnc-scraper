use derive_getters::Getters;
use log::Level;
use serde::Deserialize;

// In real-world application this boilerplate should be moved into separated module in order to use in sub-projects.
// Logging won't include writing things to any files - use pipes instead.

#[derive(Getters, Deserialize, Clone, Debug)]
pub struct LogCfg {
    pub level: Level,
    pub enabled: bool,
}

impl Default for LogCfg {
    fn default() -> Self {
        Self {
            level: Level::Info,
            enabled: true,
        }
    }
}

/// Load default logging implementation from the specified logging configuration.
pub fn setup_logger(cfg: &LogCfg) -> Result<(), fern::InitError> {
    if !cfg.enabled {
        return Ok(());
    }

    let logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(cfg.level.to_level_filter())
        .chain(std::io::stdout());

    logger.apply()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppCfg;
    use log::info;

    // Loads current app's configuration and ensures that logger is loadable.
    #[test]
    fn it_setup_app_logger() {
        let app_cfg = AppCfg::load().unwrap();
        setup_logger(app_cfg.logging()).unwrap();
        info!("App's logger is able to send some INFO-leveled data!")
    }
}
