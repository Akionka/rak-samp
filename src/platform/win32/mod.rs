//! Private Windows x86 hook implementation.
//!
//! The module is intentionally unavailable for other targets. Its only public
//! boundary is the safe `Runtime` API in the parent crate.

mod backend;
mod chat_entries;
mod commands;
mod gangzones;
mod handles;
mod hooks;
mod native_bitstream;
mod native_client;
mod native_profile;
mod objects;
mod packets;
mod players;
pub(crate) mod r1_client;
mod r3_client;
mod reads;
mod refresh;
mod requests;
mod sampfuncs;
mod text_labels;
mod textdraws;
mod vehicles;

pub(crate) use sampfuncs::{SampfuncsLogError, sampfuncs_loaded, sampfuncs_log_console};

use crate::{
    AddressSet, AttachError, BitStream, SampVersion, SendError, SendOptions,
    command::{CommandError, CommandId, CommandQueue, QueuedCommand},
    event::Registry,
    runtime::{
        AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, ClientHookStatus, CodecError,
        DirectClientError, GangzoneSnapshot, InCarSyncSnapshot, LocalChatMessageRequest,
        LocalDeathMessageRequest, LocalDialogRequest, LocalDialogResponseSnapshot,
        LocalDialogSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot, PacketPriority,
        PacketReliability, PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot,
        ServerInfoSnapshot, TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
    },
};
use hooks::{HookStorage, InlineHook, VtableHook};
#[cfg(test)]
use native_bitstream::native_bit_length;
use native_bitstream::{NativeBitStream, RawBitStream};
use native_client::profile::NativeClientProfile;
use native_profile::NativeProfile;
use std::{
    collections::{HashMap, VecDeque},
    ffi::c_void,
    ptr,
    sync::{
        Arc, Mutex, MutexGuard, OnceLock, TryLockError, Weak,
        atomic::{AtomicBool, AtomicI32, AtomicU16, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::System::{LibraryLoader::GetModuleHandleA, Threading::GetCurrentThreadId};

const ID_TIMESTAMP: u8 = 40;
const ID_RPC: u8 = 20;
const OUTGOING_PACKET_SLOT: usize = 6;
const INCOMING_PACKET_SLOT: usize = 8;
const DEALLOCATE_PACKET_SLOT: usize = 9;
const OUTGOING_RPC_SLOT: usize = 25;
const PEER_PACKET_QUEUE_OFFSET: usize = 0xDB6;
const PLAYER_INFO_REQUEST_QUEUE_CAPACITY: usize = 32;
const REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY: usize = 32;
const REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP: usize = 4;
const STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY: usize = 32;
const STREAMED_OUT_PLAYER_POSITION_REQUESTS_PER_PUMP: usize = 4;
const ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY: usize = 32;
const ONFOOT_SYNC_REQUESTS_PER_PUMP: usize = 4;
const INCAR_SYNC_REQUEST_QUEUE_CAPACITY: usize = 32;
const INCAR_SYNC_REQUESTS_PER_PUMP: usize = 4;
const PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY: usize = 32;
const PASSENGER_SYNC_REQUESTS_PER_PUMP: usize = 4;
const TRAILER_SYNC_REQUEST_QUEUE_CAPACITY: usize = 32;
const TRAILER_SYNC_REQUESTS_PER_PUMP: usize = 4;
const AIM_SYNC_REQUEST_QUEUE_CAPACITY: usize = 32;
const AIM_SYNC_REQUESTS_PER_PUMP: usize = 4;
const PLAYER_INFO_REQUESTS_PER_PUMP: usize = 4;
const VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY: usize = 32;
const VEHICLE_EXISTS_REQUESTS_PER_PUMP: usize = 4;
const TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY: usize = 32;
const TEXT_LABEL_EXISTS_REQUESTS_PER_PUMP: usize = 4;
const TEXT_LABEL_REQUEST_QUEUE_CAPACITY: usize = 32;
const TEXT_LABEL_REQUESTS_PER_PUMP: usize = 4;
const TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY: usize = 32;
const TEXTDRAW_EXISTS_REQUESTS_PER_PUMP: usize = 4;
const TEXTDRAW_REQUEST_QUEUE_CAPACITY: usize = 32;
const TEXTDRAW_REQUESTS_PER_PUMP: usize = 4;
const CHAT_ENTRY_REQUEST_QUEUE_CAPACITY: usize = 32;
const CHAT_ENTRY_REQUESTS_PER_PUMP: usize = 4;
const OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY: usize = 32;
const OBJECT_EXISTS_REQUESTS_PER_PUMP: usize = 4;
const GANGZONE_REQUEST_QUEUE_CAPACITY: usize = 32;
const GANGZONE_REQUESTS_PER_PUMP: usize = 4;
const OBJECT_HANDLE_REQUEST_QUEUE_CAPACITY: usize = 32;
const OBJECT_HANDLE_REQUESTS_PER_PUMP: usize = 4;
const PICKUP_HANDLE_REQUEST_QUEUE_CAPACITY: usize = 32;
const PICKUP_HANDLE_REQUESTS_PER_PUMP: usize = 4;
const VEHICLE_HANDLE_REQUEST_QUEUE_CAPACITY: usize = 32;
const VEHICLE_HANDLE_REQUESTS_PER_PUMP: usize = 4;
const PLAYER_HANDLE_REQUEST_QUEUE_CAPACITY: usize = 32;
const PLAYER_HANDLE_REQUESTS_PER_PUMP: usize = 4;
const OBJECT_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY: usize = 16;
const OBJECT_HANDLE_REVERSE_REQUESTS_PER_PUMP: usize = 2;
const PICKUP_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY: usize = 16;
const PICKUP_HANDLE_REVERSE_REQUESTS_PER_PUMP: usize = 2;
const VEHICLE_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY: usize = 16;
const VEHICLE_HANDLE_REVERSE_REQUESTS_PER_PUMP: usize = 2;
const PLAYER_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY: usize = 16;
const PLAYER_HANDLE_REVERSE_REQUESTS_PER_PUMP: usize = 2;
const MAX_SAMP_PLAYERS: usize = 1004;
const MAX_SAMP_VEHICLES: usize = 2000;
const MAX_SAMP_TEXT_LABELS: usize = 2048;
const MAX_SAMP_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
const MAX_SAMP_TEXTDRAWS: usize = 2304;
const MAX_TEXTDRAW_CREATE_TEXT_BYTES: usize = 800;
const MAX_CHAT_ENTRIES: usize = 100;
const MAX_SAMP_OBJECTS: usize = 2100;
const MAX_SAMP_PICKUPS: usize = 4096;
const MAX_SAMP_GANGZONES: usize = 1024;
const R1_CONNECTED_GAME_STATE: i32 = 14;

/// GTA SA 1.0 US `CGame::Process`. This target is independent of SA-MP's
/// module base and is supported only for the fixed GTA executable selected by
/// the host's R1/GTA configuration.
const GTA_SA_10_US_CGAME_PROCESS: usize = 0x53BEE0;

fn try_lock_direct<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, DirectClientError> {
    match mutex.try_lock() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::WouldBlock) => Err(DirectClientError::Busy),
        Err(TryLockError::Poisoned(_)) => Err(DirectClientError::NotReady),
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum ClientHookInstallState {
    Pending,
    Ready,
    Failed,
}

impl ClientHookInstallState {
    const fn as_raw(self) -> u32 {
        self as u32
    }

    const fn from_raw(value: u32) -> Self {
        if value == Self::Ready.as_raw() {
            Self::Ready
        } else if value == Self::Failed.as_raw() {
            Self::Failed
        } else {
            Self::Pending
        }
    }

    const fn as_public(self) -> ClientHookStatus {
        match self {
            Self::Pending => ClientHookStatus::Pending,
            Self::Ready => ClientHookStatus::Ready,
            Self::Failed => ClientHookStatus::Failed,
        }
    }
}

pub(crate) struct Backend {
    state: Arc<BackendState>,
}

/// Immutable configuration captured when the native backend attaches.
struct BackendContext {
    registry: Arc<Registry>,
    module_base: usize,
    version: SampVersion,
    addresses: AddressSet,
    native_client_profile: Option<NativeClientProfile>,
    native_profile: Option<NativeProfile>,
}

impl BackendContext {
    /// Returns the narrow fixed-layout profile permitted to publish cached
    /// CNetGame scalars on this build.
    fn scalar_profile(&self) -> Option<NativeProfile> {
        self.native_profile
    }

    fn connection_profile(&self) -> Option<NativeClientProfile> {
        self.native_client_profile
    }
}

struct BackendState {
    context: BackendContext,
    rak_client: AtomicUsize,
    raw_player_pool: AtomicUsize,
    raw_vehicle_pool: AtomicUsize,
    raw_local_player: AtomicUsize,
    rpc_receiver: AtomicUsize,
    player_address: AtomicU32,
    player_port: AtomicU16,
    constructor_trampoline: AtomicUsize,
    incoming_rpc_trampoline: AtomicUsize,
    game_process_trampoline: AtomicUsize,
    dialog_close_trampoline: AtomicUsize,
    game_thread_id: AtomicU32,
    outgoing_packet_original: AtomicUsize,
    incoming_packet_original: AtomicUsize,
    deallocate_packet_original: AtomicUsize,
    outgoing_rpc_original: AtomicUsize,
    client_hook_status: AtomicU32,
    incoming_packet_diagnostic_logged: AtomicBool,
    game_process_diagnostic_logged: AtomicBool,
    game_command_snapshot_diagnostic_logged: AtomicBool,
    game_command_completion_diagnostic_logged: AtomicBool,
    string_codec: Mutex<()>,
    game_commands: CommandQueue<GameCommand, ()>,
    auto_text_label_creates: Mutex<HashMap<CommandId, Option<u16>>>,
    local_player_snapshot: Mutex<Option<LocalPlayerSnapshot>>,
    local_player_candidate: Mutex<Option<LocalPlayerSnapshot>>,
    player_info_cache: Mutex<Vec<PlayerInfoCacheEntry>>,
    player_info_requests: Mutex<VecDeque<u16>>,
    remote_player_state_cache: Mutex<Vec<RemotePlayerStateCacheEntry>>,
    remote_player_state_requests: Mutex<VecDeque<u16>>,
    marker_sync_positions: Mutex<Vec<Option<Vector3>>>,
    streamed_out_player_position_cache: Mutex<Vec<StreamedOutPlayerPositionCacheEntry>>,
    streamed_out_player_position_requests: Mutex<VecDeque<u16>>,
    onfoot_sync_cache: Mutex<Vec<OnFootSyncCacheEntry>>,
    onfoot_sync_requests: Mutex<VecDeque<u16>>,
    incar_sync_cache: Mutex<Vec<InCarSyncCacheEntry>>,
    incar_sync_requests: Mutex<VecDeque<u16>>,
    passenger_sync_cache: Mutex<Vec<PassengerSyncCacheEntry>>,
    passenger_sync_requests: Mutex<VecDeque<u16>>,
    trailer_sync_cache: Mutex<Vec<TrailerSyncCacheEntry>>,
    trailer_sync_requests: Mutex<VecDeque<u16>>,
    aim_sync_cache: Mutex<Vec<AimSyncCacheEntry>>,
    aim_sync_requests: Mutex<VecDeque<u16>>,
    vehicle_exists_cache: Mutex<Vec<VehicleExistsCacheEntry>>,
    vehicle_exists_requests: Mutex<VecDeque<u16>>,
    text_label_exists_cache: Mutex<Vec<TextLabelExistsCacheEntry>>,
    text_label_exists_requests: Mutex<VecDeque<u16>>,
    text_label_cache: Mutex<Vec<TextLabelCacheEntry>>,
    text_label_requests: Mutex<VecDeque<u16>>,
    textdraw_exists_cache: Mutex<Vec<TextdrawExistsCacheEntry>>,
    textdraw_exists_requests: Mutex<VecDeque<u16>>,
    textdraw_cache: Mutex<Vec<TextdrawCacheEntry>>,
    textdraw_requests: Mutex<VecDeque<u16>>,
    chat_entry_cache: Mutex<Vec<ChatEntryCacheEntry>>,
    chat_entry_requests: Mutex<VecDeque<u16>>,
    object_exists_cache: Mutex<Vec<ObjectExistsCacheEntry>>,
    object_exists_requests: Mutex<VecDeque<u16>>,
    gangzone_cache: Mutex<Vec<GangzoneCacheEntry>>,
    gangzone_requests: Mutex<VecDeque<u16>>,
    object_handle_cache: Mutex<Vec<HandleCacheEntry>>,
    object_handle_requests: Mutex<VecDeque<u16>>,
    object_handle_reverse_cache: Mutex<HashMap<i32, Option<u16>>>,
    object_handle_reverse_requests: Mutex<VecDeque<i32>>,
    pickup_handle_cache: Mutex<Vec<HandleCacheEntry>>,
    pickup_handle_requests: Mutex<VecDeque<u16>>,
    pickup_handle_reverse_cache: Mutex<HashMap<i32, Option<u16>>>,
    pickup_handle_reverse_requests: Mutex<VecDeque<i32>>,
    vehicle_handle_cache: Mutex<Vec<HandleCacheEntry>>,
    vehicle_handle_requests: Mutex<VecDeque<u16>>,
    vehicle_handle_reverse_cache: Mutex<HashMap<i32, Option<u16>>>,
    vehicle_handle_reverse_requests: Mutex<VecDeque<i32>>,
    player_handle_cache: Mutex<Vec<HandleCacheEntry>>,
    player_handle_requests: Mutex<VecDeque<u16>>,
    player_handle_reverse_cache: Mutex<HashMap<i32, Option<u16>>>,
    player_handle_reverse_requests: Mutex<VecDeque<i32>>,
    player_count_including_npcs: AtomicI32,
    player_count_excluding_npcs: AtomicI32,
    player_count_ready: AtomicBool,
    player_max_id: AtomicI32,
    player_max_id_ready: AtomicBool,
    server_info_snapshot: Mutex<Option<ServerInfoSnapshot>>,
    samp_game_state: AtomicI32,
    samp_game_state_ready: AtomicBool,
    local_chat_display_mode: AtomicI32,
    local_chat_display_mode_ready: AtomicBool,
    local_cursor_mode: AtomicI32,
    local_cursor_mode_ready: AtomicBool,
    local_scoreboard_open: AtomicBool,
    local_scoreboard_open_ready: AtomicBool,
    local_dialog_active: AtomicBool,
    local_dialog_active_ready: AtomicBool,
    local_dialog_snapshot: Mutex<Option<LocalDialogSnapshot>>,
    local_dialog_snapshot_ready: AtomicBool,
    local_dialog_response: Mutex<Option<LocalDialogResponseSnapshot>>,
    local_chat_input_active: AtomicBool,
    local_chat_input_active_ready: AtomicBool,
    local_chat_input_text: Mutex<Option<Vec<u8>>>,
    local_chat_input_text_ready: AtomicBool,
    local_chat_input_commands: Mutex<Option<Vec<Vec<u8>>>>,
    local_chat_input_commands_ready: AtomicBool,
    animation_catalog: Mutex<Option<Vec<AnimationSnapshot>>>,
    cache_generation: AtomicU64,
    hooks: Mutex<HookStorage>,
}

impl std::ops::Deref for BackendState {
    type Target = BackendContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

#[derive(Clone)]
enum PlayerInfoCacheEntry {
    Unknown,
    Known(Option<PlayerInfoSnapshot>),
}

#[derive(Clone, Copy)]
enum RemotePlayerStateCacheEntry {
    Unknown,
    Known(Option<RemotePlayerStateSnapshot>),
}

#[derive(Clone, Copy)]
enum StreamedOutPlayerPositionCacheEntry {
    Unknown,
    Known(Option<Vector3>),
}

#[derive(Clone, Copy)]
enum OnFootSyncCacheEntry {
    Unknown,
    Known(Option<OnFootSyncSnapshot>),
}

#[derive(Clone, Copy)]
enum InCarSyncCacheEntry {
    Unknown,
    Known(Option<InCarSyncSnapshot>),
}

#[derive(Clone, Copy)]
enum PassengerSyncCacheEntry {
    Unknown,
    Known(Option<PassengerSyncSnapshot>),
}

#[derive(Clone, Copy)]
enum TrailerSyncCacheEntry {
    Unknown,
    Known(Option<TrailerSyncSnapshot>),
}
#[derive(Clone, Copy)]
enum AimSyncCacheEntry {
    Unknown,
    Known(Option<AimSyncSnapshot>),
}

#[derive(Clone, Copy)]
enum VehicleExistsCacheEntry {
    Unknown,
    Known(bool),
}

#[derive(Clone, Copy)]
enum TextLabelExistsCacheEntry {
    Unknown,
    Known(bool),
}

#[derive(Clone)]
enum TextLabelCacheEntry {
    Unknown,
    Known(Option<TextLabelSnapshot>),
}

#[derive(Clone, Copy)]
enum TextdrawExistsCacheEntry {
    Unknown,
    Known(bool),
}

#[derive(Clone)]
enum TextdrawCacheEntry {
    Unknown,
    Known(Option<TextdrawSnapshot>),
}

#[derive(Clone)]
enum ChatEntryCacheEntry {
    Unknown,
    Known(ChatEntrySnapshot),
}

#[derive(Clone, Copy)]
enum ObjectExistsCacheEntry {
    Unknown,
    Known(bool),
}

#[derive(Clone)]
enum GangzoneCacheEntry {
    Unknown,
    Known(Option<GangzoneSnapshot>),
}

#[derive(Clone, Copy)]
enum HandleCacheEntry {
    Unknown,
    Known(Option<i32>),
}

#[derive(Debug)]
enum GameCommand {
    ShowDialog(LocalDialogRequest),
    AddChatMessage(LocalChatMessageRequest),
    AddDeathMessage(LocalDeathMessageRequest),
    CloseDialog(u8),
    SetChatInputText(Vec<u8>),
    SetChatInputEnabled(bool),
    ProcessChatInput(Vec<u8>),
    RegisterChatCommand {
        subscription: u64,
        slot: u8,
        name: Vec<u8>,
    },
    UnregisterChatCommand {
        subscription: u64,
        name: Vec<u8>,
    },
    SetChatDisplayMode(i32),
    SetChatEntry {
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    },
    SetCursorMode(i32),
    ToggleCursor(bool),
    SetScoreboardOpen(bool),
    SetDialogClientSide(bool),
    SetDialogSelectedItem(i32),
    SetDialogEditboxText(Vec<u8>),
    SetGameState(i32),
    ConnectToServer {
        address: Vec<u8>,
        port: u16,
    },
    DisconnectWithReason(u32),
    DeleteTextLabel(u16),
    CreateTextLabel {
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    },
    CreateTextLabelAuto {
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    },
    SetTextLabelText {
        id: u16,
        text: Vec<u8>,
    },
    CreateTextdraw {
        id: u16,
        text: Vec<u8>,
        x: f32,
        y: f32,
    },
    DeleteTextdraw(u16),
    SetTextdrawPosition {
        id: u16,
        x: f32,
        y: f32,
    },
    SetTextdrawStyle {
        id: u16,
        style: i32,
    },
    SetTextdrawLetterStyle {
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    },
    SetTextdrawProportional {
        id: u16,
        proportional: bool,
    },
    SetTextdrawShadow {
        id: u16,
        shadow: u8,
        colour: u32,
    },
    SetTextdrawOutline {
        id: u16,
        outline: u8,
        colour: u32,
    },
    SetTextdrawBox {
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    },
    SetTextdrawAlignment {
        id: u16,
        alignment: u8,
    },
    SetTextdrawString {
        id: u16,
        text: Vec<u8>,
    },
    SetTextdrawModelStyle {
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    },
    SpawnLocalPlayer,
    SetLocalPlayerSpecialAction(u8),
    SetLocalPlayerName(Vec<u8>),
    ForceUnoccupiedSync {
        vehicle: u16,
        seat: u8,
    },
    ForceAimSync,
    ForceOnfootSync,
    ForceStatsSync,
    ForceTrailerSync {
        trailer: u16,
    },
    ForcePassengerSync {
        vehicle: u16,
        seat: u8,
    },
    ForceWeaponsSync,
    ForceVehicleSync {
        vehicle: u16,
    },
    SetPlayerColour {
        id: u16,
        colour: u32,
    },
    SetSendRate {
        kind: u8,
        milliseconds: u32,
    },
    SendPacket {
        id: u8,
        payload: BitStream,
        options: SendOptions,
    },
    SendRpc {
        id: u8,
        payload: BitStream,
        options: SendOptions,
    },
    EmulateIncomingPacket {
        id: u8,
        payload: BitStream,
    },
    EmulateIncomingRpc {
        id: u8,
        payload: BitStream,
    },
}

static ACTIVE_BACKEND: OnceLock<Mutex<Option<Weak<BackendState>>>> = OnceLock::new();

pub(crate) fn attach(registry: Arc<Registry>) -> Result<Backend, AttachError> {
    let module_base = loaded_samp_module()?;
    let entry_point = unsafe { pe_entry_point(module_base)? };
    let version = SampVersion::from_entry_point(entry_point)
        .ok_or(AttachError::UnsupportedClient { entry_point })?;
    let addresses = AddressSet::for_version(version);
    let selected_native_profile = NativeClientProfile::select(module_base, version, entry_point);
    if let Some(profile) = selected_native_profile {
        log::info!(
            "{} direct client helpers are enabled",
            profile.spec.identity.name
        );
    }
    let native_profile = selected_native_profile.map(NativeProfile::from_native_client_profile);

    let active = ACTIVE_BACKEND.get_or_init(|| Mutex::new(None));
    let mut active = active.lock().unwrap_or_else(|error| error.into_inner());
    if active.as_ref().and_then(Weak::upgrade).is_some() {
        return Err(AttachError::AlreadyAttached);
    }

    let state = Arc::new(BackendState {
        context: BackendContext {
            registry,
            module_base,
            version,
            addresses,
            native_client_profile: selected_native_profile,
            native_profile,
        },
        rak_client: AtomicUsize::new(0),
        raw_player_pool: AtomicUsize::new(0),
        raw_vehicle_pool: AtomicUsize::new(0),
        raw_local_player: AtomicUsize::new(0),
        rpc_receiver: AtomicUsize::new(0),
        player_address: AtomicU32::new(0),
        player_port: AtomicU16::new(0),
        constructor_trampoline: AtomicUsize::new(0),
        incoming_rpc_trampoline: AtomicUsize::new(0),
        game_process_trampoline: AtomicUsize::new(0),
        dialog_close_trampoline: AtomicUsize::new(0),
        game_thread_id: AtomicU32::new(0),
        outgoing_packet_original: AtomicUsize::new(0),
        incoming_packet_original: AtomicUsize::new(0),
        deallocate_packet_original: AtomicUsize::new(0),
        outgoing_rpc_original: AtomicUsize::new(0),
        client_hook_status: AtomicU32::new(ClientHookInstallState::Pending.as_raw()),
        incoming_packet_diagnostic_logged: AtomicBool::new(false),
        game_process_diagnostic_logged: AtomicBool::new(false),
        game_command_snapshot_diagnostic_logged: AtomicBool::new(false),
        game_command_completion_diagnostic_logged: AtomicBool::new(false),
        string_codec: Mutex::new(()),
        game_commands: CommandQueue::new(),
        auto_text_label_creates: Mutex::new(HashMap::new()),
        local_player_snapshot: Mutex::new(None),
        local_player_candidate: Mutex::new(None),
        player_info_cache: Mutex::new(vec![PlayerInfoCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        player_info_requests: Mutex::new(VecDeque::with_capacity(
            PLAYER_INFO_REQUEST_QUEUE_CAPACITY,
        )),
        remote_player_state_cache: Mutex::new(vec![
            RemotePlayerStateCacheEntry::Unknown;
            MAX_SAMP_PLAYERS
        ]),
        remote_player_state_requests: Mutex::new(VecDeque::with_capacity(
            REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY,
        )),
        marker_sync_positions: Mutex::new(vec![None; MAX_SAMP_PLAYERS]),
        streamed_out_player_position_cache: Mutex::new(vec![
            StreamedOutPlayerPositionCacheEntry::Unknown;
            MAX_SAMP_PLAYERS
        ]),
        streamed_out_player_position_requests: Mutex::new(VecDeque::with_capacity(
            STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY,
        )),
        onfoot_sync_cache: Mutex::new(vec![OnFootSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        onfoot_sync_requests: Mutex::new(VecDeque::with_capacity(
            ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY,
        )),
        incar_sync_cache: Mutex::new(vec![InCarSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        incar_sync_requests: Mutex::new(VecDeque::with_capacity(INCAR_SYNC_REQUEST_QUEUE_CAPACITY)),
        passenger_sync_cache: Mutex::new(vec![PassengerSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        passenger_sync_requests: Mutex::new(VecDeque::with_capacity(
            PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY,
        )),
        trailer_sync_cache: Mutex::new(vec![TrailerSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        trailer_sync_requests: Mutex::new(VecDeque::with_capacity(
            TRAILER_SYNC_REQUEST_QUEUE_CAPACITY,
        )),
        aim_sync_cache: Mutex::new(vec![AimSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        aim_sync_requests: Mutex::new(VecDeque::with_capacity(AIM_SYNC_REQUEST_QUEUE_CAPACITY)),
        vehicle_exists_cache: Mutex::new(vec![VehicleExistsCacheEntry::Unknown; MAX_SAMP_VEHICLES]),
        vehicle_exists_requests: Mutex::new(VecDeque::with_capacity(
            VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        text_label_exists_cache: Mutex::new(vec![
            TextLabelExistsCacheEntry::Unknown;
            MAX_SAMP_TEXT_LABELS
        ]),
        text_label_exists_requests: Mutex::new(VecDeque::with_capacity(
            TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        text_label_cache: Mutex::new(vec![TextLabelCacheEntry::Unknown; MAX_SAMP_TEXT_LABELS]),
        text_label_requests: Mutex::new(VecDeque::with_capacity(TEXT_LABEL_REQUEST_QUEUE_CAPACITY)),
        textdraw_exists_cache: Mutex::new(vec![
            TextdrawExistsCacheEntry::Unknown;
            MAX_SAMP_TEXTDRAWS
        ]),
        textdraw_exists_requests: Mutex::new(VecDeque::with_capacity(
            TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        textdraw_cache: Mutex::new(vec![TextdrawCacheEntry::Unknown; MAX_SAMP_TEXTDRAWS]),
        textdraw_requests: Mutex::new(VecDeque::with_capacity(TEXTDRAW_REQUEST_QUEUE_CAPACITY)),
        chat_entry_cache: Mutex::new(vec![ChatEntryCacheEntry::Unknown; MAX_CHAT_ENTRIES]),
        chat_entry_requests: Mutex::new(VecDeque::with_capacity(CHAT_ENTRY_REQUEST_QUEUE_CAPACITY)),
        object_exists_cache: Mutex::new(vec![ObjectExistsCacheEntry::Unknown; MAX_SAMP_OBJECTS]),
        object_exists_requests: Mutex::new(VecDeque::with_capacity(
            OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY,
        )),
        gangzone_cache: Mutex::new(vec![GangzoneCacheEntry::Unknown; MAX_SAMP_GANGZONES]),
        gangzone_requests: Mutex::new(VecDeque::with_capacity(GANGZONE_REQUEST_QUEUE_CAPACITY)),
        object_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_OBJECTS]),
        object_handle_requests: Mutex::new(VecDeque::with_capacity(
            OBJECT_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        object_handle_reverse_cache: Mutex::new(HashMap::new()),
        object_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            OBJECT_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        pickup_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_PICKUPS]),
        pickup_handle_requests: Mutex::new(VecDeque::with_capacity(
            PICKUP_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        pickup_handle_reverse_cache: Mutex::new(HashMap::new()),
        pickup_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            PICKUP_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        vehicle_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_VEHICLES]),
        vehicle_handle_requests: Mutex::new(VecDeque::with_capacity(
            VEHICLE_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        vehicle_handle_reverse_cache: Mutex::new(HashMap::new()),
        vehicle_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            VEHICLE_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        player_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        player_handle_requests: Mutex::new(VecDeque::with_capacity(
            PLAYER_HANDLE_REQUEST_QUEUE_CAPACITY,
        )),
        player_handle_reverse_cache: Mutex::new(HashMap::new()),
        player_handle_reverse_requests: Mutex::new(VecDeque::with_capacity(
            PLAYER_HANDLE_REVERSE_REQUEST_QUEUE_CAPACITY,
        )),
        player_count_including_npcs: AtomicI32::new(0),
        player_count_excluding_npcs: AtomicI32::new(0),
        player_count_ready: AtomicBool::new(false),
        player_max_id: AtomicI32::new(0),
        player_max_id_ready: AtomicBool::new(false),
        server_info_snapshot: Mutex::new(None),
        samp_game_state: AtomicI32::new(0),
        samp_game_state_ready: AtomicBool::new(false),
        local_chat_display_mode: AtomicI32::new(0),
        local_chat_display_mode_ready: AtomicBool::new(false),
        local_cursor_mode: AtomicI32::new(0),
        local_cursor_mode_ready: AtomicBool::new(false),
        local_scoreboard_open: AtomicBool::new(false),
        local_scoreboard_open_ready: AtomicBool::new(false),
        local_dialog_active: AtomicBool::new(false),
        local_dialog_active_ready: AtomicBool::new(false),
        local_dialog_snapshot: Mutex::new(None),
        local_dialog_snapshot_ready: AtomicBool::new(false),
        local_dialog_response: Mutex::new(None),
        local_chat_input_active: AtomicBool::new(false),
        local_chat_input_active_ready: AtomicBool::new(false),
        local_chat_input_text: Mutex::new(None),
        local_chat_input_text_ready: AtomicBool::new(false),
        local_chat_input_commands: Mutex::new(None),
        local_chat_input_commands_ready: AtomicBool::new(false),
        animation_catalog: Mutex::new(None),
        cache_generation: AtomicU64::new(0),
        hooks: Mutex::new(HookStorage::default()),
    });
    *active = Some(Arc::downgrade(&state));
    drop(active);

    if let Err(error) = state.install_game_process_hook() {
        clear_active_backend(&state);
        return Err(error);
    }
    if let Err(error) = state.install_dialog_close_hook() {
        state.shutdown();
        return Err(error);
    }
    if let Err(error) = state.install_constructor_hook() {
        state.shutdown();
        return Err(error);
    }
    Ok(Backend { state })
}

impl BackendState {
    fn install_game_process_hook(&self) -> Result<(), AttachError> {
        let (mut detour, trampoline) = InlineHook::create(
            "CGame::Process",
            GTA_SA_10_US_CGAME_PROCESS,
            hooks::game_process_detour as *const () as usize,
        )
        .map_err(|_| AttachError::HookInstallFailed("CGame::Process detour"))?;
        self.game_process_trampoline
            .store(trampoline, Ordering::Release);
        if detour.enable().is_err() {
            self.game_process_trampoline.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling CGame::Process detour",
            ));
        }
        self.hooks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .game_process = Some(detour);
        Ok(())
    }

    fn install_dialog_close_hook(&self) -> Result<(), AttachError> {
        let Some(profile) = self.connection_profile() else {
            return Ok(());
        };
        let target = profile
            .dialog_close_target()
            .ok_or(AttachError::HookInstallFailed("CDialog::Close target"))?;
        let (mut detour, trampoline) = InlineHook::create(
            "CDialog::Close",
            target,
            hooks::dialog_close_detour as *const () as usize,
        )
        .map_err(|_| AttachError::HookInstallFailed("CDialog::Close detour"))?;
        self.dialog_close_trampoline
            .store(trampoline, Ordering::Release);
        if detour.enable().is_err() {
            self.dialog_close_trampoline.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling CDialog::Close detour",
            ));
        }
        self.hooks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .dialog_close = Some(detour);
        Ok(())
    }

    fn install_constructor_hook(self: &Arc<Self>) -> Result<(), AttachError> {
        let target = self.module_base + self.addresses.rak_client_constructor as usize;
        let (mut detour, trampoline) = InlineHook::create(
            "RakClient constructor",
            target,
            hooks::rak_client_constructor_detour as *const () as usize,
        )
        .map_err(|_| AttachError::HookInstallFailed("RakClient constructor detour"))?;
        self.constructor_trampoline
            .store(trampoline, Ordering::Release);
        if detour.enable().is_err() {
            self.constructor_trampoline.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling RakClient constructor detour",
            ));
        }
        self.hooks
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .constructor = Some(detour);
        Ok(())
    }

    fn install_client_hooks(&self, client: *mut c_void) -> Result<(), AttachError> {
        if client.is_null() {
            return Err(AttachError::ClientNotReady);
        }
        if self
            .rak_client
            .compare_exchange(0, client as usize, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }

        let incoming_target = self.module_base + self.addresses.incoming_rpc_handler as usize;
        let (mut incoming_rpc, trampoline) = InlineHook::create(
            "RakClient::HandleRPCPacket",
            incoming_target,
            hooks::incoming_rpc_detour as *const () as usize,
        )
        .map_err(|_| {
            self.rak_client.store(0, Ordering::Release);
            AttachError::HookInstallFailed("incoming RPC detour")
        })?;
        self.incoming_rpc_trampoline
            .store(trampoline, Ordering::Release);
        if incoming_rpc.enable().is_err() {
            self.incoming_rpc_trampoline.store(0, Ordering::Release);
            self.rak_client.store(0, Ordering::Release);
            return Err(AttachError::HookInstallFailed(
                "enabling incoming RPC detour",
            ));
        }

        let vtable = match unsafe { VtableHook::install(client, self) } {
            Ok(vtable) => vtable,
            Err(error) => {
                incoming_rpc.disable();
                self.incoming_rpc_trampoline.store(0, Ordering::Release);
                self.rak_client.store(0, Ordering::Release);
                self.outgoing_packet_original.store(0, Ordering::Release);
                self.incoming_packet_original.store(0, Ordering::Release);
                self.deallocate_packet_original.store(0, Ordering::Release);
                self.outgoing_rpc_original.store(0, Ordering::Release);
                return Err(error);
            }
        };
        let mut hooks = self.hooks.lock().unwrap_or_else(|error| error.into_inner());
        hooks.incoming_rpc = Some(incoming_rpc);
        hooks.vtable = Some(vtable);
        self.client_hook_status
            .store(ClientHookInstallState::Ready.as_raw(), Ordering::Release);
        Ok(())
    }

    fn prepare_game_tick(&self) -> Option<Vec<QueuedCommand<GameCommand>>> {
        (self.rak_client.load(Ordering::Acquire) != 0)
            .then(|| self.game_commands.take_tick_snapshot())
    }

    /// Executes one post-process game tick. `commands` is captured before the
    /// native process call, so submissions made while that call or this drain
    /// is running remain owned by the following tick.
    fn pump_game_tick(&self, commands: Vec<QueuedCommand<GameCommand>>) {
        self.execute_game_commands(commands);
        let Some(profile) = self.scalar_profile() else {
            return;
        };
        // Odd generations are in-flight. Readers only observe the next even
        // generation after every cache path below has had one tick to refresh.
        self.cache_generation.fetch_add(1, Ordering::AcqRel);
        let connection_profile = self.connection_profile();
        if let Some(connection_profile) = connection_profile {
            self.refresh_samp_game_state(connection_profile);
            self.refresh_server_info_snapshot(connection_profile);
            self.refresh_player_info(connection_profile);
            self.refresh_remote_player_state(connection_profile);
            self.refresh_streamed_out_player_position(connection_profile);
            self.refresh_onfoot_sync(connection_profile);
            self.refresh_incar_sync(connection_profile);
            self.refresh_passenger_sync(connection_profile);
            self.refresh_trailer_sync(connection_profile);
            self.refresh_aim_sync(connection_profile);
            self.refresh_vehicle_exists(connection_profile);
            self.refresh_object_exists(connection_profile);
            self.refresh_gangzones(connection_profile);
            self.refresh_object_handles(connection_profile);
            self.refresh_pickup_handles(connection_profile);
            self.refresh_vehicle_handles(connection_profile);
            self.refresh_player_handles(connection_profile);
            self.refresh_object_handle_ids(connection_profile);
            self.refresh_pickup_handle_ids(connection_profile);
            self.refresh_vehicle_handle_ids(connection_profile);
            self.refresh_player_handle_ids(connection_profile);
            self.refresh_local_chat_display_mode(connection_profile);
            self.refresh_local_cursor_mode(connection_profile);
            self.refresh_local_scoreboard_open(connection_profile);
            self.refresh_local_dialog_active(connection_profile);
            self.refresh_local_dialog_state(connection_profile);
            self.refresh_local_chat_input_active(connection_profile);
            self.refresh_local_chat_input_commands(connection_profile);
            self.refresh_local_chat_input_text(connection_profile);
            self.refresh_chat_entries(connection_profile);
            self.refresh_text_label_exists(connection_profile);
            self.refresh_text_labels(connection_profile);
        }
        self.refresh_local_player_snapshot(connection_profile);
        if let Some(connection_profile) = connection_profile {
            self.refresh_player_count(connection_profile);
            self.refresh_player_max_id(connection_profile);
        } else {
            self.player_count_ready.store(false, Ordering::Release);
            self.player_max_id_ready.store(false, Ordering::Release);
        }
        self.refresh_animation_catalog(profile);
        if let Some(connection_profile) = self.connection_profile() {
            self.refresh_raw_pool_addresses(connection_profile);
            self.refresh_textdraw_exists(connection_profile);
            self.refresh_textdraws(connection_profile);
        }
        self.cache_generation.fetch_add(1, Ordering::Release);
    }

    fn cache_local_player_snapshot(&self, snapshot: Option<LocalPlayerSnapshot>) {
        let Ok(mut candidate) = self.local_player_candidate.try_lock() else {
            return;
        };
        let Ok(mut cached) = self.local_player_snapshot.try_lock() else {
            return;
        };

        let Some(snapshot) = snapshot else {
            *candidate = None;
            *cached = None;
            return;
        };

        match cached.as_ref() {
            Some(current) if current.id == snapshot.id => {
                *cached = Some(snapshot);
                *candidate = None;
            }
            Some(_) => {
                *cached = None;
                *candidate = Some(snapshot);
            }
            None if candidate
                .as_ref()
                .is_some_and(|prior| prior.id == snapshot.id) =>
            {
                *cached = Some(snapshot);
                *candidate = None;
            }
            None => *candidate = Some(snapshot),
        }
    }

    fn clear_player_info_cache(&self) {
        if let Ok(mut cache) = self.player_info_cache.try_lock() {
            cache.fill(PlayerInfoCacheEntry::Unknown);
        }
    }

    fn clear_remote_player_state_cache(&self) {
        if let Ok(mut cache) = self.remote_player_state_cache.try_lock() {
            cache.fill(RemotePlayerStateCacheEntry::Unknown);
        }
    }

    fn clear_streamed_out_player_position_cache(&self) {
        if let Ok(mut cache) = self.streamed_out_player_position_cache.try_lock() {
            cache.fill(StreamedOutPlayerPositionCacheEntry::Unknown);
        }
    }

    fn clear_marker_sync_positions(&self) {
        if let Ok(mut positions) = self.marker_sync_positions.try_lock() {
            positions.fill(None);
        }
    }

    fn clear_onfoot_sync_cache(&self) {
        if let Ok(mut cache) = self.onfoot_sync_cache.try_lock() {
            cache.fill(OnFootSyncCacheEntry::Unknown);
        }
    }

    fn clear_incar_sync_cache(&self) {
        if let Ok(mut cache) = self.incar_sync_cache.try_lock() {
            cache.fill(InCarSyncCacheEntry::Unknown);
        }
    }

    fn clear_passenger_sync_cache(&self) {
        if let Ok(mut cache) = self.passenger_sync_cache.try_lock() {
            cache.fill(PassengerSyncCacheEntry::Unknown);
        }
    }

    fn clear_trailer_sync_cache(&self) {
        if let Ok(mut cache) = self.trailer_sync_cache.try_lock() {
            cache.fill(TrailerSyncCacheEntry::Unknown);
        }
    }
    fn clear_aim_sync_cache(&self) {
        if let Ok(mut cache) = self.aim_sync_cache.try_lock() {
            cache.fill(AimSyncCacheEntry::Unknown);
        }
    }

    fn clear_vehicle_exists_cache(&self) {
        if let Ok(mut cache) = self.vehicle_exists_cache.try_lock() {
            cache.fill(VehicleExistsCacheEntry::Unknown);
        }
    }

    fn clear_text_label_exists_cache(&self) {
        if let Ok(mut cache) = self.text_label_exists_cache.try_lock() {
            cache.fill(TextLabelExistsCacheEntry::Unknown);
        }
    }

    fn clear_text_label_cache(&self) {
        if let Ok(mut cache) = self.text_label_cache.try_lock() {
            cache.fill(TextLabelCacheEntry::Unknown);
        }
    }

    fn clear_textdraw_exists_cache(&self) {
        if let Ok(mut cache) = self.textdraw_exists_cache.try_lock() {
            cache.fill(TextdrawExistsCacheEntry::Unknown);
        }
    }

    fn clear_textdraw_cache(&self) {
        if let Ok(mut cache) = self.textdraw_cache.try_lock() {
            cache.fill(TextdrawCacheEntry::Unknown);
        }
    }

    fn clear_chat_entry_cache(&self) {
        if let Ok(mut cache) = self.chat_entry_cache.try_lock() {
            cache.fill(ChatEntryCacheEntry::Unknown);
        }
    }

    fn clear_object_exists_cache(&self) {
        if let Ok(mut cache) = self.object_exists_cache.try_lock() {
            cache.fill(ObjectExistsCacheEntry::Unknown);
        }
    }

    fn clear_gangzone_cache(&self) {
        if let Ok(mut cache) = self.gangzone_cache.try_lock() {
            cache.fill(GangzoneCacheEntry::Unknown);
        }
    }

    fn clear_handle_cache(&self, cache: &Mutex<Vec<HandleCacheEntry>>) {
        if let Ok(mut cache) = cache.try_lock() {
            cache.fill(HandleCacheEntry::Unknown);
        }
    }

    fn invalidate_after_disconnect(&self) {
        self.rpc_receiver.store(0, Ordering::Release);
        self.player_address.store(0, Ordering::Release);
        self.player_port.store(0, Ordering::Release);
        self.invalidate_connection_state();
    }

    /// Invalidates every cache tied to one server connection. This runs on the
    /// game thread at a connection boundary and intentionally acquires each
    /// host cache lock: serving a prior server's entity data is worse than a
    /// short first-read `NotReady` while a plugin finishes copying a snapshot.
    fn invalidate_connection_state(&self) {
        self.raw_player_pool.store(0, Ordering::Release);
        self.raw_vehicle_pool.store(0, Ordering::Release);
        self.raw_local_player.store(0, Ordering::Release);
        *self
            .local_player_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
        *self
            .local_player_candidate
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;

        self.player_info_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(PlayerInfoCacheEntry::Unknown);
        self.player_info_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.remote_player_state_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(RemotePlayerStateCacheEntry::Unknown);
        self.remote_player_state_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.streamed_out_player_position_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(StreamedOutPlayerPositionCacheEntry::Unknown);
        self.marker_sync_positions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(None);
        self.streamed_out_player_position_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.onfoot_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(OnFootSyncCacheEntry::Unknown);
        self.onfoot_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.incar_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(InCarSyncCacheEntry::Unknown);
        self.incar_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.passenger_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(PassengerSyncCacheEntry::Unknown);
        self.passenger_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.trailer_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TrailerSyncCacheEntry::Unknown);
        self.trailer_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.aim_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(AimSyncCacheEntry::Unknown);
        self.aim_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.vehicle_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(VehicleExistsCacheEntry::Unknown);
        self.vehicle_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.text_label_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextLabelExistsCacheEntry::Unknown);
        self.text_label_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.text_label_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextLabelCacheEntry::Unknown);
        self.text_label_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.textdraw_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextdrawExistsCacheEntry::Unknown);
        self.textdraw_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.textdraw_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextdrawCacheEntry::Unknown);
        self.textdraw_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.chat_entry_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(ChatEntryCacheEntry::Unknown);
        self.chat_entry_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.object_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(ObjectExistsCacheEntry::Unknown);
        self.object_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.gangzone_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(GangzoneCacheEntry::Unknown);
        self.gangzone_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.object_handle_cache);
        self.object_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.object_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.object_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.pickup_handle_cache);
        self.pickup_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.pickup_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.pickup_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.vehicle_handle_cache);
        self.vehicle_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.vehicle_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.vehicle_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.player_handle_cache);
        self.player_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.player_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.player_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();

        self.player_count_ready.store(false, Ordering::Release);
        self.player_max_id_ready.store(false, Ordering::Release);
        *self
            .server_info_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
    }

    fn ready_client(&self) -> Result<*mut c_void, SendError> {
        let client = self.rak_client.load(Ordering::Acquire) as *mut c_void;
        if client.is_null() {
            Err(SendError::ClientNotReady)
        } else {
            Ok(client)
        }
    }

    fn ready_rpc_receiver(&self) -> Result<*mut c_void, SendError> {
        let receiver = self.rpc_receiver.load(Ordering::Acquire) as *mut c_void;
        if receiver.is_null() {
            Err(SendError::ClientNotReady)
        } else {
            Ok(receiver)
        }
    }

    fn incoming_emulation_ready(&self) -> bool {
        self.rpc_receiver.load(Ordering::Acquire) != 0
            && self.incoming_rpc_trampoline.load(Ordering::Acquire) != 0
    }

    fn cache_is_published(&self) -> bool {
        let generation = self.cache_generation.load(Ordering::Acquire);
        generation != 0 && generation.is_multiple_of(2)
    }

    fn is_game_thread(&self) -> bool {
        let game_thread = self.game_thread_id.load(Ordering::Acquire);
        game_thread != 0 && game_thread == unsafe { GetCurrentThreadId() }
    }

    unsafe fn run_game_process_tick(&self, original: GameProcessFn) {
        // Publish this before entering GTA so a plugin reached from the native
        // process path cannot block the game thread on its own command receipt.
        self.game_thread_id
            .store(unsafe { GetCurrentThreadId() }, Ordering::Release);
        let commands = self.prepare_game_tick();
        if let Some(commands) = commands.as_ref().filter(|commands| !commands.is_empty())
            && !self
                .game_command_snapshot_diagnostic_logged
                .swap(true, Ordering::AcqRel)
        {
            let first_id = commands[0].id;
            let last_id = commands.last().map_or(first_id, |command| command.id);
            // Snapshot metadata lets a live smoke prove the command crossed
            // the game-thread boundary without exposing plugin payloads.
            log::debug!(
                "captured first game command snapshot: count={}, first_id={first_id}, last_id={last_id}",
                commands.len(),
            );
        }
        unsafe { original() };
        if let Some(commands) = commands {
            self.pump_game_tick(commands);
        }
    }

    fn shutdown(&self) {
        let mut hooks = self.hooks.lock().unwrap_or_else(|error| error.into_inner());
        hooks.vtable.take();
        if let Some(detour) = hooks.game_process.take() {
            detour.disable();
        }
        if let Some(detour) = hooks.dialog_close.take() {
            detour.disable();
        }
        if let Some(detour) = hooks.incoming_rpc.take() {
            detour.disable();
        }
        if let Some(detour) = hooks.constructor.take() {
            detour.disable();
        }
        drop(hooks);

        // No new native calls can enter our detours after the vtable and inline
        // hooks have been removed. Existing detour calls hold an Arc from
        // active_state and can still reach their original functions safely.
        clear_active_backend(self);
        self.game_thread_id.store(0, Ordering::Release);
        self.dialog_close_trampoline.store(0, Ordering::Release);
        self.rak_client.store(0, Ordering::Release);
        self.raw_player_pool.store(0, Ordering::Release);
        self.raw_vehicle_pool.store(0, Ordering::Release);
        self.raw_local_player.store(0, Ordering::Release);
        self.game_commands.shutdown();
        if let Ok(mut snapshot) = self.local_player_snapshot.try_lock() {
            *snapshot = None;
        }
        if let Ok(mut candidate) = self.local_player_candidate.try_lock() {
            *candidate = None;
        }
        self.clear_player_info_cache();
        if let Ok(mut requests) = self.player_info_requests.try_lock() {
            requests.clear();
        }
        self.clear_remote_player_state_cache();
        if let Ok(mut requests) = self.remote_player_state_requests.try_lock() {
            requests.clear();
        }
        self.clear_streamed_out_player_position_cache();
        self.clear_marker_sync_positions();
        if let Ok(mut requests) = self.streamed_out_player_position_requests.try_lock() {
            requests.clear();
        }
        self.clear_onfoot_sync_cache();
        if let Ok(mut requests) = self.onfoot_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_incar_sync_cache();
        if let Ok(mut requests) = self.incar_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_passenger_sync_cache();
        if let Ok(mut requests) = self.passenger_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_trailer_sync_cache();
        if let Ok(mut requests) = self.trailer_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_aim_sync_cache();
        if let Ok(mut requests) = self.aim_sync_requests.try_lock() {
            requests.clear();
        }
        self.clear_vehicle_exists_cache();
        if let Ok(mut requests) = self.vehicle_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_text_label_exists_cache();
        if let Ok(mut requests) = self.text_label_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_text_label_cache();
        if let Ok(mut requests) = self.text_label_requests.try_lock() {
            requests.clear();
        }
        self.clear_textdraw_exists_cache();
        if let Ok(mut requests) = self.textdraw_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_textdraw_cache();
        if let Ok(mut requests) = self.textdraw_requests.try_lock() {
            requests.clear();
        }
        self.clear_chat_entry_cache();
        if let Ok(mut requests) = self.chat_entry_requests.try_lock() {
            requests.clear();
        }
        self.clear_object_exists_cache();
        if let Ok(mut requests) = self.object_exists_requests.try_lock() {
            requests.clear();
        }
        self.clear_gangzone_cache();
        if let Ok(mut requests) = self.gangzone_requests.try_lock() {
            requests.clear();
        }
        if let Ok(mut snapshot) = self.server_info_snapshot.try_lock() {
            *snapshot = None;
        }
        self.samp_game_state_ready.store(false, Ordering::Release);
        self.local_chat_display_mode_ready
            .store(false, Ordering::Release);
        self.local_cursor_mode_ready.store(false, Ordering::Release);
        self.local_scoreboard_open_ready
            .store(false, Ordering::Release);
        self.local_dialog_active_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_dialog_snapshot.try_lock() {
            *snapshot = None;
        }
        if let Ok(mut response) = self.local_dialog_response.try_lock() {
            *response = None;
        }
        self.local_dialog_snapshot_ready
            .store(false, Ordering::Release);
        self.local_chat_input_active_ready
            .store(false, Ordering::Release);
        self.local_chat_input_text_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_chat_input_text.try_lock() {
            *snapshot = None;
        }
        self.local_chat_input_commands_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_chat_input_commands.try_lock() {
            *snapshot = None;
        }
        self.player_count_ready.store(false, Ordering::Release);
        self.player_max_id_ready.store(false, Ordering::Release);
        if let Ok(mut catalog) = self.animation_catalog.try_lock() {
            *catalog = None;
        }
    }
}

fn player_info_from_local(player: &LocalPlayerSnapshot) -> PlayerInfoSnapshot {
    PlayerInfoSnapshot {
        id: player.id,
        defined: true,
        paused: false,
        nickname: player.nickname.clone(),
        is_local: true,
        is_npc: false,
        colour: player.colour,
        score: player.score,
        ping: player.ping,
    }
}

fn is_connected_game_state(game_state: i32) -> bool {
    game_state == R1_CONNECTED_GAME_STATE
}

fn command_send_error(error: CommandError) -> SendError {
    match error {
        CommandError::QueueFull => SendError::QueueFull,
        CommandError::ShuttingDown | CommandError::UnknownReceipt => SendError::ClientNotReady,
        CommandError::NativeFailure | CommandError::TimedOut | CommandError::WaitRejected => {
            SendError::NativeCallFailed
        }
    }
}

fn sent_game_command_result(sent: bool) -> Result<(), SendError> {
    sent.then_some(()).ok_or(SendError::NativeCallFailed)
}

fn crosses_connection_boundary(was_ready: bool, previous: i32, current: i32) -> bool {
    was_ready
        && previous != current
        && (is_connected_game_state(previous) || is_connected_game_state(current))
}

fn cached_direct_client_value<T>(
    profile_available: bool,
    client_available: bool,
    cache_published: bool,
    cached: Option<T>,
) -> Result<T, DirectClientError> {
    if !profile_available {
        Err(DirectClientError::UnsupportedVersion)
    } else if !client_available || !cache_published {
        Err(DirectClientError::NotReady)
    } else {
        cached.ok_or(DirectClientError::NotReady)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RpcPlayerId {
    binary_address: u32,
    port: u16,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PacketPlayerId {
    binary_address: u32,
    port: u16,
}

#[repr(C, packed)]
struct RawPacket {
    player_index: u16,
    player_id: PacketPlayerId,
    length: u32,
    bit_size: u32,
    data: *mut u8,
    delete_data: bool,
}

#[cfg(test)]
mod layout_tests;

#[cfg(test)]
mod profile_layout_tests;

#[cfg(test)]
mod vtable_tests;

#[cfg(test)]
mod inline_hook_tests;

type GameProcessFn = unsafe extern "C" fn();
type StringWriteEncoderFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, i32, *mut RawBitStream, i32);
type StringReadDecoderFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut i8, i32, *mut RawBitStream, i32) -> bool;
type OutgoingPacketFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut RawBitStream, i32, i32, i8) -> bool;
type OutgoingRpcFn = unsafe extern "thiscall" fn(
    *mut c_void,
    *mut i32,
    *mut RawBitStream,
    i32,
    i32,
    i8,
    bool,
) -> bool;
type IncomingRpcFn = unsafe extern "thiscall" fn(*mut c_void, *mut u8, i32, RpcPlayerId) -> bool;
type AllocatePacketFn = unsafe extern "C" fn(i32) -> *mut RawPacket;
type QueueWriteLockFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut *mut RawPacket;
type QueueWriteUnlockFn = unsafe extern "thiscall" fn(*mut c_void);

#[cfg(test)]
mod native_packet_bit_length_tests;

fn packet_stream(id: u8, payload: &BitStream) -> Result<BitStream, SendError> {
    let mut stream = BitStream::new();
    stream
        .write_u8(id)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_stream(payload)
        .map_err(|_| SendError::PayloadTooLarge)?;
    Ok(stream)
}

fn remaining_stream(stream: &mut BitStream, bit_len: usize) -> BitStream {
    let mut payload = BitStream::new();
    copy_remaining(stream, bit_len, &mut payload);
    payload
}

fn remaining_stream_bounded(
    stream: &mut BitStream,
    bit_len: usize,
    capacity_bits: usize,
) -> BitStream {
    let mut payload = BitStream::with_capacity_bits(capacity_bits);
    copy_remaining(stream, bit_len, &mut payload);
    payload
}

fn copy_remaining(stream: &mut BitStream, bit_len: usize, payload: &mut BitStream) {
    for _ in 0..bit_len {
        if let Ok(bit) = stream.read_bool() {
            let _ = payload.write_bool(bit);
        }
    }
}

fn active_state() -> Option<Arc<BackendState>> {
    ACTIVE_BACKEND.get().and_then(|slot| {
        slot.lock()
            .ok()
            .and_then(|state| state.as_ref().and_then(Weak::upgrade))
    })
}

fn clear_active_backend(target: &BackendState) {
    let Some(slot) = ACTIVE_BACKEND.get() else {
        return;
    };
    let mut active = slot.lock().unwrap_or_else(|error| error.into_inner());
    if active
        .as_ref()
        .and_then(Weak::upgrade)
        .is_some_and(|state| ptr::eq(Arc::as_ptr(&state), target))
    {
        *active = None;
    }
}

fn loaded_samp_module() -> Result<usize, AttachError> {
    let handle = unsafe { GetModuleHandleA(c"samp.dll".as_ptr().cast()) };
    if handle.is_null() {
        Err(AttachError::SampNotLoaded)
    } else {
        Ok(handle as usize)
    }
}

unsafe fn pe_entry_point(base: usize) -> Result<u32, AttachError> {
    let image = base as *const u8;
    if unsafe { image.cast::<u16>().read_unaligned() } != 0x5A4D {
        return Err(AttachError::UnsupportedClient { entry_point: 0 });
    }
    let nt_offset = unsafe { image.add(0x3C).cast::<u32>().read_unaligned() } as usize;
    let nt_header = unsafe { image.add(nt_offset) };
    if unsafe { nt_header.cast::<u32>().read_unaligned() } != 0x0000_4550 {
        return Err(AttachError::UnsupportedClient { entry_point: 0 });
    }
    if unsafe { nt_header.add(24).cast::<u16>().read_unaligned() } != 0x10B {
        return Err(AttachError::UnsupportedClient { entry_point: 0 });
    }
    Ok(unsafe { nt_header.add(40).cast::<u32>().read_unaligned() })
}

const fn priority_value(priority: PacketPriority) -> i32 {
    match priority {
        PacketPriority::System => 0,
        PacketPriority::High => 1,
        PacketPriority::Medium => 2,
        PacketPriority::Low => 3,
    }
}

const fn reliability_value(reliability: PacketReliability) -> i32 {
    match reliability {
        PacketReliability::Unreliable => 6,
        PacketReliability::UnreliableSequenced => 7,
        PacketReliability::Reliable => 8,
        PacketReliability::ReliableOrdered => 9,
        PacketReliability::ReliableSequenced => 10,
    }
}
