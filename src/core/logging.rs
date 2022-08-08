use derive_getters::Getters;
use log::LevelFilter;
use serde::Deserialize;
use std::fs::create_dir_all;
use std::path::Path;

// In real-world application this boilerplate should be moved into separated module in order to use in sub-projects.
// Logging won't include writing things to any files - use pipes instead.

#[derive(Getters, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LogCfg {
    pub level: LevelFilter,
    pub enabled: bool,
    pub logfile: String,
}

impl Default for LogCfg {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
            enabled: true,
            logfile: String::from("logs/default.log"),
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
        .level(cfg.level);

    // Create parent directory for specified logfile.
    let path = Path::new(&cfg.logfile);
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    let logger = logger.chain(fern::log_file(&cfg.logfile)?);

    logger.apply()?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::AppCfg;
    use log::info;
    use log::LevelFilter::Debug;

    pub fn setup_test_logger() {
        setup_logger(&LogCfg {
            level: Debug,
            ..Default::default()
        })
        .ok();
    }

    // Loads current app's configuration and ensures that logger is loadable.
    #[ignore]
    #[test]
    fn it_setup_app_logger() {
        let app_cfg = AppCfg::load().unwrap();
        setup_logger(app_cfg.logging()).unwrap();
        info!("App's logger is able to send some INFO-leveled data!")
    }
}
