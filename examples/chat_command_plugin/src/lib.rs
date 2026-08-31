//! Example ASI that handles `/sampclientsdk` through exact-version SA-MP services.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk_chat_command_example supports only 32-bit Windows x86 targets");

use samp::{
    DialogRequest, DialogStyle, ProtocolSendError, Samp, Subscription, events::ProtocolAction,
};
use samp_protocol::rpc::outgoing::chat::SEND_COMMAND;
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

const COMMAND: &[u8] = b"/sampclientsdk";
const CHAT_MESSAGE: &[u8] = b"samp-client-sdk example: SEND_CHAT RPC works";
const DIALOG_ID: u16 = 0x7F00;
const NOT_RUN: u32 = u32::MAX;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();
static LAST_CHAT_RESULT: AtomicU32 = AtomicU32::new(NOT_RUN);
static LAST_DIALOG_RESULT: AtomicU32 = AtomicU32::new(NOT_RUN);

struct PluginState {
    subscriptions: Vec<Subscription>,
    initializing: bool,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
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
                .name("samp-client-sdk-chat-command-init".into())
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
    let net = samp.net();
    let subscriptions = match net.on_outgoing_typed_rpc(SEND_COMMAND, move |command| {
        if is_samp_client_sdk_command(&command) {
            run_example(samp);
            ProtocolAction::Block
        } else {
            ProtocolAction::Continue
        }
    }) {
        Ok(subscription) => vec![subscription],
        Err(_) => {
            return;
        }
    };
    state.subscriptions = subscriptions;
}

fn run_example(samp: Samp) {
    let chat_result = send_chat(samp);
    LAST_CHAT_RESULT.store(chat_result, Ordering::Release);

    let info = format!(
        "Exact-version SA-MP services\nSEND_CHAT result: {chat_result}\n\nThis dialog is direct and local; it does not emulate RPC 61 or intercept a dialog response.",
    );
    let dialog_result = show_local_dialog(samp, info.into_bytes());
    LAST_DIALOG_RESULT.store(dialog_result, Ordering::Release);
}

fn send_chat(samp: Samp) -> u32 {
    match samp.net().send_chat(CHAT_MESSAGE) {
        Ok(_receipt) => 0,
        Err(ProtocolSendError::Encode(_)) => modkit_abi::MOD_INVALID_ARGUMENT.0 as u32,
        Err(ProtocolSendError::Host(result)) => result.0 as u32,
    }
}

fn show_local_dialog(samp: Samp, text: Vec<u8>) -> u32 {
    match samp.ui().dialogs().show(DialogRequest {
        id: DIALOG_ID,
        style: DialogStyle::MessageBox,
        title: b"samp service example",
        text: &text,
        button1: b"Close",
        button2: b"",
    }) {
        Ok(_receipt) => 0,
        Err(error) => error.0 as u32,
    }
}

fn is_samp_client_sdk_command(command: &[u8]) -> bool {
    command.eq_ignore_ascii_case(COMMAND)
}

/// Stops the callback before an unload manager calls `FreeLibrary`.
///
/// Call this from a worker thread, never from `DllMain` or a service callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkChatCommand_Shutdown() -> BOOL {
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
    let mut failed = Vec::new();
    for mut subscription in subscriptions {
        if subscription
            .unregister_and_wait(Duration::from_secs(10))
            .is_err()
        {
            failed.push(subscription);
        }
    }
    if failed.is_empty() {
        TRUE
    } else {
        STATE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .subscriptions = failed;
        0
    }
}

/// Returns the numeric result of the most recent explicit `SEND_CHAT`, or `u32::MAX` if unused.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkChatCommand_LastChatResult() -> u32 {
    LAST_CHAT_RESULT.load(Ordering::Acquire)
}

/// Returns the numeric result of the most recent direct local-dialog request, or `u32::MAX` if unused.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkChatCommand_LastDialogResult() -> u32 {
    LAST_DIALOG_RESULT.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_only_the_local_command() {
        assert!(is_samp_client_sdk_command(b"/sampclientsdk"));
        assert!(is_samp_client_sdk_command(b"/SAMPCLIENTSDK"));
        assert!(!is_samp_client_sdk_command(b"/sampclientsdk extra"));
        assert!(!is_samp_client_sdk_command(b"sampclientsdk"));
    }
}
