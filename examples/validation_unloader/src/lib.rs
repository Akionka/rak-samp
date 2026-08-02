//! External validation manager for synchronized runtime ASI unload.

use std::{
    ffi::c_void,
    fs::{File, OpenOptions},
    io::Write,
    mem,
    path::Path,
    sync::{Mutex, OnceLock},
    time::{Duration, Instant},
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use windows_sys::Win32::{
    Foundation::{FreeLibrary, HINSTANCE, TRUE},
    System::{
        LibraryLoader::{DisableThreadLibraryCalls, GetModuleHandleA, GetProcAddress},
        SystemServices::DLL_PROCESS_ATTACH,
    },
};
use windows_sys::core::BOOL;

const ENABLE_MARKER: &str = "rak-rs-validation-unload.enabled";
const TARGET_MODULE: &[u8] = b"rak_rs_validation.asi\0";
const TARGET_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const SELF_TEST_WAIT_TIMEOUT: Duration = Duration::from_secs(120);
const POLL_INTERVAL: Duration = Duration::from_millis(10);

type ValidationFn = unsafe extern "system" fn() -> BOOL;

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        unsafe { DisableThreadLibraryCalls(instance) };
        let _ = std::thread::Builder::new()
            .name("rak-rs-validation-unloader".into())
            .spawn(run);
    }
    TRUE
}

fn run() {
    initialize_log();
    write_log(&format!(
        "session started: process_id={}",
        std::process::id()
    ));
    if !Path::new(ENABLE_MARKER).is_file() {
        write_log("unload validation disabled; marker file is absent");
        return;
    }

    let Some(module) = wait_for_target() else {
        write_log("target validation ASI did not load within 30 seconds");
        return;
    };
    let Some(self_tests_complete) = resolve(module, c"RakRsValidation_SelfTestsComplete") else {
        write_log("target is missing RakRsValidation_SelfTestsComplete");
        return;
    };
    let Some(shutdown) = resolve(module, c"RakRsPlugin_Shutdown") else {
        write_log("target is missing RakRsPlugin_Shutdown");
        return;
    };

    let deadline = Instant::now() + SELF_TEST_WAIT_TIMEOUT;
    while unsafe { self_tests_complete() } != TRUE {
        if Instant::now() >= deadline {
            write_log("target self-tests did not finish within 120 seconds; unload cancelled");
            return;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
    write_log("target self-tests finished; requesting synchronized shutdown");
    if unsafe { shutdown() } != TRUE {
        write_log("target shutdown failed; unload cancelled");
        return;
    }
    write_log("target shutdown succeeded; releasing the ASI loader reference");

    if unsafe { FreeLibrary(module) } == 0 {
        write_log(&format!(
            "FreeLibrary failed: {}",
            std::io::Error::last_os_error()
        ));
        return;
    }
    if unsafe { GetModuleHandleA(TARGET_MODULE.as_ptr()) }.is_null() {
        write_log("validation ASI unloaded successfully");
    } else {
        write_log("FreeLibrary returned success, but validation ASI remains loaded");
    }
}

fn wait_for_target() -> Option<HINSTANCE> {
    let deadline = Instant::now() + TARGET_WAIT_TIMEOUT;
    loop {
        let module = unsafe { GetModuleHandleA(TARGET_MODULE.as_ptr()) };
        if !module.is_null() {
            return Some(module);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

fn resolve(module: HINSTANCE, name: &std::ffi::CStr) -> Option<ValidationFn> {
    let symbol = unsafe { GetProcAddress(module, name.as_ptr().cast()) }?;
    Some(unsafe { mem::transmute::<unsafe extern "system" fn() -> isize, ValidationFn>(symbol) })
}

fn initialize_log() {
    let result = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("rak-rs-validation-unloader.log");
    if let Ok(file) = result {
        let _ = LOG_FILE.set(Mutex::new(file));
    }
}

fn write_log(message: &str) {
    let Some(file) = LOG_FILE.get() else {
        return;
    };
    let mut file = file.lock().unwrap_or_else(|error| error.into_inner());
    let _ = file.write_all(format_log_line("INFO", "validation-unloader", message).as_bytes());
    let _ = file.flush();
}

fn format_log_line(level: &str, source: &str, message: &str) -> String {
    format!(
        "{} {} {} {}\n",
        rfc3339_now(),
        level,
        source,
        message.replace(['\r', '\n'], " ")
    )
}

fn rfc3339_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}
