//! Minimal independently loaded ASI plugin using exact-version SA-MP services.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk_sample_plugin supports only 32-bit Windows x86 targets");

use samp::{Samp, Subscription, events::ProtocolAction};
use samp_protocol::rpc::incoming::common::SERVER_MESSAGE;
use std::{
    ffi::c_void,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
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
static SERVER_MESSAGES: AtomicUsize = AtomicUsize::new(0);

struct PluginState {
    subscription: Option<Subscription>,
    initializing: bool,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            subscription: None,
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
                .name("samp-client-sdk-sample-init".into())
                .spawn(initialize)
                .is_err()
            {
                STATE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .initializing = false;
                INITIALIZATION_FINISHED.notify_all();
            }
        }
        DLL_PROCESS_DETACH => {}
        _ => {}
    }
    TRUE
}

fn initialize() {
    let _initialization = InitializationGuard;
    let Ok(samp) = Samp::connect(Duration::from_secs(30)) else {
        return;
    };
    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    if state.shutting_down {
        return;
    }

    let subscription = samp
        .net()
        .on_incoming_typed_rpc(SERVER_MESSAGE, |_message| {
            SERVER_MESSAGES.fetch_add(1, Ordering::Relaxed);
            ProtocolAction::Continue
        });
    if let Ok(subscription) = subscription {
        state.subscription = Some(subscription);
    }
}

/// Stops callbacks before an unload manager calls `FreeLibrary`.
///
/// This must be called from a worker thread, not from `DllMain` or a service callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkPlugin_Shutdown() -> BOOL {
    let mut subscription = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        let Some(subscription) = state.subscription.take() else {
            return TRUE;
        };
        subscription
    };

    match subscription.unregister_and_wait(Duration::from_secs(10)) {
        Ok(()) => TRUE,
        Err(_) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .subscription = Some(subscription);
            0
        }
    }
}

/// Returns how many typed `onServerMessage` events this sample observed.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkSample_ServerMessageCount() -> usize {
    SERVER_MESSAGES.load(Ordering::Relaxed)
}
