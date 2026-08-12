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
const DIALOG_REQUEST_MARKER: &[u8] = b"R3_SDK_DIALOG_REQUEST_20260812";
const CHAT_INPUT_TEXT_MARKER: &[u8] = b"R3_SDK_TEXT_CACHE_20260812";
const INITIALIZATION_TIMEOUT: Duration = Duration::from_secs(45);
const SCALAR_CACHE_TIMEOUT: Duration = Duration::from_secs(45);
const CHAT_INPUT_CACHE_TIMEOUT: Duration = Duration::from_secs(60);
const SCOREBOARD_CACHE_TIMEOUT: Duration = Duration::from_secs(60);
const DIALOG_ACTIVE_CACHE_TIMEOUT: Duration = Duration::from_secs(15);
const INCOMING_READY_TIMEOUT: Duration = Duration::from_secs(60);
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(15);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const STATUS_FILE: &str = "samp-client-sdk-r3-network-probe.status";
const R3_1_ENTRY_POINT_RVA: u32 = 0x0CC4D0;
const R3_WAIT_JOIN_STATE: i32 = 6;
const MAX_PE_HEADER_OFFSET: usize = 0x1000;

/// The SDK host API resolved for this probe.
pub const STATUS_HOST_CONNECTED: u32 = 1 << 0;
/// Public host status, version, and the opaque module base identify the pinned R3-1 image.
pub const STATUS_RUNTIME_IDENTITY: u32 = 1 << 1;
/// The R3 CNetGame scalar cache reported the expected connected server values.
pub const STATUS_CNETGAME_SCALARS: u32 = 1 << 6;
/// The R3 cached local-player snapshot passed bounded sanity checks.
pub const STATUS_LOCAL_PLAYER_SNAPSHOT: u32 = 1 << 7;
/// The R3 cached player-pool count pair and largest ID matched the loopback session.
pub const STATUS_PLAYER_POOL_SCALARS: u32 = 1 << 12;
/// The R3 cached scoreboard flag observed both the open and closed states.
pub const STATUS_SCOREBOARD_CACHE: u32 = 1 << 13;
/// The R3 cached chat display mode returned a documented native value.
pub const STATUS_CHAT_DISPLAY_MODE: u32 = 1 << 14;
/// The R3 cached cursor mode returned a documented native value.
pub const STATUS_CURSOR_MODE: u32 = 1 << 15;
/// The R3 cached chat-input active flag and command names passed the live check.
pub const STATUS_CHAT_INPUT_CACHE: u32 = 1 << 8;
/// The R3 cached dialog active flag observed the disposable server dialog.
pub const STATUS_DIALOG_ACTIVE_CACHE: u32 = 1 << 9;
/// The loopback dialog-request command completed on the game thread.
pub const STATUS_DIALOG_REQUEST_RECEIPT: u32 = 1 << 10;
/// The R3 cached chat-input text matched the operator-entered marker.
pub const STATUS_CHAT_INPUT_TEXT_CACHE: u32 = 1 << 11;
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
static SCALAR_OBSERVATION: Mutex<Option<ScalarObservation>> = Mutex::new(None);
static PLAYER_POOL_OBSERVATION: Mutex<Option<PlayerPoolObservation>> = Mutex::new(None);

struct PluginState {
    subscription: Option<Subscription>,
    initializing: bool,
    shutting_down: bool,
}

/// A bounded copied snapshot emitted only by this opt-in local validation probe.
struct ScalarObservation {
    game_state: i32,
    address: Vec<u8>,
    hostname: Vec<u8>,
    port: u16,
}

/// The copied player-pool values observed only by this opt-in local probe.
struct PlayerPoolObservation {
    including_npcs: u16,
    excluding_npcs: u16,
    max_id: Option<u16>,
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
            *SCALAR_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            *PLAYER_POOL_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
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
    verify_cached_cnetgame_scalars(samp)?;
    STATUS.fetch_or(STATUS_CNETGAME_SCALARS, Ordering::AcqRel);
    publish_status();
    verify_cached_local_player(samp)?;
    STATUS.fetch_or(STATUS_LOCAL_PLAYER_SNAPSHOT, Ordering::AcqRel);
    publish_status();
    verify_cached_player_pool_scalars(samp)?;
    STATUS.fetch_or(STATUS_PLAYER_POOL_SCALARS, Ordering::AcqRel);
    publish_status();
    verify_cached_chat_display_mode(samp)?;
    STATUS.fetch_or(STATUS_CHAT_DISPLAY_MODE, Ordering::AcqRel);
    publish_status();
    verify_cached_cursor_mode(samp)?;
    STATUS.fetch_or(STATUS_CURSOR_MODE, Ordering::AcqRel);
    publish_status();

    let receipt = samp.net().send_chat(OUTBOUND_MARKER)?;
    wait_for_receipt(receipt)?;
    STATUS.fetch_or(STATUS_OUTBOUND_RECEIPT, Ordering::AcqRel);
    publish_status();

    wait_for_status(STATUS_REPLY_OBSERVED, CALLBACK_TIMEOUT)
        .and_then(|()| verify_cached_scoreboard_transition(samp))?;
    STATUS.fetch_or(STATUS_SCOREBOARD_CACHE, Ordering::AcqRel);
    publish_status();
    verify_cached_chat_input(samp)?;
    STATUS.fetch_or(STATUS_CHAT_INPUT_CACHE, Ordering::AcqRel);
    publish_status();
    verify_cached_chat_input_text(samp)?;
    STATUS.fetch_or(STATUS_CHAT_INPUT_TEXT_CACHE, Ordering::AcqRel);
    publish_status();
    let dialog_receipt = samp.net().send_chat(DIALOG_REQUEST_MARKER)?;
    wait_for_receipt(dialog_receipt)?;
    STATUS.fetch_or(STATUS_DIALOG_REQUEST_RECEIPT, Ordering::AcqRel);
    publish_status();
    verify_cached_dialog_active(samp)?;
    STATUS.fetch_or(STATUS_DIALOG_ACTIVE_CACHE, Ordering::AcqRel);
    publish_status();
    Ok(())
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

fn verify_cached_cnetgame_scalars(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match (samp.game_state(), samp.server().info()) {
            (Ok(game_state), Ok(info)) => {
                record_scalar_observation(game_state, &info);
                if game_state == R3_WAIT_JOIN_STATE
                    && info.address == b"127.0.0.1"
                    && info.hostname == b"SA-MP"
                    && info.port == 7777
                {
                    return Ok(());
                }
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            (Err(SampClientSdkResult::NotReady), _) | (_, Err(SampClientSdkResult::NotReady)) => {}
            (Err(error), _) | (_, Err(error)) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn record_scalar_observation(game_state: i32, info: &samp_client_sdk::ServerInfo) {
    *SCALAR_OBSERVATION
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(ScalarObservation {
        game_state,
        address: info.address.clone(),
        hostname: info.hostname.clone(),
        port: info.port,
    });
}

fn verify_cached_local_player(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.local().player() {
            Ok(player) => {
                if !local_player_snapshot_is_valid(&player) {
                    return Err(SampClientSdkResult::NativeCallFailed);
                }
                if player.spawned {
                    return Ok(());
                }
            }
            Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn local_player_snapshot_is_valid(player: &samp_client_sdk::LocalPlayer) -> bool {
    player.id < 1004
        && !player.nickname.is_empty()
        && player.nickname.len() <= 256
        && player.health.is_finite()
        && player.armour.is_finite()
        && player.position.x.is_finite()
        && player.position.y.is_finite()
        && player.position.z.is_finite()
        && player.velocity.x.is_finite()
        && player.velocity.y.is_finite()
        && player.velocity.z.is_finite()
        && player.vehicle_id.is_none_or(|id| id < 2000)
}

fn verify_cached_player_pool_scalars(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match (
            samp.players().count(true),
            samp.players().count(false),
            samp.players().max_id(),
        ) {
            (Ok(including_npcs), Ok(excluding_npcs), Ok(max_id)) => {
                let max_id = max_id.map(|id| id.get());
                *PLAYER_POOL_OBSERVATION
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()) = Some(PlayerPoolObservation {
                    including_npcs,
                    excluding_npcs,
                    max_id,
                });
                if (including_npcs, excluding_npcs, max_id) == (0, 0, Some(0)) {
                    return Ok(());
                }
                return Err(SampClientSdkResult::NativeCallFailed);
            }
            (Err(SampClientSdkResult::NotReady), _, _)
            | (_, Err(SampClientSdkResult::NotReady), _)
            | (_, _, Err(SampClientSdkResult::NotReady)) => {}
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_cached_chat_display_mode(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.chat().display_mode() {
            Ok(_) => return Ok(()),
            Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_cached_cursor_mode(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.cursor().mode() {
            Ok(_) => return Ok(()),
            Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_cached_chat_input(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + CHAT_INPUT_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let input = samp.chat_input();
        match (
            input.is_active(),
            input.is_command_defined(b"quit"),
            input.is_command_defined(b"r3_sdk_probe_missing_command"),
        ) {
            (Ok(true), Ok(true), Ok(false)) => return Ok(()),
            (Ok(_), Ok(_), Ok(false)) => {}
            (Err(SampClientSdkResult::NotReady), _, _)
            | (_, Err(SampClientSdkResult::NotReady), _)
            | (_, _, Err(SampClientSdkResult::NotReady)) => {}
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => return Err(error),
            _ => return Err(SampClientSdkResult::NativeCallFailed),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_cached_chat_input_text(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + CHAT_INPUT_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.chat_input().text() {
            Ok(text) if text == CHAT_INPUT_TEXT_MARKER => return Ok(()),
            Ok(_) | Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_cached_scoreboard_transition(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCOREBOARD_CACHE_TIMEOUT;
    let mut observed_open = false;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.scoreboard().is_open() {
            Ok(true) => observed_open = true,
            Ok(false) if observed_open => return Ok(()),
            Ok(false) | Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_cached_dialog_active(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + DIALOG_ACTIVE_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match samp.dialogs().is_active() {
            Ok(true) => return Ok(()),
            Ok(false) | Err(SampClientSdkResult::NotReady) => {}
            Err(error) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
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
            SCALAR_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref(),
            PLAYER_POOL_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref(),
        ),
    );
}

fn status_record(
    status: u32,
    failure: u32,
    scalar: Option<&ScalarObservation>,
    player_pool: Option<&PlayerPoolObservation>,
) -> String {
    let mut record = format!("status=0x{status:08X}\nfailure={failure}\n");
    if let Some(scalar) = scalar {
        use std::fmt::Write;

        let _ = writeln!(record, "game_state={}", scalar.game_state);
        let _ = writeln!(record, "address_hex={}", hex(&scalar.address));
        let _ = writeln!(record, "hostname_hex={}", hex(&scalar.hostname));
        let _ = writeln!(record, "port={}", scalar.port);
    }
    if let Some(player_pool) = player_pool {
        use std::fmt::Write;

        let _ = writeln!(
            record,
            "player_count_including_npcs={}",
            player_pool.including_npcs
        );
        let _ = writeln!(
            record,
            "player_count_excluding_npcs={}",
            player_pool.excluding_npcs
        );
        let _ = writeln!(record, "player_max_id={:?}", player_pool.max_id);
    }
    record
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
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

/// Returns the probe stage bitset; success after all cache and dialog checks is `0x7FFF`.
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
            status_record(
                STATUS_HOST_CONNECTED | STATUS_OUTBOUND_RECEIPT,
                0,
                None,
                None
            ),
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

    #[test]
    fn valid_unspawned_local_player_remains_retryable() {
        let player = samp_client_sdk::LocalPlayer {
            id: 0,
            nickname: b"R3 probe".to_vec(),
            colour: 0,
            spawned: false,
            health: 100.0,
            armour: 0.0,
            position: samp_client_sdk::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            velocity: samp_client_sdk::Vector3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            special_action: 0,
            animation_id: 0,
            vehicle_id: None,
            score: 0,
            ping: 0,
        };

        assert!(local_player_snapshot_is_valid(&player));
        assert!(!player.spawned);
    }
}
