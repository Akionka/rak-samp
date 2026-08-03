//! Minimal independently loaded ASI plugin using the rak-samp host ABI.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_sample_plugin supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    HostApi, RakSampDirection, RakSampEventV1, RakSampHookAction, RakSampResult,
    RakSampSubscription,
    events::{RpcAction, rpc::incoming},
    wait_for_default_host,
};
use std::{
    ffi::c_void,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
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
    api: Option<HostApi>,
    subscription: Option<RakSampSubscription>,
    initializing: bool,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            api: None,
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
                .name("rak-samp-sample-init".into())
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
    let deadline = Instant::now() + Duration::from_secs(30);
    let api = loop {
        if STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .shutting_down
        {
            return;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return;
        }
        match wait_for_default_host(remaining.min(Duration::from_millis(100))) {
            Ok(api) => break api,
            Err(rak_samp_plugin_api::ResolveError::TimedOut) => {}
            Err(_) => return,
        }
    };
    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    if state.shutting_down {
        return;
    }

    let mut subscription = RakSampSubscription::default();
    let result = unsafe {
        (api.raw().register_rpc)(
            RakSampDirection::Incoming,
            Some(on_incoming_rpc),
            std::ptr::null_mut(),
            &raw mut subscription,
        )
    };
    if result == RakSampResult::Ok {
        state.api = Some(api);
        state.subscription = Some(subscription);
    }
}

unsafe extern "system" fn on_incoming_rpc(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    let api = {
        let state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.api
    };
    let Some(api) = api else {
        return RakSampHookAction::Continue;
    };
    unsafe {
        incoming::on_server_message(api, event, |_message| {
            SERVER_MESSAGES.fetch_add(1, Ordering::Relaxed);
            RpcAction::Continue
        })
    }
    .unwrap_or(RakSampHookAction::Continue)
}

/// Stops callbacks before an unload manager calls `FreeLibrary`.
///
/// This must be called from a worker thread, not from `DllMain` or a rak-samp callback.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampPlugin_Shutdown() -> BOOL {
    let (api, subscription) = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        let Some(api) = state.api else {
            return TRUE;
        };
        let Some(subscription) = state.subscription.take() else {
            return TRUE;
        };
        (api, subscription)
    };

    match api.unregister_and_wait(subscription) {
        RakSampResult::Ok | RakSampResult::SubscriptionNotFound => TRUE,
        _ => {
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
pub extern "system" fn RakSampSample_ServerMessageCount() -> usize {
    SERVER_MESSAGES.load(Ordering::Relaxed)
}
