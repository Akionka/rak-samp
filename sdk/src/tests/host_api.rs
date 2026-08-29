//! Host API resolution, direct operation, and conversion tests.

use super::*;

#[test]
fn default_host_module_matches_the_deploy_artifact() {
    assert_eq!(DEFAULT_HOST_MODULE, b"samp_client_sdk.asi\0");
}

#[test]
fn ready_fixture_host_reports_samp_available() {
    let api = test_support::test_api();
    assert!(api.is_samp_loaded());
    assert!(api.is_samp_available());
    assert!(api.sampfuncs_loaded());
    assert!(api.incoming_emulation_ready());
    assert_eq!(api.sampfuncs_log_console(b"host bridge test"), Ok(()));
    assert_eq!(
        api.sampfuncs_log_console(b"interior\0nul"),
        Err(SampClientSdkResult::InvalidArgument)
    );
}

#[test]
fn direct_dialog_rejects_nuls_and_oversized_fields_before_the_abi_call() {
    let api = test_support::test_api();
    let valid = LocalDialog {
        id: 7,
        style: LocalDialogStyle::MessageBox,
        title: b"title",
        text: b"text",
        button1: b"ok",
        button2: b"",
    };
    assert_eq!(api.show_local_dialog(valid), SampClientSdkResult::Ok);

    let nul = LocalDialog {
        title: b"bad\0title",
        ..valid
    };
    assert_eq!(
        api.show_local_dialog(nul),
        SampClientSdkResult::InvalidArgument
    );

    let too_long = [b'x'; 256];
    let long_title = LocalDialog {
        title: &too_long,
        ..valid
    };
    assert_eq!(
        api.show_local_dialog(long_title),
        SampClientSdkResult::InvalidArgument
    );
}

#[test]
fn direct_chat_rejects_nuls_and_native_entry_overflows_before_the_abi_call() {
    let api = test_support::test_api();
    let valid = LocalChatMessage {
        style: LocalChatMessageStyle::Debug,
        text: b"local message",
        prefix: b"[samp-client-sdk]",
        text_colour: 0xFF_A9_C4_E4,
        prefix_colour: u32::MAX,
    };
    assert_eq!(api.show_local_chat_message(valid), SampClientSdkResult::Ok);
    assert_eq!(
        api.show_local_chat_message(LocalChatMessage {
            text: b"bad\0text",
            ..valid
        }),
        SampClientSdkResult::InvalidArgument
    );
    let too_long_text = [b'x'; 144];
    assert_eq!(
        api.show_local_chat_message(LocalChatMessage {
            text: &too_long_text,
            ..valid
        }),
        SampClientSdkResult::InvalidArgument
    );
    let too_long_prefix = [b'x'; 28];
    assert_eq!(
        api.show_local_chat_message(LocalChatMessage {
            prefix: &too_long_prefix,
            ..valid
        }),
        SampClientSdkResult::InvalidArgument
    );
}

#[test]
fn direct_death_window_rejects_nuls_and_native_name_overflows_before_the_abi_call() {
    let api = test_support::test_api();
    let valid = LocalDeathMessage {
        killer: b"killer",
        victim: b"victim",
        killer_colour: 0xFFFF_0000,
        victim_colour: 0xFF00_FF00,
        weapon: 24,
    };
    assert_eq!(api.show_local_death_message(valid), SampClientSdkResult::Ok);
    assert_eq!(
        api.show_local_death_message(LocalDeathMessage {
            killer: b"bad\0killer",
            ..valid
        }),
        SampClientSdkResult::InvalidArgument
    );
    let too_long = [b'x'; 25];
    assert_eq!(
        api.show_local_death_message(LocalDeathMessage {
            victim: &too_long,
            ..valid
        }),
        SampClientSdkResult::InvalidArgument
    );
}

#[test]
fn direct_commands_return_owned_receipts_that_poll_wait_and_release() {
    let api = test_support::test_api();
    let mut dialog = api
        .submit_local_dialog(LocalDialog {
            id: 7,
            style: LocalDialogStyle::MessageBox,
            title: b"title",
            text: b"text",
            button1: b"ok",
            button2: b"",
        })
        .expect("fixture accepts dialog submissions");
    assert_eq!(dialog.id(), 1);
    assert_eq!(dialog.try_take(), Ok(Some(())));

    let mut chat = api
        .submit_local_chat_message(LocalChatMessage {
            style: LocalChatMessageStyle::Debug,
            text: b"local message",
            prefix: b"[samp-client-sdk]",
            text_colour: 0xFF_A9_C4_E4,
            prefix_colour: u32::MAX,
        })
        .expect("fixture accepts chat submissions");
    assert_eq!(chat.id(), 2);
    assert_eq!(chat.wait(Duration::ZERO), Ok(()));

    let death = api
        .submit_local_death_message(LocalDeathMessage {
            killer: b"killer",
            victim: b"victim",
            killer_colour: 0xFFFF_0000,
            victim_colour: 0xFF00_FF00,
            weapon: 24,
        })
        .expect("fixture accepts death-window submissions");
    assert_eq!(death.id(), 3);
    assert_eq!(death.release(), Ok(()));
}

#[test]
fn local_player_snapshot_is_owned_and_converted_from_the_abi_buffer() {
    let snapshot = test_support::test_api()
        .local_player()
        .expect("test host publishes a snapshot");
    assert_eq!(snapshot.id, 42);
    assert_eq!(snapshot.nickname, b"fixture");
    assert_eq!(snapshot.vehicle_id, Some(19));
    assert_eq!(
        snapshot.position,
        Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
}

#[test]
fn player_directory_entry_is_owned_and_handles_a_cached_disconnect() {
    let api = test_support::test_api();
    assert_eq!(
        api.player_info(7),
        Ok(Some(PlayerInfo {
            id: 7,
            nickname: b"remote".to_vec(),
            is_local: false,
            is_npc: true,
            colour: 0xFF22_4466,
            score: -10,
            ping: 55,
        }))
    );
    assert_eq!(api.is_player_connected(7), Ok(true));
    assert_eq!(api.is_player_defined(7), Ok(true));
    assert_eq!(api.is_player_paused(7), Ok(false));
    assert_eq!(api.player_nickname(7), Ok(Some(b"remote".to_vec())));
    assert_eq!(api.is_player_npc(7), Ok(Some(true)));
    assert_eq!(api.player_colour(7), Ok(Some(0xFF22_4466)));
    assert_eq!(api.player_score(7), Ok(Some(-10)));
    assert_eq!(api.player_ping(7), Ok(Some(55)));
    assert_eq!(
        api.remote_player_state(7),
        Ok(Some(RemotePlayerState {
            id: 7,
            health: 75.0,
            armour: 25.0,
            special_action: 3,
            animation_id: 123,
        }))
    );
    assert_eq!(api.player_health(7), Ok(Some(75.0)));
    assert_eq!(api.player_armour(7), Ok(Some(25.0)));
    assert_eq!(api.player_special_action(7), Ok(Some(3)));
    assert_eq!(api.player_animation_id(7), Ok(Some(123)));
    assert_eq!(
        api.streamed_out_player_position(7),
        Ok(Some(Vector3 {
            x: 100.0,
            y: -200.0,
            z: 15.0,
        }))
    );
    assert_eq!(api.remote_player_state(8), Ok(None));
    assert_eq!(api.streamed_out_player_position(8), Ok(None));
    assert_eq!(api.player_info(8), Ok(None));
    assert_eq!(api.is_player_connected(8), Ok(false));
    assert_eq!(api.is_player_defined(8), Ok(false));
    assert_eq!(api.is_player_paused(8), Ok(false));
    assert_eq!(api.is_player_paused(9), Ok(true));
    assert_eq!(api.player_count(true), Ok(3));
    assert_eq!(api.player_count(false), Ok(2));
    assert_eq!(api.player_max_id(), Ok(42));
    assert_eq!(api.is_vehicle_defined(7), Ok(true));
    assert_eq!(api.is_vehicle_defined(8), Ok(false));
    assert_eq!(
        api.is_vehicle_defined(MAX_SAMP_VEHICLES),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(api.is_text_label_defined(7), Ok(true));
    assert_eq!(api.is_text_label_defined(8), Ok(false));
    assert_eq!(
        api.is_text_label_defined(MAX_SAMP_TEXT_LABELS),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(api.is_textdraw_defined(7), Ok(true));
    assert_eq!(api.is_textdraw_defined(8), Ok(false));
    assert_eq!(
        api.is_textdraw_defined(MAX_SAMP_TEXTDRAWS),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(api.is_object_defined(7), Ok(true));
    assert_eq!(api.is_object_defined(8), Ok(false));
    assert_eq!(
        api.is_object_defined(MAX_SAMP_OBJECTS),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(
        api.gangzone(7),
        Ok(Some(Gangzone {
            id: 7,
            left: -1.0,
            bottom: -2.0,
            right: 3.0,
            top: 4.0,
            colour: 0xFF11_2233,
            alternate_colour: 0xFF44_5566,
        }))
    );
    assert_eq!(api.gangzone(8), Ok(None));
    assert_eq!(
        api.gangzone(MAX_SAMP_GANGZONES),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(
        api.text_label(7),
        Ok(Some(TextLabel {
            id: 7,
            text: b"fixture".to_vec(),
            colour: 0xFF11_2233,
            position: Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            draw_distance: 25.0,
            behind_walls: true,
            attached_player_id: Some(8),
            attached_vehicle_id: None,
        }))
    );
    assert_eq!(api.text_label(8), Ok(None));
    assert_eq!(
        api.text_label(MAX_SAMP_TEXT_LABELS),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(
        api.textdraw(7),
        Ok(Some(TextDraw {
            pool_index: 7,
            text: b"fixture".to_vec(),
            letter_width: 1.0,
            letter_height: 2.0,
            letter_colour: 0xFF11_2233,
            x: 3.0,
            y: 4.0,
            shadow: 2,
            outline: 3,
            background_colour: 0xFF44_5566,
            style: 5,
            proportional: true,
            align_left: false,
            align_center: true,
            align_right: false,
            box_enabled: true,
            box_width: 6.0,
            box_height: 7.0,
            box_colour: 0xFF77_8899,
            model_id: 10,
            rotation: Vector3 {
                x: 8.0,
                y: 9.0,
                z: 10.0,
            },
            zoom: 11.0,
            model_colour1: 12,
            model_colour2: 13,
        }))
    );
    assert_eq!(api.textdraw(8), Ok(None));
    assert_eq!(
        api.textdraw(MAX_SAMP_TEXTDRAWS),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(
        api.chat_entry(7),
        Ok(ChatEntry {
            id: 7,
            text: b"fixture".to_vec(),
            prefix: b"prefix".to_vec(),
            text_colour: 0xFF11_2233,
            prefix_colour: 0xFF44_5566,
        })
    );
    assert_eq!(
        api.chat_entry(MAX_SAMP_CHAT_ENTRIES),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(
        api.active_local_dialog(),
        Ok(Some(LocalDialogState {
            id: 7,
            style: LocalDialogStyle::Input,
            title: b"fixture".to_vec(),
            server_side: false,
            text: b"fixture".to_vec(),
            editbox_text: Some(b"fixture".to_vec()),
            items: vec![b"fixture".to_vec(); 3],
        }))
    );
    assert_eq!(
        api.player_info(MAX_SAMP_PLAYERS),
        Err(SampClientSdkResult::InvalidArgument)
    );
}

#[test]
fn dialog_snapshot_preserves_an_absent_editbox() {
    let raw = SampClientSdkDialogSnapshotV1 {
        active: 1,
        style: 0,
        id: 7,
        ..Default::default()
    };
    let dialog = local_dialog_state_from_abi(raw)
        .expect("canonical dialog snapshot")
        .expect("active dialog");

    assert_eq!(dialog.style, LocalDialogStyle::MessageBox);
    assert_eq!(dialog.editbox_text(), None);
}

#[test]
fn dialog_response_abi_is_owned_and_requires_a_bounded_input() {
    let mut raw = SampClientSdkDialogResponseV1 {
        available: 1,
        dialog_id: 7,
        button: 1,
        list_item: 2,
        input_len: 7,
        ..Default::default()
    };
    raw.input[..7].copy_from_slice(b"fixture");
    assert_eq!(
        local_dialog_response_from_abi(raw),
        Ok(Some(LocalDialogResponse {
            dialog_id: 7,
            button: 1,
            list_item: 2,
            input: b"fixture".to_vec(),
        }))
    );

    raw.input_len = 129;
    assert_eq!(
        local_dialog_response_from_abi(raw),
        Err(SampClientSdkResult::NativeCallFailed)
    );
}

#[test]
fn server_info_snapshot_is_owned_and_converted_from_the_abi_buffer() {
    let info = test_support::test_api()
        .server_info()
        .expect("test host publishes server metadata");
    assert_eq!(info.address, b"127.0.0.1");
    assert_eq!(info.hostname, b"fixture");
    assert_eq!(info.port, 7777);
}

#[test]
fn samp_game_state_is_returned_from_the_scalar_abi_output() {
    assert_eq!(test_support::test_api().samp_game_state(), Ok(14));
}

#[test]
fn local_chat_display_mode_is_converted_from_the_scalar_abi_output() {
    let api = test_support::test_api();
    assert_eq!(
        api.local_chat_display_mode(),
        Ok(LocalChatDisplayMode::Normal)
    );
    assert_eq!(api.is_local_chat_visible(), Ok(true));
    assert_eq!(LocalChatDisplayMode::from_raw(3), None);
}

#[test]
fn local_cursor_and_scoreboard_state_are_converted_from_scalar_abi_outputs() {
    let api = test_support::test_api();
    assert_eq!(api.local_cursor_mode(), Ok(LocalCursorMode::LockCamera));
    assert_eq!(api.is_local_cursor_active(), Ok(true));
    assert_eq!(api.is_local_scoreboard_open(), Ok(false));
    assert_eq!(api.is_local_dialog_active(), Ok(false));
    assert_eq!(api.is_local_chat_input_active(), Ok(false));
    assert_eq!(LocalCursorMode::from_raw(5), None);
}

#[test]
fn local_animation_table_uses_owned_bounded_abi_storage() {
    let api = test_support::test_api();
    assert_eq!(
        api.local_animation(0),
        Ok(LocalAnimation {
            name: b"AIRPORT".to_vec(),
            file: b"THRW_BARL_THRW".to_vec(),
        })
    );
    assert_eq!(
        api.local_animation_id(b"AIRPORT", b"THRW_BARL_THRW"),
        Ok(Some(0))
    );
    assert_eq!(api.local_animation_id(b"missing", b"entry"), Ok(None));
    assert_eq!(
        api.local_animation_id(b"", b"entry"),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(
        api.local_animation_id(&[b'x'; 36], b"entry"),
        Err(SampClientSdkResult::InvalidArgument)
    );
}

#[test]
fn samp_version_is_converted_from_the_scalar_abi_output() {
    assert_eq!(
        test_support::test_api().samp_version(),
        Ok(SampClientSdkClientVersion::R1)
    );
}

#[test]
fn decode_string_returns_owned_bytes_and_advances_the_owned_stream() {
    let api = test_support::test_api();
    let mut stream = samp_protocol::BitStream::from_bits(vec![0b1010_0000], 3)
        .expect("fixture bit stream is valid");

    assert_eq!(api.decode_string(&mut stream), Ok(b"fixture".to_vec()));
    assert_eq!(stream.read_offset_bits(), 3);

    let mut rejected = samp_protocol::BitStream::from_bits(vec![0b0100_0000], 3)
        .expect("fixture bit stream is valid");
    rejected.set_read_offset(1).expect("cursor is valid");
    assert_eq!(
        api.decode_string(&mut rejected),
        Err(SampClientSdkResult::InvalidArgument)
    );
    assert_eq!(rejected.read_offset_bits(), 1);
}

#[test]
fn local_player_query_conveniences_reuse_the_safe_snapshot() {
    let api = test_support::test_api();
    assert_eq!(api.local_player_id(), Ok(42));
    assert_eq!(api.local_player_nickname(), Ok(b"fixture".to_vec()));
    assert_eq!(api.local_player_colour(), Ok(0xFF00_00FF));
    assert_eq!(api.is_local_player_spawned(), Ok(true));
    assert_eq!(api.local_player_health(), Ok(99.0));
    assert_eq!(api.local_player_armour(), Ok(50.0));
    assert_eq!(api.local_player_special_action(), Ok(3));
    assert_eq!(api.local_player_animation_id(), Ok(12));
    assert_eq!(api.local_player_score(), Ok(123));
    assert_eq!(api.local_player_ping(), Ok(45));
}
