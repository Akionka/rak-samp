//! Isolated live smoke checks for native network paths.
//!
//! This test plugin never sends traffic to the server. It round-trips a fixed
//! string through SA-MP's native codec, then injects fixed three-bit packet and
//! RPC payloads which its own listeners block before SA-MP can consume them.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp_client_sdk_network_smoke_plugin supports only 32-bit Windows x86 targets");

use samp_client_sdk::{
    CommandReceipt, Samp, SampClientSdkDirection, SampClientSdkHookAction, SampClientSdkResult,
    SubscriptionSet,
};
use samp_protocol::BitStream;
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

const TEST_ID: u8 = 0xFE;
const TEST_PAYLOAD: [u8; 1] = [0b1010_0000];
const TEST_BITS: usize = 3;
const CODEC_VALUE: &[u8] = b"samp-client-sdk-network-smoke";
const INITIALIZATION_TIMEOUT: Duration = Duration::from_secs(30);
const INCOMING_EMULATION_READY_TIMEOUT: Duration = Duration::from_secs(45);
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(5);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const STATUS_FILE: &str = "samp-client-sdk-network-smoke.status";

/// The host ABI was resolved and the smoke worker started.
pub const STATUS_HOST_CONNECTED: u32 = 1 << 0;
/// Both self-blocking incoming listeners were registered.
pub const STATUS_LISTENERS_REGISTERED: u32 = 1 << 1;
/// The native string compressor encoded and decoded the fixed test string.
pub const STATUS_CODEC_ROUND_TRIP: u32 = 1 << 2;
/// The packet emulation command completed after native allocation and queueing.
pub const STATUS_PACKET_QUEUED: u32 = 1 << 3;
/// The three-bit packet payload reached, and was blocked by, this plugin's listener.
pub const STATUS_PACKET_EXACT_BITS: u32 = 1 << 4;
/// The RPC emulation command completed after dispatching its listener.
pub const STATUS_RPC_QUEUED: u32 = 1 << 5;
/// The three-bit RPC payload reached, and was blocked by, this plugin's listener.
pub const STATUS_RPC_EXACT_BITS: u32 = 1 << 6;
/// A registration, codec, queue, or exact-bit assertion failed.
pub const STATUS_FAILED: u32 = 1 << 31;

static STATE: Mutex<PluginState> = Mutex::new(PluginState::new());
static INITIALIZATION_FINISHED: Condvar = Condvar::new();
static STATUS: AtomicU32 = AtomicU32::new(0);
static FAILURE: AtomicU32 = AtomicU32::new(SampClientSdkResult::Ok as u32);

struct PluginState {
    subscriptions: Option<SubscriptionSet>,
    initializing: bool,
    shutting_down: bool,
}

impl PluginState {
    const fn new() -> Self {
        Self {
            subscriptions: None,
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
            if std::thread::Builder::new()
                .name("samp-client-sdk-network-smoke-init".into())
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

    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    if state.shutting_down {
        return;
    }
    match register_blocking_listeners(samp) {
        Ok(subscriptions) => state.subscriptions = Some(subscriptions),
        Err((error, subscriptions)) => {
            state.subscriptions = (!subscriptions.is_empty()).then_some(subscriptions);
            drop(state);
            record_failure(error);
            publish_status();
            announce(
                samp,
                b"[samp-client-sdk] network smoke failed during listener setup",
            );
            return;
        }
    }
    drop(state);
    STATUS.fetch_or(STATUS_LISTENERS_REGISTERED, Ordering::AcqRel);
    publish_status();

    let result = run_smoke(samp);
    if let Err(error) = result {
        record_failure(error);
        publish_status();
        announce(
            samp,
            b"[samp-client-sdk] network smoke failed; inspect its exported status",
        );
    } else {
        publish_status();
        announce(
            samp,
            b"[samp-client-sdk] network smoke passed (codec and blocked exact-bit emulation)",
        );
    }
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

fn register_blocking_listeners(
    samp: Samp,
) -> Result<SubscriptionSet, (SampClientSdkResult, SubscriptionSet)> {
    let mut subscriptions = SubscriptionSet::new();
    let packet = samp
        .net()
        .on_packet_id(SampClientSdkDirection::Incoming, TEST_ID, |event| {
            if exact_payload(event) {
                STATUS.fetch_or(STATUS_PACKET_EXACT_BITS, Ordering::AcqRel);
            } else {
                record_failure(SampClientSdkResult::NativeCallFailed);
            }
            // This plugin must never deliver a synthetic or colliding test packet to SA-MP.
            SampClientSdkHookAction::Block
        });
    match packet {
        Ok(subscription) => subscriptions.push(subscription),
        Err(error) => return Err((error, subscriptions)),
    }

    let rpc = samp
        .net()
        .on_rpc_id(SampClientSdkDirection::Incoming, TEST_ID, |event| {
            if exact_payload(event) {
                STATUS.fetch_or(STATUS_RPC_EXACT_BITS, Ordering::AcqRel);
            } else {
                record_failure(SampClientSdkResult::NativeCallFailed);
            }
            // The original incoming-RPC handler is intentionally not called by this smoke test.
            SampClientSdkHookAction::Block
        });
    match rpc {
        Ok(subscription) => {
            subscriptions.push(subscription);
            Ok(subscriptions)
        }
        Err(error) => Err((error, subscriptions)),
    }
}

fn exact_payload(event: &mut samp_client_sdk::events::Event<'_>) -> bool {
    event.remaining_bits() == TEST_BITS
        && matches!(event.read_bits(TEST_BITS), Ok(payload) if payload == TEST_PAYLOAD)
        && event.remaining_bits() == 0
}

fn run_smoke(samp: Samp) -> Result<(), SampClientSdkResult> {
    retry_codec_readiness(|| native_codec_round_trip(samp))?;
    STATUS.fetch_or(STATUS_CODEC_ROUND_TRIP, Ordering::AcqRel);
    publish_status();

    wait_for_incoming_emulation_ready(samp)?;
    emulate_packet(samp)?;
    STATUS.fetch_or(STATUS_PACKET_QUEUED, Ordering::AcqRel);
    wait_for_status(STATUS_PACKET_EXACT_BITS, CALLBACK_TIMEOUT)?;
    publish_status();

    emulate_rpc(samp)?;
    STATUS.fetch_or(STATUS_RPC_QUEUED, Ordering::AcqRel);
    wait_for_status(STATUS_RPC_EXACT_BITS, CALLBACK_TIMEOUT)?;
    publish_status();
    Ok(())
}

fn native_codec_round_trip(samp: Samp) -> Result<(), SampClientSdkResult> {
    let encoded = samp.net().encode_string(CODEC_VALUE)?;
    let mut encoded_stream = BitStream::from_bits(encoded.as_bytes().to_vec(), encoded.len_bits())
        .map_err(|_| SampClientSdkResult::NativeCallFailed)?;
    let decoded = samp.net().decode_string(&mut encoded_stream)?;
    if decoded == CODEC_VALUE && encoded_stream.remaining_bits() == 0 {
        Ok(())
    } else {
        Err(SampClientSdkResult::NativeCallFailed)
    }
}

fn emulate_packet(samp: Samp) -> Result<(), SampClientSdkResult> {
    let receipt = samp
        .net()
        .emulate_incoming_packet(TEST_ID, &TEST_PAYLOAD, TEST_BITS)?;
    wait_for_receipt(receipt)
}

fn emulate_rpc(samp: Samp) -> Result<(), SampClientSdkResult> {
    let receipt = samp
        .net()
        .emulate_incoming_rpc(TEST_ID, &TEST_PAYLOAD, TEST_BITS)?;
    wait_for_receipt(receipt)
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

fn retry_codec_readiness(
    operation: impl Fn() -> Result<(), SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + INITIALIZATION_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match operation() {
            Ok(()) => return Ok(()),
            Err(SampClientSdkResult::NotReady | SampClientSdkResult::Busy)
                if !deadline.saturating_duration_since(Instant::now()).is_zero() =>
            {
                thread::sleep(RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }
}

fn wait_for_incoming_emulation_ready(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + INCOMING_EMULATION_READY_TIMEOUT;
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
        value if value == SampClientSdkResult::InvalidArgument as u32 => {
            SampClientSdkResult::InvalidArgument
        }
        value if value == SampClientSdkResult::UnsupportedVersion as u32 => {
            SampClientSdkResult::UnsupportedVersion
        }
        value if value == SampClientSdkResult::PayloadTooLarge as u32 => {
            SampClientSdkResult::PayloadTooLarge
        }
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

fn announce(samp: Samp, message: &[u8]) {
    if samp.probe().is_sampfuncs_loaded() {
        let _ = samp.sampfuncs().log_console(message);
    }
}

/// Stops callbacks before an unload manager calls `FreeLibrary`.
///
/// This must run on a worker thread, not from `DllMain` or a samp-client-sdk callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkNetworkSmoke_Shutdown() -> BOOL {
    let subscriptions = {
        let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
        state.shutting_down = true;
        while state.initializing {
            state = INITIALIZATION_FINISHED
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
        state.subscriptions.take()
    };

    let Some(subscriptions) = subscriptions else {
        return TRUE;
    };
    match subscriptions.unregister_and_wait() {
        Ok(()) => TRUE,
        Err(error) => {
            STATE
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .subscriptions = Some(error.into_subscriptions());
            0
        }
    }
}

/// Returns a bitset describing the completed smoke stages.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkNetworkSmoke_Status() -> u32 {
    STATUS.load(Ordering::Acquire)
}

/// Returns the first `SampClientSdkResult` value recorded by a failing stage.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkNetworkSmoke_Failure() -> u32 {
    FAILURE.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_record_is_bounded_and_machine_readable() {
        assert_eq!(
            status_record(STATUS_HOST_CONNECTED | STATUS_CODEC_ROUND_TRIP, 7),
            "status=0x00000005\nfailure=7\n"
        );
    }
}
