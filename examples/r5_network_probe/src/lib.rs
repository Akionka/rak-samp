//! Loopback-only R5 network delivery validation.
//!
//! This probe sends one bounded chat marker only to the disposable local server
//! filter. Its matching server-message listener always continues, allowing a
//! human to verify that SA-MP's original incoming-RPC handler displayed reply.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk_r5_network_probe supports only 32-bit Windows x86 targets");

use samp_client_sdk::{
    CommandReceipt, Samp, SampClientSdkDirection, SampClientSdkResult, Subscription,
    events::{RpcAction, rpc::incoming},
};
use std::{
    ffi::c_void,
    fs,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    thread,
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

const OUTBOUND_MARKER: &[u8] = b"R5_SDK_OUTBOUND_20260812";
const INCOMING_MARKER: &[u8] = b"R5_SDK_INCOMING_20260812";
const INITIALIZATION_TIMEOUT: Duration = Duration::from_secs(45);
const INCOMING_READY_TIMEOUT: Duration = Duration::from_secs(60);
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(15);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const STATUS_FILE: &str = "samp-client-sdk-r5-network-probe.status";

/// The SDK host API resolved for this probe.
pub const STATUS_HOST_CONNECTED: u32 = 1 << 0;
/// The non-blocking RPC 93 listener was registered.
pub const STATUS_REPLY_LISTENER_REGISTERED: u32 = 1 << 1;
/// A real inbound RPC supplied the native receiver required for a connected client.
pub const STATUS_INCOMING_READY: u32 = 1 << 2;
/// The single outgoing chat command completed successfully on the game thread.
pub const STATUS_OUTBOUND_RECEIPT: u32 = 1 << 3;
/// The matching server reply was observed before continuing to the original handler.
pub const STATUS_REPLY_OBSERVED: u32 = 1 << 4;
/// An initialization, command, or reply stage failed.
pub const STATUS_FAILED: u32 = 1 << 31;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();
static STATUS: AtomicU32 = AtomicU32::new(0);
static FAILURE: AtomicU32 = AtomicU32::new(SampClientSdkResult::Ok as u32);

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
            STATUS.store(0, Ordering::Release);
            FAILURE.store(SampClientSdkResult::Ok as u32, Ordering::Release);
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .initializing = true;
            if thread::Builder::new()
                .name("samp-client-sdk-r5-network-probe-init".into())
                .spawn(initialize)
                .is_err()
            {
                record_failure(SampClientSdkResult::NativeCallFailed);
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
    publish_status();
    let Some(samp) = connect_host() else {
        publish_status();
        return;
    };
    STATUS.fetch_or(STATUS_HOST_CONNECTED, Ordering::AcqRel);
    publish_status();

    let subscription = match samp.net().on_typed_rpc(
        SampClientSdkDirection::Incoming,
        incoming::SERVER_MESSAGE,
        |message| {
            if message.text == INCOMING_MARKER {
                STATUS.fetch_or(STATUS_REPLY_OBSERVED, Ordering::AcqRel);
            }
            // The visible normal-chat reply is the required human proof that
            // SA-MP's original incoming-RPC handler ran after this callback.
            RpcAction::Continue
        },
    ) {
        Ok(subscription) => subscription,
        Err(error) => {
            record_failure(error);
            publish_status();
            return;
        }
    };
    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    if state.shutting_down {
        return;
    }
    state.subscription = Some(subscription);
    drop(state);
    STATUS.fetch_or(STATUS_REPLY_LISTENER_REGISTERED, Ordering::AcqRel);
    publish_status();

    let result = run_probe(samp);
    if let Err(error) = result {
        record_failure(error);
    }
    publish_status();
}

fn run_probe(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_incoming_ready(samp)?;
    STATUS.fetch_or(STATUS_INCOMING_READY, Ordering::AcqRel);
    publish_status();

    let receipt = samp.net().send_chat(OUTBOUND_MARKER)?;
    wait_for_receipt(receipt)?;
    STATUS.fetch_or(STATUS_OUTBOUND_RECEIPT, Ordering::AcqRel);
    publish_status();

    wait_for_status(STATUS_REPLY_OBSERVED, CALLBACK_TIMEOUT)
}

fn connect_host() -> Option<Samp> {
    let deadline = Instant::now() + INITIALIZATION_TIMEOUT;
    loop {
        if is_shutting_down() {
            return None;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            record_failure(SampClientSdkResult::TimedOut);
            return None;
        }
        match Samp::connect(remaining.min(RETRY_DELAY)) {
            Ok(samp) => return Some(samp),
            Err(samp_client_sdk::ResolveError::TimedOut) => {}
            Err(_) => {
                record_failure(SampClientSdkResult::NotReady);
                return None;
            }
        }
    }
}

fn wait_for_incoming_ready(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + INCOMING_READY_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        if samp.net().incoming_emulation_ready() {
            return Ok(());
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn wait_for_receipt(mut receipt: CommandReceipt<()>) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + INITIALIZATION_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        match receipt.wait(remaining.min(RETRY_DELAY)) {
            Err(SampClientSdkResult::TimedOut) => {}
            result => return result,
        }
    }
}

fn wait_for_status(required: u32, timeout: Duration) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if STATUS.load(Ordering::Acquire) & STATUS_FAILED != 0 {
            return Err(stored_failure());
        }
        if STATUS.load(Ordering::Acquire) & required == required {
            return Ok(());
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn is_shutting_down() -> bool {
    STATE
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .shutting_down
}

fn record_failure(error: SampClientSdkResult) {
    FAILURE
        .compare_exchange(
            SampClientSdkResult::Ok as u32,
            error as u32,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .ok();
    STATUS.fetch_or(STATUS_FAILED, Ordering::AcqRel);
}

fn stored_failure() -> SampClientSdkResult {
    match FAILURE.load(Ordering::Acquire) {
        value if value == SampClientSdkResult::NotReady as u32 => SampClientSdkResult::NotReady,
        value if value == SampClientSdkResult::TimedOut as u32 => SampClientSdkResult::TimedOut,
        value if value == SampClientSdkResult::ShuttingDown as u32 => {
            SampClientSdkResult::ShuttingDown
        }
        _ => SampClientSdkResult::NativeCallFailed,
    }
}

fn publish_status() {
    let _ = fs::write(
        STATUS_FILE,
        status_record(
            STATUS.load(Ordering::Acquire),
            FAILURE.load(Ordering::Acquire),
        ),
    );
}

fn status_record(status: u32, failure: u32) -> String {
    format!("status=0x{status:08X}\nfailure={failure}\n")
}

/// Stops the callback before an unload manager calls `FreeLibrary`.
///
/// This must run on a worker thread, not from `DllMain` or a callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Shutdown() -> BOOL {
    let subscription = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        state.subscription.take()
    };

    let Some(subscription) = subscription else {
        return TRUE;
    };
    match subscription.unregister_and_wait() {
        Ok(()) => TRUE,
        Err(error) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .subscription = Some(error.into_subscription());
            0
        }
    }
}

/// Returns the probe stage bitset; success before visual confirmation is `0x1F`.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Status() -> u32 {
    STATUS.load(Ordering::Acquire)
}

/// Returns the first failing `SampClientSdkResult` value, if any.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Failure() -> u32 {
    FAILURE.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_record_is_bounded_and_machine_readable() {
        assert_eq!(
            status_record(STATUS_HOST_CONNECTED | STATUS_OUTBOUND_RECEIPT, 0),
            "status=0x00000009\nfailure=0\n"
        );
    }
}
