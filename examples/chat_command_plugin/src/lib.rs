//! Example ASI that handles `/raksamp` through the process-wide rak-samp host.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_chat_command_example supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    ABI_VERSION_V1, HostApi, LocalDialog, LocalDialogStyle, RakSampDirection, RakSampResult,
    RakSampSendOptions, SubscriptionSet,
    events::{RpcAction, rpc::outgoing},
    register_handlers, wait_for_default_host,
};
use std::{
    ffi::c_void,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicU32, Ordering},
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

const COMMAND: &[u8] = b"/raksamp";
const CHAT_MESSAGE: &[u8] = b"rak-samp example: SEND_CHAT RPC works";
const DIALOG_ID: u16 = 0x7F00;
const NOT_RUN: u32 = u32::MAX;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();
static LAST_CHAT_RESULT: AtomicU32 = AtomicU32::new(NOT_RUN);
static LAST_DIALOG_RESULT: AtomicU32 = AtomicU32::new(NOT_RUN);

struct PluginState {
    subscriptions: SubscriptionSet,
    initializing: bool,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            subscriptions: SubscriptionSet::new(),
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
                .name("rak-samp-chat-command-init".into())
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
    let subscriptions = match register_handlers!(api;
        typed_rpc(
            RakSampDirection::Outgoing,
            outgoing::SEND_COMMAND,
            move |command| {
                if is_raksamp_command(&command) {
                    run_example(api);
                    RpcAction::Block
                } else {
                    RpcAction::Continue
                }
            }
        ),
    ) {
        Ok(subscriptions) => subscriptions,
        Err(error) => {
            if let Err(error) = error.into_subscriptions().unregister_and_wait() {
                state.subscriptions = error.into_subscriptions();
            }
            return;
        }
    };
    state.subscriptions = subscriptions;
}

fn run_example(api: HostApi) {
    let chat_result = send_chat(api);
    LAST_CHAT_RESULT.store(chat_result as u32, Ordering::Release);

    let info = format!(
        "rak-samp host status: {:?}\nABI version: {}\nSEND_CHAT result: {:?}\n\nThis dialog is direct and local; it does not emulate RPC 61 or intercept a dialog response.",
        api.status(),
        ABI_VERSION_V1,
        chat_result,
    );
    let dialog_result = show_local_dialog(api, info.into_bytes());
    LAST_DIALOG_RESULT.store(dialog_result as u32, Ordering::Release);
}

fn send_chat(api: HostApi) -> RakSampResult {
    let Ok(payload) = outgoing::SEND_CHAT.encode(api, CHAT_MESSAGE.to_vec()) else {
        return RakSampResult::InvalidArgument;
    };
    api.send_rpc(
        outgoing::SEND_CHAT.id(),
        payload.as_bytes(),
        payload.len_bits(),
        RakSampSendOptions::default(),
    )
}

fn show_local_dialog(api: HostApi, text: Vec<u8>) -> RakSampResult {
    api.show_local_dialog(LocalDialog {
        id: DIALOG_ID,
        style: LocalDialogStyle::MessageBox,
        title: b"rak-samp example",
        text: &text,
        button1: b"Close",
        button2: b"",
    })
}

fn is_raksamp_command(command: &[u8]) -> bool {
    command.eq_ignore_ascii_case(COMMAND)
}

/// Stops the callback before an unload manager calls `FreeLibrary`.
///
/// Call this from a worker thread, never from `DllMain` or a rak-samp callback.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampChatCommand_Shutdown() -> BOOL {
    let subscriptions = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        std::mem::take(&mut state.subscriptions)
    };

    if subscriptions.is_empty() {
        return TRUE;
    }
    match subscriptions.unregister_and_wait() {
        Ok(()) => TRUE,
        Err(error) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .subscriptions = error.into_subscriptions();
            0
        }
    }
}

/// Returns the numeric result of the most recent explicit `SEND_CHAT`, or `u32::MAX` if unused.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampChatCommand_LastChatResult() -> u32 {
    LAST_CHAT_RESULT.load(Ordering::Acquire)
}

/// Returns the numeric result of the most recent direct local-dialog request, or `u32::MAX` if unused.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampChatCommand_LastDialogResult() -> u32 {
    LAST_DIALOG_RESULT.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_the_local_command() {
        assert!(is_raksamp_command(b"/raksamp"));
        assert!(is_raksamp_command(b"/RAKSAMP"));
        assert!(!is_raksamp_command(b"/raksamp extra"));
        assert!(!is_raksamp_command(b"raksamp"));
    }
}
