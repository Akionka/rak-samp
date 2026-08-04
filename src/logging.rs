use log::LevelFilter;
use simplelog::{ConfigBuilder, WriteLogger};
use std::{fs::OpenOptions, sync::Once};

pub(crate) const LOG_FILE_NAME: &str = "samp-client-sdk.log";

/// Configures the host logger once, outside the Windows loader lock.
pub(crate) fn initialize() {
    static INITIALIZE_LOGGER: Once = Once::new();

    INITIALIZE_LOGGER.call_once(|| {
        let file = match OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(LOG_FILE_NAME)
        {
            Ok(file) => file,
            Err(error) => {
                eprintln!("samp-client-sdk: could not open {LOG_FILE_NAME}: {error}");
                return;
            }
        };
        let config = ConfigBuilder::new().set_time_format_rfc3339().build();
        if let Err(error) = WriteLogger::init(LevelFilter::Debug, config, file) {
            eprintln!("samp-client-sdk: could not initialize logging: {error}");
        }
    });
}
