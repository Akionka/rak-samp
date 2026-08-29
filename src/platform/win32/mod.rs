//! Private Windows x86 hook implementation.
//!
//! The module is intentionally unavailable for other targets. Its only public
//! boundary is the safe `Runtime` API in the parent crate.

mod backend;
mod cache_lifecycle;
mod chat_entries;
mod commands;
use commands::GameCommand;
#[cfg(test)]
use commands::{NetworkCommand, TextLabelCommand, UiCommand};
mod gangzones;
mod handles;
mod hooks;
mod lifecycle;
mod native_abi;
mod native_bitstream;
pub(crate) mod native_client;
mod objects;
mod packets;
mod players;
mod reads;
mod refresh;
mod requests;
mod sampfuncs;
mod text_labels;
mod textdraws;
mod tick;
mod vehicles;

use cache_lifecycle::player_info_from_local;
#[cfg(test)]
use cache_lifecycle::{crosses_connection_boundary, is_connected_game_state};
use lifecycle::active_state;
pub(crate) use lifecycle::attach;
#[cfg(test)]
use lifecycle::{ACTIVE_BACKEND, clear_active_backend};
#[cfg(test)]
use native_abi::PacketPlayerId;
use native_abi::{
    AllocatePacketFn, IncomingRpcFn, OutgoingPacketFn, OutgoingRpcFn, QueueWriteLockFn,
    QueueWriteUnlockFn, RawPacket, RpcPlayerId, StringReadDecoderFn, StringWriteEncoderFn,
    priority_value, reliability_value,
};
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
use gta_sa_native::{GameTickParticipant, GameTickRuntime, GtaProfile};
use hooks::{HookStorage, VtableHook};
use modkit_win32::InlineHook;
#[cfg(test)]
use native_bitstream::native_bit_length;
use native_bitstream::{NativeBitStream, RawBitStream};
use native_client::profile::NativeClientProfile;
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
}

impl BackendContext {
    /// Returns the selected immutable profile for cached direct operations.
    fn scalar_profile(&self) -> Option<NativeClientProfile> {
        self.native_client_profile
    }

    fn connection_profile(&self) -> Option<NativeClientProfile> {
        self.native_client_profile
    }
}

struct BackendState {
    context: BackendContext,
    game_tick: GameTickRuntime,
    rak_client: AtomicUsize,
    raw_player_pool: AtomicUsize,
    raw_vehicle_pool: AtomicUsize,
    raw_local_player: AtomicUsize,
    rpc_receiver: AtomicUsize,
    player_address: AtomicU32,
    player_port: AtomicU16,
    constructor_trampoline: AtomicUsize,
    incoming_rpc_trampoline: AtomicUsize,
    dialog_close_trampoline: AtomicUsize,
    outgoing_packet_original: AtomicUsize,
    incoming_packet_original: AtomicUsize,
    deallocate_packet_original: AtomicUsize,
    outgoing_rpc_original: AtomicUsize,
    client_hook_status: AtomicU32,
    incoming_packet_diagnostic_logged: AtomicBool,
    game_command_snapshot_diagnostic_logged: AtomicBool,
    game_command_completion_diagnostic_logged: AtomicBool,
    string_codec: Mutex<()>,
    pending_game_tick: Mutex<Option<Vec<QueuedCommand<GameCommand>>>>,
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

fn command_send_error(error: CommandError) -> SendError {
    match error {
        CommandError::QueueFull | CommandError::IdExhausted => SendError::QueueFull,
        CommandError::ShuttingDown | CommandError::UnknownReceipt => SendError::ClientNotReady,
        CommandError::NativeFailure | CommandError::TimedOut | CommandError::WaitRejected => {
            SendError::NativeCallFailed
        }
    }
}

fn sent_game_command_result(sent: bool) -> Result<(), SendError> {
    sent.then_some(()).ok_or(SendError::NativeCallFailed)
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

#[cfg(test)]
mod layout_tests;

#[cfg(test)]
mod profile_layout_tests;

#[cfg(test)]
mod vtable_tests;

#[cfg(test)]
mod inline_hook_tests;

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
