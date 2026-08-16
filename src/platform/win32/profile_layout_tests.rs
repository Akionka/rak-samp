//! Independent native-client profile layout fixtures.
//!
//! These tests cover the native structures consumed by the enabled R3-1,
//! R5-1, and DL-R1 direct-helper profiles.

use crate::client::SampVersion;
use crate::platform::win32::native_client::profiles::dl::DL_SPEC;
use crate::platform::win32::native_client::profiles::r3::R3_SPEC;
use crate::platform::win32::native_client::profiles::r5::R5_SPEC;

type FixtureFn = unsafe extern "C" fn() -> usize;

unsafe extern "C" {
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
