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
        context: BackendContext {
            registry: Registry::new(),
            module_base: 0,
            version: SampVersion::R1,
            addresses: AddressSet::for_version(SampVersion::R1),
            r1_client: None,
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
        game_thread_id: AtomicU32::new(0),
        outgoing_packet_original: AtomicUsize::new(0),
        incoming_packet_original: AtomicUsize::new(0),
        deallocate_packet_original: AtomicUsize::new(0),
        outgoing_rpc_original: AtomicUsize::new(0),
        client_hook_status: AtomicU32::new(ClientHookInstallState::Pending.as_raw()),
        incoming_packet_diagnostic_logged: AtomicBool::new(false),
        string_codec: Mutex::new(()),
        game_commands: CommandQueue::new(),
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
        onfoot_sync_cache: Mutex::new(vec![OnFootSyncCacheEntry::Unknown; MAX_SAMP_PLAYERS]),
        onfoot_sync_requests: Mutex::new(VecDeque::new()),
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
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
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
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
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
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
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
fn cached_chat_command_lookup_uses_exact_published_names() {
    let mut state = test_backend_state();
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(
        state.local_chat_command_defined(b"sdk"),
        Err(DirectClientError::NotReady)
    );
    *state.local_chat_input_commands.lock().unwrap() = Some(vec![b"sdk".to_vec()]);
    state
        .local_chat_input_commands_ready
        .store(true, Ordering::Release);

    assert_eq!(state.local_chat_command_defined(b"sdk"), Ok(true));
    assert_eq!(state.local_chat_command_defined(b"SDK"), Ok(false));
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
fn typed_text_label_receipt_returns_the_game_thread_selected_id() {
    let state = test_backend_state();
    let command = state
        .game_commands
        .submit(GameCommand::DeleteTextLabel(0))
        .unwrap();
    state
        .auto_text_label_creates
        .lock()
        .unwrap()
        .insert(command, Some(7));
    state.game_commands.complete(command, Ok(()));

    assert_eq!(state.try_take_created_text_label(command), Ok(Some(Ok(7))));
    assert!(state.auto_text_label_creates.lock().unwrap().is_empty());
}

#[test]
fn text_label_text_update_copies_nonempty_text_into_the_game_command() {
    let mut state = test_backend_state();
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
    state.rak_client.store(0x1000, Ordering::Release);

    let mut text = b"updated".to_vec();
    state.submit_set_text_label_text(7, text.clone()).unwrap();
    text[0] = b'X';

    let snapshot = state.game_commands.take_tick_snapshot();
    assert!(matches!(
        &snapshot[0].command,
        GameCommand::SetTextLabelText { id: 7, text } if text.as_slice() == b"updated"
    ));
    assert_eq!(
        state.submit_set_text_label_text(7, Vec::new()),
        Err(DirectClientError::NotReady)
    );
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
fn onfoot_sync_reads_owned_cache_and_queues_a_refresh() {
    let mut state = test_backend_state();
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(state.onfoot_sync(7), Err(DirectClientError::NotReady));
    assert_eq!(
        state.onfoot_sync_requests.lock().unwrap().as_slices().0,
        &[7]
    );

    let snapshot = OnFootSyncSnapshot {
        id: 7,
        controller_left_stick_x: -100,
        controller_left_stick_y: 200,
        controller_buttons: 0x1234,
        position: crate::runtime::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        quaternion: [0.0, 0.0, 0.0, 1.0],
        health: 75,
        armour: 25,
        weapon: 24,
        special_action: 3,
        speed: crate::runtime::Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        surfing_offset: crate::runtime::Vector3 {
            x: 7.0,
            y: 8.0,
            z: 9.0,
        },
        surfing_vehicle_id: u16::MAX,
        animation: 0x1234_5678,
    };
    state.onfoot_sync_cache.lock().unwrap()[7] = OnFootSyncCacheEntry::Known(Some(snapshot));

    assert_eq!(state.onfoot_sync(7), Ok(Some(snapshot)));
}

#[test]
fn onfoot_sync_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_onfoot_sync_request(7).unwrap();
    state.queue_onfoot_sync_request(7).unwrap();
    for id in 8..(7 + ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_onfoot_sync_request(id).unwrap();
    }
    assert_eq!(
        state.queue_onfoot_sync_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_onfoot_sync_requests();
    assert_eq!(drained.len(), ONFOOT_SYNC_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.onfoot_sync_requests.lock().unwrap().len(),
        ONFOOT_SYNC_REQUEST_QUEUE_CAPACITY - ONFOOT_SYNC_REQUESTS_PER_PUMP
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
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
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
fn contended_request_enqueue_returns_busy_without_losing_work() {
    let state = test_backend_state();
    let _guard = state.player_info_requests.lock().unwrap();

    assert_eq!(
        state.queue_player_info_request(7),
        Err(DirectClientError::Busy)
    );
}

#[test]
fn contended_request_drain_preserves_the_queue() {
    let state = test_backend_state();
    state.queue_player_info_request(7).unwrap();
    let guard = state.player_info_requests.lock().unwrap();

    assert!(state.take_player_info_requests().is_empty());
    drop(guard);
    assert_eq!(state.take_player_info_requests(), vec![7]);
}

#[test]
fn contended_direct_cache_read_returns_busy() {
    let mut state = test_backend_state();
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);
    let _guard = state.player_info_cache.lock().unwrap();

    assert_eq!(state.player_info(7), Err(DirectClientError::Busy));
}

#[test]
fn known_direct_cache_value_survives_refresh_queue_contention() {
    let mut state = test_backend_state();
    state.context.r1_client = R1ClientProfile::verify(0x10000, 0x31DF13);
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);
    let expected = player_info_from_local(&test_snapshot(7));
    state.player_info_cache.lock().unwrap()[7] =
        PlayerInfoCacheEntry::Known(Some(expected.clone()));
    let _guard = state.player_info_requests.lock().unwrap();

    assert_eq!(state.player_info(7), Ok(Some(expected)));
}

#[test]
fn poisoned_public_lock_maps_to_not_ready() {
    let mutex = Arc::new(Mutex::new(()));
    let poisoned = Arc::clone(&mutex);
    let _ = std::thread::spawn(move || {
        let _guard = poisoned.lock().unwrap();
        panic!("poison the test mutex");
    })
    .join();

    assert!(matches!(
        try_lock_direct(&mutex),
        Err(DirectClientError::NotReady)
    ));
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
