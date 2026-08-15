//! Independent minimal layout gates for unenabled native client profiles.
//!
//! The runtime keeps direct helpers R1-only apart from the narrow, read-only
//! R3 CNetGame scalar cache. These tests record the first three structures any
//! future R3-1, R5-1, or DL profile must prove before a broader gate is relaxed.

use crate::client::SampVersion;

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
