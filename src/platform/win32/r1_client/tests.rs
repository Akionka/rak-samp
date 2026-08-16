use super::addresses;
use super::{
    CHAT_ENTRIES_OFFSET, CHAT_ENTRY_SIZE, DIALOG_ACTIVE_OFFSET, DIALOG_CAPTION_OFFSET,
    DIALOG_EDITBOX_OFFSET, DIALOG_ID_OFFSET, DIALOG_LISTBOX_OFFSET, DIALOG_SERVER_SIDE_OFFSET,
    DIALOG_TEXT_OFFSET, DIALOG_TYPE_OFFSET, DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET,
    DXUT_LISTBOX_ITEM_COUNT_OFFSET, DXUT_LISTBOX_ITEM_DATA_OFFSET, DXUT_LISTBOX_ITEM_SIZE,
    DXUT_LISTBOX_ITEM_TEXT_CAPACITY, DXUT_LISTBOX_ITEM_TEXT_OFFSET,
    DXUT_LISTBOX_ITEM_VISIBLE_OFFSET, DXUT_LISTBOX_ITEMS_OFFSET, DXUT_LISTBOX_SELECTED_OFFSET,
    ENTITY_HANDLE_OFFSET, GAME_CURSOR_MODE_OFFSET, GANGZONE_POOL_NOT_EMPTY_OFFSET,
    INPUT_ENABLED_OFFSET, LABEL_ATTACHED_PLAYER_OFFSET, LABEL_ATTACHED_VEHICLE_OFFSET,
    LABEL_BEHIND_WALLS_OFFSET, LABEL_COLOUR_OFFSET, LABEL_DRAW_DISTANCE_OFFSET,
    LABEL_POOL_NOT_EMPTY_OFFSET, LABEL_POSITION_OFFSET, LABEL_SIZE, LABEL_TEXT_OFFSET,
    LOCAL_PLAYER_ACTIVE_OFFSET, LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET, LOCAL_PLAYER_INCAR_OFFSET,
    LOCAL_PLAYER_INCAR_POSITION_OFFSET, LOCAL_PLAYER_INCAR_SPEED_OFFSET,
    LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET, LOCAL_PLAYER_ONFOOT_OFFSET,
    LOCAL_PLAYER_ONFOOT_POSITION_OFFSET, LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET,
    LOCAL_PLAYER_ONFOOT_SPEED_OFFSET, LOCAL_PLAYER_PASSENGER_OFFSET, LOCAL_PLAYER_TRAILER_OFFSET,
    MAX_TEXT_LABEL_TEXT_BYTES, NET_GAME_GAME_STATE_OFFSET, NET_GAME_HOST_ADDRESS_OFFSET,
    NET_GAME_HOSTNAME_OFFSET, NET_GAME_POOLS_GANGZONE_POOL_OFFSET,
    NET_GAME_POOLS_LABEL_POOL_OFFSET, NET_GAME_POOLS_OBJECT_POOL_OFFSET, NET_GAME_POOLS_OFFSET,
    NET_GAME_POOLS_PICKUP_POOL_OFFSET, NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET, NET_GAME_PORT_OFFSET,
    NET_GAME_SERVER_SETTINGS_OFFSET, NativeDxutComboBoxItem, OBJECT_POOL_NOT_EMPTY_OFFSET,
    OBJECT_POOL_OBJECTS_OFFSET, PICKUP_POOL_HANDLES_OFFSET, PLAYER_POOL_LARGEST_ID_OFFSET,
    PLAYER_POOL_LOCAL_ID_OFFSET, REMOTE_PLAYER_INCAR_OFFSET, REMOTE_PLAYER_ONFOOT_OFFSET,
    REMOTE_PLAYER_PASSENGER_OFFSET, REMOTE_PLAYER_TRAILER_OFFSET, SAMP_PED_GAME_PED_OFFSET,
    SCOREBOARD_ENABLED_OFFSET, TEXTDRAW_ALIGN_CENTER_OFFSET, TEXTDRAW_ALIGN_LEFT_OFFSET,
    TEXTDRAW_ALIGN_RIGHT_OFFSET, TEXTDRAW_BACKGROUND_COLOUR_OFFSET, TEXTDRAW_BOX_COLOUR_OFFSET,
    TEXTDRAW_BOX_ENABLED_OFFSET, TEXTDRAW_BOX_HEIGHT_OFFSET, TEXTDRAW_BOX_WIDTH_OFFSET,
    TEXTDRAW_DATA_OFFSET, TEXTDRAW_LETTER_COLOUR_OFFSET, TEXTDRAW_LETTER_HEIGHT_OFFSET,
    TEXTDRAW_LETTER_WIDTH_OFFSET, TEXTDRAW_MODEL_COLOUR1_OFFSET, TEXTDRAW_MODEL_COLOUR2_OFFSET,
    TEXTDRAW_MODEL_ID_OFFSET, TEXTDRAW_NATIVE_SIZE, TEXTDRAW_OUTLINE_OFFSET,
    TEXTDRAW_POOL_NOT_EMPTY_OFFSET, TEXTDRAW_POOL_OBJECTS_OFFSET, TEXTDRAW_PROPORTIONAL_OFFSET,
    TEXTDRAW_ROTATION_OFFSET, TEXTDRAW_SET_TEXT_RVA, TEXTDRAW_SHADOW_OFFSET, TEXTDRAW_STYLE_OFFSET,
    TEXTDRAW_TRANSMIT_SIZE, TEXTDRAW_TRANSMIT_X_OFFSET, TEXTDRAW_TRANSMIT_Y_OFFSET,
    TEXTDRAW_X_OFFSET, TEXTDRAW_Y_OFFSET, TEXTDRAW_ZOOM_OFFSET, VEHICLE_POOL_GAME_OBJECTS_OFFSET,
    VEHICLE_POOL_NOT_EMPTY_OFFSET, assigned_player_id, bounded_c_string,
    bounded_dxut_listbox_item_text, mem, nul_terminated,
};
use crate::platform::win32::native_client::profile::{
    ForceSyncReset, GameStateCodec, ListItemTextLayout, LocalPlayerSource, NativeBoolean,
    TextdrawCallStrategy,
};
use crate::platform::win32::native_client::profiles::r1::R1_SPEC;

unsafe extern "C" {
    fn samp_client_sdk_fixture_r1_onfoot_size() -> usize;
    fn samp_client_sdk_fixture_r1_incar_size() -> usize;
    fn samp_client_sdk_fixture_r1_local_player_prefix_size() -> usize;
    fn samp_client_sdk_fixture_r1_local_active_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_current_vehicle_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_r1_remote_onfoot_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_incar_offset() -> usize;
    fn samp_client_sdk_fixture_r1_remote_incar_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_passenger_offset() -> usize;
    fn samp_client_sdk_fixture_r1_remote_passenger_offset() -> usize;
    fn samp_client_sdk_fixture_r1_local_trailer_offset() -> usize;
    fn samp_client_sdk_fixture_r1_remote_trailer_offset() -> usize;
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
    fn samp_client_sdk_fixture_r1_textdraw_letter_height_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_letter_colour_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_align_center_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_box_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_box_width_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_box_height_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_box_colour_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_proportional_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_background_colour_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_shadow_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_outline_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_align_left_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_align_right_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_style_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_x_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_y_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_model_id_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_rotation_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_zoom_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_model_colour1_offset() -> usize;
    fn samp_client_sdk_fixture_r1_textdraw_model_colour2_offset() -> usize;
    fn samp_client_sdk_fixture_r1_object_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset() -> usize;
    fn samp_client_sdk_fixture_r1_gangzone_size() -> usize;
    fn samp_client_sdk_fixture_r1_game_cursor_mode_offset() -> usize;
    fn samp_client_sdk_fixture_r1_scoreboard_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_active_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_listbox_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_editbox_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_text_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_listbox_selected_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_listbox_items_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_listbox_item_count_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_combobox_item_text_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_combobox_item_text_capacity() -> usize;
    fn samp_client_sdk_fixture_dxut_combobox_item_data_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_combobox_item_active_rect_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_combobox_item_visible_offset() -> usize;
    fn samp_client_sdk_fixture_dxut_combobox_item_size() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_type_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_id_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_caption_offset() -> usize;
    fn samp_client_sdk_fixture_r1_dialog_server_side_offset() -> usize;
    fn samp_client_sdk_fixture_r1_input_enabled_offset() -> usize;
    fn samp_client_sdk_fixture_r1_chat_entries_offset() -> usize;
    fn samp_client_sdk_fixture_r1_chat_entry_size() -> usize;
}

#[test]
fn r1_sync_offsets_match_the_independent_x86_fixture() {
    unsafe {
        assert_eq!(samp_client_sdk_fixture_r1_onfoot_size(), 68);
        assert_eq!(samp_client_sdk_fixture_r1_incar_size(), 63);
        assert_eq!(samp_client_sdk_fixture_r1_local_player_prefix_size(), 92);
        assert_eq!(
            samp_client_sdk_fixture_r1_local_active_offset(),
            LOCAL_PLAYER_ACTIVE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_current_vehicle_offset(),
            LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_onfoot_offset(),
            LOCAL_PLAYER_ONFOOT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_remote_onfoot_offset(),
            REMOTE_PLAYER_ONFOOT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_incar_offset(),
            LOCAL_PLAYER_INCAR_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_remote_incar_offset(),
            REMOTE_PLAYER_INCAR_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_passenger_offset(),
            LOCAL_PLAYER_PASSENGER_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_remote_passenger_offset(),
            REMOTE_PLAYER_PASSENGER_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_local_trailer_offset(),
            LOCAL_PLAYER_TRAILER_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_remote_trailer_offset(),
            REMOTE_PLAYER_TRAILER_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_position_offset(),
            LOCAL_PLAYER_ONFOOT_POSITION_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_speed_offset(),
            LOCAL_PLAYER_ONFOOT_SPEED_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_special_action_offset(),
            LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_onfoot_animation_offset(),
            LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET
        );
        assert_eq!(
            LOCAL_PLAYER_ONFOOT_OFFSET + 68 + 24 + 54,
            LOCAL_PLAYER_INCAR_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_incar_position_offset(),
            LOCAL_PLAYER_INCAR_POSITION_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_incar_speed_offset(),
            LOCAL_PLAYER_INCAR_SPEED_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_ped_game_ped_offset(),
            SAMP_PED_GAME_PED_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_player_pool_local_id_offset(),
            PLAYER_POOL_LOCAL_ID_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_player_pool_largest_id_offset(),
            PLAYER_POOL_LARGEST_ID_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_vehicle_pool_not_empty_offset(),
            VEHICLE_POOL_NOT_EMPTY_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_vehicle_pool_game_objects_offset(),
            VEHICLE_POOL_GAME_OBJECTS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_object_pool_objects_offset(),
            OBJECT_POOL_OBJECTS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_pickup_pool_handles_offset(),
            PICKUP_POOL_HANDLES_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_entity_handle_offset(),
            ENTITY_HANDLE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_host_address_offset(),
            NET_GAME_HOST_ADDRESS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_hostname_offset(),
            NET_GAME_HOSTNAME_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_port_offset(),
            NET_GAME_PORT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_game_state_offset(),
            NET_GAME_GAME_STATE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_server_settings_offset(),
            NET_GAME_SERVER_SETTINGS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_offset(),
            NET_GAME_POOLS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_label_offset(),
            NET_GAME_POOLS_LABEL_POOL_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_text_draw_offset(),
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_object_offset(),
            NET_GAME_POOLS_OBJECT_POOL_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_gang_zone_offset(),
            NET_GAME_POOLS_GANGZONE_POOL_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_net_game_pools_pickup_offset(),
            NET_GAME_POOLS_PICKUP_POOL_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_label_pool_not_empty_offset(),
            LABEL_POOL_NOT_EMPTY_OFFSET
        );
        assert_eq!(samp_client_sdk_fixture_r1_text_label_size(), LABEL_SIZE);
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_text_offset(),
            LABEL_TEXT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_colour_offset(),
            LABEL_COLOUR_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_position_offset(),
            LABEL_POSITION_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_draw_distance_offset(),
            LABEL_DRAW_DISTANCE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_behind_walls_offset(),
            LABEL_BEHIND_WALLS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_attached_player_offset(),
            LABEL_ATTACHED_PLAYER_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_text_label_attached_vehicle_offset(),
            LABEL_ATTACHED_VEHICLE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_pool_not_empty_offset(),
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_transmit_size(),
            TEXTDRAW_TRANSMIT_SIZE
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_transmit_x_offset(),
            TEXTDRAW_TRANSMIT_X_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_textdraw_transmit_y_offset(),
            TEXTDRAW_TRANSMIT_Y_OFFSET
        );
        let textdraw_offsets = [
            (
                samp_client_sdk_fixture_r1_textdraw_pool_objects_offset(),
                TEXTDRAW_POOL_OBJECTS_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_data_offset(),
                TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_size(),
                TEXTDRAW_NATIVE_SIZE,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_letter_width_offset(),
                TEXTDRAW_LETTER_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_letter_height_offset(),
                TEXTDRAW_LETTER_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_letter_colour_offset(),
                TEXTDRAW_LETTER_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_align_center_offset(),
                TEXTDRAW_ALIGN_CENTER_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_box_enabled_offset(),
                TEXTDRAW_BOX_ENABLED_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_box_width_offset(),
                TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_box_height_offset(),
                TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_box_colour_offset(),
                TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_proportional_offset(),
                TEXTDRAW_PROPORTIONAL_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_background_colour_offset(),
                TEXTDRAW_BACKGROUND_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_shadow_offset(),
                TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_outline_offset(),
                TEXTDRAW_OUTLINE_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_align_left_offset(),
                TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_align_right_offset(),
                TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_style_offset(),
                TEXTDRAW_STYLE_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_x_offset(),
                TEXTDRAW_X_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_y_offset(),
                TEXTDRAW_Y_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_model_id_offset(),
                TEXTDRAW_MODEL_ID_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_rotation_offset(),
                TEXTDRAW_ROTATION_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_zoom_offset(),
                TEXTDRAW_ZOOM_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_model_colour1_offset(),
                TEXTDRAW_MODEL_COLOUR1_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
            (
                samp_client_sdk_fixture_r1_textdraw_model_colour2_offset(),
                TEXTDRAW_MODEL_COLOUR2_OFFSET - TEXTDRAW_DATA_OFFSET,
            ),
        ];
        for (actual, expected) in textdraw_offsets {
            assert_eq!(actual, expected);
        }
        assert_eq!(
            samp_client_sdk_fixture_r1_object_pool_not_empty_offset(),
            OBJECT_POOL_NOT_EMPTY_OFFSET
        );
        assert_eq!(samp_client_sdk_fixture_r1_gangzone_size(), 0x18);
        assert_eq!(
            samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset(),
            GANGZONE_POOL_NOT_EMPTY_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_game_cursor_mode_offset(),
            GAME_CURSOR_MODE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_scoreboard_enabled_offset(),
            SCOREBOARD_ENABLED_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_active_offset(),
            DIALOG_ACTIVE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_listbox_offset(),
            DIALOG_LISTBOX_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_editbox_offset(),
            DIALOG_EDITBOX_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_text_offset(),
            DIALOG_TEXT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_listbox_selected_offset(),
            DXUT_LISTBOX_SELECTED_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_listbox_items_offset(),
            DXUT_LISTBOX_ITEMS_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_listbox_item_count_offset(),
            DXUT_LISTBOX_ITEM_COUNT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_combobox_item_text_offset(),
            DXUT_LISTBOX_ITEM_TEXT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_combobox_item_text_capacity(),
            DXUT_LISTBOX_ITEM_TEXT_CAPACITY
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_combobox_item_data_offset(),
            DXUT_LISTBOX_ITEM_DATA_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_combobox_item_active_rect_offset(),
            DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_combobox_item_visible_offset(),
            DXUT_LISTBOX_ITEM_VISIBLE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_dxut_combobox_item_size(),
            DXUT_LISTBOX_ITEM_SIZE
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_type_offset(),
            DIALOG_TYPE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_id_offset(),
            DIALOG_ID_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_caption_offset(),
            DIALOG_CAPTION_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_dialog_server_side_offset(),
            DIALOG_SERVER_SIDE_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_input_enabled_offset(),
            INPUT_ENABLED_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_chat_entries_offset(),
            CHAT_ENTRIES_OFFSET
        );
        assert_eq!(
            samp_client_sdk_fixture_r1_chat_entry_size(),
            CHAT_ENTRY_SIZE
        );
    }
}

#[test]
fn native_dialog_strings_are_terminated_only_after_copying() {
    assert_eq!(nul_terminated(b"dialog".to_vec()), b"dialog\0");
}

#[test]
fn r1_textdraw_setter_rva_is_pinned() {
    assert_eq!(TEXTDRAW_SET_TEXT_RVA, 0xAC870);
}

#[test]
fn bounded_label_copy_accepts_the_full_r1_text_limit() {
    let mut text = vec![b'x'; MAX_TEXT_LABEL_TEXT_BYTES];
    text.push(0);
    assert_eq!(
        unsafe { bounded_c_string(text.as_ptr(), MAX_TEXT_LABEL_TEXT_BYTES + 1) },
        Some(vec![b'x'; MAX_TEXT_LABEL_TEXT_BYTES])
    );
    assert_eq!(
        unsafe {
            bounded_c_string(
                text[..MAX_TEXT_LABEL_TEXT_BYTES].as_ptr(),
                MAX_TEXT_LABEL_TEXT_BYTES,
            )
        },
        None
    );
}

#[test]
fn native_dxut_combobox_item_mirror_matches_the_fixture_layout() {
    assert_eq!(
        mem::offset_of!(NativeDxutComboBoxItem, str_text),
        DXUT_LISTBOX_ITEM_TEXT_OFFSET
    );
    assert_eq!(
        mem::offset_of!(NativeDxutComboBoxItem, data),
        DXUT_LISTBOX_ITEM_DATA_OFFSET
    );
    assert_eq!(
        mem::offset_of!(NativeDxutComboBoxItem, active_rect),
        DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET
    );
    assert_eq!(
        mem::offset_of!(NativeDxutComboBoxItem, visible),
        DXUT_LISTBOX_ITEM_VISIBLE_OFFSET
    );
    assert_eq!(
        mem::size_of::<NativeDxutComboBoxItem>(),
        DXUT_LISTBOX_ITEM_SIZE
    );
    assert_eq!(mem::size_of::<windows_sys::Win32::Foundation::RECT>(), 16);
    assert_eq!(mem::align_of::<NativeDxutComboBoxItem>(), 4);
}

#[test]
fn listbox_item_text_read_stays_inside_the_native_text_field() {
    let mut item = NativeDxutComboBoxItem {
        str_text: [b'x'; DXUT_LISTBOX_ITEM_TEXT_CAPACITY],
        data: std::ptr::null_mut(),
        active_rect: windows_sys::Win32::Foundation::RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        visible: false,
    };

    assert_eq!(
        unsafe { bounded_dxut_listbox_item_text(item.str_text.as_ptr()) },
        None
    );
    item.str_text[DXUT_LISTBOX_ITEM_TEXT_CAPACITY - 1] = 0;
    assert_eq!(
        unsafe { bounded_dxut_listbox_item_text(item.str_text.as_ptr()) },
        Some(vec![b'x'; DXUT_LISTBOX_ITEM_TEXT_CAPACITY - 1])
    );
}

#[test]
fn unassigned_local_player_id_is_not_a_snapshot() {
    assert_eq!(assigned_player_id(u16::MAX), None);
    assert_eq!(assigned_player_id(42), Some(42));
}

#[test]
fn r1_spec_copies_the_existing_rvas_layouts_limits_and_strategies() {
    assert_eq!(R1_SPEC.identity.entry_point, addresses::SAMP_R1_ENTRY_POINT);
    assert_eq!(
        R1_SPEC.net_game.singleton_rva.get(),
        addresses::NET_GAME_SINGLETON_RVA
    );
    assert_eq!(
        R1_SPEC.net_game.get_state_rva.get(),
        addresses::NET_GAME_GET_STATE_RVA
    );
    assert_eq!(
        R1_SPEC.net_game.get_player_pool_rva.get(),
        addresses::NET_GAME_GET_PLAYER_POOL_RVA
    );
    assert_eq!(
        R1_SPEC.net_game.get_vehicle_pool_rva.get(),
        addresses::NET_GAME_GET_VEHICLE_POOL_RVA
    );
    assert_eq!(
        R1_SPEC.net_game.shutdown_for_restart_rva.get(),
        addresses::NET_GAME_SHUTDOWN_FOR_RESTART_RVA
    );
    assert_eq!(
        R1_SPEC.ui.dialog.singleton_rva.get(),
        addresses::DIALOG_SINGLETON_RVA
    );
    assert_eq!(R1_SPEC.ui.dialog.show_rva.get(), addresses::DIALOG_SHOW_RVA);
    assert_eq!(
        R1_SPEC.ui.dialog.close_rva.get(),
        addresses::DIALOG_CLOSE_RVA
    );
    assert_eq!(
        R1_SPEC.ui.input.singleton_rva.get(),
        addresses::INPUT_SINGLETON_RVA
    );
    assert_eq!(R1_SPEC.ui.input.open_rva.get(), addresses::INPUT_OPEN_RVA);
    assert_eq!(R1_SPEC.ui.input.close_rva.get(), addresses::INPUT_CLOSE_RVA);
    assert_eq!(
        R1_SPEC.ui.input.get_command_handler_rva.get(),
        addresses::INPUT_GET_COMMAND_HANDLER_RVA
    );
    assert_eq!(
        R1_SPEC.ui.input.add_command_rva.get(),
        addresses::INPUT_ADD_COMMAND_RVA
    );
    assert_eq!(
        R1_SPEC.ui.input.process_rva.get(),
        addresses::INPUT_PROCESS_RVA
    );
    assert_eq!(
        R1_SPEC.ui.input.edit_box_set_text_rva.get(),
        addresses::DXUT_EDIT_BOX_SET_TEXT_RVA
    );
    assert_eq!(
        R1_SPEC.ui.input.edit_box_get_text_rva.get(),
        addresses::DXUT_EDIT_BOX_GET_TEXT_RVA
    );
    assert_eq!(
        R1_SPEC.ui.chat.singleton_rva.get(),
        addresses::CHAT_SINGLETON_RVA
    );
    assert_eq!(
        R1_SPEC.ui.chat.add_entry_rva.get(),
        addresses::CHAT_ADD_ENTRY_RVA
    );
    assert_eq!(
        R1_SPEC.ui.chat.get_mode_rva.get(),
        addresses::CHAT_GET_MODE_RVA
    );
    assert_eq!(
        R1_SPEC.ui.scoreboard.singleton_rva.get(),
        addresses::SCOREBOARD_SINGLETON_RVA
    );
    assert_eq!(
        R1_SPEC.ui.death_window.singleton_rva.get(),
        addresses::DEATH_WINDOW_SINGLETON_RVA
    );
    assert_eq!(
        R1_SPEC.ui.death_window.add_message_rva.get(),
        addresses::DEATH_WINDOW_ADD_MESSAGE_RVA
    );
    assert_eq!(
        R1_SPEC.ui.game.singleton_rva.get(),
        addresses::GAME_SINGLETON_RVA
    );
    assert_eq!(
        R1_SPEC.ui.game.set_cursor_mode_rva.get(),
        addresses::GAME_SET_CURSOR_MODE_RVA
    );
    assert_eq!(
        R1_SPEC.ui.game.process_input_enabling_rva.get(),
        addresses::GAME_PROCESS_INPUT_ENABLING_RVA
    );
    assert_eq!(
        R1_SPEC.pools.vehicle.does_exist_rva.get(),
        addresses::VEHICLE_POOL_DOES_EXIST_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_local_player.get(),
        addresses::PLAYER_POOL_GET_LOCAL_PLAYER_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_local_score.get(),
        addresses::PLAYER_POOL_GET_LOCAL_SCORE_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_local_ping.get(),
        addresses::PLAYER_POOL_GET_LOCAL_PING_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.is_connected.get(),
        addresses::PLAYER_POOL_IS_CONNECTED_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_remote_player.get(),
        addresses::PLAYER_POOL_GET_REMOTE_PLAYER_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.is_npc.get(),
        addresses::PLAYER_POOL_IS_NPC_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_name.get(),
        addresses::PLAYER_POOL_GET_NAME_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_score.get(),
        addresses::PLAYER_POOL_GET_SCORE_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_ping.get(),
        addresses::PLAYER_POOL_GET_PING_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.get_count.get(),
        addresses::PLAYER_POOL_GET_COUNT_RVA
    );
    assert_eq!(
        R1_SPEC.players.pool_rvas.set_local_player_name.get(),
        addresses::PLAYER_POOL_SET_LOCAL_PLAYER_NAME_RVA
    );
    assert_eq!(
        R1_SPEC.players.remote_rvas.get_colour_argb.get(),
        addresses::REMOTE_PLAYER_GET_COLOUR_ARGB_RVA
    );
    assert_eq!(
        R1_SPEC.players.remote_rvas.set_colour.get(),
        addresses::REMOTE_PLAYER_SET_COLOUR_RVA
    );
    assert_eq!(
        R1_SPEC.players.remote_rvas.does_exist.get(),
        addresses::REMOTE_PLAYER_DOES_EXIST_RVA
    );
    assert_eq!(
        R1_SPEC.players.remote_rvas.get_status.get(),
        addresses::REMOTE_PLAYER_GET_STATUS_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.get_ped.get(),
        addresses::LOCAL_PLAYER_GET_PED_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.get_colour_argb.get(),
        addresses::LOCAL_PLAYER_GET_COLOUR_ARGB_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.set_colour.get(),
        addresses::LOCAL_PLAYER_SET_COLOUR_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.set_special_action.get(),
        addresses::LOCAL_PLAYER_SET_SPECIAL_ACTION_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.spawn.get(),
        addresses::LOCAL_PLAYER_SPAWN_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_unoccupied_data.get(),
        addresses::LOCAL_PLAYER_SEND_UNOCCUPIED_DATA_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_aim_data.get(),
        addresses::LOCAL_PLAYER_SEND_AIM_DATA_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_onfoot_data.get(),
        addresses::LOCAL_PLAYER_SEND_ONFOOT_DATA_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_stats.get(),
        addresses::LOCAL_PLAYER_SEND_STATS_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_trailer_data.get(),
        addresses::LOCAL_PLAYER_SEND_TRAILER_DATA_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_passenger_data.get(),
        addresses::LOCAL_PLAYER_SEND_PASSENGER_DATA_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.send_incar_data.get(),
        addresses::LOCAL_PLAYER_SEND_INCAR_DATA_RVA
    );
    assert_eq!(
        R1_SPEC.players.local_rvas.update_weapons.get(),
        addresses::LOCAL_PLAYER_UPDATE_WEAPONS_RVA
    );
    assert_eq!(
        R1_SPEC.players.ped_rvas.get_health.get(),
        addresses::PED_GET_HEALTH_RVA
    );
    assert_eq!(
        R1_SPEC.players.ped_rvas.get_armour.get(),
        addresses::PED_GET_ARMOUR_RVA
    );
    assert_eq!(
        R1_SPEC.players.animation.rva.get(),
        addresses::ANIMATION_TABLE_RVA
    );
    assert_eq!(
        R1_SPEC.sync.send_rates.onfoot.get(),
        addresses::ONFOOT_SEND_RATE_RVA
    );
    assert_eq!(
        R1_SPEC.sync.send_rates.incar.get(),
        addresses::INCAR_SEND_RATE_RVA
    );
    assert_eq!(
        R1_SPEC.sync.send_rates.aim.get(),
        addresses::AIM_SEND_RATE_RVA
    );
    assert_eq!(
        R1_SPEC.text_labels.create_rva.get(),
        addresses::LABEL_POOL_CREATE_RVA
    );
    assert_eq!(
        R1_SPEC.text_labels.delete_rva.get(),
        addresses::LABEL_POOL_DELETE_RVA
    );
    assert_eq!(
        R1_SPEC.textdraws.create_rva.get(),
        addresses::TEXTDRAW_POOL_CREATE_RVA
    );
    assert_eq!(
        R1_SPEC.textdraws.delete_rva.get(),
        addresses::TEXTDRAW_POOL_DELETE_RVA
    );
    assert_eq!(
        R1_SPEC.net_game.host_address_offset.get(),
        NET_GAME_HOST_ADDRESS_OFFSET
    );
    assert_eq!(
        R1_SPEC.net_game.hostname_offset.get(),
        NET_GAME_HOSTNAME_OFFSET
    );
    assert_eq!(R1_SPEC.net_game.port_offset.get(), NET_GAME_PORT_OFFSET);
    assert_eq!(
        R1_SPEC.net_game.game_state_offset.get(),
        NET_GAME_GAME_STATE_OFFSET
    );
    assert_eq!(
        R1_SPEC.net_game.server_settings_offset.get(),
        NET_GAME_SERVER_SETTINGS_OFFSET
    );
    assert_eq!(R1_SPEC.pools.limits.players.get(), 1004);
    assert_eq!(R1_SPEC.pools.limits.vehicles.get(), 2000);
    assert_eq!(R1_SPEC.pools.limits.objects.get(), 1000);
    assert_eq!(R1_SPEC.pools.limits.text_labels.get(), 2048);
    assert_eq!(R1_SPEC.pools.limits.textdraws.get(), 2304);
    assert_eq!(R1_SPEC.pools.limits.gangzones.get(), 1024);
    assert_eq!(R1_SPEC.pools.limits.pickups.get(), 4096);
    assert_eq!(
        R1_SPEC.pools.player.largest_id_offset.get(),
        PLAYER_POOL_LARGEST_ID_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.player.local_id_offset.get(),
        PLAYER_POOL_LOCAL_ID_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.vehicle.not_empty_offset.get(),
        VEHICLE_POOL_NOT_EMPTY_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.vehicle.game_objects_offset.get(),
        VEHICLE_POOL_GAME_OBJECTS_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.object.not_empty_offset.get(),
        OBJECT_POOL_NOT_EMPTY_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.object.objects_offset.get(),
        OBJECT_POOL_OBJECTS_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.pickup.handles_offset.get(),
        PICKUP_POOL_HANDLES_OFFSET
    );
    assert_eq!(
        R1_SPEC.pools.entity_handle_offset.get(),
        ENTITY_HANDLE_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.active_offset.get(),
        LOCAL_PLAYER_ACTIVE_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.current_vehicle_offset.get(),
        LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.onfoot_offset.get(),
        LOCAL_PLAYER_ONFOOT_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.passenger_offset.get(),
        LOCAL_PLAYER_PASSENGER_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.trailer_offset.get(),
        LOCAL_PLAYER_TRAILER_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.incar_offset.get(),
        LOCAL_PLAYER_INCAR_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.onfoot.position_offset.get(),
        LOCAL_PLAYER_ONFOOT_POSITION_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.onfoot.speed_offset.get(),
        LOCAL_PLAYER_ONFOOT_SPEED_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.onfoot.special_action_offset.get(),
        LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.onfoot.animation_offset.get(),
        LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.incar.position_offset.get(),
        LOCAL_PLAYER_INCAR_POSITION_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.incar.speed_offset.get(),
        LOCAL_PLAYER_INCAR_SPEED_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.local.game_ped_offset.get(),
        SAMP_PED_GAME_PED_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.remote.onfoot_offset.get(),
        REMOTE_PLAYER_ONFOOT_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.remote.incar_offset.get(),
        REMOTE_PLAYER_INCAR_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.remote.passenger_offset.get(),
        REMOTE_PLAYER_PASSENGER_OFFSET
    );
    assert_eq!(
        R1_SPEC.players.remote.trailer_offset.get(),
        REMOTE_PLAYER_TRAILER_OFFSET
    );
    assert_eq!(R1_SPEC.sync.onfoot.size.get(), unsafe {
        samp_client_sdk_fixture_r1_onfoot_size()
    });
    assert_eq!(R1_SPEC.sync.incar.size.get(), unsafe {
        samp_client_sdk_fixture_r1_incar_size()
    });
    assert_eq!(R1_SPEC.ui.chat.entries_offset.get(), CHAT_ENTRIES_OFFSET);
    assert_eq!(R1_SPEC.ui.chat.entry_size.get(), CHAT_ENTRY_SIZE);
    assert_eq!(
        R1_SPEC.ui.scoreboard.enabled_offset.get(),
        SCOREBOARD_ENABLED_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.game.cursor_mode_offset.get(),
        GAME_CURSOR_MODE_OFFSET
    );
    assert_eq!(R1_SPEC.ui.dialog.active_offset.get(), DIALOG_ACTIVE_OFFSET);
    assert_eq!(
        R1_SPEC.ui.dialog.dialog_type_offset.get(),
        DIALOG_TYPE_OFFSET
    );
    assert_eq!(R1_SPEC.ui.dialog.id_offset.get(), DIALOG_ID_OFFSET);
    assert_eq!(
        R1_SPEC.ui.dialog.listbox_offset.get(),
        DIALOG_LISTBOX_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.editbox_offset.get(),
        DIALOG_EDITBOX_OFFSET
    );
    assert_eq!(R1_SPEC.ui.dialog.text_offset.get(), DIALOG_TEXT_OFFSET);
    assert_eq!(
        R1_SPEC.ui.dialog.caption_offset.get(),
        DIALOG_CAPTION_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.server_side_offset.get(),
        DIALOG_SERVER_SIDE_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.selected_offset.get(),
        DXUT_LISTBOX_SELECTED_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.items_offset.get(),
        DXUT_LISTBOX_ITEMS_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.item_count_offset.get(),
        DXUT_LISTBOX_ITEM_COUNT_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.item_text_offset.get(),
        DXUT_LISTBOX_ITEM_TEXT_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.item_data_offset.get(),
        DXUT_LISTBOX_ITEM_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.item_active_rect_offset.get(),
        DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.item_visible_offset.get(),
        DXUT_LISTBOX_ITEM_VISIBLE_OFFSET
    );
    assert_eq!(
        R1_SPEC.ui.dialog.listbox.item_size.get(),
        DXUT_LISTBOX_ITEM_SIZE
    );
    assert_eq!(R1_SPEC.text_labels.size.get(), LABEL_SIZE);
    assert_eq!(R1_SPEC.text_labels.text_offset.get(), LABEL_TEXT_OFFSET);
    assert_eq!(R1_SPEC.text_labels.colour_offset.get(), LABEL_COLOUR_OFFSET);
    assert_eq!(
        R1_SPEC.text_labels.position_offset.get(),
        LABEL_POSITION_OFFSET
    );
    assert_eq!(
        R1_SPEC.text_labels.draw_distance_offset.get(),
        LABEL_DRAW_DISTANCE_OFFSET
    );
    assert_eq!(
        R1_SPEC.text_labels.behind_walls_offset.get(),
        LABEL_BEHIND_WALLS_OFFSET
    );
    assert_eq!(
        R1_SPEC.text_labels.attached_player_offset.get(),
        LABEL_ATTACHED_PLAYER_OFFSET
    );
    assert_eq!(
        R1_SPEC.text_labels.attached_vehicle_offset.get(),
        LABEL_ATTACHED_VEHICLE_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.text_setter_rva.get(),
        TEXTDRAW_SET_TEXT_RVA
    );
    assert_eq!(R1_SPEC.textdraws.native_size.get(), TEXTDRAW_NATIVE_SIZE);
    assert_eq!(R1_SPEC.textdraws.data_offset.get(), TEXTDRAW_DATA_OFFSET);
    assert_eq!(
        R1_SPEC.textdraws.transmit.size.get(),
        TEXTDRAW_TRANSMIT_SIZE
    );
    assert_eq!(
        R1_SPEC.textdraws.transmit.x.get(),
        TEXTDRAW_TRANSMIT_X_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.transmit.y.get(),
        TEXTDRAW_TRANSMIT_Y_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.width.get(),
        TEXTDRAW_LETTER_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.height.get(),
        TEXTDRAW_LETTER_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.colour.get(),
        TEXTDRAW_LETTER_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.align_center.get(),
        TEXTDRAW_ALIGN_CENTER_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.box_enabled.get(),
        TEXTDRAW_BOX_ENABLED_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.box_width.get(),
        TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.box_height.get(),
        TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.box_colour.get(),
        TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.proportional.get(),
        TEXTDRAW_PROPORTIONAL_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.background_colour.get(),
        TEXTDRAW_BACKGROUND_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.shadow.get(),
        TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.outline.get(),
        TEXTDRAW_OUTLINE_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.align_left.get(),
        TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.align_right.get(),
        TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.style.get(),
        TEXTDRAW_STYLE_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.x.get(),
        TEXTDRAW_X_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.y.get(),
        TEXTDRAW_Y_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.model_id.get(),
        TEXTDRAW_MODEL_ID_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.rotation.get(),
        TEXTDRAW_ROTATION_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.zoom.get(),
        TEXTDRAW_ZOOM_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.model_colour1.get(),
        TEXTDRAW_MODEL_COLOUR1_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.textdraws.data.model_colour2.get(),
        TEXTDRAW_MODEL_COLOUR2_OFFSET - TEXTDRAW_DATA_OFFSET
    );
    assert_eq!(
        R1_SPEC.strategies.game_state_codec,
        GameStateCodec::Identity
    );
    assert_eq!(
        R1_SPEC.strategies.local_player_source,
        LocalPlayerSource::PlayerPoolGetter
    );
    assert_eq!(R1_SPEC.strategies.i32_boolean, NativeBoolean::ValidatedI32);
    assert_eq!(R1_SPEC.strategies.u8_boolean, NativeBoolean::ValidatedU8);
    assert_eq!(
        R1_SPEC.strategies.force_sync_reset,
        ForceSyncReset::ClearLastAnyUpdate
    );
    assert_eq!(
        R1_SPEC.strategies.list_item_text_layout,
        ListItemTextLayout::DxutComboBoxItem
    );
    assert_eq!(
        R1_SPEC.strategies.textdraw_calls,
        TextdrawCallStrategy::NativeMethods
    );
}
