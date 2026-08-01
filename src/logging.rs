use log::LevelFilter;
use simplelog::{Config, WriteLogger};
use std::{fs::OpenOptions, sync::Once};

pub(crate) const LOG_FILE_NAME: &str = "rak-rs.log";

/// Configures the host logger once, outside the Windows loader lock.
pub(crate) fn initialize() {
    static INITIALIZE_LOGGER: Once = Once::new();

    INITIALIZE_LOGGER.call_once(|| {
        let file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(LOG_FILE_NAME)
        {
            Ok(file) => file,
            Err(error) => {
                eprintln!("rak-rs: could not open {LOG_FILE_NAME}: {error}");
                return;
            }
        };

        if let Err(error) = WriteLogger::init(LevelFilter::Debug, Config::default(), file) {
            eprintln!("rak-rs: could not initialize logging: {error}");
        }
    });
}
