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
    TEXTDRAW_MODEL_ID_OFFSET, TEXTDRAW_OUTLINE_OFFSET, TEXTDRAW_POOL_NOT_EMPTY_OFFSET,
    TEXTDRAW_POOL_OBJECTS_OFFSET, TEXTDRAW_PROPORTIONAL_OFFSET, TEXTDRAW_ROTATION_OFFSET,
    TEXTDRAW_SHADOW_OFFSET, TEXTDRAW_STYLE_OFFSET, TEXTDRAW_X_OFFSET, TEXTDRAW_Y_OFFSET,
    TEXTDRAW_ZOOM_OFFSET, VEHICLE_POOL_GAME_OBJECTS_OFFSET, VEHICLE_POOL_NOT_EMPTY_OFFSET,
    assigned_player_id, bounded_c_string, bounded_dxut_listbox_item_text, mem, nul_terminated,
};

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
