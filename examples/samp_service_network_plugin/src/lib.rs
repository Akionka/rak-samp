//! Phase 7 typed network service example.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp-service-network-plugin supports only 32-bit Windows x86 targets");

use samp::{ChatStyle, Samp, Subscription, events::ProtocolAction};
use samp_protocol::rpc::outgoing::chat::SendChat;
use std::{
    ffi::c_void,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicU32, Ordering},
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
static OBSERVED_CHAT_COUNT: AtomicU32 = AtomicU32::new(0);
const OBSERVED_MESSAGE: &[u8] = b"Phase 7 SampNetServiceV1 typed RPC path works";

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
                .name("samp-service-network-init".into())
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
        DLL_PROCESS_DETACH => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .shutting_down = true;
        }
        _ => {}
    }
    TRUE
}

fn initialize() {
    let _guard = InitializationGuard;
    let Some(samp) = connect() else {
        return;
    };
    let subscription = match samp.net().on_outgoing_typed_rpc(SendChat, move |_| {
        OBSERVED_CHAT_COUNT.fetch_add(1, Ordering::Relaxed);
        let _receipt = samp
            .chat()
            .add(ChatStyle::Info, OBSERVED_MESSAGE, b"", 0xFFFF_FFFF, 0);
        ProtocolAction::Continue
    }) {
        Ok(subscription) => subscription,
        Err(error) => {
            eprintln!("samp-service-network-plugin: registration failed: {error:?}");
            return;
        }
    };

    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    state.subscription = Some(subscription);
}

fn connect() -> Option<Samp> {
    match Samp::connect(Duration::from_secs(30)) {
        Ok(samp) => Some(samp),
        Err(error) => {
            eprintln!("samp-service-network-plugin: connect failed: {error}");
            None
        }
    }
}

/// Stops the callback before an unload manager calls `FreeLibrary`.
#[unsafe(no_mangle)]
pub extern "system" fn SampServiceNetworkPlugin_Shutdown() -> BOOL {
    let mut pending = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        state.subscription.take()
    };
    let Some(subscription) = pending.as_mut() else {
        return TRUE;
    };
    match subscription.unregister_and_wait(Duration::from_secs(10)) {
        Ok(()) => TRUE,
        Err(_) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .subscription = pending.take();
            0
        }
    }
}

/// Returns the number of valid outgoing chat RPCs observed by the typed callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampServiceNetworkPlugin_ObservedChatCount() -> u32 {
    OBSERVED_CHAT_COUNT.load(Ordering::Relaxed)
}
