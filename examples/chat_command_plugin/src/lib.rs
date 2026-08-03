//! Example ASI that handles `/raksamp` through the process-wide rak-samp host.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_chat_command_example supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    ABI_VERSION_V1, HostApi, RakSampDirection, RakSampEventV1, RakSampHookAction, RakSampResult,
    RakSampSendOptions, RakSampSubscription,
    events::{
        RpcAction,
        rpc::{incoming, outgoing},
    },
    wait_for_default_host,
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
const DIALOG_STYLE_MESSAGE_BOX: u8 = 0;
const NOT_RUN: u32 = u32::MAX;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();
static LAST_CHAT_RESULT: AtomicU32 = AtomicU32::new(NOT_RUN);
static LAST_DIALOG_RESULT: AtomicU32 = AtomicU32::new(NOT_RUN);

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
    let mut subscription = RakSampSubscription::default();
    let result = unsafe {
        (api.raw().register_rpc)(
            RakSampDirection::Outgoing,
            Some(on_outgoing_rpc),
            std::ptr::null_mut(),
            &raw mut subscription,
        )
    };
    if result == RakSampResult::Ok {
        state.api = Some(api);
        state.subscription = Some(subscription);
    }
}

unsafe extern "system" fn on_outgoing_rpc(
    _user_data: *mut c_void,
    event: *mut RakSampEventV1,
) -> RakSampHookAction {
    let api = STATE.lock().unwrap_or_else(|error| error.into_inner()).api;
    let Some(api) = api else {
        return RakSampHookAction::Continue;
    };

    let command_action = unsafe {
        outgoing::on_send_command(api, event, |command| {
            if is_raksamp_command(&command) {
                run_example(api);
                RpcAction::Block
            } else {
                RpcAction::Continue
            }
        })
    }
    .unwrap_or(RakSampHookAction::Continue);
    if command_action == RakSampHookAction::Block {
        return command_action;
    }

    unsafe {
        outgoing::on_send_dialog_response(api, event, |response| {
            if response.dialog_id == DIALOG_ID {
                RpcAction::Block
            } else {
                RpcAction::Continue
            }
        })
    }
    .unwrap_or(RakSampHookAction::Continue)
}

fn run_example(api: HostApi) {
    let chat_result = send_chat(api);
    LAST_CHAT_RESULT.store(chat_result as u32, Ordering::Release);

    let info = format!(
        "rak-samp host status: {:?}\nABI version: {}\nSEND_CHAT result: {:?}\n\nThis dialog was generated locally through fake incoming RPC 61.",
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
    let dialog = incoming::ShowDialog {
        dialog_id: DIALOG_ID,
        style: DIALOG_STYLE_MESSAGE_BOX,
        title: b"rak-samp example".to_vec(),
        button1: b"Close".to_vec(),
        button2: Vec::new(),
        text,
    };
    let Ok(payload) = incoming::SHOW_DIALOG.encode(api, dialog) else {
        return RakSampResult::NativeCallFailed;
    };
    api.emulate_incoming_rpc(
        incoming::SHOW_DIALOG.id(),
        payload.as_bytes(),
        payload.len_bits(),
    )
}

fn is_raksamp_command(command: &[u8]) -> bool {
    command.eq_ignore_ascii_case(COMMAND)
}

/// Stops the callback before an unload manager calls `FreeLibrary`.
///
/// Call this from a worker thread, never from `DllMain` or a rak-samp callback.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampChatCommand_Shutdown() -> BOOL {
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

/// Returns the numeric result of the most recent explicit `SEND_CHAT`, or `u32::MAX` if unused.
#[unsafe(no_mangle)]
pub extern "system" fn RakSampChatCommand_LastChatResult() -> u32 {
    LAST_CHAT_RESULT.load(Ordering::Acquire)
}

/// Returns the numeric result of the most recent local dialog emulation, or `u32::MAX` if unused.
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
