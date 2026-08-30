//! Shared Win32 backend test fixtures.

use super::*;

pub(super) fn test_backend_state() -> BackendState {
    BackendState {
        context: BackendContext {
            registry: Registry::new(),
            module_base: 0,
            version: SampVersion::R1,
            addresses: AddressSet::for_version(SampVersion::R1),
            native_client_profile: None,
            gta_profile: GtaProfile::select(0x0040_0000, gta_sa_native::GTA_SA_10_US_SHA256)
                .unwrap(),
        },
        game_tick: GameTickRuntime::new(
            GtaProfile::select(0x0040_0000, gta_sa_native::GTA_SA_10_US_SHA256).unwrap(),
        ),
        game_scope: GameThreadScope::new(),
        rak_client: AtomicUsize::new(0),
        raw_player_pool: AtomicUsize::new(0),
        raw_vehicle_pool: AtomicUsize::new(0),
        raw_local_player: AtomicUsize::new(0),
        rpc_receiver: AtomicUsize::new(0),
        player_address: AtomicU32::new(0),
        player_port: AtomicU16::new(0),
        constructor_trampoline: AtomicUsize::new(0),
        incoming_rpc_trampoline: AtomicUsize::new(0),
        dialog_close_trampoline: AtomicUsize::new(0),
        outgoing_packet_original: AtomicUsize::new(0),
        incoming_packet_original: AtomicUsize::new(0),
        deallocate_packet_original: AtomicUsize::new(0),
        outgoing_rpc_original: AtomicUsize::new(0),
        client_hook_status: AtomicU32::new(ClientHookInstallState::Pending.as_raw()),
        incoming_packet_diagnostic_logged: AtomicBool::new(false),
        game_command_snapshot_diagnostic_logged: AtomicBool::new(false),
        game_command_completion_diagnostic_logged: AtomicBool::new(false),
        string_codec: Mutex::new(()),
        pending_game_tick: Mutex::new(None),
        game_commands: CommandQueue::new(),
        gta_read_results: Mutex::new(HashMap::new()),
        auto_text_label_creates: Mutex::new(HashMap::new()),
        local_player_snapshot: Mutex::new(None),
        local_player_candidate: Mutex::new(None),
        player_info_cache: Mutex::new(vec![PlayerInfoCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        player_info_requests: Mutex::new(VecDeque::new()),
        remote_player_state_cache: Mutex::new(vec![
            RemotePlayerStateCacheEntry::Unknown;
            MAX_SAMP_PLAYERS
        ]),
        remote_player_state_requests: Mutex::new(VecDeque::new()),
        marker_sync_positions: Mutex::new(vec![None; MAX_SAMP_PLAYERS]),
        streamed_out_player_position_cache: Mutex::new(vec![
            StreamedOutPlayerPositionCacheEntry::Unknown;
            MAX_SAMP_PLAYERS
        ]),
        streamed_out_player_position_requests: Mutex::new(VecDeque::new()),
        onfoot_sync_cache: Mutex::new(vec![OnFootSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        onfoot_sync_requests: Mutex::new(VecDeque::new()),
        incar_sync_cache: Mutex::new(vec![InCarSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        incar_sync_requests: Mutex::new(VecDeque::new()),
        passenger_sync_cache: Mutex::new(vec![PassengerSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        passenger_sync_requests: Mutex::new(VecDeque::new()),
        trailer_sync_cache: Mutex::new(vec![TrailerSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        trailer_sync_requests: Mutex::new(VecDeque::new()),
        aim_sync_cache: Mutex::new(vec![AimSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        aim_sync_requests: Mutex::new(VecDeque::new()),
        vehicle_exists_cache: Mutex::new(vec![VehicleExistsCacheEntry::Unknown; MAX_SAMP_VEHICLES]),
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
        object_exists_cache: Mutex::new(vec![ObjectExistsCacheEntry::Unknown; MAX_SAMP_OBJECTS]),
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
        local_dialog_response: Mutex::new(None),
        local_chat_input_active: AtomicBool::new(false),
        local_chat_input_active_ready: AtomicBool::new(false),
        local_chat_input_text: Mutex::new(None),
        local_chat_input_text_ready: AtomicBool::new(false),
        local_chat_input_commands: Mutex::new(None),
        local_chat_input_commands_ready: AtomicBool::new(false),
        animation_catalog: Mutex::new(None),
        cache_generation: AtomicU64::new(2),
        hooks: Mutex::new(HookStorage::default()),
    }
}

pub(super) fn r1_native_profile() -> Option<NativeClientProfile> {
    NativeClientProfile::select(0x10000, SampVersion::R1, SampVersion::R1.entry_point())
}

pub(super) fn r3_native_profile() -> Option<NativeClientProfile> {
    NativeClientProfile::select(0x10000, SampVersion::R3_1, SampVersion::R3_1.entry_point())
}

pub(super) fn r3_native_client_profile() -> Option<NativeClientProfile> {
    NativeClientProfile::select(0x10000, SampVersion::R3_1, SampVersion::R3_1.entry_point())
}

pub(super) fn test_dialog(id: u16) -> LocalDialogRequest {
    LocalDialogRequest {
        id,
        style: crate::runtime::LocalDialogStyle::MessageBox,
        title: b"title".to_vec(),
        text: b"text".to_vec(),
        button1: b"ok".to_vec(),
        button2: Vec::new(),
    }
}

pub(super) fn test_chat_message() -> LocalChatMessageRequest {
    LocalChatMessageRequest {
        style: crate::runtime::LocalChatMessageStyle::Debug,
        text: b"text".to_vec(),
        prefix: b"prefix".to_vec(),
        text_colour: 0,
        prefix_colour: 0,
    }
}

pub(super) fn test_death_message() -> LocalDeathMessageRequest {
    LocalDeathMessageRequest {
        killer: b"killer".to_vec(),
        victim: b"victim".to_vec(),
        killer_colour: 0,
        victim_colour: 0,
        weapon: 24,
    }
}

pub(super) fn test_snapshot(id: u16) -> LocalPlayerSnapshot {
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
