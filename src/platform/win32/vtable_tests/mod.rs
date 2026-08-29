mod cache;
mod commands_tick;
mod fixtures;
mod hooks_native;
mod requests;

use super::players::MARKERS_SYNC_PACKET_ID;
use super::*;
use crate::{BitStream, Direction, command::GAME_COMMAND_QUEUE_CAPACITY, event::HookAction};
use fixtures::*;
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

unsafe extern "C" fn fake_game_process() {
    GAME_PROCESS_CALLS.fetch_add(1, Ordering::AcqRel);
}

#[test]
fn shared_refresh_helpers_accept_every_native_profile() {
    let profiles = [
        (
            r1_native_profile().expect("R1 must select its verified native profile"),
            NativeClientProfile::select(0x10000, SampVersion::R1, SampVersion::R1.entry_point())
                .expect("R1 must select its immutable profile"),
        ),
        (
            r3_native_profile().expect("R3 must select its verified native profile"),
            r3_native_client_profile().expect("R3 must select its immutable profile"),
        ),
        (
            NativeClientProfile::select(
                0x10000,
                SampVersion::R5_1,
                SampVersion::R5_1.entry_point(),
            )
            .expect("R5 must select its verified native profile"),
            NativeClientProfile::select(
                0x10000,
                SampVersion::R5_1,
                SampVersion::R5_1.entry_point(),
            )
            .expect("R5 must select its immutable profile"),
        ),
        (
            NativeClientProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
                .expect("DL must select its verified native profile"),
            NativeClientProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
                .expect("DL must select its immutable profile"),
        ),
    ];

    for (_profile, native_client) in profiles {
        let state = test_backend_state();
        state.raw_local_player.store(1, Ordering::Release);
        state.player_info_requests.lock().unwrap().push_back(7);
        state
            .remote_player_state_requests
            .lock()
            .unwrap()
            .push_back(7);
        state
            .streamed_out_player_position_requests
            .lock()
            .unwrap()
            .push_back(7);
        state.onfoot_sync_requests.lock().unwrap().push_back(7);
        state.incar_sync_requests.lock().unwrap().push_back(7);
        state.passenger_sync_requests.lock().unwrap().push_back(7);
        state.trailer_sync_requests.lock().unwrap().push_back(7);
        state.aim_sync_requests.lock().unwrap().push_back(7);

        state.refresh_local_player_snapshot(None);
        state.refresh_player_info(native_client);
        state.refresh_remote_player_state(native_client);
        state.refresh_streamed_out_player_position(native_client);
        state.refresh_onfoot_sync(native_client);
        state.refresh_incar_sync(native_client);
        state.refresh_passenger_sync(native_client);
        state.refresh_trailer_sync(native_client);
        state.refresh_aim_sync(native_client);

        assert_eq!(state.raw_local_player.load(Ordering::Acquire), 0);
        assert!(state.player_info_requests.lock().unwrap().is_empty());
        assert!(
            state
                .remote_player_state_requests
                .lock()
                .unwrap()
                .is_empty()
        );
        assert!(
            state
                .streamed_out_player_position_requests
                .lock()
                .unwrap()
                .is_empty()
        );
        assert!(state.onfoot_sync_requests.lock().unwrap().is_empty());
        assert!(state.incar_sync_requests.lock().unwrap().is_empty());
        assert!(state.passenger_sync_requests.lock().unwrap().is_empty());
        assert!(state.trailer_sync_requests.lock().unwrap().is_empty());
        assert!(state.aim_sync_requests.lock().unwrap().is_empty());
    }
}

#[test]
fn game_tick_uses_one_generation_bracket_for_every_native_profile() {
    let profiles = [
        r1_native_profile().expect("R1 must select its verified native profile"),
        r3_native_profile().expect("R3 must select its verified native profile"),
        NativeClientProfile::select(0x10000, SampVersion::R5_1, SampVersion::R5_1.entry_point())
            .expect("R5 must select its verified native profile"),
        NativeClientProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
            .expect("DL must select its verified native profile"),
    ];

    for profile in profiles {
        let mut state = test_backend_state();
        state.context.native_client_profile = Some(profile);
        state.cache_generation.store(2, Ordering::Release);

        state.pump_game_tick(Vec::new());

        assert_eq!(state.cache_generation.load(Ordering::Acquire), 4);
    }
}

#[test]
fn dialog_response_take_is_one_shot() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(1, Ordering::Release);
    *state
        .local_dialog_response
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(LocalDialogResponseSnapshot {
        dialog_id: 7,
        button: 1,
        list_item: 2,
        input: b"fixture".to_vec(),
    });

    assert_eq!(
        state.take_local_dialog_response(),
        Ok(Some(LocalDialogResponseSnapshot {
            dialog_id: 7,
            button: 1,
            list_item: 2,
            input: b"fixture".to_vec(),
        }))
    );
    assert_eq!(state.take_local_dialog_response(), Ok(None));
}

#[test]
fn r3_dialog_response_take_is_one_shot() {
    let mut state = test_backend_state();
    state.context.version = SampVersion::R3_1;
    state.context.native_client_profile = r3_native_profile();
    state.rak_client.store(1, Ordering::Release);
    *state
        .local_dialog_response
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(LocalDialogResponseSnapshot {
        dialog_id: 8,
        button: 0,
        list_item: 0,
        input: b"r3 fixture".to_vec(),
    });

    assert_eq!(
        state.take_local_dialog_response(),
        Ok(Some(LocalDialogResponseSnapshot {
            dialog_id: 8,
            button: 0,
            list_item: 0,
            input: b"r3 fixture".to_vec(),
        }))
    );
    assert_eq!(state.take_local_dialog_response(), Ok(None));
}

#[test]
fn direct_helpers_require_a_verified_native_profile() {
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
    state.context.native_client_profile = r1_native_profile();
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
    state.context.native_client_profile = r1_native_profile();
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
    state.context.native_client_profile = r1_native_profile();
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
        GameCommand::Ui(UiCommand::SetDialogEditboxText(text)) if text == b"fixture"
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
fn r3_cached_reads_include_local_player_without_enabling_r1_helpers() {
    let mut state = test_backend_state();
    state.context.version = SampVersion::R3_1;
    state.context.native_client_profile = r3_native_profile();
    state.context.native_client_profile = r3_native_client_profile();
    state.rak_client.store(1, Ordering::Release);
    state.samp_game_state.store(6, Ordering::Release);
    state.samp_game_state_ready.store(true, Ordering::Release);
    *state
        .server_info_snapshot
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(ServerInfoSnapshot {
        address: b"127.0.0.1".to_vec(),
        hostname: b"R3 probe".to_vec(),
        port: 7777,
    });
    *state
        .local_player_snapshot
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(LocalPlayerSnapshot {
        id: 0,
        nickname: b"R3 probe".to_vec(),
        colour: 0xFF00_FF00,
        spawned: true,
        health: 100.0,
        armour: 0.0,
        position: Vector3::default(),
        velocity: Vector3::default(),
        special_action: 0,
        animation_id: 0,
        vehicle_id: None,
        score: 0,
        ping: 1,
    });

    assert_eq!(state.samp_game_state(), Ok(6));
    assert_eq!(
        state.server_info(),
        Ok(ServerInfoSnapshot {
            address: b"127.0.0.1".to_vec(),
            hostname: b"R3 probe".to_vec(),
            port: 7777,
        })
    );
    assert_eq!(state.local_player().map(|player| player.id), Ok(0));
    assert_eq!(
        state.local_chat_display_mode(),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(state.local_cursor_mode(), Err(DirectClientError::NotReady));
}

#[test]
fn r3_player_pool_scalars_use_exact_published_values() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r3_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(state.player_count(true), Err(DirectClientError::NotReady));
    assert_eq!(state.player_max_id(), Err(DirectClientError::NotReady));

    state
        .player_count_including_npcs
        .store(3, Ordering::Release);
    state
        .player_count_excluding_npcs
        .store(2, Ordering::Release);
    state.player_count_ready.store(true, Ordering::Release);
    state.player_max_id.store(42, Ordering::Release);
    state.player_max_id_ready.store(true, Ordering::Release);

    assert_eq!(state.player_count(true), Ok(3));
    assert_eq!(state.player_count(false), Ok(2));
    assert_eq!(state.player_max_id(), Ok(42));
}

#[test]
fn r3_player_directory_uses_local_and_published_remote_states() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r3_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);
    state.cache_local_player_snapshot(Some(test_snapshot(42)));
    state.cache_local_player_snapshot(Some(test_snapshot(42)));

    assert_eq!(state.player_defined(42), Ok(true));
    assert_eq!(state.player_defined(7), Err(DirectClientError::NotReady));
    assert_eq!(
        state.player_info_requests.lock().unwrap().as_slices().0,
        [7]
    );

    state.player_info_cache.lock().unwrap()[7] = PlayerInfoCacheEntry::Known(None);
    state.player_info_cache.lock().unwrap()[8] =
        PlayerInfoCacheEntry::Known(Some(PlayerInfoSnapshot {
            id: 8,
            defined: true,
            paused: false,
            nickname: b"remote".to_vec(),
            is_local: false,
            is_npc: false,
            colour: 0,
            score: 0,
            ping: 0,
        }));

    assert_eq!(state.player_defined(7), Ok(false));
    assert_eq!(state.player_defined(8), Ok(true));
    assert_eq!(state.player_info(7), Ok(None));
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
fn r3_cached_ui_reads_use_exact_published_values() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r3_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(
        state.local_dialog_active(),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(
        state.local_scoreboard_open(),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(
        state.local_chat_display_mode(),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(state.local_cursor_mode(), Err(DirectClientError::NotReady));
    assert_eq!(
        state.local_chat_input_active(),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(
        state.local_chat_input_text(),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(
        state.local_chat_command_defined(b"sdk"),
        Err(DirectClientError::NotReady)
    );
    state.local_chat_input_active.store(true, Ordering::Release);
    state
        .local_chat_input_active_ready
        .store(true, Ordering::Release);
    *state.local_chat_input_text.lock().unwrap() = Some(b"/r3".to_vec());
    state
        .local_chat_input_text_ready
        .store(true, Ordering::Release);
    state.local_dialog_active.store(true, Ordering::Release);
    state
        .local_dialog_active_ready
        .store(true, Ordering::Release);
    state.local_scoreboard_open.store(true, Ordering::Release);
    state
        .local_scoreboard_open_ready
        .store(true, Ordering::Release);
    state.local_chat_display_mode.store(2, Ordering::Release);
    state
        .local_chat_display_mode_ready
        .store(true, Ordering::Release);
    state.local_cursor_mode.store(3, Ordering::Release);
    state.local_cursor_mode_ready.store(true, Ordering::Release);
    *state.local_chat_input_commands.lock().unwrap() = Some(vec![b"sdk".to_vec()]);
    state
        .local_chat_input_commands_ready
        .store(true, Ordering::Release);

    assert_eq!(state.local_dialog_active(), Ok(true));
    assert_eq!(state.local_scoreboard_open(), Ok(true));
    assert_eq!(state.local_chat_display_mode(), Ok(2));
    assert_eq!(state.local_cursor_mode(), Ok(3));
    assert_eq!(state.local_chat_input_active(), Ok(true));
    assert_eq!(state.local_chat_input_text(), Ok(b"/r3".to_vec()));
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
        GameCommand::Ui(UiCommand::ShowDialog(request)) if request.id == 7
    ));
    assert!(matches!(
        &snapshot[1].command,
        GameCommand::Ui(UiCommand::AddChatMessage(_))
    ));
    assert!(matches!(
        &snapshot[2].command,
        GameCommand::Ui(UiCommand::AddDeathMessage(_))
    ));
    assert!(matches!(
        &snapshot[3].command,
        GameCommand::Ui(UiCommand::ShowDialog(request)) if request.id == 3
    ));
}

#[test]
fn typed_text_label_receipt_returns_the_game_thread_selected_id() {
    let state = test_backend_state();
    let command = state
        .game_commands
        .submit(GameCommand::TextLabel(TextLabelCommand::DeleteTextLabel(0)))
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
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);

    let mut text = b"updated".to_vec();
    state.submit_set_text_label_text(7, text.clone()).unwrap();
    text[0] = b'X';

    let snapshot = state.game_commands.take_tick_snapshot();
    assert!(matches!(
        &snapshot[0].command,
        GameCommand::TextLabel(TextLabelCommand::SetTextLabelText { id: 7, text })
            if text.as_slice() == b"updated"
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
        GameCommand::Network(NetworkCommand::SendPacket {
            id: 99,
            payload: queued,
            options: SendOptions { .. },
        }) if queued.as_bytes() == [0xAB]
    ));
}

#[test]
fn game_tick_calls_original_once_and_marks_the_game_thread() {
    let state = test_backend_state();
    GAME_PROCESS_CALLS.store(0, Ordering::Release);

    unsafe { state.run_game_process_tick(fake_game_process) };

    assert_eq!(GAME_PROCESS_CALLS.load(Ordering::Acquire), 1);
    assert!(state.is_game_thread());
}

#[test]
fn game_tick_leaves_commands_pending_until_the_rak_client_is_ready() {
    let state = test_backend_state();
    let id = state
        .submit_game_command(GameCommand::Ui(UiCommand::ShowDialog(test_dialog(1))))
        .unwrap();

    unsafe { state.run_game_process_tick(fake_game_process) };

    assert_eq!(state.game_commands.try_take(id), Ok(None));
}

#[test]
fn game_tick_completes_commands_after_the_rak_client_is_ready() {
    let state = test_backend_state();
    state.rak_client.store(1, Ordering::Release);
    let id = state
        .submit_game_command(GameCommand::Ui(UiCommand::ShowDialog(test_dialog(1))))
        .unwrap();

    unsafe { state.run_game_process_tick(fake_game_process) };

    assert_eq!(
        state.game_commands.try_take(id),
        Ok(Some(Err(CommandError::NativeFailure)))
    );
}

#[test]
fn disconnect_invalidation_preserves_the_captured_rak_client_for_reconnect() {
    let mut state = test_backend_state();
    state.context.version = SampVersion::R3_1;
    state.context.native_client_profile = r3_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.rpc_receiver.store(0x2000, Ordering::Release);
    state.player_address.store(0x0100007F, Ordering::Release);
    state.player_port.store(7777, Ordering::Release);

    state.invalidate_after_disconnect();

    assert_eq!(state.rak_client.load(Ordering::Acquire), 0x1000);
    assert_eq!(state.rpc_receiver.load(Ordering::Acquire), 0);
    assert_eq!(state.player_address.load(Ordering::Acquire), 0);
    assert_eq!(state.player_port.load(Ordering::Acquire), 0);
    assert!(
        state
            .submit_connect_to_server(b"127.0.0.1".to_vec(), 7777)
            .is_ok()
    );
}

#[test]
fn incoming_emulation_readiness_requires_the_receiver_and_rpc_trampoline() {
    let state = test_backend_state();
    assert!(!state.incoming_emulation_ready());

    state.rpc_receiver.store(1, Ordering::Release);
    assert!(!state.incoming_emulation_ready());

    state.incoming_rpc_trampoline.store(1, Ordering::Release);
    assert!(state.incoming_emulation_ready());
}

#[test]
fn command_wait_is_rejected_on_the_published_game_thread() {
    let state = Arc::new(test_backend_state());
    state
        .game_thread_id
        .store(unsafe { GetCurrentThreadId() }, Ordering::Release);
    let id = state
        .game_commands
        .submit(GameCommand::Ui(UiCommand::ShowDialog(test_dialog(1))))
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
    state.streamed_out_player_position_cache.lock().unwrap()[7] =
        StreamedOutPlayerPositionCacheEntry::Known(Some(Vector3 {
            x: 100.0,
            y: -200.0,
            z: 15.0,
        }));
    state.marker_sync_positions.lock().unwrap()[7] = Some(Vector3 {
        x: 100.0,
        y: -200.0,
        z: 15.0,
    });
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
    state
        .streamed_out_player_position_requests
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
        state.streamed_out_player_position_cache.lock().unwrap()[7],
        StreamedOutPlayerPositionCacheEntry::Unknown
    ));
    assert_eq!(state.marker_sync_positions.lock().unwrap()[7], None);
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
    assert!(
        state
            .streamed_out_player_position_requests
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
fn deleted_ui_entities_publish_absent_cache_entries() {
    let state = test_backend_state();
    state.text_label_exists_cache.lock().unwrap()[7] = TextLabelExistsCacheEntry::Known(true);
    state.text_label_cache.lock().unwrap()[7] =
        TextLabelCacheEntry::Known(Some(TextLabelSnapshot {
            id: 7,
            text: b"stale".to_vec(),
            colour: 0xFFFFFFFF,
            position: Vector3::default(),
            draw_distance: 50.0,
            behind_walls: false,
            attached_player_id: None,
            attached_vehicle_id: None,
        }));
    state.textdraw_exists_cache.lock().unwrap()[7] = TextdrawExistsCacheEntry::Known(true);
    state.textdraw_cache.lock().unwrap()[7] = TextdrawCacheEntry::Unknown;

    state.publish_deleted_text_label(7);
    state.publish_deleted_textdraw(7);

    assert!(matches!(
        state.text_label_exists_cache.lock().unwrap()[7],
        TextLabelExistsCacheEntry::Known(false)
    ));
    assert!(matches!(
        state.text_label_cache.lock().unwrap()[7],
        TextLabelCacheEntry::Known(None)
    ));
    assert!(matches!(
        state.textdraw_exists_cache.lock().unwrap()[7],
        TextdrawExistsCacheEntry::Known(false)
    ));
    assert!(matches!(
        state.textdraw_cache.lock().unwrap()[7],
        TextdrawCacheEntry::Known(None)
    ));
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
fn streamed_out_player_position_reads_owned_cache_and_queues_a_refresh() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(
        state.streamed_out_player_position(7),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(
        state
            .streamed_out_player_position_requests
            .lock()
            .unwrap()
            .as_slices()
            .0,
        &[7]
    );

    let position = Vector3 {
        x: 100.0,
        y: -200.0,
        z: 15.0,
    };
    state.streamed_out_player_position_cache.lock().unwrap()[7] =
        StreamedOutPlayerPositionCacheEntry::Known(Some(position));

    assert_eq!(state.streamed_out_player_position(7), Ok(Some(position)));
}

#[test]
fn marker_sync_capture_preserves_active_positions_and_ignores_inactive_records() {
    let state = test_backend_state();
    let mut payload = BitStream::new();
    payload.write_i32(2).unwrap();
    payload.write_u16(7).unwrap();
    payload.write_bool(true).unwrap();
    payload.write_i16(100).unwrap();
    payload.write_i16(-200).unwrap();
    payload.write_i16(15).unwrap();
    payload.write_u16(8).unwrap();
    payload.write_bool(false).unwrap();

    state.capture_marker_sync(MARKERS_SYNC_PACKET_ID, &payload);

    assert_eq!(
        state.marker_sync_positions.lock().unwrap()[7],
        Some(Vector3 {
            x: 100.0,
            y: -200.0,
            z: 15.0,
        })
    );
    assert_eq!(state.marker_sync_positions.lock().unwrap()[8], None);

    let mut inactive = BitStream::new();
    inactive.write_i32(1).unwrap();
    inactive.write_u16(7).unwrap();
    inactive.write_bool(false).unwrap();
    state.capture_marker_sync(MARKERS_SYNC_PACKET_ID, &inactive);

    assert_eq!(
        state.marker_sync_positions.lock().unwrap()[7],
        Some(Vector3 {
            x: 100.0,
            y: -200.0,
            z: 15.0,
        })
    );
}

#[test]
fn marker_sync_is_captured_without_packet_listeners() {
    let state = test_backend_state();
    let mut payload = BitStream::new();
    payload.write_i32(1).unwrap();
    payload.write_u16(7).unwrap();
    payload.write_bool(true).unwrap();
    payload.write_i16(100).unwrap();
    payload.write_i16(-200).unwrap();
    payload.write_i16(15).unwrap();
    let stream = packet_stream(MARKERS_SYNC_PACKET_ID, &payload).unwrap();
    let mut bytes = stream.as_bytes().to_vec();
    let mut packet = RawPacket {
        player_index: 0,
        player_id: PacketPlayerId {
            binary_address: 0,
            port: 0,
        },
        length: bytes.len() as u32,
        bit_size: stream.len_bits() as u32,
        data: bytes.as_mut_ptr(),
        delete_data: false,
    };

    assert_eq!(
        unsafe { hooks::dispatch_raw_packet(&state, &mut packet) },
        HookAction::Continue
    );
    assert_eq!(
        state.marker_sync_positions.lock().unwrap()[7],
        Some(Vector3 {
            x: 100.0,
            y: -200.0,
            z: 15.0,
        })
    );
}

#[test]
fn streamed_out_player_position_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_streamed_out_player_position_request(7).unwrap();
    state.queue_streamed_out_player_position_request(7).unwrap();
    for id in 8..(7 + STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY as u16) {
        state
            .queue_streamed_out_player_position_request(id)
            .unwrap();
    }
    assert_eq!(
        state.queue_streamed_out_player_position_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_streamed_out_player_position_requests();
    assert_eq!(
        drained.len(),
        STREAMED_OUT_PLAYER_POSITION_REQUESTS_PER_PUMP
    );
    assert_eq!(drained[0], 7);
    assert_eq!(
        state
            .streamed_out_player_position_requests
            .lock()
            .unwrap()
            .len(),
        STREAMED_OUT_PLAYER_POSITION_REQUEST_QUEUE_CAPACITY
            - STREAMED_OUT_PLAYER_POSITION_REQUESTS_PER_PUMP
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
    state.context.native_client_profile = r1_native_profile();
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
fn incar_sync_reads_owned_cache_and_queues_a_refresh() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(state.vehicle_sync(7), Err(DirectClientError::NotReady));
    assert_eq!(
        state.incar_sync_requests.lock().unwrap().as_slices().0,
        &[7]
    );

    let snapshot = InCarSyncSnapshot {
        id: 7,
        vehicle_id: 411,
        controller_left_stick_x: -100,
        controller_left_stick_y: 200,
        controller_buttons: 0x1234,
        quaternion: [0.0, 0.0, 0.0, 1.0],
        position: crate::runtime::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        speed: crate::runtime::Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        vehicle_health: 900.0,
        driver_health: 75,
        driver_armour: 25,
        weapon: 24,
        siren: true,
        landing_gear: false,
        trailer_id: u16::MAX,
        vehicle_specific: [1, 2, 3, 4],
    };
    state.incar_sync_cache.lock().unwrap()[7] = InCarSyncCacheEntry::Known(Some(snapshot));

    assert_eq!(state.vehicle_sync(7), Ok(Some(snapshot)));
}

#[test]
fn incar_sync_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_incar_sync_request(7).unwrap();
    state.queue_incar_sync_request(7).unwrap();
    for id in 8..(7 + INCAR_SYNC_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_incar_sync_request(id).unwrap();
    }
    assert_eq!(
        state.queue_incar_sync_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_incar_sync_requests();
    assert_eq!(drained.len(), INCAR_SYNC_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.incar_sync_requests.lock().unwrap().len(),
        INCAR_SYNC_REQUEST_QUEUE_CAPACITY - INCAR_SYNC_REQUESTS_PER_PUMP
    );
}

#[test]
fn passenger_sync_reads_owned_cache_and_queues_a_refresh() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);

    assert_eq!(state.passenger_sync(7), Err(DirectClientError::NotReady));
    assert_eq!(
        state.passenger_sync_requests.lock().unwrap().as_slices().0,
        &[7]
    );

    let snapshot = PassengerSyncSnapshot {
        id: 7,
        vehicle_id: 411,
        seat_id: 2,
        weapon: 24,
        health: 75,
        armour: 25,
        controller_left_stick_x: -100,
        controller_left_stick_y: 200,
        controller_buttons: 0x1234,
        position: crate::runtime::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
    };
    state.passenger_sync_cache.lock().unwrap()[7] = PassengerSyncCacheEntry::Known(Some(snapshot));

    assert_eq!(state.passenger_sync(7), Ok(Some(snapshot)));
}

#[test]
fn passenger_sync_requests_are_bounded_deduplicated_and_pump_limited() {
    let state = test_backend_state();
    state.queue_passenger_sync_request(7).unwrap();
    state.queue_passenger_sync_request(7).unwrap();
    for id in 8..(7 + PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY as u16) {
        state.queue_passenger_sync_request(id).unwrap();
    }
    assert_eq!(
        state.queue_passenger_sync_request(99),
        Err(DirectClientError::QueueFull)
    );
    let drained = state.take_passenger_sync_requests();
    assert_eq!(drained.len(), PASSENGER_SYNC_REQUESTS_PER_PUMP);
    assert_eq!(drained[0], 7);
    assert_eq!(
        state.passenger_sync_requests.lock().unwrap().len(),
        PASSENGER_SYNC_REQUEST_QUEUE_CAPACITY - PASSENGER_SYNC_REQUESTS_PER_PUMP
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
    state.context.native_client_profile = r1_native_profile();
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
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);
    let _guard = state.player_info_cache.lock().unwrap();

    assert_eq!(state.player_info(7), Err(DirectClientError::Busy));
}

#[test]
fn known_direct_cache_value_survives_refresh_queue_contention() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
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
    assert!(is_connected_game_state(14));
    assert!(!is_connected_game_state(13));
    assert!(!crosses_connection_boundary(false, 0, 14));
    assert!(crosses_connection_boundary(true, 13, 14));
    assert!(crosses_connection_boundary(true, 14, 18));
    assert!(!crosses_connection_boundary(true, 14, 14));
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
