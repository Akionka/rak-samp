//! Minimal independently loaded ASI plugin that connects through the new
//! `GtaModHost_GetApiV1` export and queries the Core and Legacy SA-MP services.
//!
//! This example demonstrates the Phase 3 service-discovery path. It never falls
//! back to the legacy `SampClientSdk_GetApiV1` export.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("modkit_connect_plugin supports only 32-bit Windows x86 targets");

use modkit_sdk::Host;
use std::{
    ffi::c_void,
    sync::{Condvar, Mutex},
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};
use windows_sys::core::BOOL;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();

struct PluginState {
    initializing: bool,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            initializing: false,
            shutting_down: false,
        }
    }
}

struct InitializationGuard;

impl Drop for InitializationGuard {
    fn drop(&mut self) {
        STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .initializing = false;
        INITIALIZATION_FINISHED.notify_all();
    }
}

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(instance) };
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .initializing = true;
            if std::thread::Builder::new()
                .name("modkit-connect-init".into())
                .spawn(initialize)
                .is_err()
            {
                STATE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .initializing = false;
                INITIALIZATION_FINISHED.notify_all();
            }
            TRUE
        }
        DLL_PROCESS_DETACH => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .shutting_down = true;
            TRUE
        }
        _ => TRUE,
    }
}

fn initialize() {
    let _guard = InitializationGuard;

    let host = match Host::connect(Duration::from_secs(10)) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("modkit-connect-plugin: host connect failed: {error}");
            return;
        }
    };
    if is_shutting_down() {
        return;
    }

    // Query the Core service and read host status.
    match host.core() {
        Ok(core) => match core.host_status() {
            Ok(status) => eprintln!("modkit-connect-plugin: host status = {status:?}"),
            Err(result) => eprintln!("modkit-connect-plugin: host_status failed: {result:?}"),
        },
        Err(error) => eprintln!("modkit-connect-plugin: core service unavailable: {error:?}"),
    }
    if is_shutting_down() {
        return;
    }

    // Query the migration-only Legacy SA-MP service.
    match host.legacy_samp() {
        Ok(legacy) => {
            eprintln!("modkit-connect-plugin: legacy samp service resolved");
            if !legacy.is_available() {
                eprintln!("modkit-connect-plugin: legacy api pointer is null");
            }
        }
        Err(error) => eprintln!("modkit-connect-plugin: legacy service unavailable: {error:?}"),
    }
    if is_shutting_down() {
        return;
    }

    // Verify an unknown service is reported as NotFound.
    match host.query_service(modkit_abi::ServiceId(0x0000_0002), 1) {
        Err(modkit_sdk::ServiceError::NotFound) => {
            eprintln!("modkit-connect-plugin: unknown service correctly reported NotFound");
        }
        Err(error) => {
            eprintln!("modkit-connect-plugin: unexpected unknown-service error: {error:?}")
        }
        Ok(_) => eprintln!("modkit-connect-plugin: unexpected unknown-service success"),
    }
}

fn is_shutting_down() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .shutting_down
}

/// Stops initialization before an unload manager calls `FreeLibrary`.
///
/// Call this from a worker thread, never from `DllMain`.
#[unsafe(no_mangle)]
pub extern "system" fn ModkitConnectPlugin_Shutdown() -> BOOL {
    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    state.shutting_down = true;
    while state.initializing {
        state = INITIALIZATION_FINISHED
            .wait(state)
            .unwrap_or_else(|error| error.into_inner());
    }
    TRUE
}
