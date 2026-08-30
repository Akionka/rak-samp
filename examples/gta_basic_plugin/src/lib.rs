//! Minimal typed GTA SA tick plugin.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("gta-basic-plugin supports only 32-bit Windows x86 targets");

use gta_sa::{Error, HostGtaSaExt, TickSubscription};
use modkit_sdk::Host;
use std::{ffi::c_void, sync::Mutex, time::Duration};
use windows_sys::Win32::{
    Foundation::{HINSTANCE, TRUE},
    System::{
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    },
};
use windows_sys::core::BOOL;

static SUBSCRIPTION: Mutex<Option<TickSubscription>> = Mutex::new(None);

#[unsafe(no_mangle)]
unsafe extern "system" fn DllMain(
    instance: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            unsafe { DisableThreadLibraryCalls(instance) };
            let _ = std::thread::Builder::new()
                .name("gta-basic-plugin-init".into())
                .spawn(initialize);
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

fn initialize() {
    let host = match Host::connect(Duration::from_secs(120)) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("gta-basic-plugin: host connection failed: {error}");
            return;
        }
    };
    let gta = match host.gta_sa() {
        Ok(gta) => gta,
        Err(error) => {
            eprintln!("gta-basic-plugin: GTA service unavailable: {error:?}");
            return;
        }
    };
    let subscription = match gta.on_tick(|context| {
        match context.player() {
            Ok(player) => {
                let snapshot = player.snapshot()?;
                let _ = (player.position(), snapshot.health);
            }
            Err(Error::NoLocalPed) => {}
            Err(error) => return Err(error),
        }
        Ok(())
    }) {
        Ok(subscription) => subscription,
        Err(error) => {
            eprintln!("gta-basic-plugin: tick registration failed: {error:?}");
            return;
        }
    };
    *SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(subscription);
}

/// Stops callbacks before an unload manager calls `FreeLibrary`.
#[unsafe(no_mangle)]
pub extern "system" fn GtaBasicPlugin_Shutdown() -> BOOL {
    let subscription = SUBSCRIPTION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .take();
    let Some(subscription) = subscription else {
        return TRUE;
    };
    if subscription
        .unregister_and_wait(Duration::from_secs(10))
        .is_ok()
    {
        TRUE
    } else {
        0
    }
}
