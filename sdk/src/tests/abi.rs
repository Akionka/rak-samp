//! ABI defaults and layout tests.

use super::*;

#[test]
fn default_options_match_raknet_defaults() {
    assert_eq!(SampClientSdkSendOptions::default().priority, 1);
    assert_eq!(SampClientSdkSendOptions::default().reliability, 9);
}

#[test]
fn zero_only_abi_defaults_are_all_zero() {
    assert_default_is_zeroed::<SampClientSdkChatInputTextV1>();
    assert_default_is_zeroed::<SampClientSdkDialogListItemV1>();
    assert_default_is_zeroed::<SampClientSdkDialogResponseV1>();
    assert_default_is_zeroed::<SampClientSdkDialogSnapshotV1>();
    assert_default_is_zeroed::<SampClientSdkActiveDialogV1>();
    assert_default_is_zeroed::<SampClientSdkLocalPlayerV1>();
    assert_default_is_zeroed::<SampClientSdkPlayerInfoV1>();
    assert_default_is_zeroed::<SampClientSdkStreamedOutPlayerPositionV1>();
    assert_default_is_zeroed::<SampClientSdkChatEntryV1>();
    assert_default_is_zeroed::<SampClientSdkTextDrawV1>();
    assert_default_is_zeroed::<SampClientSdkTextLabelV1>();
    assert_default_is_zeroed::<SampClientSdkTextLabelCreateResultV1>();
    assert_default_is_zeroed::<SampClientSdkServerInfoV1>();
    assert_default_is_zeroed::<SampClientSdkAnimationV1>();
}

#[test]
fn busy_result_has_a_stable_discriminant() {
    assert_eq!(SampClientSdkResult::Busy as i32, 14);
}

#[test]
fn newer_functions_are_appended_to_abi_v1() {
    let function_size = mem::size_of::<*const c_void>();
    const LEGACY_API_FIELD_COUNT: usize = 145;
    const LEGACY_API_SIZE: usize = 580;
    const LEGACY_API_ALIGNMENT: usize = 4;
    const LEGACY_API_FIELD_SIZE: usize = 4;

    let legacy_api_field_offsets: [usize; LEGACY_API_FIELD_COUNT] = [
        mem::offset_of!(SampClientSdkApiV1, abi_version),
        mem::offset_of!(SampClientSdkApiV1, size),
        mem::offset_of!(SampClientSdkApiV1, host_status),
        mem::offset_of!(SampClientSdkApiV1, register_packet),
        mem::offset_of!(SampClientSdkApiV1, register_rpc),
        mem::offset_of!(SampClientSdkApiV1, unregister),
        mem::offset_of!(SampClientSdkApiV1, event_id),
        mem::offset_of!(SampClientSdkApiV1, event_reset_read),
        mem::offset_of!(SampClientSdkApiV1, event_clear),
        mem::offset_of!(SampClientSdkApiV1, event_read_u8),
        mem::offset_of!(SampClientSdkApiV1, event_read_u16),
        mem::offset_of!(SampClientSdkApiV1, event_read_u32),
        mem::offset_of!(SampClientSdkApiV1, event_read_f32),
        mem::offset_of!(SampClientSdkApiV1, event_read_bytes),
        mem::offset_of!(SampClientSdkApiV1, event_write_u8),
        mem::offset_of!(SampClientSdkApiV1, event_write_u16),
        mem::offset_of!(SampClientSdkApiV1, event_write_u32),
        mem::offset_of!(SampClientSdkApiV1, event_write_f32),
        mem::offset_of!(SampClientSdkApiV1, event_write_bytes),
        mem::offset_of!(SampClientSdkApiV1, send_packet),
        mem::offset_of!(SampClientSdkApiV1, send_rpc),
        mem::offset_of!(SampClientSdkApiV1, event_replace_bytes),
        mem::offset_of!(SampClientSdkApiV1, unregister_and_wait),
        mem::offset_of!(SampClientSdkApiV1, emulate_incoming_packet),
        mem::offset_of!(SampClientSdkApiV1, emulate_incoming_rpc),
        mem::offset_of!(SampClientSdkApiV1, event_remaining_bits),
        mem::offset_of!(SampClientSdkApiV1, event_read_bits),
        mem::offset_of!(SampClientSdkApiV1, event_replace_bits),
        mem::offset_of!(SampClientSdkApiV1, encode_string),
        mem::offset_of!(SampClientSdkApiV1, event_read_encoded_string),
        mem::offset_of!(SampClientSdkApiV1, show_local_dialog),
        mem::offset_of!(SampClientSdkApiV1, local_player),
        mem::offset_of!(SampClientSdkApiV1, samp_game_state),
        mem::offset_of!(SampClientSdkApiV1, samp_version),
        mem::offset_of!(SampClientSdkApiV1, decode_string),
        mem::offset_of!(SampClientSdkApiV1, server_info),
        mem::offset_of!(SampClientSdkApiV1, show_local_chat_message),
        mem::offset_of!(SampClientSdkApiV1, show_local_death_message),
        mem::offset_of!(SampClientSdkApiV1, local_chat_display_mode),
        mem::offset_of!(SampClientSdkApiV1, local_cursor_mode),
        mem::offset_of!(SampClientSdkApiV1, local_scoreboard_open),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_active),
        mem::offset_of!(SampClientSdkApiV1, local_chat_input_active),
        mem::offset_of!(SampClientSdkApiV1, local_animation),
        mem::offset_of!(SampClientSdkApiV1, local_animation_id),
        mem::offset_of!(SampClientSdkApiV1, player_info),
        mem::offset_of!(SampClientSdkApiV1, player_count),
        mem::offset_of!(SampClientSdkApiV1, player_max_id),
        mem::offset_of!(SampClientSdkApiV1, vehicle_exists),
        mem::offset_of!(SampClientSdkApiV1, active_local_dialog),
        mem::offset_of!(SampClientSdkApiV1, text_label_exists),
        mem::offset_of!(SampClientSdkApiV1, textdraw_exists),
        mem::offset_of!(SampClientSdkApiV1, object_exists),
        mem::offset_of!(SampClientSdkApiV1, gangzone_info),
        mem::offset_of!(SampClientSdkApiV1, text_label_info),
        mem::offset_of!(SampClientSdkApiV1, textdraw_info),
        mem::offset_of!(SampClientSdkApiV1, player_defined),
        mem::offset_of!(SampClientSdkApiV1, player_paused),
        mem::offset_of!(SampClientSdkApiV1, remote_player_state),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_message),
        mem::offset_of!(SampClientSdkApiV1, submit_local_death_message),
        mem::offset_of!(SampClientSdkApiV1, command_try_take),
        mem::offset_of!(SampClientSdkApiV1, command_wait),
        mem::offset_of!(SampClientSdkApiV1, command_release),
        mem::offset_of!(SampClientSdkApiV1, submit_packet),
        mem::offset_of!(SampClientSdkApiV1, submit_rpc),
        mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_packet),
        mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_rpc),
        mem::offset_of!(SampClientSdkApiV1, raw_rakclient),
        mem::offset_of!(SampClientSdkApiV1, raw_player_pool),
        mem::offset_of!(SampClientSdkApiV1, raw_vehicle_pool),
        mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_mode),
        mem::offset_of!(SampClientSdkApiV1, submit_local_scoreboard_open),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_client_side),
        mem::offset_of!(SampClientSdkApiV1, submit_samp_game_state),
        mem::offset_of!(SampClientSdkApiV1, raw_local_player),
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_spawn),
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_special_action),
        mem::offset_of!(SampClientSdkApiV1, submit_send_rate),
        mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_toggle),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_display_mode),
        mem::offset_of!(SampClientSdkApiV1, raw_rakpeer),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_close),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_text),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_enabled),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_process),
        mem::offset_of!(SampClientSdkApiV1, local_chat_input_text),
        mem::offset_of!(SampClientSdkApiV1, submit_player_colour),
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_name),
        mem::offset_of!(SampClientSdkApiV1, submit_force_unoccupied_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_aim_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_onfoot_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_stats_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_connect_to_server),
        mem::offset_of!(SampClientSdkApiV1, submit_disconnect_with_reason),
        mem::offset_of!(SampClientSdkApiV1, submit_delete_textdraw),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_position),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_letter_style),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_proportional),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_shadow),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_outline),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_box),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_alignment),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_string),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_selected_item),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_selected_item),
        mem::offset_of!(SampClientSdkApiV1, submit_delete_text_label),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_list_item_count),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_model_style),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_entry),
        mem::offset_of!(SampClientSdkApiV1, chat_entry_info),
        mem::offset_of!(SampClientSdkApiV1, submit_create_text_label),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_snapshot),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_editbox_text),
        mem::offset_of!(SampClientSdkApiV1, local_object_handle),
        mem::offset_of!(SampClientSdkApiV1, local_object_id_by_handle),
        mem::offset_of!(SampClientSdkApiV1, local_pickup_handle),
        mem::offset_of!(SampClientSdkApiV1, local_pickup_id_by_handle),
        mem::offset_of!(SampClientSdkApiV1, local_vehicle_handle),
        mem::offset_of!(SampClientSdkApiV1, local_vehicle_id_by_handle),
        mem::offset_of!(SampClientSdkApiV1, local_player_ped_handle),
        mem::offset_of!(SampClientSdkApiV1, local_player_id_by_ped_handle),
        mem::offset_of!(SampClientSdkApiV1, submit_register_chat_command),
        mem::offset_of!(SampClientSdkApiV1, local_chat_command_defined),
        mem::offset_of!(SampClientSdkApiV1, submit_create_text_label_auto),
        mem::offset_of!(SampClientSdkApiV1, text_label_create_try_take),
        mem::offset_of!(SampClientSdkApiV1, text_label_create_wait),
        mem::offset_of!(SampClientSdkApiV1, submit_set_text_label_text),
        mem::offset_of!(SampClientSdkApiV1, onfoot_sync),
        mem::offset_of!(SampClientSdkApiV1, vehicle_sync),
        mem::offset_of!(SampClientSdkApiV1, passenger_sync),
        mem::offset_of!(SampClientSdkApiV1, trailer_sync),
        mem::offset_of!(SampClientSdkApiV1, aim_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_trailer_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_vehicle_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_create_textdraw),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_style),
        mem::offset_of!(SampClientSdkApiV1, take_local_dialog_response),
        mem::offset_of!(SampClientSdkApiV1, submit_force_passenger_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_weapons_sync),
        mem::offset_of!(SampClientSdkApiV1, streamed_out_player_position),
        mem::offset_of!(SampClientSdkApiV1, sampfuncs_loaded),
        mem::offset_of!(SampClientSdkApiV1, sampfuncs_log_console),
        mem::offset_of!(SampClientSdkApiV1, incoming_emulation_ready),
    ];

    assert_eq!(mem::size_of::<SampClientSdkApiV1>(), LEGACY_API_SIZE);
    assert_eq!(mem::align_of::<SampClientSdkApiV1>(), LEGACY_API_ALIGNMENT);
    for (index, offset) in legacy_api_field_offsets.iter().copied().enumerate() {
        assert_eq!(offset, index * LEGACY_API_FIELD_SIZE);
    }
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, incoming_emulation_ready),
        LEGACY_API_SIZE - LEGACY_API_FIELD_SIZE
    );
    assert_eq!(
        mem::size_of::<SampClientSdkApiV1>(),
        mem::offset_of!(SampClientSdkApiV1, incoming_emulation_ready) + LEGACY_API_FIELD_SIZE
    );

    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, emulate_incoming_packet),
        mem::offset_of!(SampClientSdkApiV1, unregister_and_wait) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, emulate_incoming_rpc),
        mem::offset_of!(SampClientSdkApiV1, emulate_incoming_packet) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, event_remaining_bits),
        mem::offset_of!(SampClientSdkApiV1, emulate_incoming_rpc) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, event_read_bits),
        mem::offset_of!(SampClientSdkApiV1, event_remaining_bits) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, event_replace_bits),
        mem::offset_of!(SampClientSdkApiV1, event_read_bits) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, encode_string),
        mem::offset_of!(SampClientSdkApiV1, event_replace_bits) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, event_read_encoded_string),
        mem::offset_of!(SampClientSdkApiV1, encode_string) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, show_local_dialog),
        mem::offset_of!(SampClientSdkApiV1, event_read_encoded_string) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_player),
        mem::offset_of!(SampClientSdkApiV1, show_local_dialog) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, samp_game_state),
        mem::offset_of!(SampClientSdkApiV1, local_player) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, samp_version),
        mem::offset_of!(SampClientSdkApiV1, samp_game_state) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, decode_string),
        mem::offset_of!(SampClientSdkApiV1, samp_version) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, server_info),
        mem::offset_of!(SampClientSdkApiV1, decode_string) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, show_local_chat_message),
        mem::offset_of!(SampClientSdkApiV1, server_info) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, show_local_death_message),
        mem::offset_of!(SampClientSdkApiV1, show_local_chat_message) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_chat_display_mode),
        mem::offset_of!(SampClientSdkApiV1, show_local_death_message) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_cursor_mode),
        mem::offset_of!(SampClientSdkApiV1, local_chat_display_mode) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_scoreboard_open),
        mem::offset_of!(SampClientSdkApiV1, local_cursor_mode) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_dialog_active),
        mem::offset_of!(SampClientSdkApiV1, local_scoreboard_open) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_chat_input_active),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_active) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_animation),
        mem::offset_of!(SampClientSdkApiV1, local_chat_input_active) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_animation_id),
        mem::offset_of!(SampClientSdkApiV1, local_animation) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, player_info),
        mem::offset_of!(SampClientSdkApiV1, local_animation_id) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, player_count),
        mem::offset_of!(SampClientSdkApiV1, player_info) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, player_max_id),
        mem::offset_of!(SampClientSdkApiV1, player_count) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, vehicle_exists),
        mem::offset_of!(SampClientSdkApiV1, player_max_id) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, active_local_dialog),
        mem::offset_of!(SampClientSdkApiV1, vehicle_exists) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, text_label_exists),
        mem::offset_of!(SampClientSdkApiV1, active_local_dialog) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, textdraw_exists),
        mem::offset_of!(SampClientSdkApiV1, text_label_exists) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, object_exists),
        mem::offset_of!(SampClientSdkApiV1, textdraw_exists) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, gangzone_info),
        mem::offset_of!(SampClientSdkApiV1, object_exists) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, text_label_info),
        mem::offset_of!(SampClientSdkApiV1, gangzone_info) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, textdraw_info),
        mem::offset_of!(SampClientSdkApiV1, text_label_info) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, player_defined),
        mem::offset_of!(SampClientSdkApiV1, textdraw_info) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, player_paused),
        mem::offset_of!(SampClientSdkApiV1, player_defined) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, remote_player_state),
        mem::offset_of!(SampClientSdkApiV1, player_paused) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog),
        mem::offset_of!(SampClientSdkApiV1, remote_player_state) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_message),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_death_message),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_message) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, command_try_take),
        mem::offset_of!(SampClientSdkApiV1, submit_local_death_message) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, command_wait),
        mem::offset_of!(SampClientSdkApiV1, command_try_take) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, command_release),
        mem::offset_of!(SampClientSdkApiV1, command_wait) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_packet),
        mem::offset_of!(SampClientSdkApiV1, command_release) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_rpc),
        mem::offset_of!(SampClientSdkApiV1, submit_packet) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_packet),
        mem::offset_of!(SampClientSdkApiV1, submit_rpc) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_rpc),
        mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_packet) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, raw_player_pool),
        mem::offset_of!(SampClientSdkApiV1, raw_rakclient) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, raw_vehicle_pool),
        mem::offset_of!(SampClientSdkApiV1, raw_player_pool) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_dialog_snapshot),
        mem::offset_of!(SampClientSdkApiV1, submit_create_text_label) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_editbox_text),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_snapshot) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_object_handle),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_editbox_text) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_object_id_by_handle),
        mem::offset_of!(SampClientSdkApiV1, local_object_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_pickup_handle),
        mem::offset_of!(SampClientSdkApiV1, local_object_id_by_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_pickup_id_by_handle),
        mem::offset_of!(SampClientSdkApiV1, local_pickup_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_vehicle_handle),
        mem::offset_of!(SampClientSdkApiV1, local_pickup_id_by_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_vehicle_id_by_handle),
        mem::offset_of!(SampClientSdkApiV1, local_vehicle_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_player_ped_handle),
        mem::offset_of!(SampClientSdkApiV1, local_vehicle_id_by_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_player_id_by_ped_handle),
        mem::offset_of!(SampClientSdkApiV1, local_player_ped_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_register_chat_command),
        mem::offset_of!(SampClientSdkApiV1, local_player_id_by_ped_handle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_chat_command_defined),
        mem::offset_of!(SampClientSdkApiV1, submit_register_chat_command) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_create_text_label_auto),
        mem::offset_of!(SampClientSdkApiV1, local_chat_command_defined) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, text_label_create_try_take),
        mem::offset_of!(SampClientSdkApiV1, submit_create_text_label_auto) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, text_label_create_wait),
        mem::offset_of!(SampClientSdkApiV1, text_label_create_try_take) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_text_label_text),
        mem::offset_of!(SampClientSdkApiV1, text_label_create_wait) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, onfoot_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_set_text_label_text) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, vehicle_sync),
        mem::offset_of!(SampClientSdkApiV1, onfoot_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, passenger_sync),
        mem::offset_of!(SampClientSdkApiV1, vehicle_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, trailer_sync),
        mem::offset_of!(SampClientSdkApiV1, passenger_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, aim_sync),
        mem::offset_of!(SampClientSdkApiV1, trailer_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_trailer_sync),
        mem::offset_of!(SampClientSdkApiV1, aim_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_vehicle_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_trailer_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_create_textdraw),
        mem::offset_of!(SampClientSdkApiV1, submit_force_vehicle_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_style),
        mem::offset_of!(SampClientSdkApiV1, submit_create_textdraw) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, streamed_out_player_position),
        mem::offset_of!(SampClientSdkApiV1, submit_force_weapons_sync) + function_size
    );
    assert_eq!(
        mem::size_of::<SampClientSdkApiV1>(),
        mem::offset_of!(SampClientSdkApiV1, incoming_emulation_ready) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, sampfuncs_loaded),
        mem::offset_of!(SampClientSdkApiV1, streamed_out_player_position) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, sampfuncs_log_console),
        mem::offset_of!(SampClientSdkApiV1, sampfuncs_loaded) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, incoming_emulation_ready),
        mem::offset_of!(SampClientSdkApiV1, sampfuncs_log_console) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, take_local_dialog_response),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_style) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_passenger_sync),
        mem::offset_of!(SampClientSdkApiV1, take_local_dialog_response) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_weapons_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_passenger_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, raw_rakclient),
        mem::offset_of!(SampClientSdkApiV1, submit_emulate_incoming_rpc) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_mode),
        mem::offset_of!(SampClientSdkApiV1, raw_vehicle_pool) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_scoreboard_open),
        mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_mode) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_client_side),
        mem::offset_of!(SampClientSdkApiV1, submit_local_scoreboard_open) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_samp_game_state),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_client_side) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, raw_local_player),
        mem::offset_of!(SampClientSdkApiV1, submit_samp_game_state) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_spawn),
        mem::offset_of!(SampClientSdkApiV1, raw_local_player) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_special_action),
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_spawn) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_send_rate),
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_special_action) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_toggle),
        mem::offset_of!(SampClientSdkApiV1, submit_send_rate) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_display_mode),
        mem::offset_of!(SampClientSdkApiV1, submit_local_cursor_toggle) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, raw_rakpeer),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_display_mode) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_close),
        mem::offset_of!(SampClientSdkApiV1, raw_rakpeer) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_text),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_close) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_enabled),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_text) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_process),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_enabled) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_chat_input_text),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_input_process) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_player_colour),
        mem::offset_of!(SampClientSdkApiV1, local_chat_input_text) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_name),
        mem::offset_of!(SampClientSdkApiV1, submit_player_colour) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_unoccupied_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_local_player_name) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_aim_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_unoccupied_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_onfoot_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_aim_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_force_stats_sync),
        mem::offset_of!(SampClientSdkApiV1, submit_force_onfoot_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_connect_to_server),
        mem::offset_of!(SampClientSdkApiV1, submit_force_stats_sync) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_disconnect_with_reason),
        mem::offset_of!(SampClientSdkApiV1, submit_connect_to_server) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_delete_textdraw),
        mem::offset_of!(SampClientSdkApiV1, submit_disconnect_with_reason) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_position),
        mem::offset_of!(SampClientSdkApiV1, submit_delete_textdraw) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_letter_style),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_position) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_proportional),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_letter_style) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_shadow),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_proportional) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_outline),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_shadow) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_box),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_outline) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_alignment),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_box) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_string),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_alignment) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_dialog_selected_item),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_string) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_selected_item),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_selected_item) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_delete_text_label),
        mem::offset_of!(SampClientSdkApiV1, submit_local_dialog_selected_item) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, local_dialog_list_item_count),
        mem::offset_of!(SampClientSdkApiV1, submit_delete_text_label) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_model_style),
        mem::offset_of!(SampClientSdkApiV1, local_dialog_list_item_count) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_entry),
        mem::offset_of!(SampClientSdkApiV1, submit_set_textdraw_model_style) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, chat_entry_info),
        mem::offset_of!(SampClientSdkApiV1, submit_local_chat_entry) + function_size
    );
    assert_eq!(
        mem::offset_of!(SampClientSdkApiV1, submit_create_text_label),
        mem::offset_of!(SampClientSdkApiV1, chat_entry_info) + function_size
    );
}

#[test]
fn dialog_list_item_abi_length_covers_its_entire_payload() {
    assert_eq!(MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES, usize::from(u8::MAX));
    assert_eq!(mem::size_of::<SampClientSdkDialogListItemV1>(), 256);
}

#[test]
fn dialog_snapshot_abi_layout_is_stable() {
    assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, id), 4);
    assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, text_len), 12);
    assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, title), 16);
    assert_eq!(
        mem::offset_of!(SampClientSdkDialogSnapshotV1, editbox_text),
        81
    );
    assert_eq!(mem::offset_of!(SampClientSdkDialogSnapshotV1, text), 209);
    assert_eq!(
        mem::offset_of!(SampClientSdkDialogSnapshotV1, listbox_items),
        4_305
    );
    assert_eq!(mem::size_of::<SampClientSdkDialogSnapshotV1>(), 29_908);
}
