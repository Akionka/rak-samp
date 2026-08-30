//! Phase 7 chat service example.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp-service-chat-plugin supports only 32-bit Windows x86 targets");

use samp::{ChatCommandRegistration, ChatStyle, Samp, Subscription};
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

const COMMAND: &[u8] = b"/sampservice";
const MESSAGE: &[u8] = b"Phase 7 SampServiceV1 chat path works";

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();

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
                .name("samp-service-chat-init".into())
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
    let registration = match samp.chat().register_command(COMMAND, move |_| {
        let _receipt = samp
            .chat()
            .add(ChatStyle::Info, MESSAGE, b"", 0xFFFF_FFFF, 0);
    }) {
        Ok(registration) => registration,
        Err(error) => {
            eprintln!("samp-service-chat-plugin: registration failed: {error:?}");
            return;
        }
    };
    let ChatCommandRegistration {
        mut subscription,
        installation,
    } = registration;
    if let Err(error) = installation.wait(Duration::from_secs(10)) {
        eprintln!("samp-service-chat-plugin: installation failed: {error:?}");
        if subscription
            .unregister_and_wait(Duration::from_secs(10))
            .is_err()
        {
            STATE
                .lock()
                .unwrap_or_else(|lock_error| lock_error.into_inner())
                .subscription = Some(subscription);
        }
        return;
    }

    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    state.subscription = Some(subscription);
}

fn connect() -> Option<Samp> {
    match Samp::connect(Duration::from_secs(30)) {
        Ok(samp) => Some(samp),
        Err(error) => {
            eprintln!("samp-service-chat-plugin: connect failed: {error}");
            None
        }
    }
}

/// Stops the callback before an unload manager calls `FreeLibrary`.
#[unsafe(no_mangle)]
pub extern "system" fn SampServiceChatPlugin_Shutdown() -> BOOL {
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
