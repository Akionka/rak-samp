//! Loopback-only R3-1 network delivery validation.
//!
//! This probe sends one bounded chat marker only to the disposable local server
//! filter. Its matching server-message listener always continues, allowing a
//! human to verify that SA-MP's original incoming-RPC handler displayed reply.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk_r3_network_probe supports only 32-bit Windows x86 targets");

use samp_client_sdk::{
    CommandReceipt, Samp, SampClientSdkClientVersion, SampClientSdkDirection,
    SampClientSdkHostStatus, SampClientSdkResult, Subscription,
    events::{RpcAction, rpc::incoming},
    raw,
};
use std::{
    ffi::c_void,
    fs, ptr,
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

const OUTBOUND_MARKER: &[u8] = b"R3_SDK_OUTBOUND_20260812";
const INCOMING_MARKER: &[u8] = b"R3_SDK_INCOMING_20260812";
const INITIALIZATION_TIMEOUT: Duration = Duration::from_secs(45);
const INCOMING_READY_TIMEOUT: Duration = Duration::from_secs(60);
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(15);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const STATUS_FILE: &str = "samp-client-sdk-r3-network-probe.status";
const R3_1_ENTRY_POINT_RVA: u32 = 0x0CC4D0;
const MAX_PE_HEADER_OFFSET: usize = 0x1000;

/// The SDK host API resolved for this probe.
pub const STATUS_HOST_CONNECTED: u32 = 1 << 0;
/// Public host status, version, and the opaque module base identify the pinned R3-1 image.
pub const STATUS_RUNTIME_IDENTITY: u32 = 1 << 1;
/// The non-blocking RPC 93 listener was registered.
pub const STATUS_REPLY_LISTENER_REGISTERED: u32 = 1 << 2;
/// A real inbound RPC supplied the native receiver required for a connected client.
pub const STATUS_INCOMING_READY: u32 = 1 << 3;
/// The single outgoing chat command completed successfully on the game thread.
pub const STATUS_OUTBOUND_RECEIPT: u32 = 1 << 4;
/// The matching server reply was observed before continuing to the original handler.
pub const STATUS_REPLY_OBSERVED: u32 = 1 << 5;
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
                .name("samp-client-sdk-r3-network-probe-init".into())
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
    if let Err(error) = verify_runtime_identity(samp) {
        record_failure(error);
        publish_status();
        return;
    }
    STATUS.fetch_or(STATUS_RUNTIME_IDENTITY, Ordering::AcqRel);
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

fn verify_runtime_identity(samp: Samp) -> Result<(), SampClientSdkResult> {
    if samp.status() != SampClientSdkHostStatus::Ready || !samp.probe().is_samp_loaded() {
        return Err(SampClientSdkResult::NotReady);
    }
    if samp.version()? != SampClientSdkClientVersion::R3_1 {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    let Some(base) = (unsafe { raw::base() }) else {
        return Err(SampClientSdkResult::NotReady);
    };
    if unsafe { module_entry_point_rva(base.as_ptr()) } != Some(R3_1_ENTRY_POINT_RVA) {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    Ok(())
}

/// Reads only the bounded DOS/PE headers from an already-loaded module.
///
/// The caller first verifies the host's ready state and version, then supplies
/// the opaque `samp.dll` base. No Rust reference is constructed for client memory.
unsafe fn module_entry_point_rva(base: *mut c_void) -> Option<u32> {
    let base = base.cast::<u8>() as usize;
    parse_pe_entry_point_rva(|offset, destination| {
        let Some(address) = base.checked_add(offset) else {
            return false;
        };
        unsafe {
            ptr::copy_nonoverlapping(
                address as *const u8,
                destination.as_mut_ptr(),
                destination.len(),
            )
        };
        true
    })
}

fn parse_pe_entry_point_rva(mut read: impl FnMut(usize, &mut [u8]) -> bool) -> Option<u32> {
    let mut dos = [0_u8; 64];
    if !read(0, &mut dos) || dos[..2] != *b"MZ" {
        return None;
    }
    let pe_offset = usize::try_from(u32::from_le_bytes(dos[0x3C..0x40].try_into().ok()?)).ok()?;
    if !(0x40..=MAX_PE_HEADER_OFFSET).contains(&pe_offset) {
        return None;
    }
    let mut header = [0_u8; 44];
    if !read(pe_offset, &mut header)
        || header[..4] != *b"PE\0\0"
        || header[24..26] != 0x10B_u16.to_le_bytes()
    {
        return None;
    }
    Some(u32::from_le_bytes(header[40..44].try_into().ok()?))
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
pub extern "system" fn SampClientSdkR3NetworkProbe_Shutdown() -> BOOL {
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

/// Returns the probe stage bitset; success before visual confirmation is `0x3F`.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR3NetworkProbe_Status() -> u32 {
    STATUS.load(Ordering::Acquire)
}

/// Returns the first failing `SampClientSdkResult` value, if any.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR3NetworkProbe_Failure() -> u32 {
    FAILURE.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_record_is_bounded_and_machine_readable() {
        assert_eq!(
            status_record(STATUS_HOST_CONNECTED | STATUS_OUTBOUND_RECEIPT, 0),
            "status=0x00000011\nfailure=0\n"
        );
    }

    #[test]
    fn parses_the_r3_entry_point_from_a_bounded_pe_header() {
        let mut image = vec![0_u8; 0x200];
        image[..2].copy_from_slice(b"MZ");
        image[0x3C..0x40].copy_from_slice(&(0x80_u32).to_le_bytes());
        image[0x80..0x84].copy_from_slice(b"PE\0\0");
        image[0x98..0x9A].copy_from_slice(&0x10B_u16.to_le_bytes());
        image[0xA8..0xAC].copy_from_slice(&R3_1_ENTRY_POINT_RVA.to_le_bytes());

        assert_eq!(
            parse_pe_entry_point_rva(|offset, destination| {
                let Some(source) = image.get(offset..offset.saturating_add(destination.len()))
                else {
                    return false;
                };
                destination.copy_from_slice(source);
                true
            }),
            Some(R3_1_ENTRY_POINT_RVA)
        );
    }
}
