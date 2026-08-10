//! Private Windows x86 hook implementation.
//!
//! The module is intentionally unavailable for other targets. Its only public
//! boundary is the safe `Runtime` API in the parent crate.

mod chat_entries;
mod commands;
mod gangzones;
mod handles;
mod hooks;
mod objects;
mod packets;
mod players;
mod r1_client;
mod reads;
mod requests;
mod text_labels;
mod textdraws;
mod vehicles;

use crate::{
    AddressSet, AttachError, BitStream, SampVersion, SendError, SendOptions,
    command::{CommandError, CommandId, CommandQueue, QueuedCommand},
    event::Registry,
    runtime::{
        AnimationSnapshot, ChatEntrySnapshot, ClientHookStatus, CodecError, DirectClientError,
        GangzoneSnapshot, LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest,
        LocalDialogSnapshot, LocalPlayerSnapshot, PacketPriority, PacketReliability,
        PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot, TextLabelSnapshot,
        TextdrawSnapshot,
    },
};
use minhook::MinHook;
use r1_client::R1ClientProfile;
use std::{
    collections::{HashMap, VecDeque},
    ffi::c_void,
    mem, ptr, slice,
    sync::{
        Arc, Mutex, OnceLock, Weak,
        atomic::{AtomicBool, AtomicI32, AtomicU16, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleA,
    Memory::{PAGE_READWRITE, VirtualProtect},
    Threading::GetCurrentThreadId,
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
const MAX_CHAT_ENTRIES: usize = 100;
const MAX_SAMP_OBJECTS: usize = 1000;
const MAX_SAMP_PICKUPS: usize = 4096;
const MAX_SAMP_GANGZONES: usize = 1024;
const R1_CONNECTED_GAME_STATE: i32 = 14;
/// GTA SA 1.0 US `CGame::Process`. This target is independent of SA-MP's
/// module base and is supported only for the fixed GTA executable selected by
/// the host's R1/GTA configuration.
const GTA_SA_10_US_CGAME_PROCESS: usize = 0x53E4B0;

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

struct BackendState {
    registry: Arc<Registry>,
    module_base: usize,
    version: SampVersion,
    addresses: AddressSet,
    r1_client: Option<R1ClientProfile>,
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
    game_thread_id: AtomicU32,
    outgoing_packet_original: AtomicUsize,
    incoming_packet_original: AtomicUsize,
    deallocate_packet_original: AtomicUsize,
    outgoing_rpc_original: AtomicUsize,
    client_hook_status: AtomicU32,
    incoming_packet_diagnostic_logged: AtomicBool,
    string_codec: Mutex<()>,
    game_commands: CommandQueue<GameCommand, ()>,
    local_player_snapshot: Mutex<Option<LocalPlayerSnapshot>>,
    local_player_candidate: Mutex<Option<LocalPlayerSnapshot>>,
    player_info_cache: Mutex<Vec<PlayerInfoCacheEntry>>,
    player_info_requests: Mutex<VecDeque<u16>>,
    remote_player_state_cache: Mutex<Vec<RemotePlayerStateCacheEntry>>,
    remote_player_state_requests: Mutex<VecDeque<u16>>,
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
    local_chat_input_active: AtomicBool,
    local_chat_input_active_ready: AtomicBool,
    local_chat_input_text: Mutex<Option<Vec<u8>>>,
    local_chat_input_text_ready: AtomicBool,
    animation_catalog: Mutex<Option<Vec<AnimationSnapshot>>>,
    cache_generation: AtomicU64,
    hooks: Mutex<HookStorage>,
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

#[derive(Default)]
struct HookStorage {
    constructor: Option<InlineHook>,
    incoming_rpc: Option<InlineHook>,
    game_process: Option<InlineHook>,
    vtable: Option<VtableHook>,
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
    DeleteTextdraw(u16),
    SetTextdrawPosition {
        id: u16,
        x: f32,
        y: f32,
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
        seat: i32,
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
    let r1_client = R1ClientProfile::verify(module_base, entry_point);
    if r1_client.is_some() {
        log::info!("direct R1 client helpers are enabled with fixed offsets");
    }

    let active = ACTIVE_BACKEND.get_or_init(|| Mutex::new(None));
    let mut active = active.lock().unwrap_or_else(|error| error.into_inner());
    if active.as_ref().and_then(Weak::upgrade).is_some() {
        return Err(AttachError::AlreadyAttached);
    }

    let state = Arc::new(BackendState {
        registry,
        module_base,
        version,
        addresses,
        r1_client,
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
        game_thread_id: AtomicU32::new(0),
        outgoing_packet_original: AtomicUsize::new(0),
        incoming_packet_original: AtomicUsize::new(0),
        deallocate_packet_original: AtomicUsize::new(0),
        outgoing_rpc_original: AtomicUsize::new(0),
        client_hook_status: AtomicU32::new(ClientHookInstallState::Pending.as_raw()),
        incoming_packet_diagnostic_logged: AtomicBool::new(false),
        string_codec: Mutex::new(()),
        game_commands: CommandQueue::new(),
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
        local_chat_input_active: AtomicBool::new(false),
        local_chat_input_active_ready: AtomicBool::new(false),
        local_chat_input_text: Mutex::new(None),
        local_chat_input_text_ready: AtomicBool::new(false),
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
    if let Err(error) = state.install_constructor_hook() {
        state.shutdown();
        return Err(error);
    }
    Ok(Backend { state })
}

impl Backend {
    pub(crate) fn client_hook_status(&self) -> ClientHookStatus {
        ClientHookInstallState::from_raw(self.state.client_hook_status.load(Ordering::Acquire))
            .as_public()
    }

    pub(crate) fn samp_version(&self) -> SampVersion {
        self.state.version
    }

    pub(crate) fn raw_rakclient(&self) -> Option<*mut c_void> {
        let client = self.state.rak_client.load(Ordering::Acquire) as *mut c_void;
        (!client.is_null()).then_some(client)
    }

    pub(crate) fn raw_rakpeer(&self) -> Option<*mut c_void> {
        let profile = self.state.r1_client?;
        let client = self.raw_rakclient()?;
        profile.rakpeer_address(client).ok()
    }

    pub(crate) fn raw_player_pool(&self) -> Option<*mut c_void> {
        let pool = self.state.raw_player_pool.load(Ordering::Acquire) as *mut c_void;
        (!pool.is_null()).then_some(pool)
    }

    pub(crate) fn raw_vehicle_pool(&self) -> Option<*mut c_void> {
        let pool = self.state.raw_vehicle_pool.load(Ordering::Acquire) as *mut c_void;
        (!pool.is_null()).then_some(pool)
    }

    pub(crate) fn raw_local_player(&self) -> Option<*mut c_void> {
        let player = self.state.raw_local_player.load(Ordering::Acquire) as *mut c_void;
        (!player.is_null()).then_some(player)
    }

    pub(crate) fn encode_string(&self, value: &[u8]) -> Result<BitStream, CodecError> {
        self.state.encode_string(value)
    }

    pub(crate) fn decode_string(
        &self,
        payload: &mut BitStream,
        output: &mut [u8],
    ) -> Result<usize, CodecError> {
        self.state.decode_string(payload, output)
    }

    pub(crate) fn send_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.state.send_packet(packet_id, payload, options)
    }

    pub(crate) fn send_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.state.send_rpc(rpc_id, payload, options)
    }

    pub(crate) fn submit_packet(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.state.submit_packet(packet_id, payload, options)
    }

    pub(crate) fn submit_rpc(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.state.submit_rpc(rpc_id, payload, options)
    }

    pub(crate) fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.state.emulate_incoming_packet(packet_id, payload)
    }

    pub(crate) fn emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.state.emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn submit_emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.state
            .submit_emulate_incoming_packet(packet_id, payload)
    }

    pub(crate) fn submit_emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.state.submit_emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn show_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        self.state.show_local_dialog(request)
    }

    pub(crate) fn show_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        self.state.show_local_chat_message(request)
    }

    pub(crate) fn show_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        self.state.show_local_death_message(request)
    }

    pub(crate) fn submit_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_dialog(request)
    }

    pub(crate) fn submit_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_chat_message(request)
    }

    pub(crate) fn submit_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_death_message(request)
    }

    pub(crate) fn submit_local_cursor_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_cursor_mode(mode)
    }

    pub(crate) fn submit_local_chat_display_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_chat_display_mode(mode)
    }

    pub(crate) fn submit_local_chat_entry(
        &self,
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state
            .submit_local_chat_entry(id, text, prefix, text_colour, prefix_colour)
    }

    pub(crate) fn submit_local_dialog_close(
        &self,
        button: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_dialog_close(button)
    }

    pub(crate) fn submit_local_chat_input_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_chat_input_text(text)
    }

    pub(crate) fn submit_local_chat_input_enabled(
        &self,
        enabled: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_chat_input_enabled(enabled)
    }

    pub(crate) fn submit_local_chat_input_process(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_chat_input_process(text)
    }

    pub(crate) fn submit_local_cursor_toggle(
        &self,
        show: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_cursor_toggle(show)
    }

    pub(crate) fn submit_local_scoreboard_open(
        &self,
        open: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_scoreboard_open(open)
    }

    pub(crate) fn submit_local_dialog_client_side(
        &self,
        client_side: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_dialog_client_side(client_side)
    }

    pub(crate) fn submit_local_dialog_selected_item(
        &self,
        selected: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_dialog_selected_item(selected)
    }

    pub(crate) fn submit_samp_game_state(
        &self,
        state: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_samp_game_state(state)
    }

    pub(crate) fn submit_connect_to_server(
        &self,
        address: Vec<u8>,
        port: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_connect_to_server(address, port)
    }

    pub(crate) fn submit_disconnect_with_reason(
        &self,
        block_duration: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_disconnect_with_reason(block_duration)
    }

    pub(crate) fn submit_delete_textdraw(&self, id: u16) -> Result<CommandId, DirectClientError> {
        self.state.submit_delete_textdraw(id)
    }

    pub(crate) fn submit_delete_text_label(&self, id: u16) -> Result<CommandId, DirectClientError> {
        self.state.submit_delete_text_label(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit_create_text_label(
        &self,
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_create_text_label(
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        )
    }

    pub(crate) fn submit_set_textdraw_position(
        &self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_set_textdraw_position(id, x, y)
    }

    pub(crate) fn submit_set_textdraw_letter_style(
        &self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state
            .submit_set_textdraw_letter_style(id, width, height, colour)
    }

    pub(crate) fn submit_set_textdraw_proportional(
        &self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.state
            .submit_set_textdraw_proportional(id, proportional)
    }

    pub(crate) fn submit_set_textdraw_shadow(
        &self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_set_textdraw_shadow(id, shadow, colour)
    }

    pub(crate) fn submit_set_textdraw_outline(
        &self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_set_textdraw_outline(id, outline, colour)
    }

    pub(crate) fn submit_set_textdraw_box(
        &self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.state
            .submit_set_textdraw_box(id, enabled, colour, width, height)
    }

    pub(crate) fn submit_set_textdraw_alignment(
        &self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_set_textdraw_alignment(id, alignment)
    }

    pub(crate) fn submit_set_textdraw_string(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_set_textdraw_string(id, text)
    }

    pub(crate) fn submit_set_textdraw_model_style(
        &self,
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.state
            .submit_set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
    }

    pub(crate) fn submit_local_player_spawn(&self) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_player_spawn()
    }

    pub(crate) fn submit_local_player_special_action(
        &self,
        action: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_player_special_action(action)
    }

    pub(crate) fn submit_local_player_name(
        &self,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_player_name(name)
    }

    pub(crate) fn submit_force_unoccupied_sync(
        &self,
        vehicle: u16,
        seat: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_force_unoccupied_sync(vehicle, seat)
    }

    pub(crate) fn submit_send_rate(
        &self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_send_rate(kind, milliseconds)
    }

    pub(crate) fn submit_player_colour(
        &self,
        id: u16,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_player_colour(id, colour)
    }

    pub(crate) fn try_take_command(
        &self,
        id: CommandId,
    ) -> Result<Option<Result<(), CommandError>>, CommandError> {
        self.state.game_commands.try_take(id)
    }

    pub(crate) fn wait_for_command(
        &self,
        id: CommandId,
        timeout: Duration,
    ) -> Result<Result<(), CommandError>, CommandError> {
        self.state.game_commands.wait(
            id,
            timeout,
            !self.state.is_game_thread() && !self.state.registry.is_dispatching_on_current_thread(),
        )
    }

    pub(crate) fn release_command(&self, id: CommandId) -> Result<(), CommandError> {
        self.state.game_commands.detach(id)
    }

    pub(crate) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        self.state.local_player()
    }

    pub(crate) fn player_info(
        &self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        self.state.player_info(id)
    }

    pub(crate) fn remote_player_state(
        &self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        self.state.remote_player_state(id)
    }

    pub(crate) fn player_defined(&self, id: u16) -> Result<bool, DirectClientError> {
        self.state.player_defined(id)
    }

    pub(crate) fn player_paused(&self, id: u16) -> Result<bool, DirectClientError> {
        self.state.player_paused(id)
    }

    pub(crate) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        self.state.player_count(include_npcs)
    }

    pub(crate) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        self.state.player_max_id()
    }

    pub(crate) fn vehicle_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.state.vehicle_exists(id)
    }

    pub(crate) fn text_label_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.state.text_label_exists(id)
    }

    pub(crate) fn text_label(
        &self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        self.state.text_label(id)
    }

    pub(crate) fn textdraw_exists(&self, pool_index: u16) -> Result<bool, DirectClientError> {
        self.state.textdraw_exists(pool_index)
    }

    pub(crate) fn textdraw(
        &self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        self.state.textdraw(pool_index)
    }

    pub(crate) fn chat_entry(&self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        self.state.chat_entry(id)
    }

    pub(crate) fn object_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.state.object_exists(id)
    }

    pub(crate) fn gangzone(&self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        self.state.gangzone(id)
    }

    pub(crate) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        self.state.server_info()
    }

    pub(crate) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        self.state.samp_game_state()
    }

    pub(crate) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        self.state.local_chat_display_mode()
    }

    pub(crate) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        self.state.local_cursor_mode()
    }

    pub(crate) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        self.state.local_scoreboard_open()
    }

    pub(crate) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        self.state.local_dialog_active()
    }

    pub(crate) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        self.state.local_dialog_state()
    }

    pub(crate) fn local_dialog_selected_item(&self) -> Result<i32, DirectClientError> {
        self.state
            .local_dialog_state()?
            .and_then(|snapshot| snapshot.selected_item)
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn local_dialog_list_item_count(&self) -> Result<i32, DirectClientError> {
        self.state
            .local_dialog_state()?
            .and_then(|snapshot| snapshot.list_item_count)
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn submit_local_dialog_editbox_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.state.submit_local_dialog_editbox_text(text)
    }

    pub(crate) fn object_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.state.object_handle(id)
    }

    pub(crate) fn object_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.state.object_id_by_handle(handle)
    }

    pub(crate) fn pickup_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.state.pickup_handle(id)
    }

    pub(crate) fn pickup_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.state.pickup_id_by_handle(handle)
    }

    pub(crate) fn vehicle_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.state.vehicle_handle(id)
    }

    pub(crate) fn vehicle_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.state.vehicle_id_by_handle(handle)
    }

    pub(crate) fn player_ped_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.state.player_ped_handle(id)
    }

    pub(crate) fn player_id_by_ped_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.state.player_id_by_ped_handle(handle)
    }

    pub(crate) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        self.state.local_chat_input_active()
    }

    pub(crate) fn local_chat_input_text(&self) -> Result<Vec<u8>, DirectClientError> {
        self.state.local_chat_input_text()
    }

    pub(crate) fn local_animation(&self, id: u16) -> Result<AnimationSnapshot, DirectClientError> {
        self.state.local_animation(id)
    }

    pub(crate) fn local_animation_id(
        &self,
        name: &[u8],
        file: &[u8],
    ) -> Result<Option<u16>, DirectClientError> {
        self.state.local_animation_id(name, file)
    }

    pub(crate) fn shutdown(&mut self) {
        self.state.shutdown();
    }
}

impl BackendState {
    fn install_game_process_hook(&self) -> Result<(), AttachError> {
        let (mut detour, trampoline) = InlineHook::create(
            GTA_SA_10_US_CGAME_PROCESS,
            game_process_detour as *const () as usize,
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

    fn install_constructor_hook(self: &Arc<Self>) -> Result<(), AttachError> {
        let target = self.module_base + self.addresses.rak_client_constructor as usize;
        let (mut detour, trampoline) =
            InlineHook::create(target, rak_client_constructor_detour as *const () as usize)
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

    fn encode_string(&self, value: &[u8]) -> Result<BitStream, CodecError> {
        if value.contains(&0) {
            return Err(CodecError::InvalidArgument);
        }
        let max_chars = value
            .len()
            .checked_add(1)
            .and_then(|length| i32::try_from(length).ok())
            .ok_or(CodecError::PayloadTooLarge)?;
        let capacity_bits = value
            .len()
            .checked_mul(16)
            .and_then(|bits| bits.checked_add(16))
            .ok_or(CodecError::PayloadTooLarge)?;
        let mut input = Vec::with_capacity(value.len() + 1);
        input.extend_from_slice(value);
        input.push(0);
        let mut native = NativeBitStream::empty_with_capacity_bits(capacity_bits)
            .map_err(|_| CodecError::PayloadTooLarge)?;
        let _codec = self
            .string_codec
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let compressor = self.ready_string_compressor()?;
        let encode: StringWriteEncoderFn = unsafe {
            mem::transmute(self.module_base + self.addresses.string_write_encoder as usize)
        };
        unsafe {
            encode(
                compressor,
                input.as_ptr().cast(),
                max_chars,
                native.as_mut_ptr(),
                0,
            )
        };
        native
            .into_stream()
            .map_err(|_| CodecError::NativeCallFailed)
    }

    fn decode_string(
        &self,
        payload: &mut BitStream,
        output: &mut [u8],
    ) -> Result<usize, CodecError> {
        let max_chars = i32::try_from(output.len()).map_err(|_| CodecError::PayloadTooLarge)?;
        if max_chars == 0 {
            return Err(CodecError::InvalidArgument);
        }
        output.fill(0);
        let mut native = NativeBitStream::from_readable_stream(payload)
            .map_err(|_| CodecError::PayloadTooLarge)?;
        let _codec = self
            .string_codec
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let compressor = self.ready_string_compressor()?;
        let decode: StringReadDecoderFn = unsafe {
            mem::transmute(self.module_base + self.addresses.string_read_decoder as usize)
        };
        if !unsafe {
            decode(
                compressor,
                output.as_mut_ptr().cast(),
                max_chars,
                native.as_mut_ptr(),
                0,
            )
        } {
            return Err(CodecError::NativeCallFailed);
        }
        let read_offset = native.read_offset().ok_or(CodecError::NativeCallFailed)?;
        let length = output
            .iter()
            .position(|byte| *byte == 0)
            .ok_or(CodecError::PayloadTooLarge)?;
        payload
            .set_read_offset_bits(read_offset)
            .map_err(|_| CodecError::NativeCallFailed)?;
        Ok(length)
    }

    fn ready_string_compressor(&self) -> Result<*mut c_void, CodecError> {
        let pointer = self.module_base + self.addresses.compressor_ptr as usize;
        let compressor = unsafe { ptr::read_unaligned(pointer as *const *mut c_void) };
        if compressor.is_null() {
            Err(CodecError::ClientNotReady)
        } else {
            Ok(compressor)
        }
    }

    fn show_local_dialog(&self, request: LocalDialogRequest) -> Result<(), DirectClientError> {
        let id = self.submit_local_dialog(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    fn show_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_chat_message(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    fn show_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let id = self.submit_local_death_message(request)?;
        self.game_commands
            .detach(id)
            .map_err(|_| DirectClientError::NotReady)
    }

    fn submit_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_dialog(request)
    }

    fn submit_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_chat_message(request)
    }

    fn submit_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_local_death_message(request)
    }

    fn submit_local_cursor_mode(&self, mode: i32) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetCursorMode(mode))
    }

    fn submit_local_chat_display_mode(&self, mode: i32) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(mode, 0..=2) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatDisplayMode(mode))
    }

    fn submit_local_chat_entry(
        &self,
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || id >= 100
            || text.len() >= 144
            || prefix.len() >= 28
            || text.contains(&0)
            || prefix.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatEntry {
            id,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
    }

    fn submit_local_dialog_close(&self, button: u8) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || button > 1 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::CloseDialog(button))
    }

    fn submit_local_chat_input_text(&self, text: Vec<u8>) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatInputText(text))
    }

    fn submit_local_chat_input_enabled(
        &self,
        enabled: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetChatInputEnabled(enabled))
    }

    fn submit_local_chat_input_process(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ProcessChatInput(text))
    }

    fn submit_local_cursor_toggle(&self, show: bool) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ToggleCursor(show))
    }

    fn submit_local_scoreboard_open(&self, open: bool) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetScoreboardOpen(open))
    }

    fn submit_local_dialog_client_side(
        &self,
        client_side: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetDialogClientSide(client_side))
    }

    fn submit_local_dialog_selected_item(
        &self,
        selected: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetDialogSelectedItem(selected))
    }

    fn submit_local_dialog_editbox_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetDialogEditboxText(text))
    }

    fn submit_samp_game_state(&self, state: i32) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !matches!(state, 0 | 9 | 13 | 14 | 15 | 18)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetGameState(state))
    }

    fn submit_connect_to_server(
        &self,
        address: Vec<u8>,
        port: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || address.is_empty()
            || address.len() > 256
            || address.contains(&0)
            || port == 0
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ConnectToServer { address, port })
    }

    fn submit_disconnect_with_reason(
        &self,
        block_duration: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DisconnectWithReason(block_duration))
    }

    fn submit_delete_textdraw(&self, id: u16) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DeleteTextdraw(id))
    }

    fn submit_delete_text_label(&self, id: u16) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::DeleteTextLabel(id))
    }

    #[allow(clippy::too_many_arguments)]
    fn submit_create_text_label(
        &self,
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: crate::runtime::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXT_LABELS
            || text.len() > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
            || text.contains(&0)
            || !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !draw_distance.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::CreateTextLabel {
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        })
    }

    fn submit_set_textdraw_position(
        &self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawPosition { id, x, y })
    }

    fn submit_set_textdraw_letter_style(
        &self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !width.is_finite()
            || !height.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawLetterStyle {
            id,
            width,
            height,
            colour,
        })
    }

    fn submit_set_textdraw_proportional(
        &self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawProportional { id, proportional })
    }

    fn submit_set_textdraw_shadow(
        &self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawShadow { id, shadow, colour })
    }

    fn submit_set_textdraw_outline(
        &self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawOutline {
            id,
            outline,
            colour,
        })
    }

    fn submit_set_textdraw_box(
        &self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !width.is_finite()
            || !height.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawBox {
            id,
            enabled,
            colour,
            width,
            height,
        })
    }

    fn submit_set_textdraw_alignment(
        &self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !(1..=3).contains(&alignment)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawAlignment { id, alignment })
    }

    fn submit_set_textdraw_string(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || text.len() > 1_601
            || text.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawString { id, text })
    }

    fn submit_set_textdraw_model_style(
        &self,
        id: u16,
        rotation: crate::runtime::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || usize::from(id) >= MAX_SAMP_TEXTDRAWS
            || !rotation.x.is_finite()
            || !rotation.y.is_finite()
            || !rotation.z.is_finite()
            || !zoom.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetTextdrawModelStyle {
            id,
            rotation,
            zoom,
            colour1,
            colour2,
        })
    }

    fn submit_local_player_spawn(&self) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SpawnLocalPlayer)
    }

    fn submit_local_player_special_action(
        &self,
        action: u8,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !matches!(action, 0..=12 | 20..=25 | 68)
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetLocalPlayerSpecialAction(action))
    }

    fn submit_local_player_name(&self, name: Vec<u8>) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || name.len() > 255 || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetLocalPlayerName(name))
    }

    fn submit_force_unoccupied_sync(
        &self,
        vehicle: u16,
        seat: i32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(vehicle) >= MAX_SAMP_VEHICLES
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::ForceUnoccupiedSync { vehicle, seat })
    }

    fn submit_send_rate(
        &self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !matches!(kind, 0..=2)
            || i32::try_from(milliseconds).is_err()
        {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetSendRate { kind, milliseconds })
    }

    fn submit_player_colour(&self, id: u16, colour: u32) -> Result<CommandId, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || usize::from(id) >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        self.queue_game_command(GameCommand::SetPlayerColour { id, colour })
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
        let Some(profile) = self.r1_client else {
            return;
        };
        // Odd generations are in-flight. Readers only observe the next even
        // generation after every cache path below has had one tick to refresh.
        self.cache_generation.fetch_add(1, Ordering::AcqRel);
        self.refresh_raw_pool_addresses(profile);
        self.refresh_samp_game_state(profile);
        self.refresh_local_chat_display_mode(profile);
        self.refresh_local_cursor_mode(profile);
        self.refresh_local_scoreboard_open(profile);
        self.refresh_local_dialog_active(profile);
        self.refresh_local_dialog_state(profile);
        self.refresh_local_chat_input_active(profile);
        self.refresh_local_chat_input_text(profile);
        self.refresh_animation_catalog(profile);
        self.refresh_server_info_snapshot(profile);
        self.refresh_local_player_snapshot(profile);
        self.refresh_player_info(profile);
        self.refresh_remote_player_state(profile);
        self.refresh_player_count(profile);
        self.refresh_player_max_id(profile);
        self.refresh_vehicle_exists(profile);
        self.refresh_text_label_exists(profile);
        self.refresh_text_labels(profile);
        self.refresh_textdraw_exists(profile);
        self.refresh_textdraws(profile);
        self.refresh_chat_entries(profile);
        self.refresh_object_exists(profile);
        self.refresh_gangzones(profile);
        self.refresh_object_handles(profile);
        self.refresh_pickup_handles(profile);
        self.refresh_vehicle_handles(profile);
        self.refresh_player_handles(profile);
        self.refresh_object_handle_ids(profile);
        self.refresh_pickup_handle_ids(profile);
        self.refresh_vehicle_handle_ids(profile);
        self.refresh_player_handle_ids(profile);
        self.cache_generation.fetch_add(1, Ordering::Release);
    }

    fn execute_game_commands(&self, commands: Vec<QueuedCommand<GameCommand>>) {
        for queued in commands {
            let result = match queued.command {
                GameCommand::ShowDialog(request) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .dialog_is_ready()
                            .then_some(())
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|()| {
                                profile
                                    .show_dialog(request)
                                    .map_err(|_| CommandError::NativeFailure)
                            })
                    }),
                GameCommand::CloseDialog(button) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .close_dialog(button)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatInputText(text) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_input_text(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatInputEnabled(enabled) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_input_enabled(enabled)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ProcessChatInput(text) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .process_chat_input(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatDisplayMode(mode) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_display_mode(mode)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetChatEntry {
                    id,
                    text,
                    prefix,
                    text_colour,
                    prefix_colour,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_chat_entry(id, &text, &prefix, text_colour, prefix_colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::AddChatMessage(request) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .chat_is_ready()
                            .then_some(())
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|()| {
                                profile
                                    .show_chat_message(request)
                                    .map_err(|_| CommandError::NativeFailure)
                            })
                    }),
                GameCommand::AddDeathMessage(request) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .death_window_is_ready()
                            .then_some(())
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|()| {
                                profile
                                    .show_death_message(request)
                                    .map_err(|_| CommandError::NativeFailure)
                            })
                    }),
                GameCommand::SetCursorMode(mode) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_cursor_mode(mode)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ToggleCursor(show) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .toggle_cursor(show)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetScoreboardOpen(open) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_scoreboard_open(open)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogClientSide(client_side) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_client_side(client_side)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogSelectedItem(selected) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_selected_item(selected)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetDialogEditboxText(text) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_dialog_editbox_text(&text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetGameState(state) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_game_state(state)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ConnectToServer { address, port } => {
                    let result =
                        self.r1_client
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|profile| {
                                profile
                                    .connect_to_server(&address, port)
                                    .map_err(|_| CommandError::NativeFailure)
                            });
                    if result.is_ok() {
                        self.invalidate_connection_state();
                    }
                    result
                }
                GameCommand::DisconnectWithReason(block_duration) => {
                    let rak_client = self.rak_client.load(Ordering::Acquire) as *mut c_void;
                    let result =
                        self.r1_client
                            .ok_or(CommandError::NativeFailure)
                            .and_then(|profile| {
                                profile
                                    .disconnect_with_reason(rak_client, block_duration)
                                    .map_err(|_| CommandError::NativeFailure)
                            });
                    if result.is_ok() {
                        self.rak_client.store(0, Ordering::Release);
                        self.rpc_receiver.store(0, Ordering::Release);
                        self.player_address.store(0, Ordering::Release);
                        self.player_port.store(0, Ordering::Release);
                        self.invalidate_connection_state();
                    }
                    result
                }
                GameCommand::DeleteTextdraw(id) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .delete_textdraw(id)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::DeleteTextLabel(id) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .delete_text_label(id)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::CreateTextLabel {
                    id,
                    text,
                    colour,
                    position,
                    draw_distance,
                    behind_walls,
                    attached_player_id,
                    attached_vehicle_id,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .create_text_label(
                                id,
                                &text,
                                colour,
                                position,
                                draw_distance,
                                behind_walls,
                                attached_player_id,
                                attached_vehicle_id,
                            )
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawPosition { id, x, y } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_position(id, x, y)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawLetterStyle {
                    id,
                    width,
                    height,
                    colour,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_letter_style(id, width, height, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawProportional { id, proportional } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_proportional(id, proportional)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawShadow { id, shadow, colour } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_shadow(id, shadow, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawOutline {
                    id,
                    outline,
                    colour,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_outline(id, outline, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawBox {
                    id,
                    enabled,
                    colour,
                    width,
                    height,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_box(id, enabled, colour, width, height)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawAlignment { id, alignment } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_alignment(id, alignment)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawString { id, text } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_string(id, &text)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetTextdrawModelStyle {
                    id,
                    rotation,
                    zoom,
                    colour1,
                    colour2,
                } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SpawnLocalPlayer => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .spawn_local_player()
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetLocalPlayerSpecialAction(action) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_local_player_special_action(action)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetLocalPlayerName(name) => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_local_player_name(&name)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::ForceUnoccupiedSync { vehicle, seat } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .force_unoccupied_sync(vehicle, seat)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetPlayerColour { id, colour } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_player_colour(id, colour)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SetSendRate { kind, milliseconds } => self
                    .r1_client
                    .ok_or(CommandError::NativeFailure)
                    .and_then(|profile| {
                        profile
                            .set_send_rate(kind, milliseconds)
                            .map_err(|_| CommandError::NativeFailure)
                    }),
                GameCommand::SendPacket {
                    id,
                    payload,
                    options,
                } => self
                    .send_packet_native(id, &payload, options)
                    .and_then(sent_game_command_result)
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::SendRpc {
                    id,
                    payload,
                    options,
                } => self
                    .send_rpc_native(id, &payload, options)
                    .and_then(sent_game_command_result)
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::EmulateIncomingPacket { id, payload } => self
                    .emulate_incoming_packet_native(id, payload)
                    .map(|_| ())
                    .map_err(|_| CommandError::NativeFailure),
                GameCommand::EmulateIncomingRpc { id, payload } => self
                    .emulate_incoming_rpc_native(id, payload)
                    .map(|_| ())
                    .map_err(|_| CommandError::NativeFailure),
            };
            match result {
                Ok(()) => self.game_commands.complete(queued.id, Ok(())),
                Err(error) => {
                    // Every command owns its plugin-provided payload. Keep logs
                    // free of dialog text, chat text, and death-window names.
                    log::debug!("game command failed: {error:?}");
                    self.game_commands
                        .complete(queued.id, Err(CommandError::NativeFailure));
                }
            }
        }
    }

    fn take_player_info_requests(&self) -> Vec<u16> {
        self.player_info_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(PLAYER_INFO_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_remote_player_state_requests(&self) -> Vec<u16> {
        self.remote_player_state_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_vehicle_exists_requests(&self) -> Vec<u16> {
        self.vehicle_exists_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(VEHICLE_EXISTS_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_text_label_exists_requests(&self) -> Vec<u16> {
        self.text_label_exists_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(TEXT_LABEL_EXISTS_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_text_label_requests(&self) -> Vec<u16> {
        self.text_label_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(TEXT_LABEL_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_textdraw_exists_requests(&self) -> Vec<u16> {
        self.textdraw_exists_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(TEXTDRAW_EXISTS_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_textdraw_requests(&self) -> Vec<u16> {
        self.textdraw_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(TEXTDRAW_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_chat_entry_requests(&self) -> Vec<u16> {
        self.chat_entry_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(CHAT_ENTRY_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_object_exists_requests(&self) -> Vec<u16> {
        self.object_exists_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(OBJECT_EXISTS_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_gangzone_requests(&self) -> Vec<u16> {
        self.gangzone_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(GANGZONE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_object_handle_requests(&self) -> Vec<u16> {
        self.object_handle_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(OBJECT_HANDLE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_pickup_handle_requests(&self) -> Vec<u16> {
        self.pickup_handle_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(PICKUP_HANDLE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_vehicle_handle_requests(&self) -> Vec<u16> {
        self.vehicle_handle_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(VEHICLE_HANDLE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_player_handle_requests(&self) -> Vec<u16> {
        self.player_handle_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(PLAYER_HANDLE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_object_handle_id_requests(&self) -> Vec<i32> {
        self.object_handle_reverse_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(OBJECT_HANDLE_REVERSE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_pickup_handle_id_requests(&self) -> Vec<i32> {
        self.pickup_handle_reverse_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(PICKUP_HANDLE_REVERSE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_vehicle_handle_id_requests(&self) -> Vec<i32> {
        self.vehicle_handle_reverse_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(VEHICLE_HANDLE_REVERSE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
    }

    fn take_player_handle_id_requests(&self) -> Vec<i32> {
        self.player_handle_reverse_requests
            .try_lock()
            .map(|mut queue| {
                let count = queue.len().min(PLAYER_HANDLE_REVERSE_REQUESTS_PER_PUMP);
                queue.drain(..count).collect()
            })
            .unwrap_or_default()
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

    fn refresh_local_player_snapshot(&self, profile: R1ClientProfile) {
        if !self.samp_game_state_ready.load(Ordering::Acquire)
            || !is_r1_connected_game_state(self.samp_game_state.load(Ordering::Acquire))
        {
            self.raw_local_player.store(0, Ordering::Release);
            self.cache_local_player_snapshot(None);
            return;
        }
        self.raw_local_player.store(
            profile
                .local_player_address()
                .map_or(0, |player| player as usize),
            Ordering::Release,
        );
        self.cache_local_player_snapshot(profile.local_player().ok());
    }

    fn refresh_player_info(&self, profile: R1ClientProfile) {
        for id in self.take_player_info_requests() {
            let Ok(snapshot) = profile.player_info(id) else {
                continue;
            };
            let Ok(mut cache) = self.player_info_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = PlayerInfoCacheEntry::Known(snapshot);
            }
        }
    }

    fn refresh_remote_player_state(&self, profile: R1ClientProfile) {
        for id in self.take_remote_player_state_requests() {
            let Ok(snapshot) = profile.remote_player_state(id) else {
                continue;
            };
            let Ok(mut cache) = self.remote_player_state_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = RemotePlayerStateCacheEntry::Known(snapshot);
            }
        }
    }

    fn refresh_player_count(&self, profile: R1ClientProfile) {
        match profile.player_counts() {
            Ok((including_npcs, excluding_npcs)) => {
                self.player_count_including_npcs
                    .store(i32::from(including_npcs), Ordering::Release);
                self.player_count_excluding_npcs
                    .store(i32::from(excluding_npcs), Ordering::Release);
                self.player_count_ready.store(true, Ordering::Release);
            }
            Err(_) => self.player_count_ready.store(false, Ordering::Release),
        }
    }

    fn refresh_player_max_id(&self, profile: R1ClientProfile) {
        match profile.player_max_id() {
            Ok(id) => {
                self.player_max_id.store(i32::from(id), Ordering::Release);
                self.player_max_id_ready.store(true, Ordering::Release);
            }
            Err(_) => self.player_max_id_ready.store(false, Ordering::Release),
        }
    }

    fn refresh_raw_pool_addresses(&self, profile: R1ClientProfile) {
        let player_pool = profile.player_pool().map_or(0, |pool| pool as usize);
        let vehicle_pool = profile.vehicle_pool().map_or(0, |pool| pool as usize);
        self.raw_player_pool.store(player_pool, Ordering::Release);
        self.raw_vehicle_pool.store(vehicle_pool, Ordering::Release);
    }

    fn refresh_vehicle_exists(&self, profile: R1ClientProfile) {
        for id in self.take_vehicle_exists_requests() {
            let Ok(exists) = profile.vehicle_exists(id) else {
                continue;
            };
            let Ok(mut cache) = self.vehicle_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = VehicleExistsCacheEntry::Known(exists);
            }
        }
    }

    fn refresh_text_label_exists(&self, profile: R1ClientProfile) {
        for id in self.take_text_label_exists_requests() {
            let Ok(exists) = profile.text_label_exists(id) else {
                continue;
            };
            let Ok(mut cache) = self.text_label_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = TextLabelExistsCacheEntry::Known(exists);
            }
        }
    }

    fn refresh_text_labels(&self, profile: R1ClientProfile) {
        for id in self.take_text_label_requests() {
            let Ok(snapshot) = profile.text_label(id) else {
                continue;
            };
            let Ok(mut cache) = self.text_label_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = TextLabelCacheEntry::Known(snapshot);
            }
        }
    }

    fn refresh_textdraw_exists(&self, profile: R1ClientProfile) {
        for pool_index in self.take_textdraw_exists_requests() {
            let Ok(exists) = profile.textdraw_exists(pool_index) else {
                continue;
            };
            let Ok(mut cache) = self.textdraw_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(pool_index)) {
                *entry = TextdrawExistsCacheEntry::Known(exists);
            }
        }
    }

    fn refresh_textdraws(&self, profile: R1ClientProfile) {
        for pool_index in self.take_textdraw_requests() {
            let Ok(snapshot) = profile.textdraw(pool_index) else {
                continue;
            };
            let Ok(mut cache) = self.textdraw_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(pool_index)) {
                *entry = TextdrawCacheEntry::Known(snapshot);
            }
        }
    }

    fn refresh_chat_entries(&self, profile: R1ClientProfile) {
        for id in self.take_chat_entry_requests() {
            let Ok(snapshot) = profile.chat_entry(id) else {
                continue;
            };
            let Ok(mut cache) = self.chat_entry_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = ChatEntryCacheEntry::Known(snapshot);
            }
        }
    }

    fn refresh_object_exists(&self, profile: R1ClientProfile) {
        for id in self.take_object_exists_requests() {
            let Ok(exists) = profile.object_exists(id) else {
                continue;
            };
            let Ok(mut cache) = self.object_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = ObjectExistsCacheEntry::Known(exists);
            }
        }
    }

    fn refresh_gangzones(&self, profile: R1ClientProfile) {
        for id in self.take_gangzone_requests() {
            let Ok(snapshot) = profile.gangzone(id) else {
                continue;
            };
            let Ok(mut cache) = self.gangzone_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = GangzoneCacheEntry::Known(snapshot);
            }
        }
    }

    fn refresh_object_handles(&self, profile: R1ClientProfile) {
        for id in self.take_object_handle_requests() {
            let Ok(handle) = profile.object_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.object_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    fn refresh_pickup_handles(&self, profile: R1ClientProfile) {
        for id in self.take_pickup_handle_requests() {
            let Ok(handle) = profile.pickup_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.pickup_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    fn refresh_vehicle_handles(&self, profile: R1ClientProfile) {
        for id in self.take_vehicle_handle_requests() {
            let Ok(handle) = profile.vehicle_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.vehicle_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    fn refresh_player_handles(&self, profile: R1ClientProfile) {
        for id in self.take_player_handle_requests() {
            let Ok(handle) = profile.player_ped_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.player_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    fn refresh_object_handle_ids(&self, profile: R1ClientProfile) {
        for handle in self.take_object_handle_id_requests() {
            let Ok(id) = profile.object_id_by_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.object_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    fn refresh_pickup_handle_ids(&self, profile: R1ClientProfile) {
        for handle in self.take_pickup_handle_id_requests() {
            let Ok(id) = profile.pickup_id_by_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.pickup_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    fn refresh_vehicle_handle_ids(&self, profile: R1ClientProfile) {
        for handle in self.take_vehicle_handle_id_requests() {
            let Ok(id) = profile.vehicle_id_by_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.vehicle_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    fn refresh_player_handle_ids(&self, profile: R1ClientProfile) {
        for handle in self.take_player_handle_id_requests() {
            let Ok(id) = profile.player_id_by_ped_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.player_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    fn refresh_server_info_snapshot(&self, profile: R1ClientProfile) {
        let Ok(mut cached) = self.server_info_snapshot.try_lock() else {
            return;
        };
        *cached = profile.server_info().ok();
    }

    fn refresh_samp_game_state(&self, profile: R1ClientProfile) {
        match profile.game_state() {
            Ok(game_state) => {
                let previous = self.samp_game_state.swap(game_state, Ordering::AcqRel);
                let was_ready = self.samp_game_state_ready.swap(true, Ordering::AcqRel);
                if crosses_r1_connection_boundary(was_ready, previous, game_state) {
                    self.invalidate_connection_state();
                }
            }
            Err(DirectClientError::NotReady) => {
                self.samp_game_state_ready.store(false, Ordering::Release);
            }
            Err(DirectClientError::UnsupportedVersion | DirectClientError::QueueFull) => {
                self.samp_game_state_ready.store(false, Ordering::Release);
            }
        }
    }

    fn refresh_local_chat_display_mode(&self, profile: R1ClientProfile) {
        match profile.chat_display_mode() {
            Ok(mode) => {
                self.local_chat_display_mode.store(mode, Ordering::Release);
                self.local_chat_display_mode_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_display_mode_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    fn refresh_local_cursor_mode(&self, profile: R1ClientProfile) {
        match profile.cursor_mode() {
            Ok(mode) => {
                self.local_cursor_mode.store(mode, Ordering::Release);
                self.local_cursor_mode_ready.store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_cursor_mode_ready.store(false, Ordering::Release);
            }
        }
    }

    fn refresh_local_scoreboard_open(&self, profile: R1ClientProfile) {
        match profile.scoreboard_is_open() {
            Ok(open) => {
                self.local_scoreboard_open.store(open, Ordering::Release);
                self.local_scoreboard_open_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_scoreboard_open_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    fn refresh_local_dialog_active(&self, profile: R1ClientProfile) {
        match profile.dialog_is_active() {
            Ok(active) => {
                self.local_dialog_active.store(active, Ordering::Release);
                self.local_dialog_active_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_dialog_active_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    fn refresh_local_dialog_state(&self, profile: R1ClientProfile) {
        match profile.dialog_state() {
            Ok(snapshot) => {
                let Ok(mut cached) = self.local_dialog_snapshot.try_lock() else {
                    return;
                };
                *cached = snapshot;
                self.local_dialog_snapshot_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => self
                .local_dialog_snapshot_ready
                .store(false, Ordering::Release),
        }
    }

    fn refresh_local_chat_input_active(&self, profile: R1ClientProfile) {
        match profile.chat_input_is_active() {
            Ok(active) => {
                self.local_chat_input_active
                    .store(active, Ordering::Release);
                self.local_chat_input_active_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_input_active_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    fn refresh_local_chat_input_text(&self, profile: R1ClientProfile) {
        match profile.chat_input_text() {
            Ok(text) => {
                let Ok(mut snapshot) = self.local_chat_input_text.try_lock() else {
                    self.local_chat_input_text_ready
                        .store(false, Ordering::Release);
                    return;
                };
                *snapshot = Some(text);
                self.local_chat_input_text_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_input_text_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    fn refresh_animation_catalog(&self, profile: R1ClientProfile) {
        let Ok(mut catalog) = self.animation_catalog.try_lock() else {
            return;
        };
        if catalog.is_none() {
            *catalog = profile.animation_catalog().ok();
        }
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

    fn cache_is_published(&self) -> bool {
        let generation = self.cache_generation.load(Ordering::Acquire);
        generation != 0 && generation.is_multiple_of(2)
    }

    fn is_game_thread(&self) -> bool {
        let game_thread = self.game_thread_id.load(Ordering::Acquire);
        game_thread != 0 && game_thread == unsafe { GetCurrentThreadId() }
    }

    unsafe fn run_game_process_tick(&self, game: *mut c_void, original: GameProcessFn) {
        // Publish this before entering GTA so a plugin reached from the native
        // process path cannot block the game thread on its own command receipt.
        self.game_thread_id
            .store(unsafe { GetCurrentThreadId() }, Ordering::Release);
        let commands = self.prepare_game_tick();
        unsafe { original(game) };
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
        self.local_dialog_snapshot_ready
            .store(false, Ordering::Release);
        self.local_chat_input_active_ready
            .store(false, Ordering::Release);
        self.local_chat_input_text_ready
            .store(false, Ordering::Release);
        if let Ok(mut snapshot) = self.local_chat_input_text.try_lock() {
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

fn is_r1_connected_game_state(game_state: i32) -> bool {
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

fn crosses_r1_connection_boundary(was_ready: bool, previous: i32, current: i32) -> bool {
    was_ready
        && previous != current
        && (is_r1_connected_game_state(previous) || is_r1_connected_game_state(current))
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
struct RawBitStream {
    number_of_bits_used: i32,
    number_of_bits_allocated: i32,
    read_offset: i32,
    data: *mut u8,
    copy_data: bool,
    stack_data: [u8; 256],
}

impl RawBitStream {
    unsafe fn copy_to_owned(&self) -> Result<BitStream, SendError> {
        if self.number_of_bits_used < 0 || self.number_of_bits_allocated < self.number_of_bits_used
        {
            return Err(SendError::NativeCallFailed);
        }
        let used = self.number_of_bits_used as usize;
        let allocated = self.number_of_bits_allocated as usize;
        let byte_len = used.div_ceil(u8::BITS as usize);
        if byte_len > 0 && self.data.is_null() {
            return Err(SendError::NativeCallFailed);
        }
        let bytes = if byte_len == 0 {
            Vec::new()
        } else {
            unsafe { slice::from_raw_parts(self.data, byte_len) }.to_vec()
        };
        BitStream::from_bytes_with_capacity(bytes, used, allocated)
            .map_err(|_| SendError::NativeCallFailed)
    }

    unsafe fn replace_from(&mut self, stream: &BitStream) -> Result<(), SendError> {
        let capacity = self.number_of_bits_allocated.max(0) as usize;
        if stream.len_bits() > capacity {
            return Err(SendError::PayloadTooLarge);
        }
        if stream.len_bytes() > 0 && self.data.is_null() {
            return Err(SendError::NativeCallFailed);
        }
        unsafe {
            ptr::copy_nonoverlapping(stream.as_bytes().as_ptr(), self.data, stream.len_bytes());
        }
        self.number_of_bits_used = stream.len_bits() as i32;
        self.read_offset = 0;
        Ok(())
    }
}

struct NativeBitStream {
    data: Vec<u8>,
    raw: RawBitStream,
}

impl NativeBitStream {
    fn new(stream: &BitStream) -> Result<Self, SendError> {
        let bit_len = native_bit_length(stream.len_bits())?;
        let mut data = stream.as_bytes().to_vec();
        let data_pointer = if data.is_empty() {
            ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        Ok(Self {
            raw: RawBitStream {
                number_of_bits_used: bit_len,
                number_of_bits_allocated: bit_len,
                read_offset: 0,
                data: data_pointer,
                copy_data: false,
                stack_data: [0; 256],
            },
            data,
        })
    }

    fn empty_with_capacity_bits(capacity_bits: usize) -> Result<Self, SendError> {
        let allocated = native_bit_length(capacity_bits)?;
        let mut data = vec![0_u8; capacity_bits.div_ceil(u8::BITS as usize)];
        let data_pointer = if data.is_empty() {
            ptr::null_mut()
        } else {
            data.as_mut_ptr()
        };
        Ok(Self {
            raw: RawBitStream {
                number_of_bits_used: 0,
                number_of_bits_allocated: allocated,
                read_offset: 0,
                data: data_pointer,
                copy_data: false,
                stack_data: [0; 256],
            },
            data,
        })
    }

    fn from_readable_stream(stream: &BitStream) -> Result<Self, SendError> {
        let mut native = Self::new(stream)?;
        native.raw.read_offset = native_bit_length(stream.read_offset_bits())?;
        Ok(native)
    }

    fn read_offset(&self) -> Option<usize> {
        let read_offset = usize::try_from(self.raw.read_offset).ok()?;
        (read_offset <= usize::try_from(self.raw.number_of_bits_used).ok()?).then_some(read_offset)
    }

    fn into_stream(mut self) -> Result<BitStream, SendError> {
        let bit_len = usize::try_from(self.raw.number_of_bits_used)
            .map_err(|_| SendError::NativeCallFailed)?;
        if bit_len > self.data.len().saturating_mul(u8::BITS as usize) {
            return Err(SendError::NativeCallFailed);
        }
        if bit_len != 0 && self.raw.data != self.data.as_mut_ptr() {
            return Err(SendError::NativeCallFailed);
        }
        let bytes = self.data[..bit_len.div_ceil(u8::BITS as usize)].to_vec();
        BitStream::from_bytes_with_bits(bytes, bit_len).map_err(|_| SendError::NativeCallFailed)
    }

    fn as_mut_ptr(&mut self) -> *mut RawBitStream {
        self.raw.data = if self.data.is_empty() {
            self.raw.stack_data.as_mut_ptr()
        } else {
            self.data.as_mut_ptr()
        };
        &mut self.raw
    }
}

fn native_bit_length(bit_len: usize) -> Result<i32, SendError> {
    i32::try_from(bit_len).map_err(|_| SendError::PayloadTooLarge)
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
mod layout_tests {
    use super::{PacketPlayerId, RawPacket};
    use std::mem::{MaybeUninit, align_of, offset_of, size_of};
    use std::ptr;

    unsafe extern "C" {
        fn samp_client_sdk_fixture_player_id_size() -> usize;
        fn samp_client_sdk_fixture_player_id_alignment() -> usize;
        fn samp_client_sdk_fixture_packet_size() -> usize;
        fn samp_client_sdk_fixture_packet_alignment() -> usize;
        fn samp_client_sdk_fixture_packet_player_index_offset() -> usize;
        fn samp_client_sdk_fixture_packet_player_id_offset() -> usize;
        fn samp_client_sdk_fixture_packet_length_offset() -> usize;
        fn samp_client_sdk_fixture_packet_bit_size_offset() -> usize;
        fn samp_client_sdk_fixture_packet_data_offset() -> usize;
        fn samp_client_sdk_fixture_packet_delete_data_offset() -> usize;
        fn samp_client_sdk_fixture_initialize_packet(memory: *mut RawPacket, data: *mut u8);
    }

    #[test]
    fn raknet_packet_layout_matches_the_cpp_x86_abi() {
        unsafe {
            assert_eq!(
                size_of::<PacketPlayerId>(),
                samp_client_sdk_fixture_player_id_size()
            );
            assert_eq!(
                align_of::<PacketPlayerId>(),
                samp_client_sdk_fixture_player_id_alignment()
            );

            assert_eq!(
                size_of::<RawPacket>(),
                samp_client_sdk_fixture_packet_size()
            );
            assert_eq!(
                align_of::<RawPacket>(),
                samp_client_sdk_fixture_packet_alignment()
            );
            assert_eq!(
                offset_of!(RawPacket, player_index),
                samp_client_sdk_fixture_packet_player_index_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, player_id),
                samp_client_sdk_fixture_packet_player_id_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, length),
                samp_client_sdk_fixture_packet_length_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, bit_size),
                samp_client_sdk_fixture_packet_bit_size_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, data),
                samp_client_sdk_fixture_packet_data_offset()
            );
            assert_eq!(
                offset_of!(RawPacket, delete_data),
                samp_client_sdk_fixture_packet_delete_data_offset()
            );
        }
    }

    #[test]
    fn reads_a_packet_initialized_by_cpp() {
        let mut data = [0xAA, 0xBB, 0xCC];
        let mut packet = MaybeUninit::<RawPacket>::uninit();
        unsafe {
            samp_client_sdk_fixture_initialize_packet(packet.as_mut_ptr(), data.as_mut_ptr());
            let packet = packet.assume_init();
            assert_eq!(ptr::addr_of!(packet.player_index).read_unaligned(), 0x1234);
            assert_eq!(
                ptr::addr_of!(packet.player_id.binary_address).read_unaligned(),
                0x01020304
            );
            assert_eq!(
                ptr::addr_of!(packet.player_id.port).read_unaligned(),
                0x5678
            );
            assert_eq!(ptr::addr_of!(packet.length).read_unaligned(), 3);
            assert_eq!(ptr::addr_of!(packet.bit_size).read_unaligned(), 17);
            assert_eq!(
                ptr::addr_of!(packet.data).read_unaligned(),
                data.as_mut_ptr()
            );
            assert!(ptr::addr_of!(packet.delete_data).read_unaligned());
        }
    }
}

#[cfg(test)]
mod vtable_tests {
    use super::*;
    use crate::{Direction, command::GAME_COMMAND_QUEUE_CAPACITY, event::HookAction};
    use std::sync::atomic::{AtomicBool, AtomicU32};

    const FAKE_VTABLE_SLOTS: usize = 55;
    static ORIGINAL_PACKET_CALLED: AtomicBool = AtomicBool::new(false);
    static GAME_PROCESS_CALLS: AtomicU32 = AtomicU32::new(0);

    #[repr(C)]
    struct FakeClient {
        vtable: *mut usize,
    }

    unsafe extern "C" fn fake_method() {}
    unsafe extern "C" fn later_method() {}
    unsafe extern "thiscall" fn fake_outgoing_packet(
        _client: *mut c_void,
        _stream: *mut RawBitStream,
        _priority: i32,
        _reliability: i32,
        _channel: i8,
    ) -> bool {
        ORIGINAL_PACKET_CALLED.store(true, Ordering::Release);
        true
    }

    unsafe extern "thiscall" fn fake_game_process(_game: *mut c_void) {
        GAME_PROCESS_CALLS.fetch_add(1, Ordering::AcqRel);
    }

    fn test_backend_state() -> BackendState {
        BackendState {
            registry: Registry::new(),
            module_base: 0,
            version: SampVersion::R1,
            addresses: AddressSet::for_version(SampVersion::R1),
            r1_client: None,
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
            game_thread_id: AtomicU32::new(0),
            outgoing_packet_original: AtomicUsize::new(0),
            incoming_packet_original: AtomicUsize::new(0),
            deallocate_packet_original: AtomicUsize::new(0),
            outgoing_rpc_original: AtomicUsize::new(0),
            client_hook_status: AtomicU32::new(ClientHookInstallState::Pending.as_raw()),
            incoming_packet_diagnostic_logged: AtomicBool::new(false),
            string_codec: Mutex::new(()),
            game_commands: CommandQueue::new(),
            local_player_snapshot: Mutex::new(None),
            local_player_candidate: Mutex::new(None),
            player_info_cache: Mutex::new(vec![PlayerInfoCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
            player_info_requests: Mutex::new(VecDeque::new()),
            remote_player_state_cache: Mutex::new(vec![
                RemotePlayerStateCacheEntry::Unknown;
                MAX_SAMP_PLAYERS
            ]),
            remote_player_state_requests: Mutex::new(VecDeque::new()),
            vehicle_exists_cache: Mutex::new(vec![
                VehicleExistsCacheEntry::Unknown;
                MAX_SAMP_VEHICLES
            ]),
            vehicle_exists_requests: Mutex::new(VecDeque::new()),
            text_label_exists_cache: Mutex::new(vec![
                TextLabelExistsCacheEntry::Unknown;
                MAX_SAMP_TEXT_LABELS
            ]),
            text_label_exists_requests: Mutex::new(VecDeque::new()),
            text_label_cache: Mutex::new(vec![TextLabelCacheEntry::Unknown; MAX_SAMP_TEXT_LABELS]),
            text_label_requests: Mutex::new(VecDeque::new()),
            textdraw_exists_cache: Mutex::new(vec![
                TextdrawExistsCacheEntry::Unknown;
                MAX_SAMP_TEXTDRAWS
            ]),
            textdraw_exists_requests: Mutex::new(VecDeque::new()),
            textdraw_cache: Mutex::new(vec![TextdrawCacheEntry::Unknown; MAX_SAMP_TEXTDRAWS]),
            textdraw_requests: Mutex::new(VecDeque::new()),
            chat_entry_cache: Mutex::new(vec![ChatEntryCacheEntry::Unknown; MAX_CHAT_ENTRIES]),
            chat_entry_requests: Mutex::new(VecDeque::new()),
            object_exists_cache: Mutex::new(vec![
                ObjectExistsCacheEntry::Unknown;
                MAX_SAMP_OBJECTS
            ]),
            object_exists_requests: Mutex::new(VecDeque::new()),
            gangzone_cache: Mutex::new(vec![GangzoneCacheEntry::Unknown; MAX_SAMP_GANGZONES]),
            gangzone_requests: Mutex::new(VecDeque::new()),
            object_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_OBJECTS]),
            object_handle_requests: Mutex::new(VecDeque::new()),
            object_handle_reverse_cache: Mutex::new(HashMap::new()),
            object_handle_reverse_requests: Mutex::new(VecDeque::new()),
            pickup_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_PICKUPS]),
            pickup_handle_requests: Mutex::new(VecDeque::new()),
            pickup_handle_reverse_cache: Mutex::new(HashMap::new()),
            pickup_handle_reverse_requests: Mutex::new(VecDeque::new()),
            vehicle_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_VEHICLES]),
            vehicle_handle_requests: Mutex::new(VecDeque::new()),
            vehicle_handle_reverse_cache: Mutex::new(HashMap::new()),
            vehicle_handle_reverse_requests: Mutex::new(VecDeque::new()),
            player_handle_cache: Mutex::new(vec![HandleCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
            player_handle_requests: Mutex::new(VecDeque::new()),
            player_handle_reverse_cache: Mutex::new(HashMap::new()),
            player_handle_reverse_requests: Mutex::new(VecDeque::new()),
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
            local_chat_input_active: AtomicBool::new(false),
            local_chat_input_active_ready: AtomicBool::new(false),
            local_chat_input_text: Mutex::new(None),
            local_chat_input_text_ready: AtomicBool::new(false),
            animation_catalog: Mutex::new(None),
            cache_generation: AtomicU64::new(2),
            hooks: Mutex::new(HookStorage::default()),
        }
    }

    fn test_dialog(id: u16) -> LocalDialogRequest {
        LocalDialogRequest {
            id,
            style: crate::runtime::LocalDialogStyle::MessageBox,
            title: b"title".to_vec(),
            text: b"text".to_vec(),
            button1: b"ok".to_vec(),
            button2: Vec::new(),
        }
    }

    fn test_chat_message() -> LocalChatMessageRequest {
        LocalChatMessageRequest {
            style: crate::runtime::LocalChatMessageStyle::Debug,
            text: b"text".to_vec(),
            prefix: b"prefix".to_vec(),
            text_colour: 0,
            prefix_colour: 0,
        }
    }

    fn test_death_message() -> LocalDeathMessageRequest {
        LocalDeathMessageRequest {
            killer: b"killer".to_vec(),
            victim: b"victim".to_vec(),
            killer_colour: 0,
            victim_colour: 0,
            weapon: 24,
        }
    }

    fn test_snapshot(id: u16) -> LocalPlayerSnapshot {
        LocalPlayerSnapshot {
            id,
            nickname: b"fixture".to_vec(),
            colour: 0,
            spawned: true,
            health: 100.0,
            armour: 0.0,
            position: crate::runtime::Vector3::default(),
            velocity: crate::runtime::Vector3::default(),
            special_action: 0,
            animation_id: 0,
            vehicle_id: None,
            score: 0,
            ping: 0,
        }
    }

    #[test]
    fn direct_helpers_are_unsupported_without_the_r1_profile() {
        let state = test_backend_state();
        assert_eq!(
            state.show_local_dialog(test_dialog(1)),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.show_local_chat_message(test_chat_message()),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.show_local_death_message(test_death_message()),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_player(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.player_info(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.player_count(true),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.player_max_id(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.vehicle_exists(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.text_label_exists(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.text_label(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.textdraw_exists(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.textdraw(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.object_exists(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.gangzone(7),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.samp_game_state(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_chat_display_mode(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_cursor_mode(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_scoreboard_open(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_dialog_active(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_dialog_state(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.submit_local_dialog_editbox_text(b"fixture".to_vec()),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_chat_input_active(),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_animation(0),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.local_animation_id(b"AIRPORT", b"THRW_BARL_THRW"),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            state.server_info(),
            Err(DirectClientError::UnsupportedVersion)
        );
    }

    #[test]
    fn handle_reads_are_deduplicated_queued_and_published_per_pump() {
        let mut state = test_backend_state();
        state.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
        state.rak_client.store(0x1000, Ordering::Release);
        state.cache_generation.store(2, Ordering::Release);

        assert_eq!(state.object_handle(7), Err(DirectClientError::NotReady));
        state
            .queue_handle_request(&state.object_handle_requests, 32, 7)
            .unwrap();
        state
            .queue_handle_request(&state.object_handle_requests, 32, 7)
            .unwrap();
        assert_eq!(state.object_handle_requests.lock().unwrap().len(), 1);

        state.object_handle_cache.lock().unwrap()[7] = HandleCacheEntry::Known(None);
        assert_eq!(state.object_handle(7), Ok(None));

        assert_eq!(
            state.object_id_by_handle(42),
            Err(DirectClientError::NotReady)
        );
        state
            .object_handle_reverse_cache
            .lock()
            .unwrap()
            .insert(42, Some(7));
        assert_eq!(state.object_id_by_handle(42), Ok(Some(7)));
    }

    #[test]
    fn handle_reverse_requests_are_deduplicated() {
        let mut state = test_backend_state();
        state.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
        state.rak_client.store(0x1000, Ordering::Release);
        state.cache_generation.store(2, Ordering::Release);

        state
            .queue_handle_id_request(&state.object_handle_reverse_requests, 16, 42)
            .unwrap();
        state
            .queue_handle_id_request(&state.object_handle_reverse_requests, 16, 42)
            .unwrap();
        assert_eq!(
            state.object_handle_reverse_requests.lock().unwrap().len(),
            1
        );

        state
            .object_handle_reverse_cache
            .lock()
            .unwrap()
            .insert(42, None);
        assert_eq!(state.object_id_by_handle(42), Ok(None));
    }

    #[test]
    fn handle_caches_are_cleared_across_connection_boundaries() {
        let state = test_backend_state();
        state.object_handle_cache.lock().unwrap()[7] = HandleCacheEntry::Known(Some(42));
        state.object_handle_requests.lock().unwrap().push_back(7);
        state
            .object_handle_reverse_cache
            .lock()
            .unwrap()
            .insert(42, Some(7));
        state
            .object_handle_reverse_requests
            .lock()
            .unwrap()
            .push_back(42);
        state.pickup_handle_cache.lock().unwrap()[7] = HandleCacheEntry::Known(Some(42));
        state.vehicle_handle_cache.lock().unwrap()[7] = HandleCacheEntry::Known(Some(42));
        state.player_handle_cache.lock().unwrap()[7] = HandleCacheEntry::Known(Some(42));

        state.invalidate_connection_state();

        assert!(matches!(
            state.object_handle_cache.lock().unwrap()[7],
            HandleCacheEntry::Unknown
        ));
        assert!(matches!(
            state.pickup_handle_cache.lock().unwrap()[7],
            HandleCacheEntry::Unknown
        ));
        assert!(matches!(
            state.vehicle_handle_cache.lock().unwrap()[7],
            HandleCacheEntry::Unknown
        ));
        assert!(matches!(
            state.player_handle_cache.lock().unwrap()[7],
            HandleCacheEntry::Unknown
        ));
        assert!(state.object_handle_requests.lock().unwrap().is_empty());
        assert!(state.object_handle_reverse_cache.lock().unwrap().is_empty());
        assert!(
            state
                .object_handle_reverse_requests
                .lock()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn dialog_editbox_text_command_is_bounded_and_queued() {
        let mut state = test_backend_state();
        state.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
        state.rak_client.store(0x1000, Ordering::Release);
        let mut oversized = vec![b'x'; 129];
        oversized.push(0);
        assert_eq!(
            state.submit_local_dialog_editbox_text(oversized),
            Err(DirectClientError::NotReady)
        );
        let id = state
            .submit_local_dialog_editbox_text(b"fixture".to_vec())
            .unwrap();
        let snapshot = state.game_commands.take_tick_snapshot();
        assert_eq!(snapshot.len(), 1);
        assert_eq!(snapshot[0].id, id);
        assert!(matches!(
            &snapshot[0].command,
            GameCommand::SetDialogEditboxText(text) if text == b"fixture"
        ));
    }

    #[test]
    fn cached_game_state_requires_the_profile_client_and_game_thread_publication() {
        assert_eq!(
            cached_direct_client_value(false, true, true, Some(14)),
            Err(DirectClientError::UnsupportedVersion)
        );
        assert_eq!(
            cached_direct_client_value(true, false, true, Some(14)),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            cached_direct_client_value(true, true, true, None::<i32>),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            cached_direct_client_value(true, true, true, Some(14)),
            Ok(14)
        );
        assert_eq!(
            cached_direct_client_value(true, true, false, Some(14)),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn cached_chat_display_mode_requires_game_thread_publication() {
        assert_eq!(
            cached_direct_client_value(true, true, true, None::<i32>),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(cached_direct_client_value(true, true, true, Some(2)), Ok(2));
    }

    #[test]
    fn cached_ui_flags_require_game_thread_publication() {
        assert_eq!(
            cached_direct_client_value(true, true, true, None::<bool>),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            cached_direct_client_value(true, true, true, Some(true)),
            Ok(true)
        );
    }

    #[test]
    fn game_command_queue_is_shared_fifo_and_bounded() {
        let state = test_backend_state();
        state.queue_local_dialog(test_dialog(7)).unwrap();
        state.queue_local_chat_message(test_chat_message()).unwrap();
        state
            .queue_local_death_message(test_death_message())
            .unwrap();
        for id in 3..GAME_COMMAND_QUEUE_CAPACITY as u16 {
            state.queue_local_dialog(test_dialog(id)).unwrap();
        }
        assert_eq!(
            state.queue_local_chat_message(test_chat_message()),
            Err(DirectClientError::QueueFull)
        );

        let snapshot = state.game_commands.take_tick_snapshot();
        assert_eq!(snapshot.len(), GAME_COMMAND_QUEUE_CAPACITY);
        assert!(matches!(
            &snapshot[0].command,
            GameCommand::ShowDialog(request) if request.id == 7
        ));
        assert!(matches!(
            &snapshot[1].command,
            GameCommand::AddChatMessage(_)
        ));
        assert!(matches!(
            &snapshot[2].command,
            GameCommand::AddDeathMessage(_)
        ));
        assert!(matches!(
            &snapshot[3].command,
            GameCommand::ShowDialog(request) if request.id == 3
        ));
    }

    #[test]
    fn network_commands_copy_payloads_and_detach_the_legacy_waiter() {
        let state = test_backend_state();
        let mut payload = BitStream::new();
        payload.write_u8(0xAB).unwrap();

        assert_eq!(
            state.send_packet(99, &payload, SendOptions::default()),
            Ok(true)
        );
        payload.write_u8(0xCD).unwrap();

        let snapshot = state.game_commands.take_tick_snapshot();
        assert_eq!(snapshot.len(), 1);
        assert!(matches!(
            &snapshot[0].command,
            GameCommand::SendPacket {
                id: 99,
                payload: queued,
                options: SendOptions { .. },
            } if queued.as_bytes() == [0xAB]
        ));
    }

    #[test]
    fn game_tick_calls_original_once_and_marks_the_game_thread() {
        let state = test_backend_state();
        GAME_PROCESS_CALLS.store(0, Ordering::Release);

        unsafe { state.run_game_process_tick(ptr::null_mut(), fake_game_process) };

        assert_eq!(GAME_PROCESS_CALLS.load(Ordering::Acquire), 1);
        assert!(state.is_game_thread());
    }

    #[test]
    fn command_wait_is_rejected_on_the_published_game_thread() {
        let state = Arc::new(test_backend_state());
        state
            .game_thread_id
            .store(unsafe { GetCurrentThreadId() }, Ordering::Release);
        let id = state
            .game_commands
            .submit(GameCommand::ShowDialog(test_dialog(1)))
            .unwrap();
        let backend = Backend {
            state: Arc::clone(&state),
        };

        assert_eq!(
            backend.wait_for_command(id, Duration::ZERO),
            Err(CommandError::WaitRejected)
        );
    }

    #[test]
    fn connection_boundary_invalidates_cached_entities_and_pending_refreshes() {
        let state = test_backend_state();
        state.cache_local_player_snapshot(Some(test_snapshot(42)));
        state.cache_local_player_snapshot(Some(test_snapshot(42)));
        state.player_info_cache.lock().unwrap()[7] =
            PlayerInfoCacheEntry::Known(Some(player_info_from_local(&test_snapshot(7))));
        state.remote_player_state_cache.lock().unwrap()[7] =
            RemotePlayerStateCacheEntry::Known(Some(RemotePlayerStateSnapshot {
                id: 7,
                health: 90.0,
                armour: 20.0,
                special_action: 0,
                animation_id: 0,
            }));
        state.vehicle_exists_cache.lock().unwrap()[7] = VehicleExistsCacheEntry::Known(true);
        state.text_label_exists_cache.lock().unwrap()[7] = TextLabelExistsCacheEntry::Known(true);
        state.text_label_cache.lock().unwrap()[7] = TextLabelCacheEntry::Known(None);
        state.textdraw_exists_cache.lock().unwrap()[7] = TextdrawExistsCacheEntry::Known(true);
        state.textdraw_cache.lock().unwrap()[7] = TextdrawCacheEntry::Known(None);
        state.object_exists_cache.lock().unwrap()[7] = ObjectExistsCacheEntry::Known(true);
        state.gangzone_cache.lock().unwrap()[7] = GangzoneCacheEntry::Known(None);
        state.player_info_requests.lock().unwrap().push_back(7);
        state
            .remote_player_state_requests
            .lock()
            .unwrap()
            .push_back(7);
        state.vehicle_exists_requests.lock().unwrap().push_back(7);
        state
            .text_label_exists_requests
            .lock()
            .unwrap()
            .push_back(7);
        state.text_label_requests.lock().unwrap().push_back(7);
        state.textdraw_exists_requests.lock().unwrap().push_back(7);
        state.textdraw_requests.lock().unwrap().push_back(7);
        state.object_exists_requests.lock().unwrap().push_back(7);
        state.gangzone_requests.lock().unwrap().push_back(7);
        state.player_count_ready.store(true, Ordering::Release);
        state.player_max_id_ready.store(true, Ordering::Release);

        state.invalidate_connection_state();

        assert!(state.local_player_snapshot.lock().unwrap().is_none());
        assert!(matches!(
            state.player_info_cache.lock().unwrap()[7],
            PlayerInfoCacheEntry::Unknown
        ));
        assert!(matches!(
            state.remote_player_state_cache.lock().unwrap()[7],
            RemotePlayerStateCacheEntry::Unknown
        ));
        assert!(matches!(
            state.vehicle_exists_cache.lock().unwrap()[7],
            VehicleExistsCacheEntry::Unknown
        ));
        assert!(matches!(
            state.text_label_exists_cache.lock().unwrap()[7],
            TextLabelExistsCacheEntry::Unknown
        ));
        assert!(matches!(
            state.text_label_cache.lock().unwrap()[7],
            TextLabelCacheEntry::Unknown
        ));
        assert!(matches!(
            state.textdraw_exists_cache.lock().unwrap()[7],
            TextdrawExistsCacheEntry::Unknown
        ));
        assert!(matches!(
            state.textdraw_cache.lock().unwrap()[7],
            TextdrawCacheEntry::Unknown
        ));
        assert!(matches!(
            state.object_exists_cache.lock().unwrap()[7],
            ObjectExistsCacheEntry::Unknown
        ));
        assert!(matches!(
            state.gangzone_cache.lock().unwrap()[7],
            GangzoneCacheEntry::Unknown
        ));
        assert!(state.player_info_requests.lock().unwrap().is_empty());
        assert!(
            state
                .remote_player_state_requests
                .lock()
                .unwrap()
                .is_empty()
        );
        assert!(state.vehicle_exists_requests.lock().unwrap().is_empty());
        assert!(state.text_label_exists_requests.lock().unwrap().is_empty());
        assert!(state.text_label_requests.lock().unwrap().is_empty());
        assert!(state.textdraw_exists_requests.lock().unwrap().is_empty());
        assert!(state.textdraw_requests.lock().unwrap().is_empty());
        assert!(state.object_exists_requests.lock().unwrap().is_empty());
        assert!(state.gangzone_requests.lock().unwrap().is_empty());
        assert!(!state.player_count_ready.load(Ordering::Acquire));
        assert!(!state.player_max_id_ready.load(Ordering::Acquire));
    }

    #[test]
    fn player_directory_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_player_info_request(7).unwrap();
        state.queue_player_info_request(7).unwrap();
        assert_eq!(state.player_info_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + PLAYER_INFO_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_player_info_request(id).unwrap();
        }
        assert_eq!(
            state.queue_player_info_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_player_info_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.player_info_requests.lock().unwrap().len(),
            PLAYER_INFO_REQUEST_QUEUE_CAPACITY - PLAYER_INFO_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn remote_player_state_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_remote_player_state_request(7).unwrap();
        state.queue_remote_player_state_request(7).unwrap();
        for id in 8..(7 + REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_remote_player_state_request(id).unwrap();
        }
        assert_eq!(
            state.queue_remote_player_state_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_remote_player_state_requests();
        assert_eq!(drained.len(), REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP);
        assert_eq!(drained[0], 7);
        assert_eq!(
            state.remote_player_state_requests.lock().unwrap().len(),
            REMOTE_PLAYER_STATE_REQUEST_QUEUE_CAPACITY - REMOTE_PLAYER_STATE_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn vehicle_exists_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_vehicle_exists_request(7).unwrap();
        state.queue_vehicle_exists_request(7).unwrap();
        assert_eq!(state.vehicle_exists_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_vehicle_exists_request(id).unwrap();
        }
        assert_eq!(
            state.queue_vehicle_exists_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_vehicle_exists_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.vehicle_exists_requests.lock().unwrap().len(),
            VEHICLE_EXISTS_REQUEST_QUEUE_CAPACITY - VEHICLE_EXISTS_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn text_label_exists_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_text_label_exists_request(7).unwrap();
        state.queue_text_label_exists_request(7).unwrap();
        assert_eq!(state.text_label_exists_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_text_label_exists_request(id).unwrap();
        }
        assert_eq!(
            state.queue_text_label_exists_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_text_label_exists_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.text_label_exists_requests.lock().unwrap().len(),
            TEXT_LABEL_EXISTS_REQUEST_QUEUE_CAPACITY - TEXT_LABEL_EXISTS_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn text_label_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_text_label_request(7).unwrap();
        state.queue_text_label_request(7).unwrap();
        assert_eq!(state.text_label_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + TEXT_LABEL_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_text_label_request(id).unwrap();
        }
        assert_eq!(
            state.queue_text_label_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_text_label_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.text_label_requests.lock().unwrap().len(),
            TEXT_LABEL_REQUEST_QUEUE_CAPACITY - TEXT_LABEL_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn textdraw_exists_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_textdraw_exists_request(7).unwrap();
        state.queue_textdraw_exists_request(7).unwrap();
        assert_eq!(state.textdraw_exists_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_textdraw_exists_request(id).unwrap();
        }
        assert_eq!(
            state.queue_textdraw_exists_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_textdraw_exists_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.textdraw_exists_requests.lock().unwrap().len(),
            TEXTDRAW_EXISTS_REQUEST_QUEUE_CAPACITY - TEXTDRAW_EXISTS_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn textdraw_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_textdraw_request(7).unwrap();
        state.queue_textdraw_request(7).unwrap();
        assert_eq!(state.textdraw_requests.lock().unwrap().len(), 1);
        for pool_index in 8..(7 + TEXTDRAW_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_textdraw_request(pool_index).unwrap();
        }
        assert_eq!(
            state.queue_textdraw_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_textdraw_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.textdraw_requests.lock().unwrap().len(),
            TEXTDRAW_REQUEST_QUEUE_CAPACITY - TEXTDRAW_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn chat_entry_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_chat_entry_request(7).unwrap();
        state.queue_chat_entry_request(7).unwrap();
        assert_eq!(state.chat_entry_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + CHAT_ENTRY_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_chat_entry_request(id).unwrap();
        }
        assert_eq!(
            state.queue_chat_entry_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_chat_entry_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.chat_entry_requests.lock().unwrap().len(),
            CHAT_ENTRY_REQUEST_QUEUE_CAPACITY - CHAT_ENTRY_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn chat_entry_reads_queue_unknown_and_return_published_snapshot() {
        let mut state = test_backend_state();
        state.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
        state.rak_client.store(0x1000, Ordering::Release);
        state.cache_generation.store(2, Ordering::Release);

        assert_eq!(state.chat_entry(7), Err(DirectClientError::NotReady));
        assert_eq!(state.chat_entry(7), Err(DirectClientError::NotReady));
        assert_eq!(state.chat_entry_requests.lock().unwrap().len(), 1);

        let snapshot = ChatEntrySnapshot {
            id: 7,
            text: b"message".to_vec(),
            prefix: b"name".to_vec(),
            text_colour: 0x1122_3344,
            prefix_colour: 0x5566_7788,
        };
        state.chat_entry_cache.lock().unwrap()[7] = ChatEntryCacheEntry::Known(snapshot.clone());

        assert_eq!(state.chat_entry(7), Ok(snapshot));
        assert_eq!(state.chat_entry_requests.lock().unwrap().len(), 1);
        assert_eq!(
            state.chat_entry(MAX_CHAT_ENTRIES as u16),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(state.chat_entry_requests.lock().unwrap().len(), 1);
    }

    #[test]
    fn object_exists_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_object_exists_request(7).unwrap();
        state.queue_object_exists_request(7).unwrap();
        assert_eq!(state.object_exists_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_object_exists_request(id).unwrap();
        }
        assert_eq!(
            state.queue_object_exists_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_object_exists_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.object_exists_requests.lock().unwrap().len(),
            OBJECT_EXISTS_REQUEST_QUEUE_CAPACITY - OBJECT_EXISTS_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn gangzone_requests_are_bounded_deduplicated_and_pump_limited() {
        let state = test_backend_state();
        state.queue_gangzone_request(7).unwrap();
        state.queue_gangzone_request(7).unwrap();
        assert_eq!(state.gangzone_requests.lock().unwrap().len(), 1);
        for id in 8..(7 + GANGZONE_REQUEST_QUEUE_CAPACITY as u16) {
            state.queue_gangzone_request(id).unwrap();
        }
        assert_eq!(
            state.queue_gangzone_request(99),
            Err(DirectClientError::QueueFull)
        );
        let drained = state.take_gangzone_requests();
        assert_eq!(drained, vec![7, 8, 9, 10]);
        assert_eq!(
            state.gangzone_requests.lock().unwrap().len(),
            GANGZONE_REQUEST_QUEUE_CAPACITY - GANGZONE_REQUESTS_PER_PUMP
        );
    }

    #[test]
    fn player_directory_reuses_the_owned_local_snapshot() {
        let player = player_info_from_local(&test_snapshot(42));
        assert_eq!(player.id, 42);
        assert_eq!(player.nickname, b"fixture");
        assert!(player.is_local);
        assert!(!player.is_npc);
    }

    #[test]
    fn local_snapshot_cache_publishes_only_a_stable_identity() {
        let state = test_backend_state();
        state.cache_local_player_snapshot(Some(test_snapshot(42)));
        assert!(state.local_player_snapshot.lock().unwrap().is_none());

        state.cache_local_player_snapshot(Some(test_snapshot(42)));
        assert_eq!(
            state
                .local_player_snapshot
                .lock()
                .unwrap()
                .as_ref()
                .map(|snapshot| snapshot.id),
            Some(42)
        );

        state.cache_local_player_snapshot(Some(test_snapshot(7)));
        assert!(state.local_player_snapshot.lock().unwrap().is_none());
        state.cache_local_player_snapshot(Some(test_snapshot(7)));
        assert_eq!(
            state
                .local_player_snapshot
                .lock()
                .unwrap()
                .as_ref()
                .map(|snapshot| snapshot.id),
            Some(7)
        );

        state.cache_local_player_snapshot(None);
        assert!(state.local_player_snapshot.lock().unwrap().is_none());
        assert!(state.local_player_candidate.lock().unwrap().is_none());
    }

    #[test]
    fn r1_connected_state_matches_the_fixed_native_value() {
        assert_eq!(R1_CONNECTED_GAME_STATE, 14);
        assert!(is_r1_connected_game_state(14));
        assert!(!is_r1_connected_game_state(13));
        assert!(!crosses_r1_connection_boundary(false, 0, 14));
        assert!(crosses_r1_connection_boundary(true, 13, 14));
        assert!(crosses_r1_connection_boundary(true, 14, 18));
        assert!(!crosses_r1_connection_boundary(true, 14, 14));
    }

    #[test]
    fn patches_only_owned_slots_and_preserves_a_later_hook() {
        let original = fake_method as *const () as usize;
        let mut table = vec![original; FAKE_VTABLE_SLOTS].into_boxed_slice();
        let untouched_slot = FAKE_VTABLE_SLOTS - 1;
        let untouched_original = table[untouched_slot];
        let mut client = FakeClient {
            vtable: table.as_mut_ptr(),
        };
        let state = test_backend_state();

        let hook = unsafe {
            VtableHook::install((&mut client as *mut FakeClient).cast::<c_void>(), &state).unwrap()
        };

        assert_eq!(
            table[OUTGOING_PACKET_SLOT],
            hooks::outgoing_packet_detour as *const () as usize
        );
        assert_eq!(
            table[INCOMING_PACKET_SLOT],
            hooks::incoming_packet_detour as *const () as usize
        );
        assert_eq!(
            table[OUTGOING_RPC_SLOT],
            hooks::outgoing_rpc_detour as *const () as usize
        );
        assert_eq!(table[untouched_slot], untouched_original);
        assert_eq!(
            state.outgoing_packet_original.load(Ordering::Acquire),
            original
        );

        let later_hook = later_method as *const () as usize;
        table[OUTGOING_PACKET_SLOT] = later_hook;
        drop(hook);

        assert_eq!(table[OUTGOING_PACKET_SLOT], later_hook);
        assert_eq!(table[INCOMING_PACKET_SLOT], original);
        assert_eq!(table[OUTGOING_RPC_SLOT], original);
        assert_eq!(table[untouched_slot], untouched_original);
    }

    #[test]
    fn captured_state_calls_original_after_active_slot_is_cleared() {
        ORIGINAL_PACKET_CALLED.store(false, Ordering::Release);
        let state = Arc::new(test_backend_state());
        state.outgoing_packet_original.store(
            fake_outgoing_packet as *const () as usize,
            Ordering::Release,
        );
        let active = ACTIVE_BACKEND.get_or_init(|| Mutex::new(None));
        *active.lock().unwrap_or_else(|error| error.into_inner()) = Some(Arc::downgrade(&state));

        let captured = Arc::clone(&state);
        clear_active_backend(&state);
        assert!(active_state().is_none());
        assert!(hooks::call_outgoing_packet(
            &captured,
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            0,
            0,
        ));
        assert!(ORIGINAL_PACKET_CALLED.load(Ordering::Acquire));
    }

    #[test]
    fn packet_emulation_requires_the_captured_rpc_receiver() {
        let state = test_backend_state();
        state.rak_client.store(0x1000, Ordering::Release);

        assert_eq!(state.ready_rpc_receiver(), Err(SendError::ClientNotReady));

        state.rpc_receiver.store(0x2000, Ordering::Release);
        assert_eq!(
            state.ready_rpc_receiver().map(|receiver| receiver as usize),
            Ok(0x2000)
        );
    }

    #[test]
    fn incoming_rpc_emulation_blocks_before_native_readiness_checks() {
        let state = test_backend_state();
        let _listener = state.registry.register_rpc(Direction::Incoming, |event| {
            assert_eq!(event.id(), 42);
            HookAction::Block
        });

        assert_eq!(
            state.emulate_incoming_rpc_native(42, BitStream::new()),
            Ok(false)
        );
    }

    #[test]
    fn client_hook_failure_is_observable_by_the_runtime() {
        let state = Arc::new(test_backend_state());
        let backend = Backend {
            state: Arc::clone(&state),
        };

        assert_eq!(backend.client_hook_status(), ClientHookStatus::Pending);
        state
            .client_hook_status
            .store(ClientHookInstallState::Failed.as_raw(), Ordering::Release);
        assert_eq!(backend.client_hook_status(), ClientHookStatus::Failed);
    }
}

#[cfg(test)]
mod inline_hook_tests {
    use super::*;
    use std::sync::{
        Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    static TEST_TRAMPOLINE: AtomicUsize = AtomicUsize::new(0);
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[inline(never)]
    unsafe extern "C" fn target(value: i32) -> i32 {
        value + 1
    }

    #[inline(never)]
    unsafe extern "C" fn detour(value: i32) -> i32 {
        let trampoline = TEST_TRAMPOLINE.load(Ordering::Acquire);
        if trampoline == 0 {
            return i32::MIN;
        }
        let original: unsafe extern "C" fn(i32) -> i32 = unsafe { mem::transmute(trampoline) };
        unsafe { original(value) + 10 }
    }

    #[test]
    fn publishes_trampoline_before_enabling_and_can_recreate_inline_hook() {
        let _serial = TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let target = target as *const () as usize;
        let detour = detour as *const () as usize;

        let (mut hook, trampoline) = InlineHook::create(target, detour).unwrap();

        // Creation must leave the target disabled until the caller publishes
        // the trampoline used by the detour.
        assert_eq!(unsafe { self::target(7) }, 8);
        TEST_TRAMPOLINE.store(trampoline, Ordering::Release);
        hook.enable().unwrap();
        assert_eq!(unsafe { self::target(7) }, 18);

        hook.disable();
        assert_eq!(unsafe { self::target(7) }, 8);

        let (recreated, recreated_trampoline) = InlineHook::create(target, detour).unwrap();
        assert_ne!(recreated_trampoline, 0);
        recreated.disable();
        TEST_TRAMPOLINE.store(0, Ordering::Release);
    }
}

struct VtableHook {
    vtable: usize,
    entries: [VtableEntry; 3],
}

#[derive(Clone, Copy)]
struct VtableEntry {
    slot: usize,
    original: usize,
    detour: usize,
}

struct InlineHook {
    target: usize,
    enabled: bool,
}

impl InlineHook {
    fn create(target: usize, detour: usize) -> Result<(Self, usize), ()> {
        let trampoline = unsafe {
            MinHook::create_hook(target as *mut c_void, detour as *mut c_void).map_err(|_| ())?
        };
        Ok((
            Self {
                target,
                enabled: false,
            },
            trampoline as usize,
        ))
    }

    fn enable(&mut self) -> Result<(), ()> {
        unsafe { MinHook::enable_hook(self.target as *mut c_void) }.map_err(|_| ())?;
        self.enabled = true;
        Ok(())
    }

    fn disable(mut self) {
        self.remove();
    }

    fn remove(&mut self) {
        if self.target == 0 {
            return;
        }
        let target = self.target as *mut c_void;
        if self.enabled {
            let _ = unsafe { MinHook::disable_hook(target) };
        }
        let _ = unsafe { MinHook::remove_hook(target) };
        self.target = 0;
        self.enabled = false;
    }
}

impl Drop for InlineHook {
    fn drop(&mut self) {
        self.remove();
    }
}

impl VtableHook {
    unsafe fn install(client: *mut c_void, state: &BackendState) -> Result<Self, AttachError> {
        let object_vtable = client.cast::<*mut usize>();
        let vtable = unsafe { object_vtable.read() };
        if vtable.is_null() {
            return Err(AttachError::ClientNotReady);
        }

        let replacements = [
            (
                OUTGOING_PACKET_SLOT,
                hooks::outgoing_packet_detour as *const () as usize,
            ),
            (
                INCOMING_PACKET_SLOT,
                hooks::incoming_packet_detour as *const () as usize,
            ),
            (
                OUTGOING_RPC_SLOT,
                hooks::outgoing_rpc_detour as *const () as usize,
            ),
        ];
        let mut entries = [VtableEntry {
            slot: 0,
            original: 0,
            detour: 0,
        }; 3];
        for (index, (slot, detour)) in replacements.into_iter().enumerate() {
            let original = unsafe { vtable.add(slot).read() };
            if original == 0 {
                return Err(AttachError::ClientNotReady);
            }
            entries[index] = VtableEntry {
                slot,
                original,
                detour,
            };
        }

        state
            .outgoing_packet_original
            .store(entries[0].original, Ordering::Release);
        state
            .incoming_packet_original
            .store(entries[1].original, Ordering::Release);
        state.deallocate_packet_original.store(
            unsafe { vtable.add(DEALLOCATE_PACKET_SLOT).read() },
            Ordering::Release,
        );
        state
            .outgoing_rpc_original
            .store(entries[2].original, Ordering::Release);

        for (index, entry) in entries.iter().enumerate() {
            if unsafe { write_protected(vtable.add(entry.slot), entry.detour) }.is_err() {
                for restore in entries[..index].iter().rev() {
                    let _ = unsafe { write_protected(vtable.add(restore.slot), restore.original) };
                }
                return Err(AttachError::HookInstallFailed("patching RakClient vtable"));
            }
        }

        Ok(Self {
            vtable: vtable as usize,
            entries,
        })
    }
}

impl Drop for VtableHook {
    fn drop(&mut self) {
        let vtable = self.vtable as *mut usize;
        for entry in self.entries.iter().rev() {
            let slot = unsafe { vtable.add(entry.slot) };
            if unsafe { slot.read() } == entry.detour {
                let _ = unsafe { write_protected(slot, entry.original) };
            }
        }
    }
}

type RakClientConstructorFn = unsafe extern "C" fn() -> *mut c_void;
type GameProcessFn = unsafe extern "thiscall" fn(*mut c_void);
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

unsafe extern "C" fn rak_client_constructor_detour() -> *mut c_void {
    let Some(state) = active_state() else {
        return ptr::null_mut();
    };
    let trampoline = state.constructor_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return ptr::null_mut();
    }
    let original: RakClientConstructorFn = unsafe { mem::transmute(trampoline) };
    let client = unsafe { original() };
    if !client.is_null()
        && let Err(error) = state.install_client_hooks(client)
    {
        state
            .client_hook_status
            .store(ClientHookInstallState::Failed.as_raw(), Ordering::Release);
        log::error!("RakClient hook installation failed: {error}");
    }
    client
}

unsafe extern "thiscall" fn game_process_detour(game: *mut c_void) {
    let Some(state) = active_state() else {
        return;
    };
    let trampoline = state.game_process_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return;
    }
    let original: GameProcessFn = unsafe { mem::transmute(trampoline) };
    unsafe { state.run_game_process_tick(game, original) };
}

#[cfg(test)]
mod packet_metadata_tests {
    use super::{hooks, native_bit_length};
    use crate::SendError;

    #[test]
    fn accepts_byte_aligned_and_partial_byte_packets() {
        assert_eq!(hooks::validated_packet_byte_len(2, 16), Some(2));
        assert_eq!(hooks::validated_packet_byte_len(2, 9), Some(2));
    }

    #[test]
    fn rejects_metadata_that_cannot_describe_the_buffer() {
        assert_eq!(hooks::validated_packet_byte_len(1, 7), None);
        assert_eq!(hooks::validated_packet_byte_len(1, 9), None);
        assert_eq!(
            hooks::validated_packet_byte_len(
                (hooks::MAX_INCOMING_PACKET_BYTES + 1) as u32,
                (hooks::MAX_INCOMING_PACKET_BYTES + 1) * 8
            ),
            None
        );
    }

    #[test]
    fn rejects_bit_lengths_that_overflow_native_i32() {
        assert_eq!(native_bit_length(i32::MAX as usize), Ok(i32::MAX));
        assert_eq!(
            native_bit_length(i32::MAX as usize + 1),
            Err(SendError::PayloadTooLarge)
        );
    }
}

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

unsafe fn write_protected<T>(address: *mut T, value: T) -> Result<(), AttachError> {
    let mut old_protection = 0;
    if unsafe {
        VirtualProtect(
            address.cast(),
            mem::size_of::<T>(),
            PAGE_READWRITE,
            &mut old_protection,
        )
    } == 0
    {
        return Err(AttachError::HookInstallFailed("changing vtable protection"));
    }
    unsafe { address.write(value) };
    let mut ignored = 0;
    let _ = unsafe {
        VirtualProtect(
            address.cast(),
            mem::size_of::<T>(),
            old_protection,
            &mut ignored,
        )
    };
    Ok(())
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
