use crate::{
    BitStream, Direction, ListenerHandle, PacketEvent, RpcEvent, SampVersion,
    command::{CommandError, CommandId},
    event::Registry,
    platform,
};
use std::{sync::Arc, time::Duration};

mod errors;
mod options;
mod requests;
mod snapshots;

pub use errors::{AttachError, SendError};
pub(crate) use errors::{CodecError, DirectClientError};
use options::validate_packet_options;
pub use options::{PacketPriority, PacketReliability, SendOptions};
pub(crate) use requests::{
    LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest, LocalDialogRequest,
    LocalDialogStyle,
};
pub(crate) use snapshots::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, GangzoneSnapshot, InCarSyncSnapshot,
    LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot,
    PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot,
    TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
};
/// A live SA-MP hook runtime.
///
/// Only one runtime may be attached in a process. Drop it before unloading the
/// containing ASI/DLL so native detours and vtable changes are restored.
pub struct Runtime {
    registry: Arc<Registry>,
    backend: platform::Backend,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ClientHookStatus {
    Pending,
    Ready,
    Failed,
}

impl Runtime {
    /// Installs the startup hook for a supported SA-MP client.
    ///
    /// Call this after `samp.dll` loads but before RakClient construction. A
    /// host ASI/DLL should wait for `samp.dll`, then attach immediately rather
    /// than waiting for the normal game-loop callback.
    pub fn attach() -> Result<Self, AttachError> {
        let registry = Registry::new();
        let backend = platform::attach(Arc::clone(&registry))?;
        Ok(Self { registry, backend })
    }

    /// Registers a synchronous packet listener.
    pub fn on_packet(
        &self,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut PacketEvent<'event>) -> crate::HookAction + Send + 'static,
    ) -> ListenerHandle {
        self.registry.register_packet(direction, callback)
    }

    /// Registers a synchronous RPC listener.
    pub fn on_rpc(
        &self,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut RpcEvent<'event>) -> crate::HookAction + Send + 'static,
    ) -> ListenerHandle {
        self.registry.register_rpc(direction, callback)
    }

    /// Queues a packet for the original SA-MP RakClient method on the game thread.
    ///
    /// This bypasses outgoing listeners to prevent recursive hook dispatch.
    pub fn send_packet(&self, packet_id: u8, payload: &BitStream) -> Result<bool, SendError> {
        self.backend
            .send_packet(packet_id, payload, SendOptions::default())
    }

    /// Queues a packet with explicit RakNet delivery settings for the game thread.
    pub fn send_packet_with_options(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        validate_packet_options(options)?;
        self.backend.send_packet(packet_id, payload, options)
    }

    /// Queues an RPC for the original SA-MP RakClient method on the game thread.
    ///
    /// This bypasses outgoing listeners to prevent recursive hook dispatch.
    pub fn send_rpc(&self, rpc_id: u8, payload: &BitStream) -> Result<bool, SendError> {
        self.backend
            .send_rpc(rpc_id, payload, SendOptions::default())
    }

    /// Queues an RPC with explicit RakNet delivery settings for the game thread.
    pub fn send_rpc_with_options(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        self.backend.send_rpc(rpc_id, payload, options)
    }

    pub(crate) fn submit_packet_with_options(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        validate_packet_options(options)?;
        self.backend.submit_packet(packet_id, payload, options)
    }

    pub(crate) fn submit_rpc_with_options(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<CommandId, SendError> {
        self.backend.submit_rpc(rpc_id, payload, options)
    }

    /// Queues a packet for game-thread emulation; incoming listeners run once then.
    pub fn emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        self.backend.emulate_incoming_packet(packet_id, payload)
    }

    /// Queues an RPC for game-thread delivery after incoming listeners run.
    pub fn emulate_incoming_rpc(&self, rpc_id: u8, payload: BitStream) -> Result<bool, SendError> {
        self.backend.emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn submit_emulate_incoming_packet(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.backend
            .submit_emulate_incoming_packet(packet_id, payload)
    }

    pub(crate) fn submit_emulate_incoming_rpc(
        &self,
        rpc_id: u8,
        payload: BitStream,
    ) -> Result<CommandId, SendError> {
        self.backend.submit_emulate_incoming_rpc(rpc_id, payload)
    }

    pub(crate) fn client_hook_status(&self) -> ClientHookStatus {
        self.backend.client_hook_status()
    }

    pub(crate) fn incoming_emulation_ready(&self) -> bool {
        self.backend.incoming_emulation_ready()
    }

    pub(crate) fn raw_rakclient(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_rakclient()
    }

    pub(crate) fn raw_rakpeer(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_rakpeer()
    }

    pub(crate) fn raw_player_pool(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_player_pool()
    }

    pub(crate) fn raw_vehicle_pool(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_vehicle_pool()
    }

    pub(crate) fn raw_local_player(&self) -> Option<*mut core::ffi::c_void> {
        self.backend.raw_local_player()
    }

    pub(crate) fn samp_version(&self) -> SampVersion {
        self.backend.samp_version()
    }

    pub(crate) fn encode_string(&self, value: &[u8]) -> Result<BitStream, CodecError> {
        self.backend.encode_string(value)
    }

    pub(crate) fn decode_string(
        &self,
        payload: &mut BitStream,
        output: &mut [u8],
    ) -> Result<usize, CodecError> {
        self.backend.decode_string(payload, output)
    }

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

    pub(crate) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        self.backend.local_player()
    }

    pub(crate) fn player_info(
        &self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        self.backend.player_info(id)
    }

    pub(crate) fn remote_player_state(
        &self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        self.backend.remote_player_state(id)
    }

    pub(crate) fn streamed_out_player_position(
        &self,
        id: u16,
    ) -> Result<Option<Vector3>, DirectClientError> {
        self.backend.streamed_out_player_position(id)
    }

    pub(crate) fn onfoot_sync(
        &self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        self.backend.onfoot_sync(id)
    }

    pub(crate) fn vehicle_sync(
        &self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        self.backend.vehicle_sync(id)
    }

    pub(crate) fn passenger_sync(
        &self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        self.backend.passenger_sync(id)
    }

    pub(crate) fn trailer_sync(
        &self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        self.backend.trailer_sync(id)
    }
    pub(crate) fn aim_sync(&self, id: u16) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        self.backend.aim_sync(id)
    }

    pub(crate) fn player_defined(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.player_defined(id)
    }

    pub(crate) fn player_paused(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.player_paused(id)
    }

    pub(crate) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        self.backend.player_count(include_npcs)
    }

    pub(crate) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        self.backend.player_max_id()
    }

    pub(crate) fn vehicle_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.vehicle_exists(id)
    }

    pub(crate) fn text_label_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.text_label_exists(id)
    }

    pub(crate) fn textdraw_exists(&self, pool_index: u16) -> Result<bool, DirectClientError> {
        self.backend.textdraw_exists(pool_index)
    }

    pub(crate) fn object_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        self.backend.object_exists(id)
    }

    pub(crate) fn gangzone(&self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        self.backend.gangzone(id)
    }

    pub(crate) fn text_label(
        &self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        self.backend.text_label(id)
    }

    pub(crate) fn textdraw(
        &self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        self.backend.textdraw(pool_index)
    }

    pub(crate) fn chat_entry(&self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        self.backend.chat_entry(id)
    }

    pub(crate) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        self.backend.server_info()
    }

    pub(crate) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        self.backend.samp_game_state()
    }

    pub(crate) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        self.backend.local_chat_display_mode()
    }

    pub(crate) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        self.backend.local_cursor_mode()
    }

    pub(crate) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        self.backend.local_scoreboard_open()
    }

    pub(crate) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_dialog_active()
    }

    pub(crate) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        self.backend.local_dialog_state()
    }

    pub(crate) fn take_local_dialog_response(
        &self,
    ) -> Result<Option<LocalDialogResponseSnapshot>, DirectClientError> {
        self.backend.take_local_dialog_response()
    }

    pub(crate) fn object_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.object_handle(id)
    }

    pub(crate) fn object_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.object_id_by_handle(handle)
    }

    pub(crate) fn pickup_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.pickup_handle(id)
    }

    pub(crate) fn pickup_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.pickup_id_by_handle(handle)
    }

    pub(crate) fn vehicle_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.vehicle_handle(id)
    }

    pub(crate) fn vehicle_id_by_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.vehicle_id_by_handle(handle)
    }

    pub(crate) fn player_ped_handle(&self, id: u16) -> Result<Option<i32>, DirectClientError> {
        self.backend.player_ped_handle(id)
    }

    pub(crate) fn player_id_by_ped_handle(
        &self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.player_id_by_ped_handle(handle)
    }

    pub(crate) fn local_dialog_selected_item(&self) -> Result<i32, DirectClientError> {
        self.backend.local_dialog_selected_item()
    }

    pub(crate) fn local_dialog_list_item_count(&self) -> Result<i32, DirectClientError> {
        self.backend.local_dialog_list_item_count()
    }

    pub(crate) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        self.backend.local_chat_input_active()
    }

    pub(crate) fn local_chat_input_text(&self) -> Result<Vec<u8>, DirectClientError> {
        self.backend.local_chat_input_text()
    }

    pub(crate) fn local_chat_command_defined(
        &self,
        name: &[u8],
    ) -> Result<bool, DirectClientError> {
        self.backend.local_chat_command_defined(name)
    }

    pub(crate) fn local_animation(&self, id: u16) -> Result<AnimationSnapshot, DirectClientError> {
        self.backend.local_animation(id)
    }

    pub(crate) fn local_animation_id(
        &self,
        name: &[u8],
        file: &[u8],
    ) -> Result<Option<u16>, DirectClientError> {
        self.backend.local_animation_id(name, file)
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.backend.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LocalChatMessageStyle, LocalDialogStyle, PacketPriority, PacketReliability, SendError,
        SendOptions, validate_packet_options,
    };

    #[test]
    fn timestamped_packet_options_are_explicitly_unsupported() {
        let options = SendOptions {
            priority: PacketPriority::High,
            reliability: PacketReliability::ReliableOrdered,
            ordering_channel: 0,
            timestamp: true,
        };

        assert_eq!(
            validate_packet_options(options),
            Err(SendError::TimestampedPacketUnsupported)
        );
    }

    #[test]
    fn direct_dialog_style_accepts_only_the_six_native_values() {
        assert_eq!(
            LocalDialogStyle::from_raw(0),
            Some(LocalDialogStyle::MessageBox)
        );
        assert_eq!(
            LocalDialogStyle::from_raw(5),
            Some(LocalDialogStyle::HeadersList)
        );
        assert_eq!(LocalDialogStyle::from_raw(6), None);
    }

    #[test]
    fn direct_chat_style_accepts_only_the_three_native_values() {
        assert_eq!(
            LocalChatMessageStyle::from_raw(2),
            Some(LocalChatMessageStyle::Chat)
        );
        assert_eq!(
            LocalChatMessageStyle::from_raw(8),
            Some(LocalChatMessageStyle::Debug)
        );
        assert_eq!(LocalChatMessageStyle::from_raw(1), None);
    }
}
