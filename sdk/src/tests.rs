use super::*;
use crate::events::{EncodedPayload, ProtocolAction, test_support};
use samp_protocol::rpc::incoming as protocol_incoming;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

static REGISTRATION_TEST_LOCK: Mutex<()> = Mutex::new(());

struct FailingReplacementCodec;

impl samp_protocol::WireCodec for FailingReplacementCodec {
    type Value = bool;

    fn decode<R: samp_protocol::BitRead>(
        reader: &mut R,
    ) -> Result<Self::Value, samp_protocol::DecodeError<R::Error>> {
        reader
            .read_left_aligned_bits(8)
            .map(|bits| bits[0] != 0)
            .map_err(samp_protocol::DecodeError::Source)
    }

    fn encode<W: samp_protocol::BitWrite>(
        _writer: &mut W,
        _value: &Self::Value,
    ) -> Result<(), samp_protocol::EncodeError<W::Error>> {
        Err(samp_protocol::EncodeError::LengthExceedsLimit {
            length: 2,
            limit: 1,
        })
    }
}

type FailingIncomingRpc =
    samp_protocol::IncomingRpc<201, FailingReplacementCodec, samp_protocol::ExactBitsPolicy>;

struct DropCounter(Arc<AtomicUsize>);

impl Drop for DropCounter {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
}

#[test]
fn default_options_match_raknet_defaults() {
    assert_eq!(SampClientSdkSendOptions::default().priority, 1);
    assert_eq!(SampClientSdkSendOptions::default().reliability, 9);
}

fn assert_default_is_zeroed<T: Default>() {
    let value = T::default();
    let bytes = unsafe {
        core::slice::from_raw_parts((&value as *const T).cast::<u8>(), core::mem::size_of::<T>())
    };
    assert!(bytes.iter().all(|byte| *byte == 0));
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

#[test]
fn owned_bit_stream_send_helpers_preserve_exact_partial_bit_lengths() {
    let mut stream = samp_protocol::BitStream::new();
    stream.write_bits(&[0b0000_0101], 3).unwrap();

    let api = test_support::test_api();
    assert_eq!(
        api.send_packet_stream(200, &stream, SampClientSdkSendOptions::default()),
        SampClientSdkResult::NativeCallFailed
    );
    assert_eq!(
        api.send_rpc_stream(62, &stream, SampClientSdkSendOptions::default()),
        SampClientSdkResult::NativeCallFailed
    );
}

#[test]
fn send_chat_uses_the_protocol_bounded_rpc_101_payload() {
    let api = test_support::test_api();
    assert_eq!(api.send_chat(b"hi"), SampClientSdkResult::Ok);
    assert_eq!(api.send_chat(b"/hi"), SampClientSdkResult::Ok);
    assert_eq!(
        api.send_chat(&[b'x'; 256]),
        SampClientSdkResult::InvalidArgument
    );
    assert_eq!(api.send_request_spawn(), SampClientSdkResult::Ok);
}

#[test]
fn local_player_protocol_actions_preserve_their_wire_vectors() {
    let api = test_support::test_api();
    assert_eq!(api.send_request_class(9), SampClientSdkResult::Ok);
    assert_eq!(api.send_interior_change(7), SampClientSdkResult::Ok);
    assert_eq!(api.send_spawn(), SampClientSdkResult::Ok);
    assert_eq!(
        api.send_enter_vehicle(0x1234, true),
        SampClientSdkResult::Ok
    );
    assert_eq!(api.send_exit_vehicle(0x1234), SampClientSdkResult::Ok);
}

#[test]
fn typed_protocol_action_conveniences_preserve_their_wire_vectors() {
    let api = test_support::test_api();
    assert_eq!(
        api.send_dialog_response(0x1234, 1, 0x3456, b"ok"),
        SampClientSdkResult::Ok
    );
    assert_eq!(api.send_click_player(0x1234, 2), SampClientSdkResult::Ok);
    assert_eq!(api.send_click_textdraw(0x1234), SampClientSdkResult::Ok);
    assert_eq!(api.send_death_by_player(0x1234, 9), SampClientSdkResult::Ok);
    assert_eq!(api.send_menu_quit(), SampClientSdkResult::Ok);
    assert_eq!(api.send_menu_select_row(7), SampClientSdkResult::Ok);
    assert_eq!(api.send_picked_up_pickup(9), SampClientSdkResult::Ok);
    assert_eq!(api.send_vehicle_destroyed(0x1234), SampClientSdkResult::Ok);
    assert_eq!(
        api.send_dialog_response(0, 0, 0, &[b'x'; 256]),
        SampClientSdkResult::InvalidArgument
    );
}

#[test]
fn additional_typed_protocol_actions_preserve_their_wire_vectors() {
    let api = test_support::test_api();
    assert_eq!(
        api.send_vehicle_damage(0x1234, 1, 2, 3, 4),
        SampClientSdkResult::Ok
    );
    assert_eq!(api.send_scm_event(4, 1, 2, 3), SampClientSdkResult::Ok);
    assert_eq!(
        api.send_give_damage(0x1234, 1.0, 24, 9),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_take_damage(0x1234, 1.0, 24, 9),
        SampClientSdkResult::Ok
    );

    let protocol_zero = samp_protocol::types::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let attached = samp_protocol::rpc::outgoing::common::EditAttachedObject {
        response: 0,
        index: 0,
        model_id: 0,
        bone: 0,
        position: protocol_zero,
        rotation: protocol_zero,
        scale: protocol_zero,
        color1: 0,
        color2: 0,
    };
    assert_eq!(
        api.send_edit_attached_object(attached),
        SampClientSdkResult::Ok
    );
    let zero = events::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    assert_eq!(
        api.send_edit_object(events::rpc::outgoing::object::EditObject {
            player_object: false,
            object_id: 0,
            response: 0,
            position: zero,
            rotation: zero,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(api.send_rcon_command(b"rcon"), SampClientSdkResult::Ok);
    assert_eq!(
        api.send_rcon_command(&[b'x'; events::MAX_STRING32_BYTES + 1]),
        SampClientSdkResult::InvalidArgument
    );
}

#[test]
fn typed_sync_send_conveniences_preserve_their_fixed_wire_vectors() {
    let api = test_support::test_api();
    let zero = samp_protocol::types::Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    assert_eq!(
        api.send_aim_sync(samp_protocol::packet::common::AimSync {
            camera_mode: 0,
            camera_front: zero,
            camera_position: zero,
            aim_z: 0.0,
            zoom_and_weapon_state: 0,
            aspect_ratio: 0,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_bullet_sync(samp_protocol::packet::common::BulletSync {
            target_type: 0,
            target_id: 0,
            origin: zero,
            target: zero,
            center: zero,
            weapon_id: 0,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_vehicle_sync(samp_protocol::packet::common::VehicleSync {
            vehicle_id: 0,
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            quaternion: [0.0; 4],
            position: zero,
            move_speed: zero,
            vehicle_health: 0.0,
            player_health: 0,
            armour: 0,
            weapon_and_special_key: 0,
            siren: 0,
            landing_gear_state: 0,
            trailer_id: 0,
            vehicle_specific: [0; 4],
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_player_sync(samp_protocol::packet::common::PlayerSync {
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
            quaternion: [0.0; 4],
            health: 0,
            armour: 0,
            weapon_and_special_key: 0,
            special_action: 0,
            move_speed: zero,
            surfing_offsets: zero,
            surfing_vehicle_id: 0,
            animation_id: 0,
            animation_flags: 0,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_spectator_sync(samp_protocol::packet::common::SpectatorSync {
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_trailer_sync(samp_protocol::packet::common::TrailerSync {
            trailer_id: 0,
            position: zero,
            quaternion: [0.0; 4],
            move_speed: zero,
            turn_speed: zero,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_passenger_sync(samp_protocol::packet::common::PassengerSync {
            vehicle_id: 0,
            seat_driveby_cuffed: 0,
            weapon_and_special_key: 0,
            health: 0,
            armour: 0,
            left_right_keys: 0,
            up_down_keys: 0,
            key_data: 0,
            position: zero,
        }),
        SampClientSdkResult::Ok
    );
    assert_eq!(
        api.send_unoccupied_sync(samp_protocol::packet::common::UnoccupiedSync {
            vehicle_id: 0,
            seat_id: 0,
            roll: zero,
            direction: zero,
            position: zero,
            move_speed: zero,
            turn_speed: zero,
            vehicle_health: 0.0,
        }),
        SampClientSdkResult::Ok
    );
}

#[test]
fn safe_rpc_registration_dispatches_and_synchronizes() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let subscription = test_support::test_api()
        .on_rpc(SampClientSdkDirection::Incoming, move |event| {
            assert_eq!(event.id(), 42);
            observed.fetch_add(1, Ordering::AcqRel);
            SampClientSdkHookAction::Block
        })
        .expect("test registration must succeed");

    assert_eq!(subscription.id(), 1);
    assert_eq!(
        test_support::invoke_registered_callback(42),
        Some(SampClientSdkHookAction::Block)
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
    assert_eq!(test_support::invoke_registered_callback(42), None);
    assert_eq!(
        test_support::registration_stats().unregister_and_wait_calls,
        1
    );
}

#[test]
fn safe_callback_panic_fails_open() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Outgoing, |_| {
            panic!("test callback panic")
        })
        .expect("test registration must succeed");

    assert_eq!(
        test_support::invoke_registered_callback(10),
        Some(SampClientSdkHookAction::Continue)
    );
    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn id_filtered_callback_ignores_unrelated_events() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let subscription = test_support::test_api()
        .on_rpc_id(SampClientSdkDirection::Incoming, 42, move |_| {
            observed.fetch_add(1, Ordering::AcqRel);
            SampClientSdkHookAction::Block
        })
        .expect("test registration must succeed");

    assert_eq!(
        test_support::invoke_registered_callback(41),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(
        test_support::invoke_registered_callback(42),
        Some(SampClientSdkHookAction::Block)
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_callback_decodes_matching_descriptor_and_fails_open() {
    use samp_protocol::WireDescriptor;

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_rpc(protocol_incoming::r1::ENABLE_STUNT_BONUS, move |enabled| {
            assert!(enabled);
            observed.fetch_add(1, Ordering::AcqRel);
            ProtocolAction::Block
        })
        .expect("test registration must succeed");
    assert_eq!(test_support::registration_stats().registered_callbacks, 1);

    assert_eq!(
        test_support::invoke_registered_callback(99),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_payload(
            protocol_incoming::r1::EnableStuntBonusRpc::ID,
            EncodedPayload::from_bits(
                protocol_incoming::r1::EnableStuntBonusRpc::encode_bits(&true)
                    .expect("the Protocol test payload must encode")
                    .as_bytes()
                    .to_vec(),
                1,
            )
            .expect("the Protocol test payload must preserve its bit length"),
        ),
        Some(SampClientSdkHookAction::Block)
    );
    assert_eq!(
        test_support::invoke_registered_callback(protocol_incoming::r1::EnableStuntBonusRpc::ID),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(calls.load(Ordering::Acquire), 1);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_chat_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::WireDescriptor;

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_outgoing_typed_rpc(
            samp_protocol::rpc::outgoing::chat::SEND_CHAT,
            |text| match text.as_slice() {
                b"continue" => ProtocolAction::Continue,
                b"block" => ProtocolAction::Block,
                b"replace" => ProtocolAction::Replace(b"changed".to_vec()),
                _ => unreachable!("test payload must select a callback action"),
            },
        )
        .expect("test registration must succeed");

    for (value, expected) in [
        (b"continue".as_slice(), SampClientSdkHookAction::Continue),
        (b"block".as_slice(), SampClientSdkHookAction::Block),
    ] {
        let bits = samp_protocol::rpc::outgoing::chat::SendChat::encode_bits(&value.to_vec())
            .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
            .expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(101, payload),
            Some(expected)
        );
    }

    let bits = samp_protocol::rpc::outgoing::chat::SendChat::encode_bits(&b"replace".to_vec())
        .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
        .expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(101, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![7, b'c', b'h', b'a', b'n', b'g', b'e', b'd'],
            64,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_common_outgoing_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::{
        WireDescriptor,
        rpc::outgoing::common::{DialogResponse, SEND_DIALOG_RESPONSE, SendDialogResponse},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let subscription = api
        .on_outgoing_typed_rpc(SEND_DIALOG_RESPONSE, |response| {
            match response.input.as_slice() {
                b"continue" => ProtocolAction::Continue,
                b"block" => ProtocolAction::Block,
                b"replace" => ProtocolAction::Replace(DialogResponse {
                    input: b"changed".to_vec(),
                    ..response
                }),
                _ => unreachable!("test payload must select a callback action"),
            }
        })
        .expect("test registration must succeed");

    for (input, expected) in [
        (b"continue".as_slice(), SampClientSdkHookAction::Continue),
        (b"block".as_slice(), SampClientSdkHookAction::Block),
    ] {
        let bits = SendDialogResponse::encode_bits(&DialogResponse {
            dialog_id: 0x1234,
            button: 1,
            list_item: 0x5678,
            input: input.to_vec(),
        })
        .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
            .expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(62, payload),
            Some(expected)
        );
    }

    let bits = SendDialogResponse::encode_bits(&DialogResponse {
        dialog_id: 0x1234,
        button: 1,
        list_item: 0x5678,
        input: b"replace".to_vec(),
    })
    .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
        .expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(62, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![
                0x34, 0x12, 1, 0x78, 0x56, 7, b'c', b'h', b'a', b'n', b'g', b'e', b'd'
            ],
            104,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_common_packet_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::{
        WireDescriptor,
        packet::common::{CONNECTION_ACCEPTED, ConnectionAccepted, ConnectionAcceptedPacket},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_packet(CONNECTION_ACCEPTED, |connection| {
            match connection.challenge {
                1 => ProtocolAction::Continue,
                2 => ProtocolAction::Block,
                3 => ProtocolAction::Replace(ConnectionAccepted {
                    challenge: 42,
                    ..connection
                }),
                _ => unreachable!("test payload must select a callback action"),
            }
        })
        .expect("test registration must succeed");

    for (challenge, expected) in [
        (1, SampClientSdkHookAction::Continue),
        (2, SampClientSdkHookAction::Block),
    ] {
        let bits = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
            ip: -1,
            port: 1,
            player_id: 2,
            challenge,
        })
        .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
            .expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(34, payload),
            Some(expected)
        );
    }

    let bits = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
        ip: -1,
        port: 1,
        player_id: 2,
        challenge: 3,
    })
    .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
        .expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(34, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![0xFF, 0xFF, 0xFF, 0xFF, 1, 0, 2, 0, 42, 0, 0, 0,],
            96,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn malformed_typed_packet_is_diagnosed_before_fail_open() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };
    use samp_protocol::{
        WireDescriptor,
        packet::common::{CONNECTION_ACCEPTED, ConnectionAccepted, ConnectionAcceptedPacket},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let handler_calls = Arc::new(AtomicUsize::new(0));
    let captured_calls = Arc::clone(&handler_calls);
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_packet(CONNECTION_ACCEPTED, move |_| {
            captured_calls.fetch_add(1, Ordering::Relaxed);
            ProtocolAction::Continue
        })
        .expect("test registration must succeed");

    let bits = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
        ip: -1,
        port: 1,
        player_id: 2,
        challenge: 3,
    })
    .unwrap();
    let mut bytes = bits.as_bytes().to_vec();
    bytes.push(0);
    let payload = EncodedPayload::from_bits(bytes, bits.len_bits() + 8).unwrap();

    assert_eq!(
        test_support::invoke_registered_callback_with_payload(34, payload),
        Some(SampClientSdkHookAction::Continue)
    );
    assert_eq!(handler_calls.load(Ordering::Relaxed), 0);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Debug,
            direction: "incoming",
            kind: "packet",
            id: 34,
            phase: CallbackFailurePhase::DecodeMalformed,
        }]
    );

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn typed_source_failure_is_warned_before_fail_open() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let handler_calls = Arc::new(AtomicUsize::new(0));
    let captured_calls = Arc::clone(&handler_calls);
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_rpc(protocol_incoming::SET_PLAYER_DRUNK, move |_| {
            captured_calls.fetch_add(1, Ordering::Relaxed);
            ProtocolAction::Continue
        })
        .unwrap();
    let payload = EncodedPayload::from_bits(vec![7, 0, 0, 0], 32).unwrap();

    let outcome = test_support::invoke_registered_callback_with_source_failure(
        35,
        payload,
        SampClientSdkResult::NativeCallFailed,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(handler_calls.load(Ordering::Relaxed), 0);
    assert_eq!(outcome.bytes, [7, 0, 0, 0]);
    assert_eq!(outcome.bit_len, 32);
    assert_eq!(outcome.replacement_calls, 0);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "rpc",
            id: 35,
            phase: CallbackFailurePhase::DecodeSource,
        }]
    );

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn replacement_encode_failure_preserves_payload_without_host_mutation() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let net = Samp::from_api(test_support::test_api()).net();
    let subscription = net
        .on_incoming_typed_rpc(FailingIncomingRpc::new(), ProtocolAction::Replace)
        .unwrap();
    let original = EncodedPayload::from_bits(vec![0x80], 8).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        201,
        original,
        SampClientSdkResult::Ok,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, [0x80]);
    assert_eq!(outcome.bit_len, 8);
    assert_eq!(outcome.replacement_calls, 0);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "rpc",
            id: 201,
            phase: CallbackFailurePhase::ReplacementEncode,
        }]
    );

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn host_rejection_preserves_incoming_rpc_and_packet_payloads() {
    use crate::events::{
        CallbackFailurePhase, TestCallbackDiagnostic, take_test_callback_diagnostics,
    };
    use samp_protocol::{
        WireDescriptor,
        packet::common::{CONNECTION_ACCEPTED, ConnectionAccepted, ConnectionAcceptedPacket},
        rpc::incoming::{SET_PLAYER_DRUNK, SetPlayerDrunk},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let net = Samp::from_api(test_support::test_api()).net();
    let rpc_subscription = net
        .on_incoming_typed_rpc(SET_PLAYER_DRUNK, |_| ProtocolAction::Replace(8))
        .unwrap();
    let original = SetPlayerDrunk::encode_bits(&7).unwrap();
    let original_bytes = original.as_bytes().to_vec();
    let original_bit_len = original.len_bits();
    let payload = EncodedPayload::from_bits(original_bytes.clone(), original_bit_len).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        35,
        payload,
        SampClientSdkResult::NativeCallFailed,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, original_bytes);
    assert_eq!(outcome.bit_len, original_bit_len);
    assert_eq!(outcome.replacement_calls, 1);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "rpc",
            id: 35,
            phase: CallbackFailurePhase::ReplacementHost,
        }]
    );
    rpc_subscription.unregister_and_wait().unwrap();

    test_support::reset_registration();
    let packet_subscription = net
        .on_incoming_typed_packet(CONNECTION_ACCEPTED, |connection| {
            ProtocolAction::Replace(ConnectionAccepted {
                challenge: 42,
                ..connection
            })
        })
        .unwrap();
    let original = ConnectionAcceptedPacket::encode_bits(&ConnectionAccepted {
        ip: -1,
        port: 1,
        player_id: 2,
        challenge: 3,
    })
    .unwrap();
    let original_bytes = original.as_bytes().to_vec();
    let original_bit_len = original.len_bits();
    let payload = EncodedPayload::from_bits(original_bytes.clone(), original_bit_len).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        34,
        payload,
        SampClientSdkResult::NativeCallFailed,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, original_bytes);
    assert_eq!(outcome.bit_len, original_bit_len);
    assert_eq!(outcome.replacement_calls, 1);
    assert_eq!(
        take_test_callback_diagnostics(),
        vec![TestCallbackDiagnostic {
            level: log::Level::Warn,
            direction: "incoming",
            kind: "packet",
            id: 34,
            phase: CallbackFailurePhase::ReplacementHost,
        }]
    );
    packet_subscription.unregister_and_wait().unwrap();
}

#[test]
fn successful_non_byte_aligned_replacement_uses_one_host_call() {
    use crate::events::take_test_callback_diagnostics;
    use samp_protocol::{WireDescriptor, packet::r1};

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    take_test_callback_diagnostics();
    let net = Samp::from_api(test_support::test_api()).net();
    let value = r1::MarkersSync {
        markers: vec![r1::Marker {
            player_id: 1,
            coordinates: None,
        }],
    };
    let replacement = value.clone();
    let subscription = net
        .on_incoming_typed_packet(r1::MARKERS_SYNC, move |_| {
            ProtocolAction::Replace(replacement.clone())
        })
        .unwrap();
    let original = r1::MarkersSyncPacket::encode_bits(&value).unwrap();
    assert_eq!(original.len_bits(), 49);
    let payload =
        EncodedPayload::from_bits(original.as_bytes().to_vec(), original.len_bits()).unwrap();

    let outcome = test_support::invoke_registered_callback_with_host_replacement(
        r1::MarkersSyncPacket::ID,
        payload,
        SampClientSdkResult::Ok,
    )
    .unwrap();

    assert_eq!(outcome.action, SampClientSdkHookAction::Continue);
    assert_eq!(outcome.bytes, original.as_bytes());
    assert_eq!(outcome.bit_len, 49);
    assert_eq!(outcome.replacement_calls, 1);
    assert!(take_test_callback_diagnostics().is_empty());

    subscription.unregister_and_wait().unwrap();
}

#[test]
fn normal_typed_methods_accept_all_descriptor_sources() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let net = Samp::from_api(test_support::test_api()).net();

    let subscriptions = [
        net.on_outgoing_typed_packet(samp_protocol::packet::common::SEND_PLAYER_SYNC, |_| {
            ProtocolAction::Continue
        })
        .expect("Protocol Packet registration must succeed"),
        net.on_incoming_typed_rpc(events::rpc::incoming::SHOW_DIALOG, |_| {
            ProtocolAction::Continue
        })
        .expect("legacy incoming RPC registration must succeed"),
        net.on_outgoing_typed_rpc(events::rpc::outgoing::damage::SEND_DAMAGE, |_| {
            ProtocolAction::Continue
        })
        .expect("legacy outgoing RPC registration must succeed"),
    ];

    assert_eq!(test_support::registration_stats().registered_callbacks, 3);
    for subscription in subscriptions {
        subscription
            .unregister_and_wait()
            .expect("test shutdown must synchronize");
    }
}

#[test]
fn normal_typed_legacy_callback_preserves_all_actions() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let subscription = Samp::from_api(test_support::test_api())
        .net()
        .on_outgoing_typed_rpc(events::rpc::outgoing::damage::SEND_DAMAGE, move |damage| {
            match observed.fetch_add(1, Ordering::AcqRel) {
                0 => ProtocolAction::Continue,
                1 => ProtocolAction::Block,
                2 => ProtocolAction::Replace(damage),
                _ => unreachable!("test invokes exactly three actions"),
            }
        })
        .expect("legacy typed registration must succeed");
    let payload = || {
        EncodedPayload::from_bits(
            vec![
                0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00, 0x00,
                0x00,
            ],
            113,
        )
        .expect("the exact damage vector is valid")
    };

    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(115, payload()),
        Some((
            SampClientSdkHookAction::Continue,
            payload().as_bytes().to_vec(),
            113,
        ))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(115, payload()),
        Some((
            SampClientSdkHookAction::Block,
            payload().as_bytes().to_vec(),
            113,
        ))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(115, payload()),
        Some((
            SampClientSdkHookAction::Continue,
            payload().as_bytes().to_vec(),
            113,
        ))
    );
    assert_eq!(calls.load(Ordering::Acquire), 3);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn normal_typed_legacy_packet_callback_preserves_all_actions() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let descriptor =
        events::OutgoingPacket::new(204, |event| event.read_u8(), |value| Ok(vec![value]));
    let subscription = Samp::from_api(test_support::test_api())
        .net()
        .on_outgoing_typed_packet(descriptor, move |value| {
            match observed.fetch_add(1, Ordering::AcqRel) {
                0 => ProtocolAction::Continue,
                1 => ProtocolAction::Block,
                2 => ProtocolAction::Replace(value + 1),
                _ => unreachable!("test invokes exactly three actions"),
            }
        })
        .expect("legacy typed Packet registration must succeed");
    let payload = || EncodedPayload::from_bits(vec![7], 8).expect("test payload must be valid");

    assert_eq!(test_support::registration_stats().registered_callbacks, 1);
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(204, payload()),
        Some((SampClientSdkHookAction::Continue, vec![7], 8))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(204, payload()),
        Some((SampClientSdkHookAction::Block, vec![7], 8))
    );
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(204, payload()),
        Some((SampClientSdkHookAction::Continue, vec![8], 8))
    );
    assert_eq!(calls.load(Ordering::Acquire), 3);

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn protocol_server_message_callback_preserves_continue_block_and_replacement() {
    use samp_protocol::{
        WireDescriptor,
        rpc::incoming::{SERVER_MESSAGE, ServerMessage, ServerMessageRpc},
    };

    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let subscription = api
        .on_incoming_typed_rpc(SERVER_MESSAGE, |message| match message.text.as_slice() {
            b"continue" => ProtocolAction::Continue,
            b"block" => ProtocolAction::Block,
            b"replace" => ProtocolAction::Replace(ServerMessage {
                color: message.color,
                text: b"changed".to_vec(),
            }),
            _ => unreachable!("test payload must select a callback action"),
        })
        .expect("test registration must succeed");

    for (text, expected) in [
        (b"continue".as_slice(), SampClientSdkHookAction::Continue),
        (b"block".as_slice(), SampClientSdkHookAction::Block),
    ] {
        let bits = ServerMessageRpc::encode_bits(&ServerMessage {
            color: 0x1122_3344,
            text: text.to_vec(),
        })
        .expect("test payload must encode");
        let (bytes, bit_len) = bits.into_parts();
        let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
            .expect("test payload must fit its storage");
        assert_eq!(
            test_support::invoke_registered_callback_with_payload(93, payload),
            Some(expected)
        );
    }

    let bits = ServerMessageRpc::encode_bits(&ServerMessage {
        color: 0x1122_3344,
        text: b"replace".to_vec(),
    })
    .expect("test payload must encode");
    let (bytes, bit_len) = bits.into_parts();
    let payload = crate::events::EncodedPayload::from_bits(bytes, bit_len)
        .expect("test payload must fit its storage");
    assert_eq!(
        test_support::invoke_registered_callback_with_replacement(93, payload),
        Some((
            SampClientSdkHookAction::Continue,
            vec![
                0x44, 0x33, 0x22, 0x11, 7, 0, 0, 0, b'c', b'h', b'a', b'n', b'g', b'e', b'd'
            ],
            120,
        ))
    );

    subscription
        .unregister_and_wait()
        .expect("test shutdown must synchronize");
}

#[test]
fn register_handlers_collects_every_supported_handler_form() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();

    let subscriptions = register_handlers!(test_support::test_api();
        packet(SampClientSdkDirection::Incoming, |_| SampClientSdkHookAction::Continue),
        rpc(SampClientSdkDirection::Outgoing, |_| SampClientSdkHookAction::Continue),
        packet_id(SampClientSdkDirection::Incoming, 1, |_| SampClientSdkHookAction::Continue),
        rpc_id(SampClientSdkDirection::Outgoing, 2, |_| SampClientSdkHookAction::Continue),
        incoming_typed_packet(
            samp_protocol::packet::r1::PLAYER_SYNC,
            |_| ProtocolAction::Continue
        ),
        incoming_typed_rpc(
            protocol_incoming::r1::ENABLE_STUNT_BONUS,
            |_| ProtocolAction::Continue
        ),
    )
    .expect("all test registrations must succeed");

    assert_eq!(subscriptions.len(), 6);
    assert_eq!(
        test_support::registration_stats().registered_callbacks,
        subscriptions.len()
    );
    subscriptions
        .unregister_and_wait()
        .expect("test shutdown must synchronize every callback");
    assert_eq!(test_support::registration_stats().registered_callbacks, 0);
}

#[test]
fn subscription_set_retains_each_failed_shutdown_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let api = test_support::test_api();
    let mut subscriptions = SubscriptionSet::new();
    subscriptions.push(
        api.on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    subscriptions.push(
        api.on_rpc(SampClientSdkDirection::Outgoing, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed"),
    );
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscriptions
        .unregister_and_wait()
        .expect_err("failed callbacks must remain available for retry");
    assert_eq!(error.failures().len(), 2);
    assert!(
        error
            .failures()
            .iter()
            .all(|failure| failure.result() == SampClientSdkResult::CallbackInProgress)
    );
    assert_eq!(test_support::registration_stats().registered_callbacks, 2);

    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    error
        .into_subscriptions()
        .unregister_and_wait()
        .expect("retry must synchronize every callback");
    let stats = test_support::registration_stats();
    assert_eq!(stats.unregister_and_wait_calls, 4);
    assert_eq!(stats.registered_callbacks, 0);
}

#[test]
fn subscription_set_preserves_earlier_registrations_after_a_registration_failure() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    let error = SubscriptionSet::new()
        .try_add(Ok(subscription))
        .and_then(|subscriptions| subscriptions.try_add(Err(SampClientSdkResult::NotReady)))
        .expect_err("the synthetic second registration must fail");
    assert_eq!(error.result(), SampClientSdkResult::NotReady);
    let subscriptions = error.into_subscriptions();
    assert_eq!(subscriptions.len(), 1);
    subscriptions
        .unregister_and_wait()
        .expect("retained subscription must remain cleanly removable");
}

#[test]
fn failed_synchronized_shutdown_keeps_the_subscription_for_retry() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let subscription = test_support::test_api()
        .on_rpc(SampClientSdkDirection::Incoming, |_| {
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");
    test_support::set_unregister_and_wait_result(SampClientSdkResult::CallbackInProgress);

    let error = subscription
        .unregister_and_wait()
        .expect_err("callback-thread shutdown must remain retryable");
    assert_eq!(error.result(), SampClientSdkResult::CallbackInProgress);
    let subscription = error.into_subscription();
    test_support::set_unregister_and_wait_result(SampClientSdkResult::Ok);
    subscription
        .unregister_and_wait()
        .expect("retry must synchronize");
}

#[test]
fn failed_registration_releases_the_handler() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    test_support::set_register_result(SampClientSdkResult::NotReady);
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));

    let result = test_support::test_api().on_packet(SampClientSdkDirection::Incoming, move |_| {
        let _ = &counter;
        SampClientSdkHookAction::Continue
    });
    assert_eq!(result.unwrap_err(), SampClientSdkResult::NotReady);
    assert_eq!(drops.load(Ordering::Acquire), 1);
}

#[test]
fn dropping_a_subscription_detaches_without_freeing_callback_state() {
    let _serial = REGISTRATION_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    test_support::reset_registration();
    let drops = Arc::new(AtomicUsize::new(0));
    let counter = DropCounter(Arc::clone(&drops));
    let subscription = test_support::test_api()
        .on_packet(SampClientSdkDirection::Incoming, move |_| {
            let _ = &counter;
            SampClientSdkHookAction::Continue
        })
        .expect("test registration must succeed");

    drop(subscription);
    assert_eq!(drops.load(Ordering::Acquire), 0);
    assert_eq!(test_support::invoke_registered_callback(1), None);
    assert_eq!(test_support::registration_stats().unregister_calls, 1);
}
