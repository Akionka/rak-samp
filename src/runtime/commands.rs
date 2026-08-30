use super::{
    DirectClientError, LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest,
    Runtime, Vector3,
};
use crate::command::{CommandError, CommandId};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GtaEntityHandle {
    Ped(gta_sa::PedHandle),
    Vehicle(gta_sa::VehicleHandle),
    Object(gta_sa::ObjectHandle),
}

impl Runtime {
    pub(crate) fn show_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<(), DirectClientError> {
        self.backend.show_local_dialog(request)
    }

    pub(crate) fn show_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        self.backend.show_local_chat_message(request)
    }

    pub(crate) fn show_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        self.backend.show_local_death_message(request)
    }

    pub(crate) fn submit_local_dialog(
        &self,
        request: LocalDialogRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog(request)
    }

    pub(crate) fn submit_local_chat_message(
        &self,
        request: LocalChatMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_message(request)
    }

    pub(crate) fn submit_local_death_message(
        &self,
        request: LocalDeathMessageRequest,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_death_message(request)
    }

    pub(crate) fn submit_local_cursor_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_cursor_mode(mode)
    }

    pub(crate) fn submit_local_chat_display_mode(
        &self,
        mode: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_display_mode(mode)
    }

    pub(crate) fn submit_local_chat_entry(
        &self,
        id: u16,
        text: Vec<u8>,
        prefix: Vec<u8>,
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_local_chat_entry(id, text, prefix, text_colour, prefix_colour)
    }

    pub(crate) fn submit_local_dialog_close(
        &self,
        button: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_close(button)
    }

    pub(crate) fn submit_local_chat_input_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_input_text(text)
    }

    pub(crate) fn submit_register_chat_command(
        &self,
        subscription: u64,
        slot: u8,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_register_chat_command(subscription, slot, name)
    }

    pub(crate) fn submit_unregister_chat_command(
        &self,
        subscription: u64,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_unregister_chat_command(subscription, name)
    }

    pub(crate) fn submit_local_chat_input_enabled(
        &self,
        enabled: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_input_enabled(enabled)
    }

    pub(crate) fn submit_local_chat_input_process(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_chat_input_process(text)
    }

    pub(crate) fn submit_local_cursor_toggle(
        &self,
        show: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_cursor_toggle(show)
    }

    pub(crate) fn submit_local_scoreboard_open(
        &self,
        open: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_scoreboard_open(open)
    }

    pub(crate) fn submit_local_dialog_client_side(
        &self,
        client_side: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_client_side(client_side)
    }

    pub(crate) fn submit_local_dialog_selected_item(
        &self,
        selected: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_selected_item(selected)
    }

    pub(crate) fn submit_local_dialog_editbox_text(
        &self,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_dialog_editbox_text(text)
    }

    pub(crate) fn submit_samp_game_state(
        &self,
        state: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_samp_game_state(state)
    }

    pub(crate) fn submit_connect_to_server(
        &self,
        address: Vec<u8>,
        port: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_connect_to_server(address, port)
    }

    pub(crate) fn submit_disconnect_with_reason(
        &self,
        block_duration: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_disconnect_with_reason(block_duration)
    }

    pub(crate) fn submit_delete_textdraw(&self, id: u16) -> Result<CommandId, DirectClientError> {
        self.backend.submit_delete_textdraw(id)
    }

    pub(crate) fn submit_create_textdraw(
        &self,
        id: u16,
        text: Vec<u8>,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_create_textdraw(id, text, x, y)
    }

    pub(crate) fn submit_delete_text_label(&self, id: u16) -> Result<CommandId, DirectClientError> {
        self.backend.submit_delete_text_label(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit_create_text_label(
        &self,
        id: u16,
        text: Vec<u8>,
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_create_text_label(
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn submit_create_text_label_auto(
        &self,
        text: Vec<u8>,
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_create_text_label_auto(
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id,
            attached_vehicle_id,
        )
    }

    pub(crate) fn submit_set_text_label_text(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_text_label_text(id, text)
    }

    pub(crate) fn submit_set_textdraw_position(
        &self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_position(id, x, y)
    }

    pub(crate) fn submit_set_textdraw_style(
        &self,
        id: u16,
        style: i32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_style(id, style)
    }

    pub(crate) fn submit_set_textdraw_letter_style(
        &self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_letter_style(id, width, height, colour)
    }

    pub(crate) fn submit_set_textdraw_proportional(
        &self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_proportional(id, proportional)
    }

    pub(crate) fn submit_set_textdraw_shadow(
        &self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_shadow(id, shadow, colour)
    }

    pub(crate) fn submit_set_textdraw_outline(
        &self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_outline(id, outline, colour)
    }

    pub(crate) fn submit_set_textdraw_box(
        &self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_box(id, enabled, colour, width, height)
    }

    pub(crate) fn submit_set_textdraw_alignment(
        &self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_alignment(id, alignment)
    }

    pub(crate) fn submit_set_textdraw_string(
        &self,
        id: u16,
        text: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_set_textdraw_string(id, text)
    }

    pub(crate) fn submit_set_textdraw_model_style(
        &self,
        id: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend
            .submit_set_textdraw_model_style(id, rotation, zoom, colour1, colour2)
    }

    pub(crate) fn submit_local_player_spawn(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_player_spawn()
    }

    pub(crate) fn submit_local_player_special_action(
        &self,
        action: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_player_special_action(action)
    }

    pub(crate) fn submit_local_player_name(
        &self,
        name: Vec<u8>,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_local_player_name(name)
    }

    pub(crate) fn submit_force_unoccupied_sync(
        &self,
        vehicle: u16,
        seat: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_unoccupied_sync(vehicle, seat)
    }
    pub(crate) fn submit_force_aim_sync(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_aim_sync()
    }
    pub(crate) fn submit_force_onfoot_sync(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_onfoot_sync()
    }
    pub(crate) fn submit_force_stats_sync(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_stats_sync()
    }

    pub(crate) fn submit_force_trailer_sync(
        &self,
        trailer: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_trailer_sync(trailer)
    }

    pub(crate) fn submit_force_vehicle_sync(
        &self,
        vehicle: u16,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_vehicle_sync(vehicle)
    }

    pub(crate) fn submit_force_passenger_sync(
        &self,
        vehicle: u16,
        seat: u8,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_passenger_sync(vehicle, seat)
    }

    pub(crate) fn submit_force_weapons_sync(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_force_weapons_sync()
    }

    pub(crate) fn submit_send_rate(
        &self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_send_rate(kind, milliseconds)
    }

    pub(crate) fn submit_player_colour(
        &self,
        id: u16,
        colour: u32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_player_colour(id, colour)
    }

    pub(crate) fn gta_local_ped_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<Option<gta_sa::PedSnapshot>, DirectClientError> {
        self.backend.gta_local_ped_snapshot(token)
    }
    pub(crate) fn gta_entity_exists(
        &self,
        token: modkit_runtime::ScopeToken,
        handle: GtaEntityHandle,
    ) -> Result<bool, DirectClientError> {
        self.backend.gta_entity_exists(token, handle)
    }

    pub(crate) fn gta_vehicle_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
        handle: gta_sa::VehicleHandle,
    ) -> Result<Option<gta_sa::VehicleSnapshot>, DirectClientError> {
        self.backend.gta_vehicle_snapshot(token, handle)
    }

    pub(crate) fn gta_find_ground_z(
        &self,
        token: modkit_runtime::ScopeToken,
        x: f32,
        y: f32,
    ) -> Result<f32, DirectClientError> {
        self.backend.gta_find_ground_z(token, x, y)
    }
    pub(crate) fn gta_timer_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<gta_sa::TimerSnapshot, DirectClientError> {
        self.backend.gta_timer_snapshot(token)
    }
    pub(crate) fn gta_camera_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<gta_sa::CameraSnapshot, DirectClientError> {
        self.backend.gta_camera_snapshot(token)
    }

    pub(crate) fn gta_teleport_local_ped(
        &self,
        token: modkit_runtime::ScopeToken,
        destination: gta_sa::Vector3,
    ) -> Result<(), DirectClientError> {
        self.backend.gta_teleport_local_ped(token, destination)
    }

    pub(crate) fn submit_gta_local_ped_snapshot(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_local_ped_snapshot()
    }

    pub(crate) fn take_gta_local_ped_snapshot(
        &self,
        id: CommandId,
    ) -> Option<Option<gta_sa::PedSnapshot>> {
        self.backend.take_gta_local_ped_snapshot(id)
    }
    pub(crate) fn submit_gta_entity_exists(
        &self,
        handle: GtaEntityHandle,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_entity_exists(handle)
    }

    pub(crate) fn take_gta_entity_exists(&self, id: CommandId) -> Option<bool> {
        self.backend.take_gta_entity_exists(id)
    }

    pub(crate) fn submit_gta_vehicle_snapshot(
        &self,
        handle: gta_sa::VehicleHandle,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_vehicle_snapshot(handle)
    }

    pub(crate) fn take_gta_vehicle_snapshot(
        &self,
        id: CommandId,
    ) -> Option<Option<gta_sa::VehicleSnapshot>> {
        self.backend.take_gta_vehicle_snapshot(id)
    }

    pub(crate) fn submit_gta_find_ground_z(
        &self,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_find_ground_z(x, y)
    }

    pub(crate) fn take_gta_find_ground_z(&self, id: CommandId) -> Option<f32> {
        self.backend.take_gta_find_ground_z(id)
    }
    pub(crate) fn submit_gta_timer_snapshot(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_timer_snapshot()
    }

    pub(crate) fn take_gta_timer_snapshot(&self, id: CommandId) -> Option<gta_sa::TimerSnapshot> {
        self.backend.take_gta_timer_snapshot(id)
    }
    pub(crate) fn submit_gta_camera_snapshot(&self) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_camera_snapshot()
    }

    pub(crate) fn take_gta_camera_snapshot(&self, id: CommandId) -> Option<gta_sa::CameraSnapshot> {
        self.backend.take_gta_camera_snapshot(id)
    }

    pub(crate) fn submit_gta_teleport_local_ped(
        &self,
        destination: gta_sa::Vector3,
    ) -> Result<CommandId, DirectClientError> {
        self.backend.submit_gta_teleport_local_ped(destination)
    }

    pub(crate) fn try_take_command(
        &self,
        id: CommandId,
    ) -> Result<Option<Result<(), CommandError>>, CommandError> {
        self.backend.try_take_command(id)
    }

    pub(crate) fn wait_for_command(
        &self,
        id: CommandId,
        timeout: Duration,
    ) -> Result<Result<(), CommandError>, CommandError> {
        self.backend.wait_for_command(id, timeout)
    }

    pub(crate) fn command_wait_allowed(&self) -> bool {
        self.backend.command_wait_allowed()
    }

    pub(crate) fn release_command(&self, id: CommandId) -> Result<(), CommandError> {
        self.backend.release_command(id)
    }

    pub(crate) fn try_take_created_text_label(
        &self,
        id: CommandId,
    ) -> Result<Option<Result<u16, CommandError>>, CommandError> {
        self.backend.try_take_created_text_label(id)
    }

    pub(crate) fn wait_for_created_text_label(
        &self,
        id: CommandId,
        timeout: Duration,
    ) -> Result<Result<u16, CommandError>, CommandError> {
        self.backend.wait_for_created_text_label(id, timeout)
    }
}
