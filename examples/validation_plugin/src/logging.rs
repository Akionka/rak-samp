use log::{Level, Log, Record};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::{
    fs::{File, OpenOptions},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    sync::OnceLock,
};
use windows_sys::Win32::{Foundation::HINSTANCE, System::LibraryLoader::GetModuleFileNameW};

static LOGGER: OnceLock<Box<WriteLogger<File>>> = OnceLock::new();
static PLUGIN_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn initialize(instance: HINSTANCE) {
    if let Some(directory) = module_directory(instance) {
        let _ = PLUGIN_DIRECTORY.set(directory);
    }
    let Ok(file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(plugin_path("rak-samp-validation.log"))
    else {
        return;
    };
    let config = ConfigBuilder::new().set_time_format_rfc3339().build();
    let logger = WriteLogger::new(LevelFilter::Debug, config, file);
    let _ = LOGGER.set(logger);
}

pub(crate) fn plugin_path(name: &str) -> PathBuf {
    PLUGIN_DIRECTORY
        .get()
        .map_or_else(|| PathBuf::from(name), |directory| directory.join(name))
}

fn module_directory(instance: HINSTANCE) -> Option<PathBuf> {
    let mut buffer = vec![0_u16; 260];
    loop {
        let length = unsafe {
            GetModuleFileNameW(
                instance,
                buffer.as_mut_ptr(),
                u32::try_from(buffer.len()).ok()?,
            )
        } as usize;
        if length == 0 {
            return None;
        }
        if length < buffer.len() {
            let module = PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length]));
            return module.parent().map(PathBuf::from);
        }
        if buffer.len() >= 32_768 {
            return None;
        }
        buffer.resize((buffer.len() * 2).min(32_768), 0);
    }
}

pub(crate) fn write(message: &str) {
    let Some(logger) = LOGGER.get() else {
        return;
    };
    let message = message.replace(['\r', '\n'], " ");
    let arguments = format_args!("{message}");
    let record = Record::builder()
        .args(arguments)
        .level(Level::Info)
        .target("validation")
        .build();
    logger.log(&record);
}
