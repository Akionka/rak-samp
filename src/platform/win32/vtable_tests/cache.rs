//! Native cache publication and invalidation tests.

use super::*;

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
fn handle_caches_are_cleared_across_connection_boundaries() {
    let mut state = test_backend_state();
    state.context.native_client_profile = r1_native_profile();
    state.rak_client.store(0x1000, Ordering::Release);
    state.cache_generation.store(2, Ordering::Release);
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

    assert_eq!(state.object_handle(7), Err(DirectClientError::NotReady));
    assert_eq!(
        state.object_id_by_handle(42),
        Err(DirectClientError::NotReady)
    );
    assert_eq!(
        state.object_handle_requests.lock().unwrap().as_slices().0,
        &[7]
    );
    assert_eq!(
        state
            .object_handle_reverse_requests
            .lock()
            .unwrap()
            .as_slices()
            .0,
        &[42]
    );
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
