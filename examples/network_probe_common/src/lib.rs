//! Loopback-only version-selected network delivery validation.
//!
//! This probe sends one bounded chat marker only to the disposable local server
//! filter. Its matching server-message listener always continues, allowing a
//! human to verify that SA-MP's original incoming-RPC handler displayed reply.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("samp-client-sdk network probes support only 32-bit Windows x86 targets");

use samp_client_sdk::{
    CommandReceipt, GangzoneId, LocalChatDisplayMode, LocalChatMessage, LocalChatMessageStyle,
    LocalCursorMode, LocalDeathMessage, LocalDialog, LocalDialogStyle, ObjectId, PlayerId, Samp,
    SampClientSdkClientVersion, SampClientSdkDirection, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkResult, SendRateKind, SpecialAction, SubscriptionSet,
    TextdrawId, Vector3, VehicleId, events::RpcAction, raw,
};
#[cfg(feature = "r1-probe")]
use samp_protocol::BitStream;
use samp_protocol::{
    WireDescriptor,
    packet::common::{
        SendAimSync, SendPassengerSync, SendPlayerSync, SendStatsUpdate, SendTrailerSync,
        SendUnoccupiedSync, SendVehicleSync, SendWeaponsUpdate,
    },
    rpc::incoming::SERVER_MESSAGE,
};
use std::{
    ffi::c_void,
    fs, ptr,
    sync::{
        Condvar, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
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

#[cfg(feature = "r1-probe")]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $r1
    };
}

#[cfg(feature = "r3-probe")]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $r3
    };
}

#[cfg(feature = "dl-probe")]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $dl
    };
}

#[cfg(not(any(feature = "r1-probe", feature = "r3-probe", feature = "dl-probe")))]
macro_rules! profile_value {
    ($r5:expr, $r1:expr, $r3:expr, $dl:expr) => {
        $r5
    };
}

const OUTBOUND_MARKER: &[u8] = profile_value!(
    b"R5_SDK_OUTBOUND_20260812",
    b"R1_SDK_OUTBOUND_20260816",
    b"R3_SDK_OUTBOUND_20260812",
    b"DL_SDK_OUTBOUND_20260812"
);
const INCOMING_MARKER: &[u8] = profile_value!(
    b"R5_SDK_INCOMING_20260812",
    b"R1_SDK_INCOMING_20260816",
    b"R3_SDK_INCOMING_20260812",
    b"DL_SDK_INCOMING_20260812"
);
const DIALOG_REQUEST_MARKER: &[u8] = profile_value!(
    b"R5_SDK_DIALOG_REQUEST_20260812",
    b"R1_SDK_DIALOG_REQUEST_20260816",
    b"R3_SDK_DIALOG_REQUEST_20260812",
    b"DL_SDK_DIALOG_REQUEST_20260812"
);
const ENTITY_REQUEST_MARKER: &[u8] = profile_value!(
    b"R5_SDK_ENTITY_REQUEST_20260813",
    b"R1_SDK_ENTITY_REQUEST_20260816",
    b"R3_SDK_ENTITY_REQUEST_20260813",
    b"DL_SDK_ENTITY_REQUEST_20260813"
);
const ENTITY_IDS_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_ENTITY_IDS_",
    b"R1_SDK_ENTITY_IDS_",
    b"R3_SDK_ENTITY_IDS_",
    b"DL_SDK_ENTITY_IDS_"
);
const CHAT_INPUT_TEXT_MARKER: &[u8] = profile_value!(
    b"R5_SDK_TEXT_CACHE_20260812",
    b"R1_SDK_TEXT_CACHE_20260816",
    b"R3_SDK_TEXT_CACHE_20260812",
    b"DL_SDK_TEXT_CACHE_20260812"
);
const LOCAL_CHAT_MARKER: &[u8] = profile_value!(
    b"R5 SDK full UI validation",
    b"R1 SDK full UI validation",
    b"R3 SDK full UI validation",
    b"DL SDK full UI validation"
);
const LOCAL_CHAT_PREFIX: &[u8] = profile_value!(b"R5 SDK", b"R1 SDK", b"R3 SDK", b"DL SDK");
const LOCAL_COMMAND_NAME: &[u8] =
    profile_value!(b"r5sdkprobe", b"r1sdkprobe", b"r3sdkprobe", b"dlsdkprobe");
const MISSING_COMMAND_NAME: &[u8] = profile_value!(
    b"r5_sdk_probe_missing_command",
    b"r1_sdk_probe_missing_command",
    b"r3_sdk_probe_missing_command",
    b"dl_sdk_probe_missing_command"
);
const LOCAL_COMMAND_TEXT: &[u8] = profile_value!(
    b"/r5sdkprobe consolidated",
    b"/r1sdkprobe consolidated",
    b"/r3sdkprobe consolidated",
    b"/dlsdkprobe consolidated"
);
const LOCAL_DIALOG_INPUT_TEXT: &[u8] = profile_value!(
    b"R5_INPUT_UPDATED",
    b"R1_INPUT_UPDATED",
    b"R3_INPUT_UPDATED",
    b"DL_INPUT_UPDATED"
);
const LOCAL_CHAT_INPUT_MUTATION: &[u8] = profile_value!(
    b"R5_SDK_MUTATION",
    b"R1_SDK_MUTATION",
    b"R3_SDK_MUTATION",
    b"DL_SDK_MUTATION"
);
const LOCAL_INPUT_DIALOG_TITLE: &[u8] = profile_value!(
    b"R5 input dialog",
    b"R1 input dialog",
    b"R3 input dialog",
    b"DL input dialog"
);
const LOCAL_INPUT_DIALOG_BODY: &[u8] = profile_value!(
    b"R5 input body",
    b"R1 input body",
    b"R3 input body",
    b"DL input body"
);
const LOCAL_LIST_DIALOG_TITLE: &[u8] = profile_value!(
    b"R5 list dialog",
    b"R1 list dialog",
    b"R3 list dialog",
    b"DL list dialog"
);
const LOCAL_LABEL_TEXT: &[u8] = profile_value!(
    b"R5 label validation",
    b"R1 label validation",
    b"R3 label validation",
    b"DL label validation"
);
const LOCAL_LABEL_UPDATED_TEXT: &[u8] = profile_value!(
    b"R5 label updated",
    b"R1 label updated",
    b"R3 label updated",
    b"DL label updated"
);
const LOCAL_TEXTDRAW_TEXT: &[u8] = profile_value!(
    b"R5 textdraw validation",
    b"R1 textdraw validation",
    b"R3 textdraw validation",
    b"DL textdraw validation"
);
const LOCAL_TEXTDRAW_UPDATED_TEXT: &[u8] = profile_value!(
    b"R5 textdraw updated",
    b"R1 textdraw updated",
    b"R3 textdraw updated",
    b"DL textdraw updated"
);
const LOCAL_DRIVER_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_DRIVER_REQUEST",
    b"R1_SDK_LOCAL_DRIVER_REQUEST",
    b"R3_SDK_LOCAL_DRIVER_REQUEST",
    b"DL_SDK_LOCAL_DRIVER_REQUEST"
);
const LOCAL_PASSENGER_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_PASSENGER_REQUEST",
    b"R1_SDK_LOCAL_PASSENGER_REQUEST",
    b"R3_SDK_LOCAL_PASSENGER_REQUEST",
    b"DL_SDK_LOCAL_PASSENGER_REQUEST"
);
const LOCAL_TRAILER_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_TRAILER_REQUEST",
    b"R1_SDK_LOCAL_TRAILER_REQUEST",
    b"R3_SDK_LOCAL_TRAILER_REQUEST",
    b"DL_SDK_LOCAL_TRAILER_REQUEST"
);
const VEHICLE_CLEANUP_REQUEST: &[u8] = profile_value!(
    b"R5_SDK_VEHICLE_CLEANUP",
    b"R1_SDK_VEHICLE_CLEANUP",
    b"R3_SDK_VEHICLE_CLEANUP",
    b"DL_SDK_VEHICLE_CLEANUP"
);
const RECONNECT_COMMAND_NAME: &[u8] = profile_value!(
    b"r5sdkreconnect",
    b"r1sdkreconnect",
    b"r3sdkreconnect",
    b"dlsdkreconnect"
);
const MAIN_PASS_MESSAGE: &[u8] = profile_value!(
    b"R5 main pass complete. Type /r5sdkreconnect for the final lifecycle pass.",
    b"R1 main pass complete. Type /r1sdkreconnect for the final lifecycle pass.",
    b"R3 main pass complete. Type /r3sdkreconnect for the final lifecycle pass.",
    b"DL main pass complete. Type /dlsdkreconnect for the final lifecycle pass."
);
const HOST_CONNECTION_TIMEOUT: Duration = Duration::from_secs(5 * 60);
const INITIALIZATION_TIMEOUT: Duration = Duration::from_secs(45);
const SCALAR_CACHE_TIMEOUT: Duration = Duration::from_secs(45);
const CHAT_INPUT_CACHE_TIMEOUT: Duration = Duration::from_secs(60);
const SCOREBOARD_CACHE_TIMEOUT: Duration = Duration::from_secs(60);
const DIALOG_ACTIVE_CACHE_TIMEOUT: Duration = Duration::from_secs(15);
const INCOMING_READY_TIMEOUT: Duration = Duration::from_secs(60);
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(15);
const RETRY_DELAY: Duration = Duration::from_millis(100);
const STATUS_FILE: &str = profile_value!(
    "samp-client-sdk-r5-network-probe.status",
    "samp-client-sdk-r1-network-probe.status",
    "samp-client-sdk-r3-network-probe.status",
    "samp-client-sdk-dl-network-probe.status"
);
const PROFILE_ENTRY_POINT_RVA: u32 = profile_value!(0x0CBC90, 0x31DF13, 0x0CC4D0, 0x0FDB60);
/// State observed after the first incoming RPC, before the local player is spawned.
const PROFILE_INITIAL_GAME_STATE: i32 = profile_value!(5, 14, 15, 5);
/// State required after a reconnect has spawned the local player.
const PROFILE_CONNECTED_STATE: i32 = profile_value!(5, 14, 14, 5);
const PROFILE_SERVER_HOSTNAME: &[u8] = profile_value!(
    b"SDK R5 loopback probe",
    b"SDK R1 loopback probe",
    b"SA-MP",
    b"SDK DL loopback probe"
);
const PROFILE_CLIENT_VERSION: SampClientSdkClientVersion = profile_value!(
    SampClientSdkClientVersion::R5_1,
    SampClientSdkClientVersion::R1,
    SampClientSdkClientVersion::R3_1,
    SampClientSdkClientVersion::Dl
);
const LOCAL_DRIVER_READY_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_DRIVER_READY_",
    b"R1_SDK_LOCAL_DRIVER_READY_",
    b"R3_SDK_LOCAL_DRIVER_READY_",
    b"DL_SDK_LOCAL_DRIVER_READY_"
);
const LOCAL_PASSENGER_READY_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_PASSENGER_READY_",
    b"R1_SDK_LOCAL_PASSENGER_READY_",
    b"R3_SDK_LOCAL_PASSENGER_READY_",
    b"DL_SDK_LOCAL_PASSENGER_READY_"
);
const LOCAL_TRAILER_READY_PREFIX: &[u8] = profile_value!(
    b"R5_SDK_LOCAL_TRAILER_READY_",
    b"R1_SDK_LOCAL_TRAILER_READY_",
    b"R3_SDK_LOCAL_TRAILER_READY_",
    b"DL_SDK_LOCAL_TRAILER_READY_"
);
const VEHICLE_CLEANUP_READY_MARKER: &[u8] = profile_value!(
    b"R5_SDK_VEHICLE_CLEANUP_READY",
    b"R1_SDK_VEHICLE_CLEANUP_READY",
    b"R3_SDK_VEHICLE_CLEANUP_READY",
    b"DL_SDK_VEHICLE_CLEANUP_READY"
);
const MAX_PE_HEADER_OFFSET: usize = 0x1000;

/// The SDK host API resolved for this probe.
pub const STATUS_HOST_CONNECTED: u32 = 1 << 0;
/// Public host status, version, and the opaque module base identify the pinned profile image.
pub const STATUS_RUNTIME_IDENTITY: u32 = 1 << 1;
/// The selected profile's CNetGame scalar cache reported the expected connected server values.
pub const STATUS_CNETGAME_SCALARS: u32 = 1 << 6;
/// The selected profile's cached local-player snapshot passed bounded sanity checks.
pub const STATUS_LOCAL_PLAYER_SNAPSHOT: u32 = 1 << 7;
/// The selected profile's cached player-pool count pair and largest ID matched the loopback session.
pub const STATUS_PLAYER_POOL_SCALARS: u32 = 1 << 12;
/// The selected profile's cached scoreboard flag observed both the open and closed states.
pub const STATUS_SCOREBOARD_CACHE: u32 = 1 << 13;
/// The selected profile's cached chat display mode returned a documented native value.
pub const STATUS_CHAT_DISPLAY_MODE: u32 = 1 << 14;
/// The selected profile's cached cursor mode returned a documented native value.
pub const STATUS_CURSOR_MODE: u32 = 1 << 15;
/// The selected profile's cached remote-player directory observed another loopback client.
pub const STATUS_REMOTE_PLAYER_DIRECTORY: u32 = 1 << 16;
/// Object, pickup, vehicle, gangzone, and player-ped handle paths round-tripped.
pub const STATUS_ENTITY_HANDLES: u32 = 1 << 17;
/// Native local-player force-sync methods completed on the game thread.
pub const STATUS_FORCE_SYNC_RECEIPTS: u32 = 1 << 18;
/// Chat, input, cursor, scoreboard, and death-window mutations completed.
pub const STATUS_UI_MUTATIONS: u32 = 1 << 19;
/// Client-side input/list dialogs and the close-response hook passed.
pub const STATUS_DIALOG_LIFECYCLE: u32 = 1 << 20;
/// A native chat command was installed, invoked, and removed synchronously.
pub const STATUS_CHAT_COMMAND_LIFECYCLE: u32 = 1 << 21;
/// The animation table round-tripped one name/file pair.
pub const STATUS_ANIMATION_TABLE: u32 = 1 << 22;
/// Local-player colour/action and all send-rate writes completed and restored.
pub const STATUS_LOCAL_MUTATIONS: u32 = 1 << 23;
/// An automatically allocated text label completed create/read/write/delete.
pub const STATUS_TEXT_LABEL_LIFECYCLE: u32 = 1 << 24;
/// A free R5 textdraw slot completed create/read/write/delete.
pub const STATUS_TEXTDRAW_LIFECYCLE: u32 = 1 << 25;
/// Local on-foot/aim and remote NPC on-foot snapshots passed bounded checks.
pub const STATUS_SYNC_SNAPSHOTS: u32 = 1 << 26;
/// Controlled local vehicle states and all vehicle force-sync packets passed.
pub const STATUS_VEHICLE_SYNC: u32 = 1 << 27;
/// Disconnect invalidated the R5 connection-bound caches.
pub const STATUS_DISCONNECT_INVALIDATION: u32 = 1 << 28;
/// Reconnect restored R5 caches and post-reconnect packet/RPC delivery.
pub const STATUS_RECONNECT_RESTORED: u32 = 1 << 29;
/// Complete connected-session validation before the opt-in reconnect phase.
pub const MAIN_SUCCESS_STATUS: u32 = 0x0FFF_FFFF;
/// Complete validation including disconnect invalidation and reconnect restoration.
pub const FULL_SUCCESS_STATUS: u32 = 0x3FFF_FFFF;
/// The R5 cached chat-input active flag and command names passed the live check.
pub const STATUS_CHAT_INPUT_CACHE: u32 = 1 << 8;
/// The R5 cached dialog active flag observed the disposable server dialog.
pub const STATUS_DIALOG_ACTIVE_CACHE: u32 = 1 << 9;
/// The loopback dialog-request command completed on the game thread.
pub const STATUS_DIALOG_REQUEST_RECEIPT: u32 = 1 << 10;
/// The cached chat-input text matched the operator-entered marker.
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
static RECONNECT_OBSERVATION: Mutex<Option<ReconnectObservation>> = Mutex::new(None);
static ENTITY_IDS: Mutex<Option<EntityIds>> = Mutex::new(None);
static CHAT_COMMAND_INVOKED: AtomicBool = AtomicBool::new(false);
static SYNC_PACKETS_OBSERVED: AtomicU32 = AtomicU32::new(0);
static SYNC_PACKET_COUNTS: [AtomicU32; 8] = [const { AtomicU32::new(0) }; 8];
static TEXT_LABEL_PHASE: Mutex<&'static str> = Mutex::new("none");
static TEXT_LABEL_INITIAL_FIELDS: AtomicU32 = AtomicU32::new(0);
static TEXT_LABEL_INITIAL_RESULT: AtomicU32 = AtomicU32::new(0);
static TEXTDRAW_PHASE: Mutex<&'static str> = Mutex::new("none");
static TEXTDRAW_SNAPSHOT_FIELDS: AtomicU32 = AtomicU32::new(0);
static TEXTDRAW_SNAPSHOT_RESULT: AtomicU32 = AtomicU32::new(0);
static VEHICLE_PHASE: Mutex<&'static str> = Mutex::new("none");
static VEHICLE_PHASES: Mutex<VehiclePhases> = Mutex::new(VehiclePhases::new());
static RECONNECT_REQUESTED: AtomicBool = AtomicBool::new(false);
static INCOMING_REPLY_COUNT: AtomicU32 = AtomicU32::new(0);

const SYNC_PACKET_AIM: u32 = 1 << 0;
const SYNC_PACKET_ONFOOT: u32 = 1 << 1;
const SYNC_PACKET_STATS: u32 = 1 << 2;
const SYNC_PACKET_WEAPONS: u32 = 1 << 3;
const SYNC_PACKET_VEHICLE: u32 = 1 << 4;
const SYNC_PACKET_PASSENGER: u32 = 1 << 5;
const SYNC_PACKET_UNOCCUPIED: u32 = 1 << 6;
const SYNC_PACKET_TRAILER: u32 = 1 << 7;
const SYNC_INDEX_AIM: usize = 0;
const SYNC_INDEX_ONFOOT: usize = 1;
const SYNC_INDEX_STATS: usize = 2;
const SYNC_INDEX_WEAPONS: usize = 3;
const SYNC_INDEX_VEHICLE: usize = 4;
const SYNC_INDEX_PASSENGER: usize = 5;
const SYNC_INDEX_UNOCCUPIED: usize = 6;
const SYNC_INDEX_TRAILER: usize = 7;

#[cfg(feature = "r1-probe")]
const R1_EXACT_BIT_TEST_ID: u8 = 0xFE;
#[cfg(feature = "r1-probe")]
const R1_EXACT_BIT_PAYLOAD: [u8; 1] = [0b1010_0000];
#[cfg(feature = "r1-probe")]
const R1_EXACT_BIT_COUNT: usize = 3;
#[cfg(feature = "r1-probe")]
const R1_CODEC_VALUE: &[u8] = b"samp-client-sdk-r1-network-probe";
#[cfg(feature = "r1-probe")]
static R1_CODEC_ROUND_TRIP: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "r1-probe")]
static R1_PACKET_EXACT_BITS: AtomicBool = AtomicBool::new(false);
#[cfg(feature = "r1-probe")]
static R1_RPC_EXACT_BITS: AtomicBool = AtomicBool::new(false);

struct PluginState {
    subscriptions: Option<SubscriptionSet>,
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

#[derive(Clone, Copy, Eq, PartialEq)]
struct ReconnectObservation {
    server_ready: bool,
    local_ready: bool,
    game_state: Option<i32>,
    spawned: Option<bool>,
    incoming_ready: bool,
}

#[derive(Clone, Copy)]
struct ProbePhaseStatus {
    text_label_phase: &'static str,
    text_label_initial_fields: u32,
    text_label_initial_result: u32,
    textdraw_phase: &'static str,
    textdraw_snapshot_fields: u32,
    textdraw_snapshot_result: u32,
    vehicle_phase: &'static str,
}

#[derive(Clone, Copy)]
struct EntityIds {
    object: u16,
    vehicle: u16,
    pickup: u16,
    gangzone: u16,
}

#[derive(Clone, Copy)]
struct VehiclePair {
    vehicle: u16,
    trailer: u16,
}

struct VehiclePhases {
    local_driver: Option<u16>,
    local_passenger: Option<u16>,
    local_trailer: Option<VehiclePair>,
    cleanup: bool,
}

impl VehiclePhases {
    const fn new() -> Self {
        Self {
            local_driver: None,
            local_passenger: None,
            local_trailer: None,
            cleanup: false,
        }
    }
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
            *SCALAR_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            *PLAYER_POOL_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            *RECONNECT_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = None;
            CHAT_COMMAND_INVOKED.store(false, Ordering::Release);
            SYNC_PACKETS_OBSERVED.store(0, Ordering::Release);
            for count in &SYNC_PACKET_COUNTS {
                count.store(0, Ordering::Release);
            }
            *TEXT_LABEL_PHASE
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = "none";
            TEXT_LABEL_INITIAL_FIELDS.store(0, Ordering::Release);
            TEXT_LABEL_INITIAL_RESULT.store(0, Ordering::Release);
            *TEXTDRAW_PHASE
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = "none";
            TEXTDRAW_SNAPSHOT_FIELDS.store(0, Ordering::Release);
            TEXTDRAW_SNAPSHOT_RESULT.store(0, Ordering::Release);
            *VEHICLE_PHASE
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = "none";
            RECONNECT_REQUESTED.store(false, Ordering::Release);
            INCOMING_REPLY_COUNT.store(0, Ordering::Release);
            #[cfg(feature = "r1-probe")]
            {
                R1_CODEC_ROUND_TRIP.store(false, Ordering::Release);
                R1_PACKET_EXACT_BITS.store(false, Ordering::Release);
                R1_RPC_EXACT_BITS.store(false, Ordering::Release);
            }
            *VEHICLE_PHASES
                .lock()
                .unwrap_or_else(|error| error.into_inner()) = VehiclePhases::new();
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
    if let Err(error) = verify_runtime_identity(samp) {
        record_failure(error);
        publish_status();
        return;
    }
    STATUS.fetch_or(STATUS_RUNTIME_IDENTITY, Ordering::AcqRel);
    publish_status();

    let reply_subscription = match samp
        .net()
        .on_incoming_protocol_rpc(SERVER_MESSAGE, |message| {
            if message.text == INCOMING_MARKER {
                STATUS.fetch_or(STATUS_REPLY_OBSERVED, Ordering::AcqRel);
                INCOMING_REPLY_COUNT.fetch_add(1, Ordering::AcqRel);
            }
            if let Some(ids) = parse_entity_ids(&message.text) {
                *ENTITY_IDS.lock().unwrap_or_else(|error| error.into_inner()) = Some(ids);
            }
            record_vehicle_phase(&message.text);
            // The visible normal-chat reply is the required human proof that
            // SA-MP's original incoming-RPC handler ran after this callback.
            RpcAction::Continue
        }) {
        Ok(subscription) => subscription,
        Err(error) => {
            record_failure(error);
            publish_status();
            return;
        }
    };
    let mut subscriptions = SubscriptionSet::new();
    subscriptions.push(reply_subscription);
    #[cfg(feature = "r1-probe")]
    if let Err(error) = register_r1_exact_bit_listeners(samp, &mut subscriptions) {
        STATE
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .subscriptions = Some(subscriptions);
        record_failure(error);
        publish_status();
        return;
    }
    for (packet_id, observed_bit, count_index) in [
        (SendAimSync::ID, SYNC_PACKET_AIM, SYNC_INDEX_AIM),
        (SendPlayerSync::ID, SYNC_PACKET_ONFOOT, SYNC_INDEX_ONFOOT),
        (SendStatsUpdate::ID, SYNC_PACKET_STATS, SYNC_INDEX_STATS),
        (
            SendWeaponsUpdate::ID,
            SYNC_PACKET_WEAPONS,
            SYNC_INDEX_WEAPONS,
        ),
        (SendVehicleSync::ID, SYNC_PACKET_VEHICLE, SYNC_INDEX_VEHICLE),
        (
            SendPassengerSync::ID,
            SYNC_PACKET_PASSENGER,
            SYNC_INDEX_PASSENGER,
        ),
        (
            SendUnoccupiedSync::ID,
            SYNC_PACKET_UNOCCUPIED,
            SYNC_INDEX_UNOCCUPIED,
        ),
        (SendTrailerSync::ID, SYNC_PACKET_TRAILER, SYNC_INDEX_TRAILER),
    ] {
        let subscription =
            match samp
                .net()
                .on_packet_id(SampClientSdkDirection::Outgoing, packet_id, move |_| {
                    SYNC_PACKETS_OBSERVED.fetch_or(observed_bit, Ordering::AcqRel);
                    SYNC_PACKET_COUNTS[count_index].fetch_add(1, Ordering::AcqRel);
                    SampClientSdkHookAction::Continue
                }) {
                Ok(subscription) => subscription,
                Err(error) => {
                    STATE
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .subscriptions = Some(subscriptions);
                    record_failure(error);
                    publish_status();
                    return;
                }
            };
        subscriptions.push(subscription);
    }

    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    if state.shutting_down {
        return;
    }
    state.subscriptions = Some(subscriptions);
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
    #[cfg(feature = "r1-probe")]
    verify_r1_codec_and_exact_bits(samp)?;
    verify_cached_cnetgame_scalars(samp)?;
    STATUS.fetch_or(STATUS_CNETGAME_SCALARS, Ordering::AcqRel);
    publish_status();
    verify_cached_local_player(samp)?;
    STATUS.fetch_or(STATUS_LOCAL_PLAYER_SNAPSHOT, Ordering::AcqRel);
    publish_status();
    verify_entity_handles(samp)?;
    STATUS.fetch_or(STATUS_ENTITY_HANDLES, Ordering::AcqRel);
    publish_status();
    verify_force_sync_receipts(samp)?;
    STATUS.fetch_or(STATUS_FORCE_SYNC_RECEIPTS, Ordering::AcqRel);
    publish_status();
    verify_cached_player_pool_scalars(samp)?;
    STATUS.fetch_or(STATUS_PLAYER_POOL_SCALARS, Ordering::AcqRel);
    publish_status();
    #[cfg(feature = "r1-probe")]
    verify_r1_raw_addresses(samp)?;
    verify_cached_remote_player_directory(samp)?;
    STATUS.fetch_or(STATUS_REMOTE_PLAYER_DIRECTORY, Ordering::AcqRel);
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
    wait_for_receipt(samp.chat_input().set_enabled(true)?)?;
    wait_for_receipt(samp.chat_input().set_text(CHAT_INPUT_TEXT_MARKER)?)?;
    verify_cached_chat_input(samp)?;
    STATUS.fetch_or(STATUS_CHAT_INPUT_CACHE, Ordering::AcqRel);
    publish_status();
    verify_cached_chat_input_text(samp)?;
    STATUS.fetch_or(STATUS_CHAT_INPUT_TEXT_CACHE, Ordering::AcqRel);
    publish_status();
    wait_for_receipt(samp.chat_input().set_enabled(false)?)?;
    let dialog_receipt = samp.net().send_chat(DIALOG_REQUEST_MARKER)?;
    wait_for_receipt(dialog_receipt)?;
    STATUS.fetch_or(STATUS_DIALOG_REQUEST_RECEIPT, Ordering::AcqRel);
    publish_status();
    verify_cached_dialog_active(samp)?;
    STATUS.fetch_or(STATUS_DIALOG_ACTIVE_CACHE, Ordering::AcqRel);
    publish_status();
    verify_dialog_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_DIALOG_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_ui_mutations(samp)?;
    STATUS.fetch_or(STATUS_UI_MUTATIONS, Ordering::AcqRel);
    publish_status();
    verify_chat_command_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_CHAT_COMMAND_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_animation_table(samp)?;
    STATUS.fetch_or(STATUS_ANIMATION_TABLE, Ordering::AcqRel);
    publish_status();
    verify_local_mutations(samp)?;
    STATUS.fetch_or(STATUS_LOCAL_MUTATIONS, Ordering::AcqRel);
    publish_status();
    verify_text_label_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_TEXT_LABEL_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_textdraw_lifecycle(samp)?;
    STATUS.fetch_or(STATUS_TEXTDRAW_LIFECYCLE, Ordering::AcqRel);
    publish_status();
    verify_sync_snapshots(samp)?;
    STATUS.fetch_or(STATUS_SYNC_SNAPSHOTS, Ordering::AcqRel);
    publish_status();
    verify_vehicle_sync(samp)?;
    STATUS.fetch_or(STATUS_VEHICLE_SYNC, Ordering::AcqRel);
    publish_status();
    verify_reconnect_on_request(samp)?;
    Ok(())
}

#[cfg(feature = "r1-probe")]
fn register_r1_exact_bit_listeners(
    samp: Samp,
    subscriptions: &mut SubscriptionSet,
) -> Result<(), SampClientSdkResult> {
    let packet = samp.net().on_packet_id(
        SampClientSdkDirection::Incoming,
        R1_EXACT_BIT_TEST_ID,
        |event| {
            if r1_exact_bit_payload(event) {
                R1_PACKET_EXACT_BITS.store(true, Ordering::Release);
            } else {
                record_failure(SampClientSdkResult::NativeCallFailed);
            }
            SampClientSdkHookAction::Block
        },
    )?;
    subscriptions.push(packet);
    let rpc = samp.net().on_rpc_id(
        SampClientSdkDirection::Incoming,
        R1_EXACT_BIT_TEST_ID,
        |event| {
            if r1_exact_bit_payload(event) {
                R1_RPC_EXACT_BITS.store(true, Ordering::Release);
            } else {
                record_failure(SampClientSdkResult::NativeCallFailed);
            }
            SampClientSdkHookAction::Block
        },
    )?;
    subscriptions.push(rpc);
    Ok(())
}

#[cfg(feature = "r1-probe")]
fn r1_exact_bit_payload(event: &mut samp_client_sdk::events::Event<'_>) -> bool {
    event.remaining_bits() == R1_EXACT_BIT_COUNT
        && matches!(event.read_bits(R1_EXACT_BIT_COUNT), Ok(payload) if payload == R1_EXACT_BIT_PAYLOAD)
        && event.remaining_bits() == 0
}

#[cfg(feature = "r1-probe")]
fn verify_r1_codec_and_exact_bits(samp: Samp) -> Result<(), SampClientSdkResult> {
    let codec_deadline = Instant::now() + INITIALIZATION_TIMEOUT;
    loop {
        match r1_codec_round_trip(samp) {
            Ok(()) => break,
            Err(SampClientSdkResult::NotReady | SampClientSdkResult::Busy)
                if !codec_deadline
                    .saturating_duration_since(Instant::now())
                    .is_zero() =>
            {
                thread::sleep(RETRY_DELAY);
            }
            Err(error) => return Err(error),
        }
    }
    R1_CODEC_ROUND_TRIP.store(true, Ordering::Release);
    publish_status();

    let emulation_deadline = Instant::now() + INCOMING_READY_TIMEOUT;
    while !samp.net().incoming_emulation_ready() {
        if emulation_deadline
            .saturating_duration_since(Instant::now())
            .is_zero()
        {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }

    wait_for_receipt(samp.net().emulate_incoming_packet(
        R1_EXACT_BIT_TEST_ID,
        &R1_EXACT_BIT_PAYLOAD,
        R1_EXACT_BIT_COUNT,
    )?)?;
    wait_for_r1_exact_bit(&R1_PACKET_EXACT_BITS)?;
    wait_for_receipt(samp.net().emulate_incoming_rpc(
        R1_EXACT_BIT_TEST_ID,
        &R1_EXACT_BIT_PAYLOAD,
        R1_EXACT_BIT_COUNT,
    )?)?;
    wait_for_r1_exact_bit(&R1_RPC_EXACT_BITS)?;
    publish_status();
    Ok(())
}

#[cfg(feature = "r1-probe")]
fn r1_codec_round_trip(samp: Samp) -> Result<(), SampClientSdkResult> {
    let encoded = samp.net().encode_string(R1_CODEC_VALUE)?;
    let mut stream = BitStream::from_bits(encoded.as_bytes().to_vec(), encoded.len_bits())
        .map_err(|_| SampClientSdkResult::NativeCallFailed)?;
    let decoded = samp.net().decode_string(&mut stream)?;
    (decoded == R1_CODEC_VALUE && stream.remaining_bits() == 0)
        .then_some(())
        .ok_or(SampClientSdkResult::NativeCallFailed)
}

#[cfg(feature = "r1-probe")]
fn wait_for_r1_exact_bit(observed: &AtomicBool) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + CALLBACK_TIMEOUT;
    loop {
        if STATUS.load(Ordering::Acquire) & STATUS_FAILED != 0 {
            return Err(stored_failure());
        }
        if observed.load(Ordering::Acquire) {
            return Ok(());
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

#[cfg(feature = "r1-probe")]
fn verify_r1_raw_addresses(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_value(SCALAR_CACHE_TIMEOUT, || {
        let base = unsafe { raw::base() }.ok_or(SampClientSdkResult::NotReady)?;
        let rakclient = unsafe { raw::rakclient(samp) }?;
        let rakpeer = unsafe { raw::rakpeer(samp) }?;
        let player_pool = unsafe { raw::player_pool(samp) }?;
        let vehicle_pool = unsafe { raw::vehicle_pool(samp) }?;
        let player = unsafe { raw::player(samp) }?;
        (base.as_ptr() != rakclient.as_ptr()
            && rakpeer.as_ptr() != rakclient.as_ptr()
            && player_pool.as_ptr() != vehicle_pool.as_ptr()
            && player.as_ptr() != rakclient.as_ptr())
        .then_some(())
        .ok_or(SampClientSdkResult::NativeCallFailed)
    })
}

fn parse_entity_ids(message: &[u8]) -> Option<EntityIds> {
    let values = message.strip_prefix(ENTITY_IDS_PREFIX)?;
    let mut fields = values
        .split(|byte| *byte == b',')
        .map(|value| std::str::from_utf8(value).ok()?.parse::<u16>().ok());
    let ids = EntityIds {
        object: fields.next()??,
        vehicle: fields.next()??,
        pickup: fields.next()??,
        gangzone: fields.next()??,
    };
    fields.next().is_none().then_some(ids)
}

fn record_vehicle_phase(message: &[u8]) {
    let mut phases = VEHICLE_PHASES
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if let Some(values) = parse_u16_fields(message, LOCAL_DRIVER_READY_PREFIX) {
        phases.local_driver = (values.len() == 1).then_some(values[0]);
    } else if let Some(values) = parse_u16_fields(message, LOCAL_PASSENGER_READY_PREFIX) {
        phases.local_passenger = (values.len() == 1).then_some(values[0]);
    } else if let Some(values) = parse_u16_fields(message, LOCAL_TRAILER_READY_PREFIX) {
        phases.local_trailer = (values.len() == 2).then(|| VehiclePair {
            vehicle: values[0],
            trailer: values[1],
        });
    } else if message == VEHICLE_CLEANUP_READY_MARKER {
        phases.cleanup = true;
    }
}

fn parse_u16_fields(message: &[u8], prefix: &[u8]) -> Option<Vec<u16>> {
    message
        .strip_prefix(prefix)?
        .split(|byte| *byte == b',')
        .map(|value| std::str::from_utf8(value).ok()?.parse::<u16>().ok())
        .collect()
}

fn verify_entity_handles(samp: Samp) -> Result<(), SampClientSdkResult> {
    let request = samp.net().send_chat(ENTITY_REQUEST_MARKER)?;
    wait_for_receipt(request)?;
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let ids = *ENTITY_IDS.lock().unwrap_or_else(|error| error.into_inner());
        let Some(ids) = ids else {
            if deadline.saturating_duration_since(Instant::now()).is_zero() {
                return Err(SampClientSdkResult::TimedOut);
            }
            thread::sleep(RETRY_DELAY);
            continue;
        };
        let object = ObjectId::new(ids.object).ok_or(SampClientSdkResult::NativeCallFailed)?;
        let vehicle = VehicleId::new(ids.vehicle).ok_or(SampClientSdkResult::NativeCallFailed)?;
        let gangzone =
            GangzoneId::new(ids.gangzone).ok_or(SampClientSdkResult::NativeCallFailed)?;
        let local = match samp.local().player() {
            Ok(local) => local,
            Err(SampClientSdkResult::NotReady) => {
                if deadline.saturating_duration_since(Instant::now()).is_zero() {
                    return Err(SampClientSdkResult::TimedOut);
                }
                thread::sleep(RETRY_DELAY);
                continue;
            }
            Err(error) => return Err(error),
        };
        let player = samp
            .players()
            .player(PlayerId::new(local.id).ok_or(SampClientSdkResult::NativeCallFailed)?);
        let entity_results = (
            samp.objects().exists(object),
            samp.vehicles().exists(vehicle),
            samp.gangzones().get(gangzone),
            samp.objects().handle(object),
            samp.vehicles().handle(vehicle),
            samp.pickups().handle(ids.pickup),
            player.ped_handle(),
        );
        match entity_results {
            (
                Ok(true),
                Ok(true),
                Ok(Some(_)),
                Ok(Some(object_handle)),
                Ok(Some(vehicle_handle)),
                Ok(Some(pickup_handle)),
                Ok(Some(ped_handle)),
            ) => {
                let reverse_results = (
                    object_handle.to_id(samp),
                    vehicle_handle.to_id(samp),
                    pickup_handle.to_id(samp),
                    ped_handle.to_id(samp),
                );
                match reverse_results {
                    (
                        Ok(Some(object_id)),
                        Ok(Some(vehicle_id)),
                        Ok(Some(pickup_id)),
                        Ok(Some(player_id)),
                    ) if object_id == object
                        && vehicle_id == vehicle
                        && pickup_id == ids.pickup
                        && player_id == player.id() =>
                    {
                        return Ok(());
                    }
                    (Err(SampClientSdkResult::NotReady), _, _, _)
                    | (_, Err(SampClientSdkResult::NotReady), _, _)
                    | (_, _, Err(SampClientSdkResult::NotReady), _)
                    | (_, _, _, Err(SampClientSdkResult::NotReady)) => {}
                    (Ok(_), Ok(_), Ok(_), Ok(_)) => {
                        return Err(SampClientSdkResult::NativeCallFailed);
                    }
                    (Err(error), _, _, _)
                    | (_, Err(error), _, _)
                    | (_, _, Err(error), _)
                    | (_, _, _, Err(error)) => return Err(error),
                }
            }
            (Err(SampClientSdkResult::NotReady), _, _, _, _, _, _)
            | (_, Err(SampClientSdkResult::NotReady), _, _, _, _, _)
            | (_, _, Err(SampClientSdkResult::NotReady), _, _, _, _)
            | (_, _, _, Err(SampClientSdkResult::NotReady), _, _, _)
            | (_, _, _, _, Err(SampClientSdkResult::NotReady), _, _)
            | (_, _, _, _, _, Err(SampClientSdkResult::NotReady), _)
            | (_, _, _, _, _, _, Err(SampClientSdkResult::NotReady))
            | (Ok(false), _, _, _, _, _, _)
            | (_, Ok(false), _, _, _, _, _)
            | (_, _, Ok(None), _, _, _, _)
            | (_, _, _, Ok(None), _, _, _)
            | (_, _, _, _, Ok(None), _, _)
            | (_, _, _, _, _, Ok(None), _)
            | (_, _, _, _, _, _, Ok(None)) => {}
            (Err(error), _, _, _, _, _, _)
            | (_, Err(error), _, _, _, _, _)
            | (_, _, Err(error), _, _, _, _)
            | (_, _, _, Err(error), _, _, _)
            | (_, _, _, _, Err(error), _, _)
            | (_, _, _, _, _, Err(error), _)
            | (_, _, _, _, _, _, Err(error)) => return Err(error),
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn verify_force_sync_receipts(samp: Samp) -> Result<(), SampClientSdkResult> {
    wait_for_receipt(samp.local().force_aim_sync()?)?;
    wait_for_receipt(samp.local().force_onfoot_sync()?)?;
    wait_for_receipt(samp.local().force_stats_sync()?)?;
    wait_for_receipt(samp.local().force_weapons_sync()?)
}

fn verify_packet_after_command(
    count_index: usize,
    submit: impl FnOnce() -> Result<CommandReceipt<()>, SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    let before = SYNC_PACKET_COUNTS[count_index].load(Ordering::Acquire);
    wait_for_receipt(submit()?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(SYNC_PACKET_COUNTS[count_index].load(Ordering::Acquire) > before)
    })
}

fn verify_runtime_identity(samp: Samp) -> Result<(), SampClientSdkResult> {
    if samp.status() != SampClientSdkHostStatus::Ready || !samp.probe().is_samp_loaded() {
        return Err(SampClientSdkResult::NotReady);
    }
    if samp.version()? != PROFILE_CLIENT_VERSION {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    let Some(base) = (unsafe { raw::base() }) else {
        return Err(SampClientSdkResult::NotReady);
    };
    if unsafe { module_entry_point_rva(base.as_ptr()) } != Some(PROFILE_ENTRY_POINT_RVA) {
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
                if game_state == PROFILE_INITIAL_GAME_STATE
                    && info.address == b"127.0.0.1"
                    && info.hostname == PROFILE_SERVER_HOSTNAME
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
                if including_npcs == excluding_npcs.saturating_add(1) && max_id.is_some() {
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

fn verify_cached_remote_player_directory(samp: Samp) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let local_id = match samp.local().player() {
            Ok(player) => PlayerId::new(player.id).ok_or(SampClientSdkResult::NativeCallFailed)?,
            Err(SampClientSdkResult::NotReady) => {
                if deadline.saturating_duration_since(Instant::now()).is_zero() {
                    return Err(SampClientSdkResult::TimedOut);
                }
                thread::sleep(RETRY_DELAY);
                continue;
            }
            Err(error) => return Err(error),
        };
        match samp.players().player(local_id).is_defined() {
            Ok(true) => {}
            Ok(false) => return Err(SampClientSdkResult::NativeCallFailed),
            Err(SampClientSdkResult::NotReady) => {
                if deadline.saturating_duration_since(Instant::now()).is_zero() {
                    return Err(SampClientSdkResult::TimedOut);
                }
                thread::sleep(RETRY_DELAY);
                continue;
            }
            Err(error) => return Err(error),
        }

        let Some(max_id) = samp.players().max_id()? else {
            if deadline.saturating_duration_since(Instant::now()).is_zero() {
                return Err(SampClientSdkResult::TimedOut);
            }
            thread::sleep(RETRY_DELAY);
            continue;
        };
        for raw_id in 0..=max_id.get() {
            let Some(id) = PlayerId::new(raw_id) else {
                return Err(SampClientSdkResult::NativeCallFailed);
            };
            if id == local_id {
                continue;
            }
            match (
                samp.players().player(id).is_defined(),
                samp.players().get(id),
            ) {
                (Ok(true), Ok(Some(player)))
                    if player.id == id.get()
                        && !player.is_local
                        && player.is_npc
                        && !player.nickname.is_empty() =>
                {
                    match samp.players().remote_state(id) {
                        Ok(Some(state))
                            if state.id == id.get()
                                && state.health.is_finite()
                                && state.armour.is_finite() =>
                        {
                            return Ok(());
                        }
                        Ok(None) | Err(SampClientSdkResult::NotReady) => {}
                        Err(error) => return Err(error),
                        _ => return Err(SampClientSdkResult::NativeCallFailed),
                    }
                }
                (Ok(false), _) => {}
                (Err(SampClientSdkResult::NotReady), _)
                | (_, Err(SampClientSdkResult::NotReady))
                | (Ok(true), Ok(None)) => {}
                (Err(error), _) | (_, Err(error)) => return Err(error),
                _ => return Err(SampClientSdkResult::NativeCallFailed),
            }
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
            input.is_command_defined(MISSING_COMMAND_NAME),
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
    let scoreboard = samp.scoreboard();
    wait_for_receipt(scoreboard.toggle(true)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || scoreboard.is_open())?;
    wait_for_receipt(scoreboard.toggle(false)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || {
        scoreboard.is_open().map(|open| !open)
    })
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

fn verify_ui_mutations(samp: Samp) -> Result<(), SampClientSdkResult> {
    let chat = samp.chat();
    let original_chat_mode = wait_for_value(SCALAR_CACHE_TIMEOUT, || chat.display_mode())?;
    for mode in [
        LocalChatDisplayMode::Off,
        LocalChatDisplayMode::NoShadow,
        LocalChatDisplayMode::Normal,
    ] {
        wait_for_receipt(chat.set_display_mode(mode)?)?;
        wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
            chat.display_mode().map(|current| current == mode)
        })?;
    }
    wait_for_receipt(chat.set_display_mode(original_chat_mode)?)?;

    wait_for_receipt(chat.set_entry(
        99,
        LOCAL_CHAT_MARKER,
        LOCAL_CHAT_PREFIX,
        0xFF6FCF97,
        0xFFFFFFFF,
    )?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        chat.entry(99).map(|entry| {
            entry.text == LOCAL_CHAT_MARKER
                && entry.prefix == LOCAL_CHAT_PREFIX
                && entry.text_colour == 0xFF6FCF97
                && entry.prefix_colour == 0xFFFFFFFF
        })
    })?;
    wait_for_receipt(chat.add(LocalChatMessage {
        style: LocalChatMessageStyle::Info,
        text: LOCAL_CHAT_MARKER,
        prefix: b"",
        text_colour: 0xFF6FCF97,
        prefix_colour: 0,
    })?)?;
    wait_for_receipt(chat.death_window().add(LocalDeathMessage {
        killer: LOCAL_CHAT_PREFIX,
        victim: b"validation",
        killer_colour: 0xFF6FCF97,
        victim_colour: 0xFFFFFFFF,
        weapon: 24,
    })?)?;

    let input = samp.chat_input();
    wait_for_receipt(input.set_text(LOCAL_CHAT_INPUT_MUTATION)?)?;
    wait_for_condition(CHAT_INPUT_CACHE_TIMEOUT, || {
        input.text().map(|text| text == LOCAL_CHAT_INPUT_MUTATION)
    })?;
    wait_for_receipt(input.set_enabled(false)?)?;
    wait_for_condition(CHAT_INPUT_CACHE_TIMEOUT, || {
        input.is_active().map(|active| !active)
    })?;

    let cursor = samp.cursor();
    let original_cursor_mode = wait_for_value(SCALAR_CACHE_TIMEOUT, || cursor.mode())?;
    wait_for_receipt(cursor.set_mode(LocalCursorMode::LockCamera)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        cursor
            .mode()
            .map(|mode| mode == LocalCursorMode::LockCamera)
    })?;
    wait_for_receipt(cursor.set_mode(original_cursor_mode)?)?;
    wait_for_receipt(cursor.toggle(true)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || cursor.is_active())?;
    wait_for_receipt(cursor.toggle(false)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        cursor.is_active().map(|active| !active)
    })?;

    let scoreboard = samp.scoreboard();
    wait_for_receipt(scoreboard.toggle(true)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || scoreboard.is_open())?;
    wait_for_receipt(scoreboard.toggle(false)?)?;
    wait_for_condition(SCOREBOARD_CACHE_TIMEOUT, || {
        scoreboard.is_open().map(|open| !open)
    })
}

fn verify_dialog_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    let dialogs = samp.dialogs();
    wait_for_receipt(dialogs.close_with_button(0)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.is_active().map(|active| !active)
    })?;

    wait_for_receipt(dialogs.show(LocalDialog {
        id: 26_000,
        style: LocalDialogStyle::Input,
        title: LOCAL_INPUT_DIALOG_TITLE,
        text: LOCAL_INPUT_DIALOG_BODY,
        button1: b"Accept",
        button2: b"Cancel",
    })?)?;
    wait_for_receipt(dialogs.set_client_side(true)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.active().map(|dialog| {
            dialog.is_some_and(|dialog| {
                dialog.id == 26_000
                    && dialog.style == LocalDialogStyle::Input
                    && dialog.title == LOCAL_INPUT_DIALOG_TITLE
                    && dialog.text == LOCAL_INPUT_DIALOG_BODY
                    && !dialog.server_side
            })
        })
    })?;
    wait_for_receipt(dialogs.set_editbox_text(LOCAL_DIALOG_INPUT_TEXT)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.active().map(|dialog| {
            dialog.is_some_and(|dialog| {
                dialog.editbox_text.as_deref() == Some(LOCAL_DIALOG_INPUT_TEXT)
            })
        })
    })?;
    wait_for_receipt(dialogs.close_with_button(1)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.last_response().map(|response| {
            response.is_some_and(|response| {
                response.dialog_id == 26_000
                    && response.button == 1
                    && response.input == LOCAL_DIALOG_INPUT_TEXT
            })
        })
    })?;

    wait_for_receipt(dialogs.show(LocalDialog {
        id: 26_001,
        style: LocalDialogStyle::List,
        title: LOCAL_LIST_DIALOG_TITLE,
        text: b"first\nsecond",
        button1: b"Select",
        button2: b"Cancel",
    })?)?;
    wait_for_receipt(dialogs.set_selected_item(1)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        let state = dialogs.active()?;
        Ok(state.is_some_and(|dialog| {
            dialog.id == 26_001
                && dialog.style == LocalDialogStyle::List
                && dialog.items == [b"first".to_vec(), b"second".to_vec()]
        }) && dialogs.selected_item()? == 1
            && dialogs.list_item_count()? == 2)
    })?;
    wait_for_receipt(dialogs.close_with_button(0)?)?;
    wait_for_condition(DIALOG_ACTIVE_CACHE_TIMEOUT, || {
        dialogs.last_response().map(|response| {
            response.is_some_and(|response| {
                response.dialog_id == 26_001 && response.button == 0 && response.list_item == 1
            })
        })
    })
}

fn verify_chat_command_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    CHAT_COMMAND_INVOKED.store(false, Ordering::Release);
    let command = samp
        .chat_input()
        .register_command(LOCAL_COMMAND_NAME, |arguments| {
            if arguments == b"consolidated" {
                CHAT_COMMAND_INVOKED.store(true, Ordering::Release);
            }
        })?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.chat_input().is_command_defined(LOCAL_COMMAND_NAME)
    })?;
    wait_for_receipt(samp.chat_input().process(LOCAL_COMMAND_TEXT)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(CHAT_COMMAND_INVOKED.load(Ordering::Acquire))
    })?;
    command
        .unregister_and_wait()
        .map_err(|error| error.result())?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.chat_input()
            .is_command_defined(LOCAL_COMMAND_NAME)
            .map(|defined| !defined)
    })
}

fn verify_animation_table(samp: Samp) -> Result<(), SampClientSdkResult> {
    let animation = wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.anim().get(0))?;
    if animation.name.is_empty() || animation.file.is_empty() {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    match wait_for_value(SCALAR_CACHE_TIMEOUT, || {
        samp.anim().find(&animation.name, &animation.file)
    })? {
        Some(0) => Ok(()),
        _ => Err(SampClientSdkResult::NativeCallFailed),
    }
}

fn verify_local_mutations(samp: Samp) -> Result<(), SampClientSdkResult> {
    let local = wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?;
    let local_id = PlayerId::new(local.id).ok_or(SampClientSdkResult::NativeCallFailed)?;
    let player = samp.players().player(local_id);
    let original_colour = local.colour;
    let probe_colour: u32 = 0xFF6FCF97;
    wait_for_receipt(player.set_colour(probe_colour.rotate_left(8))?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        player.colour().map(|colour| colour == Some(probe_colour))
    })?;
    wait_for_receipt(player.set_colour(original_colour.rotate_left(8))?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        player
            .colour()
            .map(|colour| colour == Some(original_colour))
    })?;
    wait_for_receipt(samp.local().set_special_action(SpecialAction::None)?)?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.local()
            .player()
            .map(|snapshot| snapshot.special_action == SpecialAction::None.raw())
    })?;
    for kind in [
        SendRateKind::OnFoot,
        SendRateKind::InVehicle,
        SendRateKind::Aim,
    ] {
        wait_for_receipt(samp.net().set_send_rate(kind, 30)?)?;
    }
    Ok(())
}

fn verify_text_label_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    set_text_label_phase("local_player_wait");
    let local = wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?;
    set_text_label_phase("create_wait");
    let mut create = samp.labels().create(
        LOCAL_LABEL_TEXT,
        0xFF6FCF97,
        Vector3 {
            x: local.position.x,
            y: local.position.y,
            z: local.position.z + 1.0,
        },
        40.0,
        false,
        None,
        None,
    )?;
    let id = loop {
        match create.wait(INITIALIZATION_TIMEOUT) {
            Ok(id) => break id,
            Err(SampClientSdkResult::TimedOut) => continue,
            Err(error) => return Err(error),
        }
    };
    set_text_label_phase("initial_cache_wait");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        let label = match samp.labels().get(id) {
            Ok(label) => {
                let result = if label.is_some() { 2 } else { 1 };
                if TEXT_LABEL_INITIAL_RESULT.swap(result, Ordering::AcqRel) != result {
                    publish_status();
                }
                label
            }
            Err(error) => {
                let result = 0x100 | error as u32;
                if TEXT_LABEL_INITIAL_RESULT.swap(result, Ordering::AcqRel) != result {
                    publish_status();
                }
                return Err(error);
            }
        };
        let fields = label.map_or(0, |label| {
            (1 << 0)
                | (u32::from(label.id == id.get()) << 1)
                | (u32::from(label.text == LOCAL_LABEL_TEXT) << 2)
                | (u32::from(label.colour == 0xFF6FCF97) << 3)
                | (u32::from(label.draw_distance == 40.0) << 4)
                | (u32::from(!label.behind_walls) << 5)
        });
        if TEXT_LABEL_INITIAL_FIELDS.swap(fields, Ordering::AcqRel) != fields {
            publish_status();
        }
        Ok(fields == 0b11_1111)
    })?;
    set_text_label_phase("set_wait");
    wait_for_receipt(samp.labels().set_text(id, LOCAL_LABEL_UPDATED_TEXT)?)?;
    set_text_label_phase("updated_cache_wait");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.labels()
            .get(id)
            .map(|label| label.is_some_and(|label| label.text == LOCAL_LABEL_UPDATED_TEXT))
    })?;
    set_text_label_phase("delete_wait");
    wait_for_receipt(samp.labels().delete(id)?)?;
    set_text_label_phase("deleted_cache_wait");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.labels().exists(id).map(|exists| !exists)
    })?;
    set_text_label_phase("complete");
    Ok(())
}

fn set_text_label_phase(phase: &'static str) {
    *TEXT_LABEL_PHASE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = phase;
    publish_status();
}

fn verify_textdraw_lifecycle(samp: Samp) -> Result<(), SampClientSdkResult> {
    let textdraws = samp.textdraws();
    let mut free = None;
    for raw in (0..2_304).rev() {
        let id = TextdrawId::new(raw).ok_or(SampClientSdkResult::NativeCallFailed)?;
        if !wait_for_value(SCALAR_CACHE_TIMEOUT, || textdraws.exists(id))? {
            free = Some(id);
            break;
        }
    }
    let id = free.ok_or(SampClientSdkResult::NativeCallFailed)?;
    verify_textdraw_mutation("create_before", "create_after", || {
        textdraws.create(id, LOCAL_TEXTDRAW_TEXT, 320.0, 180.0)
    })?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        textdraws
            .get(id)
            .map(|textdraw| textdraw.is_some_and(|textdraw| textdraw.text == LOCAL_TEXTDRAW_TEXT))
    })?;
    verify_textdraw_mutation("position_before", "position_after", || {
        textdraws.set_position(id, 300.0, 170.0)
    })?;
    verify_textdraw_mutation("style_before", "style_after", || textdraws.set_style(id, 1))?;
    verify_textdraw_mutation("letter_before", "letter_after", || {
        textdraws.set_letter_style(id, 0.3, 1.2, 0xFFFFFFFF)
    })?;
    verify_textdraw_mutation("proportional_before", "proportional_after", || {
        textdraws.set_proportional(id, false)
    })?;
    verify_textdraw_mutation("shadow_before", "shadow_after", || {
        textdraws.set_shadow(id, 2, 0xFF101010)
    })?;
    verify_textdraw_mutation("outline_before", "outline_after", || {
        textdraws.set_outline(id, 1, 0xFF202020)
    })?;
    verify_textdraw_mutation("box_before", "box_after", || {
        textdraws.set_box(id, true, 0x80202020, 180.0, 30.0)
    })?;
    verify_textdraw_mutation("alignment_before", "alignment_after", || {
        textdraws.set_alignment(id, 2)
    })?;
    verify_textdraw_mutation("string_before", "string_after", || {
        textdraws.set_text(id, LOCAL_TEXTDRAW_UPDATED_TEXT)
    })?;
    verify_textdraw_mutation("model_before", "model_after", || {
        textdraws.set_model_style(
            id,
            Vector3 {
                x: 10.0,
                y: 20.0,
                z: 30.0,
            },
            1.25,
            1,
            2,
        )
    })?;
    set_textdraw_phase("snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || match textdraws.get(id) {
        Ok(textdraw) => {
            let fields = textdraw.as_ref().map_or(0, textdraw_snapshot_fields);
            publish_textdraw_snapshot_observation(if textdraw.is_some() { 2 } else { 1 }, fields);
            Ok(fields == 0x7FF)
        }
        Err(error) => {
            publish_textdraw_snapshot_observation(0x100 | error as u32, 0);
            Err(error)
        }
    })?;
    set_textdraw_phase("snapshot_after");
    verify_textdraw_mutation("delete_before", "delete_after", || textdraws.delete(id))?;
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        textdraws.exists(id).map(|exists| !exists)
    })?;
    set_textdraw_phase("complete");
    Ok(())
}

fn verify_textdraw_mutation(
    before: &'static str,
    after: &'static str,
    submit: impl FnOnce() -> Result<CommandReceipt<()>, SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    set_textdraw_phase(before);
    wait_for_receipt(submit()?)?;
    set_textdraw_phase(after);
    Ok(())
}

fn set_textdraw_phase(phase: &'static str) {
    *TEXTDRAW_PHASE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = phase;
    publish_status();
}

fn textdraw_snapshot_fields(textdraw: &samp_client_sdk::TextDraw) -> u32 {
    (1 << 0)
        | (u32::from(textdraw.text == LOCAL_TEXTDRAW_UPDATED_TEXT) << 1)
        | (u32::from(textdraw.position() == (300.0, 170.0)) << 2)
        | (u32::from(textdraw.style() == 1) << 3)
        | (u32::from(textdraw.letter_style() == (0.3, 1.2, 0xFFFFFFFF)) << 4)
        | (u32::from(!textdraw.is_proportional()) << 5)
        | (u32::from(textdraw.shadow() == 2) << 6)
        | (u32::from(textdraw.outline() == 1) << 7)
        | (u32::from(textdraw.alignment() == (false, true, false)) << 8)
        | (u32::from(textdraw.box_style() == (true, 180.0, 30.0, 0x80202020)) << 9)
        | (u32::from(
            textdraw.model_style()
                == (
                    0,
                    Vector3 {
                        x: 10.0,
                        y: 20.0,
                        z: 30.0,
                    },
                    1.25,
                    1,
                    2,
                ),
        ) << 10)
}

fn publish_textdraw_snapshot_observation(result: u32, fields: u32) {
    let result_changed = TEXTDRAW_SNAPSHOT_RESULT.swap(result, Ordering::AcqRel) != result;
    let fields_changed = TEXTDRAW_SNAPSHOT_FIELDS.swap(fields, Ordering::AcqRel) != fields;
    if result_changed || fields_changed {
        publish_status();
    }
}

fn verify_vehicle_sync(samp: Samp) -> Result<(), SampClientSdkResult> {
    let local_id =
        PlayerId::new(wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?.id)
            .ok_or(SampClientSdkResult::NativeCallFailed)?;

    set_vehicle_phase("driver_request_before");
    wait_for_receipt(samp.net().send_chat(LOCAL_DRIVER_REQUEST)?)?;
    set_vehicle_phase("driver_request_after");
    let local_vehicle = wait_for_vehicle_phase(SCALAR_CACHE_TIMEOUT, |phases| phases.local_driver)?;
    let local_vehicle_id =
        VehicleId::new(local_vehicle).ok_or(SampClientSdkResult::NativeCallFailed)?;
    set_vehicle_phase("driver_snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.players().player(local_id).vehicle_sync().map(|sync| {
            sync.is_some_and(|sync| in_car_sync_is_valid(sync, local_id, local_vehicle))
        })
    })?;
    set_vehicle_phase("driver_snapshot_after");
    set_vehicle_phase("driver_force_before");
    verify_packet_after_command(SYNC_INDEX_VEHICLE, || {
        samp.local().force_vehicle_sync(local_vehicle_id)
    })?;
    set_vehicle_phase("driver_force_after");

    set_vehicle_phase("passenger_request_before");
    wait_for_receipt(samp.net().send_chat(LOCAL_PASSENGER_REQUEST)?)?;
    set_vehicle_phase("passenger_request_after");
    let passenger_vehicle =
        wait_for_vehicle_phase(SCALAR_CACHE_TIMEOUT, |phases| phases.local_passenger)?;
    if passenger_vehicle != local_vehicle {
        return Err(SampClientSdkResult::NativeCallFailed);
    }
    set_vehicle_phase("passenger_snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        samp.players()
            .player(local_id)
            .passenger_sync()
            .map(|sync| {
                sync.is_some_and(|sync| {
                    sync.id == local_id.get()
                        && sync.vehicle_id == local_vehicle
                        && sync.seat_id == 1
                        && vector_is_finite(sync.position)
                })
            })
    })?;
    set_vehicle_phase("passenger_snapshot_after");
    set_vehicle_phase("passenger_force_before");
    verify_packet_after_command(SYNC_INDEX_PASSENGER, || {
        samp.local().force_passenger_sync(local_vehicle_id, 1)
    })?;
    set_vehicle_phase("passenger_force_after");

    set_vehicle_phase("trailer_request_before");
    wait_for_receipt(samp.net().send_chat(LOCAL_TRAILER_REQUEST)?)?;
    set_vehicle_phase("trailer_request_after");
    let local_trailer =
        wait_for_vehicle_phase(SCALAR_CACHE_TIMEOUT, |phases| phases.local_trailer)?;
    let trailer_id =
        VehicleId::new(local_trailer.trailer).ok_or(SampClientSdkResult::NativeCallFailed)?;
    set_vehicle_phase("trailer_snapshot_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        let player = samp.players().player(local_id);
        let in_car = player.vehicle_sync()?;
        let trailer = player.trailer_sync()?;
        Ok(in_car.is_some_and(|sync| {
            in_car_sync_is_valid(sync, local_id, local_trailer.vehicle)
                && sync.trailer_id == local_trailer.trailer
        }) && trailer.is_some_and(|sync| {
            sync.id == local_id.get()
                && sync.trailer_id == local_trailer.trailer
                && vector_is_finite(sync.position)
                && vector_is_finite(sync.speed)
                && vector_is_finite(sync.turn_speed)
                && sync.quaternion.into_iter().all(f32::is_finite)
        }))
    })?;
    set_vehicle_phase("trailer_snapshot_after");
    set_vehicle_phase("trailer_force_before");
    verify_packet_after_command(SYNC_INDEX_TRAILER, || {
        samp.local().force_trailer_sync(trailer_id)
    })?;
    set_vehicle_phase("trailer_force_after");
    let truck_id =
        VehicleId::new(local_trailer.vehicle).ok_or(SampClientSdkResult::NativeCallFailed)?;
    set_vehicle_phase("unoccupied_force_before");
    verify_packet_after_command(SYNC_INDEX_UNOCCUPIED, || {
        samp.local().force_unoccupied_sync(truck_id, 0)
    })?;
    set_vehicle_phase("unoccupied_force_after");

    let vehicle_packet_mask =
        SYNC_PACKET_VEHICLE | SYNC_PACKET_PASSENGER | SYNC_PACKET_UNOCCUPIED | SYNC_PACKET_TRAILER;
    set_vehicle_phase("packet_mask_before");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(
            SYNC_PACKETS_OBSERVED.load(Ordering::Acquire) & vehicle_packet_mask
                == vehicle_packet_mask,
        )
    })?;
    set_vehicle_phase("packet_mask_after");

    set_vehicle_phase("cleanup_request_before");
    wait_for_receipt(samp.net().send_chat(VEHICLE_CLEANUP_REQUEST)?)?;
    set_vehicle_phase("cleanup_request_after");
    wait_for_condition(SCALAR_CACHE_TIMEOUT, || {
        Ok(VEHICLE_PHASES
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .cleanup)
    })?;
    set_vehicle_phase("complete");
    Ok(())
}

fn set_vehicle_phase(phase: &'static str) {
    *VEHICLE_PHASE
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = phase;
    publish_status();
}

fn wait_for_vehicle_phase<T: Copy>(
    timeout: Duration,
    read: impl Fn(&VehiclePhases) -> Option<T>,
) -> Result<T, SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(value) = read(
            &VEHICLE_PHASES
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        ) {
            return Ok(value);
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn in_car_sync_is_valid(sync: samp_client_sdk::InCarSync, player: PlayerId, vehicle: u16) -> bool {
    sync.id == player.get()
        && sync.vehicle_id == vehicle
        && vector_is_finite(sync.position)
        && vector_is_finite(sync.speed)
        && sync.vehicle_health.is_finite()
        && sync.quaternion.into_iter().all(f32::is_finite)
}

fn verify_sync_snapshots(samp: Samp) -> Result<(), SampClientSdkResult> {
    let local_id =
        PlayerId::new(wait_for_value(SCALAR_CACHE_TIMEOUT, || samp.local().player())?.id)
            .ok_or(SampClientSdkResult::NativeCallFailed)?;
    let remote_id = find_remote_player(samp, SCALAR_CACHE_TIMEOUT)?;
    wait_for_condition(CHAT_INPUT_CACHE_TIMEOUT, || {
        let local = samp.players().player(local_id);
        let remote = samp.players().player(remote_id);
        let Some(local_onfoot) = local.onfoot_sync()? else {
            return Ok(false);
        };
        let Some(local_aim) = local.aim_sync()? else {
            return Ok(false);
        };
        let Some(remote_onfoot) = remote.onfoot_sync()? else {
            return Ok(false);
        };
        Ok(local_onfoot.id == local_id.get()
            && local_aim.id == local_id.get()
            && remote_onfoot.id == remote_id.get()
            && vector_is_finite(local_onfoot.position)
            && vector_is_finite(local_onfoot.speed)
            && local_onfoot.quaternion.into_iter().all(f32::is_finite)
            && vector_is_finite(local_aim.aim_first)
            && vector_is_finite(local_aim.aim_position)
            && local_aim.aim_z.is_finite()
            && vector_is_finite(remote_onfoot.position)
            && vector_is_finite(remote_onfoot.speed)
            && remote_onfoot.quaternion.into_iter().all(f32::is_finite))
    })
}

fn find_remote_player(samp: Samp, timeout: Duration) -> Result<PlayerId, SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        let local_id = PlayerId::new(samp.local().player()?.id)
            .ok_or(SampClientSdkResult::NativeCallFailed)?;
        if let Some(max_id) = samp.players().max_id()? {
            for raw in 0..=max_id.get() {
                let id = PlayerId::new(raw).ok_or(SampClientSdkResult::NativeCallFailed)?;
                if id != local_id && samp.players().player(id).is_defined().unwrap_or(false) {
                    return Ok(id);
                }
            }
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
}

fn vector_is_finite(value: Vector3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn wait_for_condition(
    timeout: Duration,
    mut condition: impl FnMut() -> Result<bool, SampClientSdkResult>,
) -> Result<(), SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match condition() {
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

fn wait_for_value<T>(
    timeout: Duration,
    mut read: impl FnMut() -> Result<T, SampClientSdkResult>,
) -> Result<T, SampClientSdkResult> {
    let deadline = Instant::now() + timeout;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        match read() {
            Ok(value) => return Ok(value),
            Err(SampClientSdkResult::NotReady) => {}
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

fn verify_reconnect_on_request(samp: Samp) -> Result<(), SampClientSdkResult> {
    RECONNECT_REQUESTED.store(false, Ordering::Release);
    let reconnect_command = samp
        .chat_input()
        .register_command(RECONNECT_COMMAND_NAME, |_| {
            RECONNECT_REQUESTED.store(true, Ordering::Release);
        })?;
    wait_for_receipt(samp.chat().add(LocalChatMessage {
        style: LocalChatMessageStyle::Info,
        text: MAIN_PASS_MESSAGE,
        prefix: b"",
        text_colour: 0xFF6FCF97,
        prefix_colour: 0,
    })?)?;

    let request = wait_for_condition(HOST_CONNECTION_TIMEOUT, || {
        Ok(RECONNECT_REQUESTED.load(Ordering::Acquire))
    });
    let unregister = reconnect_command
        .unregister_and_wait()
        .map_err(|error| error.result());
    request?;
    unregister?;

    wait_for_receipt(samp.net().disconnect(500)?)?;
    let deadline = Instant::now() + SCALAR_CACHE_TIMEOUT;
    loop {
        if is_shutting_down() {
            return Err(SampClientSdkResult::ShuttingDown);
        }
        let server_not_ready = matches!(samp.server().info(), Err(SampClientSdkResult::NotReady));
        let local_not_ready = matches!(samp.local().player(), Err(SampClientSdkResult::NotReady));
        #[cfg(feature = "r1-probe")]
        let raw_connection_state_invalidated = matches!(
            (
                unsafe { raw::player_pool(samp) },
                unsafe { raw::vehicle_pool(samp) },
                unsafe { raw::player(samp) },
            ),
            (
                Err(SampClientSdkResult::NotReady),
                Err(SampClientSdkResult::NotReady),
                Err(SampClientSdkResult::NotReady),
            )
        );
        #[cfg(not(feature = "r1-probe"))]
        let raw_connection_state_invalidated = true;
        if server_not_ready
            && local_not_ready
            && raw_connection_state_invalidated
            && !samp.net().incoming_emulation_ready()
        {
            break;
        }
        if deadline.saturating_duration_since(Instant::now()).is_zero() {
            return Err(SampClientSdkResult::TimedOut);
        }
        thread::sleep(RETRY_DELAY);
    }
    STATUS.fetch_or(STATUS_DISCONNECT_INVALIDATION, Ordering::AcqRel);
    publish_status();

    wait_for_receipt(samp.net().connect(b"127.0.0.1", 7777)?)?;
    wait_for_condition(HOST_CONNECTION_TIMEOUT, || {
        let server = samp.server().info();
        let local = samp.local().player();
        let game_state = samp.game_state().ok();
        let observation = ReconnectObservation {
            server_ready: server
                .as_ref()
                .is_ok_and(|server| server.address == b"127.0.0.1" && server.port == 7777),
            local_ready: local.is_ok(),
            game_state,
            spawned: local.as_ref().ok().map(|local| local.spawned),
            incoming_ready: samp.net().incoming_emulation_ready(),
        };
        let ready = observation.server_ready
            && observation.game_state == Some(PROFILE_CONNECTED_STATE)
            && observation.spawned == Some(true)
            && observation.incoming_ready;
        let mut published = RECONNECT_OBSERVATION
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if published.as_ref() != Some(&observation) {
            *published = Some(observation);
            drop(published);
            publish_status();
        }
        Ok(ready)
    })?;

    let replies_before = INCOMING_REPLY_COUNT.load(Ordering::Acquire);
    wait_for_receipt(samp.net().send_chat(OUTBOUND_MARKER)?)?;
    wait_for_condition(CALLBACK_TIMEOUT, || {
        Ok(INCOMING_REPLY_COUNT.load(Ordering::Acquire) > replies_before)
    })?;
    STATUS.fetch_or(STATUS_RECONNECT_RESTORED, Ordering::AcqRel);
    publish_status();
    Ok(())
}

fn connect_host() -> Option<Samp> {
    let deadline = Instant::now() + HOST_CONNECTION_TIMEOUT;
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
            RECONNECT_OBSERVATION
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .as_ref(),
            ProbePhaseStatus {
                text_label_phase: *TEXT_LABEL_PHASE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()),
                text_label_initial_fields: TEXT_LABEL_INITIAL_FIELDS.load(Ordering::Acquire),
                text_label_initial_result: TEXT_LABEL_INITIAL_RESULT.load(Ordering::Acquire),
                textdraw_phase: *TEXTDRAW_PHASE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()),
                textdraw_snapshot_fields: TEXTDRAW_SNAPSHOT_FIELDS.load(Ordering::Acquire),
                textdraw_snapshot_result: TEXTDRAW_SNAPSHOT_RESULT.load(Ordering::Acquire),
                vehicle_phase: *VEHICLE_PHASE
                    .lock()
                    .unwrap_or_else(|error| error.into_inner()),
            },
        ),
    );
}

fn status_record(
    status: u32,
    failure: u32,
    scalar: Option<&ScalarObservation>,
    player_pool: Option<&PlayerPoolObservation>,
    reconnect: Option<&ReconnectObservation>,
    phases: ProbePhaseStatus,
) -> String {
    use std::fmt::Write;

    let mut record = format!("status=0x{status:08X}\nfailure={failure}\n");
    if let Some(scalar) = scalar {
        let _ = writeln!(record, "game_state={}", scalar.game_state);
        let _ = writeln!(record, "address_hex={}", hex(&scalar.address));
        let _ = writeln!(record, "hostname_hex={}", hex(&scalar.hostname));
        let _ = writeln!(record, "port={}", scalar.port);
    }
    if let Some(player_pool) = player_pool {
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
    if let Some(reconnect) = reconnect {
        let _ = writeln!(record, "reconnect_server_ready={}", reconnect.server_ready);
        let _ = writeln!(record, "reconnect_local_ready={}", reconnect.local_ready);
        let _ = writeln!(record, "reconnect_game_state={:?}", reconnect.game_state);
        let _ = writeln!(record, "reconnect_spawned={:?}", reconnect.spawned);
        let _ = writeln!(
            record,
            "reconnect_incoming_ready={}",
            reconnect.incoming_ready
        );
    }
    let _ = writeln!(record, "text_label_phase={}", phases.text_label_phase);
    let _ = writeln!(
        record,
        "text_label_initial_fields=0x{:02X}",
        phases.text_label_initial_fields
    );
    let _ = writeln!(
        record,
        "text_label_initial_result=0x{:03X}",
        phases.text_label_initial_result
    );
    let _ = writeln!(record, "textdraw_phase={}", phases.textdraw_phase);
    let _ = writeln!(
        record,
        "textdraw_snapshot_fields=0x{:03X}",
        phases.textdraw_snapshot_fields
    );
    let _ = writeln!(
        record,
        "textdraw_snapshot_result=0x{:03X}",
        phases.textdraw_snapshot_result
    );
    let _ = writeln!(record, "vehicle_phase={}", phases.vehicle_phase);
    #[cfg(feature = "r1-probe")]
    {
        let _ = writeln!(
            record,
            "codec_round_trip={}",
            R1_CODEC_ROUND_TRIP.load(Ordering::Acquire)
        );
        let _ = writeln!(
            record,
            "incoming_packet_bits={}",
            usize::from(R1_PACKET_EXACT_BITS.load(Ordering::Acquire)) * R1_EXACT_BIT_COUNT
        );
        let _ = writeln!(
            record,
            "incoming_rpc_bits={}",
            usize::from(R1_RPC_EXACT_BITS.load(Ordering::Acquire)) * R1_EXACT_BIT_COUNT
        );
    }
    record
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02X}")).collect()
}

/// Stops the callbacks before an unload manager calls `FreeLibrary`.
///
/// This must run on a worker thread, not from `DllMain` or a callback.
#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdkR5NetworkProbe_Shutdown() -> BOOL {
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
                .unwrap_or_else(|poison| poison.into_inner())
                .subscriptions = Some(error.into_subscriptions());
            0
        }
    }
}

/// Returns the probe stage bitset.
///
/// The connected pass is complete at `0x0FFFFFFF`; the opt-in reconnect pass
/// completes at `0x3FFFFFFF`.
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
    fn selected_probe_configuration_matches_its_client_version() {
        #[cfg(feature = "r1-probe")]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::R1,
                0x31DF13,
                14,
                14,
                b"SDK R1 loopback probe".as_slice()
            )
        );
        #[cfg(feature = "r3-probe")]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::R3_1,
                0x0CC4D0,
                15,
                14,
                b"SA-MP".as_slice()
            )
        );
        #[cfg(feature = "dl-probe")]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::Dl,
                0x0FDB60,
                5,
                5,
                b"SDK DL loopback probe".as_slice()
            )
        );
        #[cfg(not(any(feature = "r1-probe", feature = "r3-probe", feature = "dl-probe")))]
        assert_eq!(
            (
                PROFILE_CLIENT_VERSION,
                PROFILE_ENTRY_POINT_RVA,
                PROFILE_INITIAL_GAME_STATE,
                PROFILE_CONNECTED_STATE,
                PROFILE_SERVER_HOSTNAME
            ),
            (
                SampClientSdkClientVersion::R5_1,
                0x0CBC90,
                5,
                5,
                b"SDK R5 loopback probe".as_slice()
            )
        );
    }

    #[test]
    fn selected_probe_ui_labels_match_its_client_version() {
        let labels = (
            LOCAL_CHAT_PREFIX,
            LOCAL_CHAT_INPUT_MUTATION,
            LOCAL_INPUT_DIALOG_TITLE,
            LOCAL_INPUT_DIALOG_BODY,
            LOCAL_LIST_DIALOG_TITLE,
            MAIN_PASS_MESSAGE,
        );
        #[cfg(feature = "r1-probe")]
        assert_eq!(
            labels,
            (
                b"R1 SDK".as_slice(),
                b"R1_SDK_MUTATION".as_slice(),
                b"R1 input dialog".as_slice(),
                b"R1 input body".as_slice(),
                b"R1 list dialog".as_slice(),
                b"R1 main pass complete. Type /r1sdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
        #[cfg(feature = "r3-probe")]
        assert_eq!(
            labels,
            (
                b"R3 SDK".as_slice(),
                b"R3_SDK_MUTATION".as_slice(),
                b"R3 input dialog".as_slice(),
                b"R3 input body".as_slice(),
                b"R3 list dialog".as_slice(),
                b"R3 main pass complete. Type /r3sdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
        #[cfg(feature = "dl-probe")]
        assert_eq!(
            labels,
            (
                b"DL SDK".as_slice(),
                b"DL_SDK_MUTATION".as_slice(),
                b"DL input dialog".as_slice(),
                b"DL input body".as_slice(),
                b"DL list dialog".as_slice(),
                b"DL main pass complete. Type /dlsdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
        #[cfg(not(any(feature = "r1-probe", feature = "r3-probe", feature = "dl-probe")))]
        assert_eq!(
            labels,
            (
                b"R5 SDK".as_slice(),
                b"R5_SDK_MUTATION".as_slice(),
                b"R5 input dialog".as_slice(),
                b"R5 input body".as_slice(),
                b"R5 list dialog".as_slice(),
                b"R5 main pass complete. Type /r5sdkreconnect for the final lifecycle pass."
                    .as_slice(),
            )
        );
    }

    #[test]
    fn status_record_is_bounded_and_machine_readable() {
        let record = status_record(
            STATUS_HOST_CONNECTED | STATUS_OUTBOUND_RECEIPT,
            0,
            None,
            None,
            None,
            ProbePhaseStatus {
                text_label_phase: "none",
                text_label_initial_fields: 0,
                text_label_initial_result: 0,
                textdraw_phase: "none",
                textdraw_snapshot_fields: 0,
                textdraw_snapshot_result: 0,
                vehicle_phase: "none",
            },
        );
        #[cfg(not(feature = "r1-probe"))]
        assert_eq!(
            record,
            "status=0x00000011\nfailure=0\ntext_label_phase=none\ntext_label_initial_fields=0x00\ntext_label_initial_result=0x000\ntextdraw_phase=none\ntextdraw_snapshot_fields=0x000\ntextdraw_snapshot_result=0x000\nvehicle_phase=none\n"
        );
        #[cfg(feature = "r1-probe")]
        assert_eq!(
            record,
            concat!(
                "status=0x00000011\nfailure=0\ntext_label_phase=none\ntext_label_initial_fields=0x00\ntext_label_initial_result=0x000\ntextdraw_phase=none\ntextdraw_snapshot_fields=0x000\ntextdraw_snapshot_result=0x000\nvehicle_phase=none\n",
                "codec_round_trip=false\nincoming_packet_bits=0\nincoming_rpc_bits=0\n"
            )
        );
    }

    #[test]
    fn parses_the_r5_entry_point_from_a_bounded_pe_header() {
        let mut image = vec![0_u8; 0x200];
        image[..2].copy_from_slice(b"MZ");
        image[0x3C..0x40].copy_from_slice(&(0x80_u32).to_le_bytes());
        image[0x80..0x84].copy_from_slice(b"PE\0\0");
        image[0x98..0x9A].copy_from_slice(&0x10B_u16.to_le_bytes());
        image[0xA8..0xAC].copy_from_slice(&PROFILE_ENTRY_POINT_RVA.to_le_bytes());

        assert_eq!(
            parse_pe_entry_point_rva(|offset, destination| {
                let Some(source) = image.get(offset..offset.saturating_add(destination.len()))
                else {
                    return false;
                };
                destination.copy_from_slice(source);
                true
            }),
            Some(PROFILE_ENTRY_POINT_RVA)
        );
    }

    #[test]
    fn valid_unspawned_local_player_remains_retryable() {
        let player = samp_client_sdk::LocalPlayer {
            id: 0,
            nickname: b"R5 probe".to_vec(),
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

    #[test]
    fn parses_the_server_owned_entity_id_marker() {
        let marker = [ENTITY_IDS_PREFIX, b"17,18,19,20"].concat();
        let ids = parse_entity_ids(&marker).unwrap();
        assert_eq!(ids.object, 17);
        assert_eq!(ids.vehicle, 18);
        assert_eq!(ids.pickup, 19);
        assert_eq!(ids.gangzone, 20);
        assert!(parse_entity_ids(&[ENTITY_IDS_PREFIX, b"17,18,19"].concat()).is_none());
        assert!(parse_entity_ids(&[ENTITY_IDS_PREFIX, b"17,18,19,20,21"].concat()).is_none());
    }

    #[test]
    fn parses_bounded_vehicle_phase_fields() {
        assert_eq!(
            parse_u16_fields(
                b"R5_SDK_REMOTE_TRAILER_READY_7,18,19",
                b"R5_SDK_REMOTE_TRAILER_READY_"
            ),
            Some(vec![7, 18, 19])
        );
        assert!(
            parse_u16_fields(
                b"R5_SDK_REMOTE_TRAILER_READY_7,nope,19",
                b"R5_SDK_REMOTE_TRAILER_READY_"
            )
            .is_none()
        );
    }

    #[test]
    fn retries_transient_not_ready_values() {
        let mut attempts = 0;
        let value = wait_for_value(Duration::from_millis(500), || {
            attempts += 1;
            if attempts < 3 {
                Err(SampClientSdkResult::NotReady)
            } else {
                Ok(37)
            }
        })
        .expect("a published cache value should be returned");

        assert_eq!(value, 37);
        assert_eq!(attempts, 3);
    }

    #[test]
    fn success_masks_cover_contiguous_status_bits() {
        assert_eq!(MAIN_SUCCESS_STATUS, (1_u32 << 28) - 1);
        assert_eq!(FULL_SUCCESS_STATUS, (1_u32 << 30) - 1);
    }
}
