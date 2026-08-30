//! Independent native-client profile layout fixtures.
//!
//! These tests cover the native structures consumed by the enabled R1, R3-1,
//! R5-1, and DL-R1 direct-helper profiles.

use crate::client::SampVersion;
use samp_native::profiles::dl::DL_SPEC;
use samp_native::profiles::r1::R1_SPEC;
use samp_native::profiles::r3::R3_SPEC;
use samp_native::profiles::r5::R5_SPEC;

type FixtureFn = unsafe extern "C" fn() -> usize;

unsafe extern "C" {
    fn samp_client_sdk_fixture_r1_onfoot_size() -> usize;
    fn samp_client_sdk_fixture_r1_incar_size() -> usize;
    fn samp_client_sdk_fixture_r1_local_active_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_current_vehicle_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_incar_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_passenger_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_trailer_offset() -> usize;
    fn samp_client_sdk_fixture_r1_onfoot_position_offset() -> usize;
    fn samp_client_sdk_fixture_r1_onfoot_speed_offset() -> usize;
    fn samp_client_sdk_fixture_r1_onfoot_special_action_offset() -> usize;
    fn samp_client_sdk_fixture_r1_onfoot_animation_offset() -> usize;
    fn samp_client_sdk_fixture_r1_incar_position_offset() -> usize;
    fn samp_client_sdk_fixture_r1_incar_speed_offset() -> usize;
    fn samp_client_sdk_fixture_r1_ped_game_ped_offset() -> usize;
    fn samp_client_sdk_fixture_r1_player_pool_local_id_offset() -> usize;
    fn samp_client_sdk_fixture_r1_player_pool_largest_id_offset() -> usize;
    fn samp_client_sdk_fixture_r1_vehicle_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_vehicle_pool_game_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r1_object_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_object_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r1_pickup_pool_handles_offset() -> usize;
    fn samp_client_sdk_fixture_r1_entity_handle_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_host_address_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_hostname_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_port_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_server_settings_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_pools_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_pools_label_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_pools_text_draw_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_pools_object_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_pools_gang_zone_offset() -> usize;
    fn samp_client_sdk_fixture_r1_net_game_pools_pickup_offset() -> usize;
    fn samp_client_sdk_fixture_r1_label_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_size() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_text_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_colour_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_position_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_draw_distance_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_behind_walls_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_attached_player_offset() -> usize;
    fn samp_client_sdk_fixture_r1_text_label_attached_vehicle_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_data_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_size() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_transmit_size() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_transmit_x_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_transmit_y_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_letter_width_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_model_id_offset() -> usize;
    fn samp_client_sdk_fixture_r1_gangzone_size() -> usize;
    fn samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_game_cursor_mode_offset() -> usize;
    fn samp_client_sdk_fixture_r1_scoreboard_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_listbox_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_editbox_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_text_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_type_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_id_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_caption_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_server_side_offset() -> usize;
    fn samp_client_sdk_fixture_r1_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r1_chat_entries_offset() -> usize;
    fn samp_client_sdk_fixture_r1_chat_entry_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_rak_client_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_host_address_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_hostname_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_port_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_netgame_pools_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_pools_label_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_text_label_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_label_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_vehicle_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_vehicle_pool_game_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_pools_object_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_object_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_object_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_pools_pickup_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_pickup_pool_handles_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_entity_handle_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_pools_gangzone_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_gangzone_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_gangzone_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_pools_textdraw_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_textdraw_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_textdraw_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_animation_entry_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_player_pool_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_player_pool_largest_id_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_player_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_player_pool_local_id_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_player_info_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_player_info_is_npc_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_remote_player_special_action_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_remote_player_reported_armour_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_remote_player_reported_health_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_remote_player_animation_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_local_player_incar_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_local_player_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_local_player_active_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_local_player_current_vehicle_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_local_player_last_any_update_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_ped_game_ped_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_editbox_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_command_count_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_command_names_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_command_name_capacity() -> usize;
    fn samp_client_sdk_fixture_r3_1_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_caption_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_listbox_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_editbox_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_type_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_id_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_text_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_dialog_server_side_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_listbox_selected_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_listbox_items_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_listbox_item_count_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_scoreboard_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_scoreboard_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r3_1_game_size() -> usize;
    fn samp_client_sdk_fixture_r3_1_game_cursor_mode_offset() -> usize;

    fn samp_client_sdk_fixture_r5_1_netgame_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_netgame_rak_client_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_netgame_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_netgame_pools_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_input_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_input_command_count_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_dialog_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_dialog_caption_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_pools_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_pools_pickup_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_pools_object_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_pools_gangzone_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_pools_label_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_pools_textdraw_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_player_info_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_player_info_is_npc_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_player_pool_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_player_pool_local_id_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_player_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_player_pool_largest_id_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_remote_player_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_remote_player_special_action_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_remote_player_animation_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_remote_player_ped_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_size() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_incar_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_aim_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_trailer_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_passenger_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_active_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_current_vehicle_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_ped_offset() -> usize;
    fn samp_client_sdk_fixture_r5_1_local_player_last_any_update_offset() -> usize;

    fn samp_client_sdk_fixture_dl_netgame_size() -> usize;
    fn samp_client_sdk_fixture_dl_netgame_rak_client_offset() -> usize;
    fn samp_client_sdk_fixture_dl_netgame_game_state_offset() -> usize;
    fn samp_client_sdk_fixture_dl_netgame_pools_offset() -> usize;
    fn samp_client_sdk_fixture_dl_input_size() -> usize;
    fn samp_client_sdk_fixture_dl_input_command_count_offset() -> usize;
    fn samp_client_sdk_fixture_dl_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_dl_dialog_size() -> usize;
    fn samp_client_sdk_fixture_dl_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_dl_dialog_caption_offset() -> usize;
    fn samp_client_sdk_fixture_dl_pools_size() -> usize;
    fn samp_client_sdk_fixture_dl_pools_pickup_offset() -> usize;
    fn samp_client_sdk_fixture_dl_pools_object_offset() -> usize;
    fn samp_client_sdk_fixture_dl_pools_gangzone_offset() -> usize;
    fn samp_client_sdk_fixture_dl_pools_label_offset() -> usize;
    fn samp_client_sdk_fixture_dl_pools_textdraw_offset() -> usize;
    fn samp_client_sdk_fixture_dl_samp_string_size() -> usize;
    fn samp_client_sdk_fixture_dl_player_info_size() -> usize;
    fn samp_client_sdk_fixture_dl_player_info_is_npc_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_local_id_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_local_player_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_largest_id_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_players_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_collision_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_ping_offset() -> usize;
    fn samp_client_sdk_fixture_dl_player_pool_score_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_size() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_ped_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_trailer_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_passenger_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_incar_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_aim_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_active_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_current_vehicle_offset() -> usize;
    fn samp_client_sdk_fixture_dl_local_player_last_any_update_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_size() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_ped_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_special_action_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_passenger_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_incar_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_trailer_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_aim_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_armour_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_health_offset() -> usize;
    fn samp_client_sdk_fixture_dl_remote_player_animation_offset() -> usize;
    fn samp_client_sdk_fixture_dl_vehicle_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_vehicle_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_dl_vehicle_pool_game_objects_offset() -> usize;
    fn samp_client_sdk_fixture_dl_object_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_object_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_dl_object_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_dl_pickup_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_pickup_pool_handles_offset() -> usize;
    fn samp_client_sdk_fixture_dl_entity_size() -> usize;
    fn samp_client_sdk_fixture_dl_entity_handle_offset() -> usize;
    fn samp_client_sdk_fixture_dl_ped_size() -> usize;
    fn samp_client_sdk_fixture_dl_ped_game_ped_offset() -> usize;
    fn samp_client_sdk_fixture_dl_gangzone_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_gangzone_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_dl_label_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_label_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_dl_textdraw_size() -> usize;
    fn samp_client_sdk_fixture_dl_textdraw_data_offset() -> usize;
    fn samp_client_sdk_fixture_dl_textdraw_pool_size() -> usize;
    fn samp_client_sdk_fixture_dl_textdraw_pool_objects_offset() -> usize;
    fn samp_client_sdk_fixture_dl_chat_size() -> usize;
    fn samp_client_sdk_fixture_dl_chat_mode_offset() -> usize;
    fn samp_client_sdk_fixture_dl_chat_entries_offset() -> usize;
    fn samp_client_sdk_fixture_dl_scoreboard_size() -> usize;
    fn samp_client_sdk_fixture_dl_game_cursor_mode_offset() -> usize;
    fn samp_client_sdk_fixture_dl_animation_entry_size() -> usize;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ProfileLayout {
    netgame_size: usize,
    netgame_rak_client_offset: usize,
    netgame_game_state_offset: usize,
    netgame_pools_offset: usize,
    input_size: usize,
    input_command_count_offset: usize,
    input_enabled_offset: usize,
    dialog_size: usize,
    dialog_active_offset: usize,
    dialog_caption_offset: usize,
}

#[derive(Clone, Copy)]
struct ProfileLayoutFixture {
    version: SampVersion,
    netgame_size: FixtureFn,
    netgame_rak_client_offset: FixtureFn,
    netgame_game_state_offset: FixtureFn,
    netgame_pools_offset: FixtureFn,
    input_size: FixtureFn,
    input_command_count_offset: FixtureFn,
    input_enabled_offset: FixtureFn,
    dialog_size: FixtureFn,
    dialog_active_offset: FixtureFn,
    dialog_caption_offset: FixtureFn,
}

impl ProfileLayoutFixture {
    unsafe fn observed(self) -> ProfileLayout {
        ProfileLayout {
            netgame_size: unsafe { (self.netgame_size)() },
            netgame_rak_client_offset: unsafe { (self.netgame_rak_client_offset)() },
            netgame_game_state_offset: unsafe { (self.netgame_game_state_offset)() },
            netgame_pools_offset: unsafe { (self.netgame_pools_offset)() },
            input_size: unsafe { (self.input_size)() },
            input_command_count_offset: unsafe { (self.input_command_count_offset)() },
            input_enabled_offset: unsafe { (self.input_enabled_offset)() },
            dialog_size: unsafe { (self.dialog_size)() },
            dialog_active_offset: unsafe { (self.dialog_active_offset)() },
            dialog_caption_offset: unsafe { (self.dialog_caption_offset)() },
        }
    }
}

const R3_1_LAYOUT: ProfileLayout = ProfileLayout {
    netgame_size: 0x3E2,
    netgame_rak_client_offset: 0x2C,
    netgame_game_state_offset: 0x3CD,
    netgame_pools_offset: 0x3DE,
    input_size: 0x1AFC,
    input_command_count_offset: 0x14DC,
    input_enabled_offset: 0x14E0,
    dialog_size: 0x29D,
    dialog_active_offset: 0x28,
    dialog_caption_offset: 0x40,
};

const R5_1_LAYOUT: ProfileLayout = ProfileLayout {
    netgame_rak_client_offset: 0x00,
    ..R3_1_LAYOUT
};

const DL_LAYOUT: ProfileLayout = R3_1_LAYOUT;

#[test]
fn r3_scalar_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_netgame_host_address_offset(),
            samp_client_sdk_fixture_r3_1_netgame_hostname_offset(),
            samp_client_sdk_fixture_r3_1_netgame_port_offset(),
        )
    };
    assert_eq!(observed, (0x30, 0x131, 0x235));
}

#[test]
fn r3_local_player_snapshot_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_player_pool_local_id_offset(),
            samp_client_sdk_fixture_r3_1_local_player_incar_offset(),
            samp_client_sdk_fixture_r3_1_local_player_onfoot_offset(),
            samp_client_sdk_fixture_r3_1_local_player_active_offset(),
            samp_client_sdk_fixture_r3_1_local_player_current_vehicle_offset(),
            samp_client_sdk_fixture_r3_1_local_player_last_any_update_offset(),
            samp_client_sdk_fixture_r3_1_ped_game_ped_offset(),
        )
    };
    assert_eq!(observed, (0x2F1C, 0x04, 0x98, 0xF4, 0xFC, 0x13F, 0x2A4));
}

#[test]
fn r3_spec_layout_fields_match_the_independent_cpp_fixture() {
    unsafe {
        assert_eq!(
            R3_SPEC.net_game.host_address_offset.get(),
            samp_client_sdk_fixture_r3_1_netgame_host_address_offset()
        );
        assert_eq!(
            R3_SPEC.net_game.hostname_offset.get(),
            samp_client_sdk_fixture_r3_1_netgame_hostname_offset()
        );
        assert_eq!(
            R3_SPEC.net_game.port_offset.get(),
            samp_client_sdk_fixture_r3_1_netgame_port_offset()
        );
        assert_eq!(
            R3_SPEC.net_game.game_state_offset.get(),
            samp_client_sdk_fixture_r3_1_netgame_game_state_offset()
        );
        assert_eq!(
            R3_SPEC.net_game.pools_offset.get(),
            samp_client_sdk_fixture_r3_1_netgame_pools_offset()
        );
        assert_eq!(
            R3_SPEC.net_game.pools.text_label_offset.get(),
            samp_client_sdk_fixture_r3_1_pools_label_offset()
        );
        assert_eq!(
            R3_SPEC.pools.text_label.not_empty_offset.get(),
            samp_client_sdk_fixture_r3_1_label_pool_not_empty_offset()
        );
        assert_eq!(
            R3_SPEC.pools.vehicle.not_empty_offset.get(),
            samp_client_sdk_fixture_r3_1_vehicle_pool_not_empty_offset()
        );
        assert_eq!(
            R3_SPEC.pools.vehicle.game_objects_offset.get(),
            samp_client_sdk_fixture_r3_1_vehicle_pool_game_objects_offset()
        );
        assert_eq!(
            R3_SPEC.pools.object.not_empty_offset.get(),
            samp_client_sdk_fixture_r3_1_object_pool_not_empty_offset()
        );
        assert_eq!(
            R3_SPEC.pools.object.objects_offset.get(),
            samp_client_sdk_fixture_r3_1_object_pool_objects_offset()
        );
        assert_eq!(
            R3_SPEC.pools.pickup.handles_offset.get(),
            samp_client_sdk_fixture_r3_1_pickup_pool_handles_offset()
        );
        assert_eq!(
            R3_SPEC.pools.entity_handle_offset.get(),
            samp_client_sdk_fixture_r3_1_entity_handle_offset()
        );
        assert_eq!(
            R3_SPEC.pools.gangzone.not_empty_offset.get(),
            samp_client_sdk_fixture_r3_1_gangzone_pool_not_empty_offset()
        );
        assert_eq!(
            R3_SPEC.pools.textdraw.objects_offset.get(),
            samp_client_sdk_fixture_r3_1_textdraw_pool_objects_offset()
        );
        assert_eq!(
            R3_SPEC.players.animation.entry_size.get(),
            samp_client_sdk_fixture_r3_1_animation_entry_size()
        );
        assert_eq!(
            R3_SPEC.pools.player.largest_id_offset.get(),
            samp_client_sdk_fixture_r3_1_player_pool_largest_id_offset()
        );
        assert_eq!(
            R3_SPEC.pools.player.objects_offset.map(|value| value.get()),
            Some(samp_client_sdk_fixture_r3_1_player_pool_objects_offset())
        );
        assert_eq!(
            R3_SPEC.pools.player.local_id_offset.get(),
            samp_client_sdk_fixture_r3_1_player_pool_local_id_offset()
        );
        assert_eq!(
            R3_SPEC
                .pools
                .player
                .player_info
                .map(|value| value.npc_offset.get()),
            Some(samp_client_sdk_fixture_r3_1_player_info_is_npc_offset())
        );
        assert_eq!(
            R3_SPEC.players.remote.special_action_offset.get(),
            samp_client_sdk_fixture_r3_1_remote_player_special_action_offset()
        );
        assert_eq!(
            R3_SPEC.players.remote.reported_armour_offset.get(),
            samp_client_sdk_fixture_r3_1_remote_player_reported_armour_offset()
        );
        assert_eq!(
            R3_SPEC.players.remote.reported_health_offset.get(),
            samp_client_sdk_fixture_r3_1_remote_player_reported_health_offset()
        );
        assert_eq!(
            R3_SPEC.players.remote.animation_offset.get(),
            samp_client_sdk_fixture_r3_1_remote_player_animation_offset()
        );
        assert_eq!(
            R3_SPEC.players.local.incar_offset.get(),
            samp_client_sdk_fixture_r3_1_local_player_incar_offset()
        );
        assert_eq!(
            R3_SPEC.players.local.onfoot_offset.get(),
            samp_client_sdk_fixture_r3_1_local_player_onfoot_offset()
        );
        assert_eq!(
            R3_SPEC.players.local.active_offset.get(),
            samp_client_sdk_fixture_r3_1_local_player_active_offset()
        );
        assert_eq!(
            R3_SPEC.players.local.current_vehicle_offset.get(),
            samp_client_sdk_fixture_r3_1_local_player_current_vehicle_offset()
        );
        assert_eq!(
            R3_SPEC.players.local.last_any_update_offset.get(),
            samp_client_sdk_fixture_r3_1_local_player_last_any_update_offset()
        );
        assert_eq!(
            R3_SPEC.players.local.game_ped_offset.get(),
            samp_client_sdk_fixture_r3_1_ped_game_ped_offset()
        );
        assert_eq!(
            R3_SPEC.ui.input.edit_box_offset.get(),
            samp_client_sdk_fixture_r3_1_input_editbox_offset()
        );
        assert_eq!(
            R3_SPEC.ui.input.command_count_offset.get(),
            samp_client_sdk_fixture_r3_1_input_command_count_offset()
        );
        assert_eq!(
            R3_SPEC.ui.input.command_name_offset.get(),
            samp_client_sdk_fixture_r3_1_input_command_names_offset()
        );
        assert_eq!(
            R3_SPEC.ui.input.command_name_capacity.get(),
            samp_client_sdk_fixture_r3_1_input_command_name_capacity()
        );
        assert_eq!(
            R3_SPEC.ui.input.enabled_offset.get(),
            samp_client_sdk_fixture_r3_1_input_enabled_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.active_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_active_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.caption_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_caption_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.listbox_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_listbox_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.editbox_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_editbox_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.dialog_type_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_type_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.id_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_id_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.text_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_text_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.server_side_offset.get(),
            samp_client_sdk_fixture_r3_1_dialog_server_side_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.listbox.selected_offset.get(),
            samp_client_sdk_fixture_r3_1_listbox_selected_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.listbox.items_offset.get(),
            samp_client_sdk_fixture_r3_1_listbox_items_offset()
        );
        assert_eq!(
            R3_SPEC.ui.dialog.listbox.item_count_offset.get(),
            samp_client_sdk_fixture_r3_1_listbox_item_count_offset()
        );
        assert_eq!(
            R3_SPEC.ui.scoreboard.enabled_offset.get(),
            samp_client_sdk_fixture_r3_1_scoreboard_enabled_offset()
        );
        assert_eq!(
            R3_SPEC.ui.game.cursor_mode_offset.get(),
            samp_client_sdk_fixture_r3_1_game_cursor_mode_offset()
        );
    }
}

#[test]
fn r5_distinct_player_and_pool_layouts_match_the_independent_cpp_fixture() {
    let pools = unsafe {
        (
            samp_client_sdk_fixture_r5_1_pools_size(),
            samp_client_sdk_fixture_r5_1_pools_pickup_offset(),
            samp_client_sdk_fixture_r5_1_pools_object_offset(),
            samp_client_sdk_fixture_r5_1_pools_gangzone_offset(),
            samp_client_sdk_fixture_r5_1_pools_label_offset(),
            samp_client_sdk_fixture_r5_1_pools_textdraw_offset(),
        )
    };
    assert_eq!(pools, (0x24, 0x08, 0x0C, 0x14, 0x18, 0x1C));

    let player_pool = unsafe {
        (
            samp_client_sdk_fixture_r5_1_player_pool_size(),
            samp_client_sdk_fixture_r5_1_player_pool_local_id_offset(),
            samp_client_sdk_fixture_r5_1_player_pool_objects_offset(),
            samp_client_sdk_fixture_r5_1_player_pool_largest_id_offset(),
            samp_client_sdk_fixture_r5_1_player_info_size(),
            samp_client_sdk_fixture_r5_1_player_info_is_npc_offset(),
        )
    };
    assert_eq!(player_pool, (0x2F3E, 0x04, 0x1F8A, 0x2F3A, 0x30, 0x08));

    let remote = unsafe {
        (
            samp_client_sdk_fixture_r5_1_remote_player_size(),
            samp_client_sdk_fixture_r5_1_remote_player_special_action_offset(),
            samp_client_sdk_fixture_r5_1_remote_player_animation_offset(),
            samp_client_sdk_fixture_r5_1_remote_player_ped_offset(),
        )
    };
    assert_eq!(remote, (0x1FD, 0x0C, 0x1B4, 0x1DD));

    let local = unsafe {
        (
            samp_client_sdk_fixture_r5_1_local_player_size(),
            samp_client_sdk_fixture_r5_1_local_player_incar_offset(),
            samp_client_sdk_fixture_r5_1_local_player_aim_offset(),
            samp_client_sdk_fixture_r5_1_local_player_trailer_offset(),
            samp_client_sdk_fixture_r5_1_local_player_onfoot_offset(),
            samp_client_sdk_fixture_r5_1_local_player_passenger_offset(),
            samp_client_sdk_fixture_r5_1_local_player_active_offset(),
            samp_client_sdk_fixture_r5_1_local_player_current_vehicle_offset(),
            samp_client_sdk_fixture_r5_1_local_player_ped_offset(),
            samp_client_sdk_fixture_r5_1_local_player_last_any_update_offset(),
        )
    };
    assert_eq!(
        local,
        (
            0x324, 0x00, 0x3F, 0x5E, 0x94, 0xD8, 0xF0, 0xF8, 0x104, 0x13F
        )
    );
}

#[test]
fn r5_spec_layout_fields_match_the_independent_cpp_fixture() {
    unsafe {
        assert_eq!(
            R5_SPEC.net_game.rak_client_offset.map(|value| value.get()),
            Some(samp_client_sdk_fixture_r5_1_netgame_rak_client_offset())
        );
        assert_eq!(
            R5_SPEC.net_game.game_state_offset.get(),
            samp_client_sdk_fixture_r5_1_netgame_game_state_offset()
        );
        assert_eq!(
            R5_SPEC.net_game.pools_offset.get(),
            samp_client_sdk_fixture_r5_1_netgame_pools_offset()
        );
        assert_eq!(
            R5_SPEC.ui.input.command_count_offset.get(),
            samp_client_sdk_fixture_r5_1_input_command_count_offset()
        );
        assert_eq!(
            R5_SPEC.ui.input.enabled_offset.get(),
            samp_client_sdk_fixture_r5_1_input_enabled_offset()
        );
        assert_eq!(
            R5_SPEC.ui.dialog.active_offset.get(),
            samp_client_sdk_fixture_r5_1_dialog_active_offset()
        );
        assert_eq!(
            R5_SPEC.ui.dialog.caption_offset.get(),
            samp_client_sdk_fixture_r5_1_dialog_caption_offset()
        );
        assert_eq!(
            R5_SPEC.net_game.pools.pickup_offset.get(),
            samp_client_sdk_fixture_r5_1_pools_pickup_offset()
        );
        assert_eq!(
            R5_SPEC.net_game.pools.object_offset.get(),
            samp_client_sdk_fixture_r5_1_pools_object_offset()
        );
        assert_eq!(
            R5_SPEC.net_game.pools.gangzone_offset.get(),
            samp_client_sdk_fixture_r5_1_pools_gangzone_offset()
        );
        assert_eq!(
            R5_SPEC.net_game.pools.text_label_offset.get(),
            samp_client_sdk_fixture_r5_1_pools_label_offset()
        );
        assert_eq!(
            R5_SPEC.net_game.pools.textdraw_offset.get(),
            samp_client_sdk_fixture_r5_1_pools_textdraw_offset()
        );
        assert_eq!(
            R5_SPEC.pools.player.local_id_offset.get(),
            samp_client_sdk_fixture_r5_1_player_pool_local_id_offset()
        );
        assert_eq!(
            R5_SPEC.pools.player.objects_offset.map(|value| value.get()),
            Some(samp_client_sdk_fixture_r5_1_player_pool_objects_offset())
        );
        assert_eq!(
            R5_SPEC
                .pools
                .player
                .player_info
                .map(|value| value.npc_offset.get()),
            Some(samp_client_sdk_fixture_r5_1_player_info_is_npc_offset())
        );
    }
}

#[test]
fn dl_pool_and_player_layouts_match_the_independent_cpp_fixture() {
    let pools = unsafe {
        [
            samp_client_sdk_fixture_dl_pools_size(),
            samp_client_sdk_fixture_dl_pools_pickup_offset(),
            samp_client_sdk_fixture_dl_pools_object_offset(),
            samp_client_sdk_fixture_dl_pools_gangzone_offset(),
            samp_client_sdk_fixture_dl_pools_label_offset(),
            samp_client_sdk_fixture_dl_pools_textdraw_offset(),
        ]
    };
    assert_eq!(pools, [0x24, 0x10, 0x14, 0x18, 0x1C, 0x20]);

    let player_pool = unsafe {
        [
            samp_client_sdk_fixture_dl_samp_string_size(),
            samp_client_sdk_fixture_dl_player_info_size(),
            samp_client_sdk_fixture_dl_player_info_is_npc_offset(),
            samp_client_sdk_fixture_dl_player_pool_size(),
            samp_client_sdk_fixture_dl_player_pool_local_id_offset(),
            samp_client_sdk_fixture_dl_player_pool_local_player_offset(),
            samp_client_sdk_fixture_dl_player_pool_largest_id_offset(),
            samp_client_sdk_fixture_dl_player_pool_players_offset(),
            samp_client_sdk_fixture_dl_player_pool_not_empty_offset(),
            samp_client_sdk_fixture_dl_player_pool_collision_offset(),
            samp_client_sdk_fixture_dl_player_pool_ping_offset(),
            samp_client_sdk_fixture_dl_player_pool_score_offset(),
        ]
    };
    assert_eq!(
        player_pool,
        [
            0x1C, 0x2C, 0x04, 0x2F3E, 0x00, 0x1E, 0x22, 0x26, 0xFD6, 0x1F86, 0x2F36, 0x2F3A,
        ]
    );
}

#[test]
fn dl_spec_layout_fields_match_the_independent_cpp_fixture() {
    unsafe {
        assert_eq!(
            DL_SPEC.net_game.rak_client_offset.map(|value| value.get()),
            Some(samp_client_sdk_fixture_dl_netgame_rak_client_offset())
        );
        assert_eq!(
            DL_SPEC.net_game.game_state_offset.get(),
            samp_client_sdk_fixture_dl_netgame_game_state_offset()
        );
        assert_eq!(
            DL_SPEC.net_game.pools_offset.get(),
            samp_client_sdk_fixture_dl_netgame_pools_offset()
        );
        assert_eq!(
            DL_SPEC.net_game.pools.object_offset.get(),
            samp_client_sdk_fixture_dl_pools_object_offset()
        );
        assert_eq!(
            DL_SPEC.pools.object.objects_offset.get(),
            samp_client_sdk_fixture_dl_object_pool_objects_offset()
        );
        assert_eq!(
            DL_SPEC.pools.player.local_id_offset.get(),
            samp_client_sdk_fixture_dl_player_pool_local_id_offset()
        );
        assert_eq!(
            DL_SPEC.pools.player.largest_id_offset.get(),
            samp_client_sdk_fixture_dl_player_pool_largest_id_offset()
        );
        assert_eq!(
            DL_SPEC.players.local.onfoot_offset.get(),
            samp_client_sdk_fixture_dl_local_player_onfoot_offset()
        );
        assert_eq!(
            DL_SPEC.players.local.last_any_update_offset.get(),
            samp_client_sdk_fixture_dl_local_player_last_any_update_offset()
        );
        assert_eq!(
            DL_SPEC.players.remote.onfoot_offset.get(),
            samp_client_sdk_fixture_dl_remote_player_onfoot_offset()
        );
        assert_eq!(
            DL_SPEC.players.remote.aim_offset.get(),
            samp_client_sdk_fixture_dl_remote_player_aim_offset()
        );
    }
}

#[test]
fn dl_local_and_remote_player_layouts_match_the_independent_cpp_fixture() {
    let local = unsafe {
        [
            samp_client_sdk_fixture_dl_local_player_size(),
            samp_client_sdk_fixture_dl_local_player_ped_offset(),
            samp_client_sdk_fixture_dl_local_player_trailer_offset(),
            samp_client_sdk_fixture_dl_local_player_onfoot_offset(),
            samp_client_sdk_fixture_dl_local_player_passenger_offset(),
            samp_client_sdk_fixture_dl_local_player_incar_offset(),
            samp_client_sdk_fixture_dl_local_player_aim_offset(),
            samp_client_sdk_fixture_dl_local_player_active_offset(),
            samp_client_sdk_fixture_dl_local_player_current_vehicle_offset(),
            samp_client_sdk_fixture_dl_local_player_last_any_update_offset(),
        ]
    };
    assert_eq!(
        local,
        [0x328, 0x00, 0x04, 0x3A, 0x7E, 0x96, 0xD5, 0xF4, 0xFC, 0x110]
    );

    let remote = unsafe {
        [
            samp_client_sdk_fixture_dl_remote_player_size(),
            samp_client_sdk_fixture_dl_remote_player_ped_offset(),
            samp_client_sdk_fixture_dl_remote_player_special_action_offset(),
            samp_client_sdk_fixture_dl_remote_player_passenger_offset(),
            samp_client_sdk_fixture_dl_remote_player_onfoot_offset(),
            samp_client_sdk_fixture_dl_remote_player_incar_offset(),
            samp_client_sdk_fixture_dl_remote_player_trailer_offset(),
            samp_client_sdk_fixture_dl_remote_player_aim_offset(),
            samp_client_sdk_fixture_dl_remote_player_armour_offset(),
            samp_client_sdk_fixture_dl_remote_player_health_offset(),
            samp_client_sdk_fixture_dl_remote_player_animation_offset(),
        ]
    };
    assert_eq!(
        remote,
        [
            0x1FD, 0x04, 0x18, 0x24, 0x3C, 0x80, 0xBF, 0xF5, 0x1AC, 0x1B0, 0x1C0,
        ]
    );
}

#[test]
fn dl_entity_pool_layouts_match_the_independent_cpp_fixture() {
    let observed = unsafe {
        [
            samp_client_sdk_fixture_dl_vehicle_pool_size(),
            samp_client_sdk_fixture_dl_vehicle_pool_not_empty_offset(),
            samp_client_sdk_fixture_dl_vehicle_pool_game_objects_offset(),
            samp_client_sdk_fixture_dl_object_pool_size(),
            samp_client_sdk_fixture_dl_object_pool_not_empty_offset(),
            samp_client_sdk_fixture_dl_object_pool_objects_offset(),
            samp_client_sdk_fixture_dl_pickup_pool_size(),
            samp_client_sdk_fixture_dl_pickup_pool_handles_offset(),
            samp_client_sdk_fixture_dl_entity_size(),
            samp_client_sdk_fixture_dl_entity_handle_offset(),
            samp_client_sdk_fixture_dl_ped_size(),
            samp_client_sdk_fixture_dl_ped_game_ped_offset(),
        ]
    };
    assert_eq!(
        observed,
        [
            0x17898, 0x3074, 0x4FB4, 0x41A4, 0x04, 0x20D4, 0x23004, 0x04, 0x48, 0x44, 0x32D, 0x2A4,
        ]
    );
}

#[test]
fn dl_ui_and_world_layouts_match_the_independent_cpp_fixture() {
    let observed = unsafe {
        [
            samp_client_sdk_fixture_dl_gangzone_pool_size(),
            samp_client_sdk_fixture_dl_gangzone_pool_not_empty_offset(),
            samp_client_sdk_fixture_dl_label_pool_size(),
            samp_client_sdk_fixture_dl_label_pool_not_empty_offset(),
            samp_client_sdk_fixture_dl_textdraw_size(),
            samp_client_sdk_fixture_dl_textdraw_data_offset(),
            samp_client_sdk_fixture_dl_textdraw_pool_size(),
            samp_client_sdk_fixture_dl_textdraw_pool_objects_offset(),
            samp_client_sdk_fixture_dl_chat_size(),
            samp_client_sdk_fixture_dl_chat_mode_offset(),
            samp_client_sdk_fixture_dl_chat_entries_offset(),
            samp_client_sdk_fixture_dl_scoreboard_size(),
            samp_client_sdk_fixture_dl_game_cursor_mode_offset(),
            samp_client_sdk_fixture_dl_animation_entry_size(),
        ]
    };
    assert_eq!(
        observed,
        [
            0x2000, 0x1000, 0x10800, 0xE800, 0x9D6, 0x963, 0x4800, 0x2400, 0x63EA, 0x08, 0x132,
            0x44, 0x61, 0x24,
        ]
    );
}

#[test]
fn r3_player_pool_scalar_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_player_pool_size(),
            samp_client_sdk_fixture_r3_1_player_pool_largest_id_offset(),
            samp_client_sdk_fixture_r3_1_player_pool_objects_offset(),
            samp_client_sdk_fixture_r3_1_player_info_size(),
            samp_client_sdk_fixture_r3_1_player_info_is_npc_offset(),
        )
    };
    assert_eq!(observed, (0x2F3E, 0x00, 0x04, 0x2C, 0x28));
}

#[test]
fn r3_remote_player_state_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_remote_player_special_action_offset(),
            samp_client_sdk_fixture_r3_1_remote_player_reported_armour_offset(),
            samp_client_sdk_fixture_r3_1_remote_player_reported_health_offset(),
            samp_client_sdk_fixture_r3_1_remote_player_animation_offset(),
        )
    };
    assert_eq!(observed, (0x18, 0x1AC, 0x1B0, 0x1C0));
}

#[test]
fn r3_chat_input_cache_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_input_editbox_offset(),
            samp_client_sdk_fixture_r3_1_input_command_names_offset(),
            samp_client_sdk_fixture_r3_1_input_command_name_capacity(),
            samp_client_sdk_fixture_r3_1_input_command_count_offset(),
            samp_client_sdk_fixture_r3_1_input_enabled_offset(),
        )
    };
    assert_eq!(observed, (0x08, 0x24C, 33, 0x14DC, 0x14E0));
}

#[test]
fn r3_dialog_active_cache_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_dialog_size(),
            samp_client_sdk_fixture_r3_1_dialog_active_offset(),
            samp_client_sdk_fixture_r3_1_dialog_caption_offset(),
        )
    };
    assert_eq!(observed, (0x29D, 0x28, 0x40));
}

#[test]
fn r3_dialog_snapshot_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_dialog_listbox_offset(),
            samp_client_sdk_fixture_r3_1_dialog_editbox_offset(),
            samp_client_sdk_fixture_r3_1_dialog_type_offset(),
            samp_client_sdk_fixture_r3_1_dialog_id_offset(),
            samp_client_sdk_fixture_r3_1_dialog_text_offset(),
            samp_client_sdk_fixture_r3_1_dialog_server_side_offset(),
            samp_client_sdk_fixture_r3_1_listbox_selected_offset(),
            samp_client_sdk_fixture_r3_1_listbox_items_offset(),
            samp_client_sdk_fixture_r3_1_listbox_item_count_offset(),
        )
    };
    assert_eq!(
        observed,
        (0x20, 0x24, 0x2C, 0x30, 0x34, 0x81, 0x143, 0x14C, 0x150)
    );
}

#[test]
fn r3_scoreboard_cache_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_scoreboard_size(),
            samp_client_sdk_fixture_r3_1_scoreboard_enabled_offset(),
        )
    };
    assert_eq!(observed, (0x44, 0x00));
}

#[test]
fn r3_cursor_mode_cache_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_game_size(),
            samp_client_sdk_fixture_r3_1_game_cursor_mode_offset(),
        )
    };
    assert_eq!(observed, (0x142, 0x61));
}

#[test]
fn r3_text_label_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_pools_label_offset(),
            samp_client_sdk_fixture_r3_1_text_label_size(),
            samp_client_sdk_fixture_r3_1_label_pool_not_empty_offset(),
        )
    };
    assert_eq!(observed, (0x1C, 0x1D, 0xE800));
}

#[test]
fn r3_vehicle_pool_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_vehicle_pool_not_empty_offset(),
            samp_client_sdk_fixture_r3_1_vehicle_pool_game_objects_offset(),
        )
    };
    assert_eq!(observed, (0x3074, 0x4FB4));
}

#[test]
fn r3_object_pool_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_pools_object_offset(),
            samp_client_sdk_fixture_r3_1_object_pool_not_empty_offset(),
        )
    };
    assert_eq!(observed, (0x14, 0x04));
    assert_eq!(
        unsafe { samp_client_sdk_fixture_r3_1_object_pool_objects_offset() },
        0xFA4
    );
}

#[test]
fn r3_handle_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_pools_pickup_offset(),
            samp_client_sdk_fixture_r3_1_pickup_pool_handles_offset(),
            samp_client_sdk_fixture_r3_1_entity_handle_offset(),
        )
    };
    assert_eq!(observed, (0x10, 0x04, 0x44));
}

#[test]
fn r3_gangzone_pool_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_pools_gangzone_offset(),
            samp_client_sdk_fixture_r3_1_gangzone_size(),
            samp_client_sdk_fixture_r3_1_gangzone_pool_not_empty_offset(),
        )
    };
    assert_eq!(observed, (0x18, 0x18, 0x1000));
}

#[test]
fn r3_textdraw_pool_layout_matches_the_independent_cpp_fixture() {
    let observed = unsafe {
        (
            samp_client_sdk_fixture_r3_1_pools_textdraw_offset(),
            samp_client_sdk_fixture_r3_1_textdraw_size(),
            samp_client_sdk_fixture_r3_1_textdraw_pool_objects_offset(),
        )
    };
    assert_eq!(observed, (0x20, 0x9D6, 0x2400));
}

#[test]
fn r3_animation_table_entry_layout_matches_the_independent_cpp_fixture() {
    assert_eq!(
        unsafe { samp_client_sdk_fixture_r3_1_animation_entry_size() },
        0x24
    );
}

#[test]
fn non_r1_profile_layout_gates_match_the_independent_cpp_fixture() {
    let fixtures = [
        (
            ProfileLayoutFixture {
                version: SampVersion::R3_1,
                netgame_size: samp_client_sdk_fixture_r3_1_netgame_size,
                netgame_rak_client_offset: samp_client_sdk_fixture_r3_1_netgame_rak_client_offset,
                netgame_game_state_offset: samp_client_sdk_fixture_r3_1_netgame_game_state_offset,
                netgame_pools_offset: samp_client_sdk_fixture_r3_1_netgame_pools_offset,
                input_size: samp_client_sdk_fixture_r3_1_input_size,
                input_command_count_offset: samp_client_sdk_fixture_r3_1_input_command_count_offset,
                input_enabled_offset: samp_client_sdk_fixture_r3_1_input_enabled_offset,
                dialog_size: samp_client_sdk_fixture_r3_1_dialog_size,
                dialog_active_offset: samp_client_sdk_fixture_r3_1_dialog_active_offset,
                dialog_caption_offset: samp_client_sdk_fixture_r3_1_dialog_caption_offset,
            },
            R3_1_LAYOUT,
        ),
        (
            ProfileLayoutFixture {
                version: SampVersion::R5_1,
                netgame_size: samp_client_sdk_fixture_r5_1_netgame_size,
                netgame_rak_client_offset: samp_client_sdk_fixture_r5_1_netgame_rak_client_offset,
                netgame_game_state_offset: samp_client_sdk_fixture_r5_1_netgame_game_state_offset,
                netgame_pools_offset: samp_client_sdk_fixture_r5_1_netgame_pools_offset,
                input_size: samp_client_sdk_fixture_r5_1_input_size,
                input_command_count_offset: samp_client_sdk_fixture_r5_1_input_command_count_offset,
                input_enabled_offset: samp_client_sdk_fixture_r5_1_input_enabled_offset,
                dialog_size: samp_client_sdk_fixture_r5_1_dialog_size,
                dialog_active_offset: samp_client_sdk_fixture_r5_1_dialog_active_offset,
                dialog_caption_offset: samp_client_sdk_fixture_r5_1_dialog_caption_offset,
            },
            R5_1_LAYOUT,
        ),
        (
            ProfileLayoutFixture {
                version: SampVersion::Dl,
                netgame_size: samp_client_sdk_fixture_dl_netgame_size,
                netgame_rak_client_offset: samp_client_sdk_fixture_dl_netgame_rak_client_offset,
                netgame_game_state_offset: samp_client_sdk_fixture_dl_netgame_game_state_offset,
                netgame_pools_offset: samp_client_sdk_fixture_dl_netgame_pools_offset,
                input_size: samp_client_sdk_fixture_dl_input_size,
                input_command_count_offset: samp_client_sdk_fixture_dl_input_command_count_offset,
                input_enabled_offset: samp_client_sdk_fixture_dl_input_enabled_offset,
                dialog_size: samp_client_sdk_fixture_dl_dialog_size,
                dialog_active_offset: samp_client_sdk_fixture_dl_dialog_active_offset,
                dialog_caption_offset: samp_client_sdk_fixture_dl_dialog_caption_offset,
            },
            DL_LAYOUT,
        ),
    ];

    for (fixture, expected) in fixtures {
        let actual = unsafe { fixture.observed() };
        assert_eq!(actual, expected, "{:#?} layout fixture", fixture.version);
    }
}

#[test]
fn r1_profile_layout_matches_the_independent_cpp_fixture() {
    let spec = R1_SPEC;
    unsafe {
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_size(),
            spec.sync.onfoot.size.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_incar_size(),
            spec.sync.incar.size.get()
        );

        assert_eq!(
            samp_client_sdk_fixture_r1_local_active_offset(),
            spec.players.local.active_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_current_vehicle_offset(),
            spec.players.local.current_vehicle_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_onfoot_offset(),
            spec.players.local.onfoot_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_incar_offset(),
            spec.players.local.incar_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_passenger_offset(),
            spec.players.local.passenger_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_trailer_offset(),
            spec.players.local.trailer_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_position_offset(),
            spec.players.local.onfoot.position_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_speed_offset(),
            spec.players.local.onfoot.speed_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_special_action_offset(),
            spec.players.local.onfoot.special_action_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_animation_offset(),
            spec.players.local.onfoot.animation_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_incar_position_offset(),
            spec.players.local.incar.position_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_incar_speed_offset(),
            spec.players.local.incar.speed_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_ped_game_ped_offset(),
            spec.players.local.game_ped_offset.get()
        );

        assert_eq!(
            samp_client_sdk_fixture_r1_player_pool_local_id_offset(),
            spec.pools.player.local_id_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_player_pool_largest_id_offset(),
            spec.pools.player.largest_id_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_vehicle_pool_not_empty_offset(),
            spec.pools.vehicle.not_empty_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_vehicle_pool_game_objects_offset(),
            spec.pools.vehicle.game_objects_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_object_pool_not_empty_offset(),
            spec.pools.object.not_empty_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_object_pool_objects_offset(),
            spec.pools.object.objects_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_pickup_pool_handles_offset(),
            spec.pools.pickup.handles_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_entity_handle_offset(),
            spec.pools.entity_handle_offset.get()
        );

        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_host_address_offset(),
            spec.net_game.host_address_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_hostname_offset(),
            spec.net_game.hostname_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_port_offset(),
            spec.net_game.port_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_game_state_offset(),
            spec.net_game.game_state_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_server_settings_offset(),
            spec.net_game.server_settings_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_offset(),
            spec.net_game.pools_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_label_offset(),
            spec.net_game.pools.text_label_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_text_draw_offset(),
            spec.net_game.pools.textdraw_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_object_offset(),
            spec.net_game.pools.object_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_gang_zone_offset(),
            spec.net_game.pools.gangzone_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_pickup_offset(),
            spec.net_game.pools.pickup_offset.get()
        );

        assert_eq!(
            samp_client_sdk_fixture_r1_label_pool_not_empty_offset(),
            spec.pools.text_label.not_empty_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_size(),
            spec.text_labels.size.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_text_offset(),
            spec.text_labels.text_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_colour_offset(),
            spec.text_labels.colour_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_position_offset(),
            spec.text_labels.position_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_draw_distance_offset(),
            spec.text_labels.draw_distance_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_behind_walls_offset(),
            spec.text_labels.behind_walls_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_attached_player_offset(),
            spec.text_labels.attached_player_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_attached_vehicle_offset(),
            spec.text_labels.attached_vehicle_offset.get()
        );

        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_pool_not_empty_offset(),
            spec.pools.textdraw.not_empty_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_pool_objects_offset(),
            spec.pools.textdraw.objects_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_data_offset(),
            spec.textdraws.data_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_size(),
            spec.textdraws.native_size.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_transmit_size(),
            spec.textdraws.transmit.size.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_transmit_x_offset(),
            spec.textdraws.transmit.x.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_transmit_y_offset(),
            spec.textdraws.transmit.y.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_letter_width_offset(),
            spec.textdraws.data.width.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_model_id_offset(),
            spec.textdraws.data.model_id.get()
        );
        assert_eq!(samp_client_sdk_fixture_r1_gangzone_size(), 0x18);
        assert_eq!(
            samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset(),
            spec.pools.gangzone.not_empty_offset.get()
        );

        assert_eq!(
            samp_client_sdk_fixture_r1_game_cursor_mode_offset(),
            spec.ui.game.cursor_mode_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_scoreboard_enabled_offset(),
            spec.ui.scoreboard.enabled_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_active_offset(),
            spec.ui.dialog.active_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_listbox_offset(),
            spec.ui.dialog.listbox_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_editbox_offset(),
            spec.ui.dialog.editbox_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_text_offset(),
            spec.ui.dialog.text_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_type_offset(),
            spec.ui.dialog.dialog_type_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_id_offset(),
            spec.ui.dialog.id_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_caption_offset(),
            spec.ui.dialog.caption_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_server_side_offset(),
            spec.ui.dialog.server_side_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_input_enabled_offset(),
            spec.ui.input.enabled_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_chat_entries_offset(),
            spec.ui.chat.entries_offset.get()
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_chat_entry_size(),
            spec.ui.chat.entry_size.get()
        );
    }
}

#[cfg(gta_sa_layout_oracle)]
unsafe extern "C" {
    fn gta_sa_fixture_vector2_size() -> usize;
    fn gta_sa_fixture_vector3_size() -> usize;
    fn gta_sa_fixture_matrix_size() -> usize;
    fn gta_sa_fixture_matrix_right_offset() -> usize;
    fn gta_sa_fixture_matrix_forward_offset() -> usize;
    fn gta_sa_fixture_matrix_up_offset() -> usize;
    fn gta_sa_fixture_matrix_position_offset() -> usize;
    fn gta_sa_fixture_matrix_attached_offset() -> usize;
    fn gta_sa_fixture_matrix_owns_attached_offset() -> usize;
    fn gta_sa_fixture_placeable_size() -> usize;
    fn gta_sa_fixture_placeable_position_offset() -> usize;
    fn gta_sa_fixture_placeable_matrix_offset() -> usize;
    fn gta_sa_fixture_entity_size() -> usize;
    fn gta_sa_fixture_ped_size() -> usize;
    fn gta_sa_fixture_ped_health_offset() -> usize;
    fn gta_sa_fixture_ped_armour_offset() -> usize;
    fn gta_sa_fixture_vehicle_size() -> usize;
    fn gta_sa_fixture_vehicle_health_offset() -> usize;
    fn gta_sa_fixture_object_size() -> usize;
    fn gta_sa_fixture_pool_size() -> usize;
    fn gta_sa_fixture_pool_objects_offset() -> usize;
    fn gta_sa_fixture_pool_flags_offset() -> usize;
    fn gta_sa_fixture_pool_capacity_offset() -> usize;
    fn gta_sa_fixture_invoke_teleport(
        target: *const (),
        object: *mut core::ffi::c_void,
        x: f32,
        y: f32,
        z: f32,
        reset_rotation: u8,
    );
}

#[cfg(gta_sa_layout_oracle)]
#[test]
fn gta_sa_profile_layout_matches_the_pinned_plugin_sdk_oracle() {
    let profile =
        gta_sa_native::GtaProfile::select(0x0040_0000, gta_sa_native::GTA_SA_10_US_SHA256).unwrap();
    unsafe {
        assert_eq!(gta_sa_fixture_vector2_size(), 0x08);
        assert_eq!(gta_sa_fixture_vector3_size(), 0x0C);
        assert_eq!(
            gta_sa_fixture_matrix_size(),
            core::mem::size_of::<gta_sa_native::RawMatrix>()
        );
        assert_eq!(gta_sa_fixture_matrix_right_offset(), 0x00);
        assert_eq!(gta_sa_fixture_matrix_forward_offset(), 0x10);
        assert_eq!(gta_sa_fixture_matrix_up_offset(), 0x20);
        assert_eq!(gta_sa_fixture_matrix_position_offset(), 0x30);
        assert_eq!(gta_sa_fixture_matrix_attached_offset(), 0x40);
        assert_eq!(gta_sa_fixture_matrix_owns_attached_offset(), 0x44);
        assert_eq!(gta_sa_fixture_placeable_size(), 0x18);
        assert_eq!(
            gta_sa_fixture_placeable_position_offset(),
            profile.spec.entity.placeable_position.get()
        );
        assert_eq!(
            gta_sa_fixture_placeable_matrix_offset(),
            profile.spec.entity.matrix_pointer.get()
        );
        assert_eq!(gta_sa_fixture_entity_size(), profile.spec.entity.size.get());
        assert_eq!(gta_sa_fixture_ped_size(), profile.spec.ped.size.get());
        assert_eq!(
            gta_sa_fixture_ped_health_offset(),
            profile.spec.ped.health.get()
        );
        assert_eq!(
            gta_sa_fixture_ped_armour_offset(),
            profile.spec.ped.armour.get()
        );
        assert_eq!(
            gta_sa_fixture_vehicle_size(),
            profile.spec.vehicle.size.get()
        );
        assert_eq!(
            gta_sa_fixture_vehicle_health_offset(),
            profile.spec.vehicle.health.get()
        );
        assert_eq!(gta_sa_fixture_object_size(), profile.spec.object.size.get());
        assert_eq!(
            gta_sa_fixture_pool_size(),
            profile.spec.pool_layout.size.get()
        );
        assert_eq!(
            gta_sa_fixture_pool_objects_offset(),
            profile.spec.pool_layout.objects.get()
        );
        assert_eq!(
            gta_sa_fixture_pool_flags_offset(),
            profile.spec.pool_layout.flags.get()
        );
        assert_eq!(
            gta_sa_fixture_pool_capacity_offset(),
            profile.spec.pool_layout.capacity.get()
        );
    }
}

#[cfg(gta_sa_layout_oracle)]
#[repr(C)]
#[derive(Default)]
struct TeleportAbiCapture {
    destination: gta_sa_native::RawVector3,
    reset_rotation: u8,
}

#[cfg(gta_sa_layout_oracle)]
unsafe extern "thiscall" fn capture_teleport_abi(
    object: *mut core::ffi::c_void,
    destination: gta_sa_native::RawVector3,
    reset_rotation: u8,
) {
    let capture = unsafe { &mut *object.cast::<TeleportAbiCapture>() };
    capture.destination = destination;
    capture.reset_rotation = reset_rotation;
}

#[cfg(gta_sa_layout_oracle)]
#[test]
fn gta_sa_teleport_thiscall_matches_the_cpp_oracle() {
    let mut capture = TeleportAbiCapture::default();
    unsafe {
        gta_sa_fixture_invoke_teleport(
            capture_teleport_abi as *const (),
            (&mut capture as *mut TeleportAbiCapture).cast(),
            12.5,
            -30.0,
            7.25,
            1,
        );
    }
    assert_eq!(
        capture.destination,
        gta_sa_native::RawVector3 {
            x: 12.5,
            y: -30.0,
            z: 7.25,
        }
    );
    assert_eq!(capture.reset_rotation, 1);
}
