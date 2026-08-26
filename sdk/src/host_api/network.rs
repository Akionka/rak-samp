//! Low-level packet/RPC send and incoming-emulation `HostApi` wrappers.

use crate::events;
use crate::{
    CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult,
    SampClientSdkSendOptions,
};
use samp_protocol::BitStream;

impl HostApi {
    /// Returns whether incoming packet emulation can be submitted without waiting for the host
    /// to capture its native receiver from a real incoming RPC.
    #[must_use]
    pub fn incoming_emulation_ready(self) -> bool {
        (self.raw.incoming_emulation_ready)() != 0
    }

    /// Sends a bounded server-bound RCON command packet (201).
    pub fn send_rcon_command(self, command: &[u8]) -> SampClientSdkResult {
        self.send_typed_packet(
            events::packet::outgoing::SEND_RCON_COMMAND,
            command.to_vec(),
        )
    }
    /// Sends a complete local aim-sync packet (203).
    pub fn send_aim_sync(self, sync: events::packet::AimSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_AIM_SYNC, sync)
    }
    /// Sends a complete local bullet-sync packet (206).
    pub fn send_bullet_sync(self, sync: events::packet::BulletSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_BULLET_SYNC, sync)
    }
    /// Sends a complete local vehicle-sync packet (200).
    pub fn send_vehicle_sync(self, sync: events::packet::VehicleSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_VEHICLE_SYNC, sync)
    }
    /// Sends a complete local on-foot player-sync packet (207).
    pub fn send_player_sync(self, sync: events::packet::PlayerSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_PLAYER_SYNC, sync)
    }
    /// Sends a complete local spectator-sync packet (212).
    pub fn send_spectator_sync(self, sync: events::packet::SpectatorSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_SPECTATOR_SYNC, sync)
    }
    /// Sends a complete local trailer-sync packet (210).
    pub fn send_trailer_sync(self, sync: events::packet::TrailerSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_TRAILER_SYNC, sync)
    }
    /// Sends a complete local passenger-sync packet (211).
    pub fn send_passenger_sync(self, sync: events::packet::PassengerSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_PASSENGER_SYNC, sync)
    }
    /// Sends a complete local unoccupied-vehicle sync packet (209).
    pub fn send_unoccupied_sync(self, sync: events::packet::UnoccupiedSync) -> SampClientSdkResult {
        self.send_typed_packet(events::packet::outgoing::SEND_UNOCCUPIED_SYNC, sync)
    }
    /// Sends a packet through SA-MP's original RakClient method.
    ///
    /// `payload` excludes the packet ID. Outgoing listeners are bypassed to prevent recursive
    /// dispatch. Timestamped packet options are currently rejected as invalid.
    pub fn send_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        unsafe {
            (self.raw.send_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len, options)
        }
    }

    /// Sends an RPC through SA-MP's original RakClient method.
    ///
    /// `payload` excludes the RPC ID. Outgoing listeners are bypassed to prevent recursive
    /// dispatch.
    pub fn send_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        unsafe { (self.raw.send_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len, options) }
    }

    /// Sends a complete owned plugin-side bit stream as a packet payload.
    pub fn send_packet_stream(
        self,
        packet_id: u8,
        payload: &BitStream,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        self.send_packet(packet_id, payload.as_bytes(), payload.len_bits(), options)
    }

    /// Sends a complete owned plugin-side bit stream as an RPC payload.
    pub fn send_rpc_stream(
        self,
        rpc_id: u8,
        payload: &BitStream,
        options: SampClientSdkSendOptions,
    ) -> SampClientSdkResult {
        self.send_rpc(rpc_id, payload.as_bytes(), payload.len_bits(), options)
    }

    /// Queues an incoming packet for SA-MP after incoming plugin listeners run.
    ///
    /// `payload` excludes the packet ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`SampClientSdkResult::Ok`].
    pub fn emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> SampClientSdkResult {
        unsafe {
            (self.raw.emulate_incoming_packet)(packet_id, payload.as_ptr(), payload.len(), bit_len)
        }
    }

    /// Dispatches an incoming RPC to plugin listeners and then SA-MP unless blocked.
    ///
    /// `payload` excludes the RPC ID. A listener may rewrite or block the event;
    /// blocking is still reported as [`SampClientSdkResult::Ok`].
    pub fn emulate_incoming_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> SampClientSdkResult {
        unsafe { (self.raw.emulate_incoming_rpc)(rpc_id, payload.as_ptr(), payload.len(), bit_len) }
    }

    /// Copies and queues a locally generated incoming packet, returning its completion receipt.
    pub fn submit_emulate_incoming_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_emulate_incoming_packet)(
                packet_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a locally generated incoming RPC, returning its completion receipt.
    pub fn submit_emulate_incoming_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_emulate_incoming_rpc)(
                rpc_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a server-bound packet, returning its game-thread completion receipt.
    pub fn submit_packet(
        self,
        packet_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_packet)(
                packet_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                options,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }

    /// Copies and queues a server-bound RPC, returning its game-thread completion receipt.
    pub fn submit_rpc(
        self,
        rpc_id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let mut receipt = SampClientSdkCommandReceipt::default();
        let result = unsafe {
            (self.raw.submit_rpc)(
                rpc_id,
                payload.as_ptr(),
                payload.len(),
                bit_len,
                options,
                &mut receipt,
            )
        };
        self.command_receipt(result, receipt)
    }
    /// Sends one server-bound SA-MP chat message (RPC 101).
    ///
    /// This is the safe equivalent of SF.lua's `sampSendChat`. The message is
    /// serialized as the protocol's bounded `string8` payload; a
    /// slash-prefixed value instead uses the command RPC (50), matching the
    /// native helper. It is real network traffic, not a local chat display
    /// action.
    pub fn send_chat(self, text: &[u8]) -> SampClientSdkResult {
        if text.first() == Some(&b'/') {
            self.send_protocol_rpc(
                samp_protocol::rpc::outgoing::chat::SEND_COMMAND,
                text.to_vec(),
            )
        } else {
            self.send_protocol_rpc(samp_protocol::rpc::outgoing::chat::SEND_CHAT, text.to_vec())
        }
    }

    /// Sends SA-MP's empty request-spawn RPC (129).
    ///
    /// This is the protocol-level equivalent of SF.lua's
    /// `sampSendRequestSpawn`; it does not call native local-player methods or
    /// mutate client state.
    pub fn send_request_spawn(self) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::session::SEND_REQUEST_SPAWN, ())
    }

    /// Sends SA-MP's request-class RPC (128).
    ///
    /// This carries the same server-bound protocol value as SF.lua's
    /// `sampRequestClass`, but does not invoke the native local-player method
    /// or update any local class-selection state.
    pub fn send_request_class(self, class_id: i32) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::session::SEND_REQUEST_CLASS, class_id)
    }

    /// Sends SA-MP's interior-change RPC (118).
    ///
    /// This is protocol-only. It does not change the GTA interior or mutate
    /// SA-MP's native local-player state.
    pub fn send_interior_change(self, interior_id: u8) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_INTERIOR_CHANGE, interior_id)
    }

    /// Sends SA-MP's empty spawn RPC (52).
    ///
    /// This is protocol-only. It does not call the native local-player spawn
    /// method or change local spawn state.
    pub fn send_spawn(self) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::session::SEND_SPAWN, ())
    }

    /// Sends SA-MP's enter-vehicle RPC (26).
    ///
    /// This is protocol-only. It does not put the local GTA ped in a vehicle
    /// or otherwise alter native local-player state.
    pub fn send_enter_vehicle(self, vehicle_id: u16, passenger: bool) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::vehicle::SEND_ENTER_VEHICLE,
            events::rpc::outgoing::vehicle::EnterVehicle {
                vehicle_id,
                passenger,
            },
        )
    }

    /// Sends SA-MP's exit-vehicle RPC (154).
    ///
    /// This is protocol-only. It does not make the local GTA ped leave a
    /// vehicle or otherwise alter native local-player state.
    pub fn send_exit_vehicle(self, vehicle_id: u16) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::vehicle::SEND_EXIT_VEHICLE,
            vehicle_id,
        )
    }

    /// Sends a server-bound dialog response (RPC 62).
    pub fn send_dialog_response(
        self,
        dialog_id: u16,
        button: u8,
        list_item: u16,
        input: &[u8],
    ) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::ui::SEND_DIALOG_RESPONSE,
            events::rpc::outgoing::ui::DialogResponse {
                dialog_id,
                button,
                list_item,
                input: input.to_vec(),
            },
        )
    }

    /// Sends a server-bound player-click action (RPC 23).
    pub fn send_click_player(self, player_id: u16, source: u8) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::ui::SEND_CLICK_PLAYER,
            events::rpc::outgoing::ui::ClickPlayer { player_id, source },
        )
    }

    /// Sends a server-bound textdraw-click action (RPC 83).
    pub fn send_click_textdraw(self, textdraw_id: u16) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::ui::SEND_CLICK_TEXT_DRAW, textdraw_id)
    }

    /// Sends a server-bound death notification naming another player (RPC 53).
    pub fn send_death_by_player(self, player_id: u16, reason: u8) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::SEND_DEATH_NOTIFICATION,
            events::rpc::outgoing::DeathNotification {
                reason,
                killer_id: player_id,
            },
        )
    }

    /// Sends the empty menu-quit RPC (140).
    pub fn send_menu_quit(self) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::ui::SEND_QUIT_MENU, ())
    }

    /// Sends a server-bound menu-row selection (RPC 132).
    pub fn send_menu_select_row(self, row: u8) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::ui::SEND_MENU_SELECT, row)
    }

    /// Sends a server-bound pickup notification (RPC 131).
    pub fn send_picked_up_pickup(self, pickup_id: i32) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::SEND_PICKED_UP_PICKUP, pickup_id)
    }

    /// Sends a server-bound vehicle-destroyed notification (RPC 136).
    pub fn send_vehicle_destroyed(self, vehicle_id: u16) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::vehicle::SEND_VEHICLE_DESTROYED,
            vehicle_id,
        )
    }

    /// Sends a server-bound vehicle-damage update (RPC 106).
    pub fn send_vehicle_damage(
        self,
        vehicle_id: u16,
        panel_damage: i32,
        door_damage: i32,
        lights: u8,
        tires: u8,
    ) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::damage::SEND_VEHICLE_DAMAGED,
            events::rpc::outgoing::damage::VehicleDamage {
                vehicle_id,
                panel_damage,
                door_damage,
                lights,
                tires,
            },
        )
    }

    /// Sends a server-bound SCM event (RPC 96).
    ///
    /// The values follow SA-MP's wire order: ID, first parameter, second
    /// parameter, then event ID.
    pub fn send_scm_event(
        self,
        event: i32,
        id: i32,
        param1: i32,
        param2: i32,
    ) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::vehicle::SEND_VEHICLE_TUNING,
            events::rpc::outgoing::vehicle::VehicleTuning {
                vehicle_id: id,
                param1,
                param2,
                event,
            },
        )
    }

    /// Sends a server-bound give-damage notification (RPC 115).
    pub fn send_give_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> SampClientSdkResult {
        self.send_damage(player_id, damage, weapon, body_part, false)
    }

    /// Sends a server-bound take-damage notification (RPC 115).
    pub fn send_take_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> SampClientSdkResult {
        self.send_damage(player_id, damage, weapon, body_part, true)
    }

    /// Sends a complete attached-object edit action (RPC 116).
    ///
    /// The typed value deliberately includes both colour fields. SF.lua's
    /// helper leaves them unspecified, so accepting its partial parameter list
    /// here could create malformed or accidentally lossy traffic.
    pub fn send_edit_attached_object(
        self,
        edit: events::rpc::outgoing::object::EditAttachedObject,
    ) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::object::SEND_EDIT_ATTACHED_OBJECT,
            edit,
        )
    }

    /// Sends a complete global or player-object edit action (RPC 117).
    pub fn send_edit_object(
        self,
        edit: events::rpc::outgoing::object::EditObject,
    ) -> SampClientSdkResult {
        self.send_typed_rpc(events::rpc::outgoing::object::SEND_EDIT_OBJECT, edit)
    }

    fn send_typed_rpc<T>(self, descriptor: events::Rpc<T>, value: T) -> SampClientSdkResult {
        self.submit_typed_rpc(descriptor, value)
            .map_or_else(|error| error, |_| SampClientSdkResult::Ok)
    }

    fn send_protocol_rpc<D>(self, descriptor: D, value: D::Value) -> SampClientSdkResult
    where
        D: samp_protocol::WireDescriptor,
    {
        self.submit_protocol_rpc(descriptor, value)
            .map_or_else(|error| error, |_| SampClientSdkResult::Ok)
    }

    fn send_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
        take: bool,
    ) -> SampClientSdkResult {
        self.send_typed_rpc(
            events::rpc::outgoing::damage::SEND_DAMAGE,
            events::rpc::outgoing::damage::Damage {
                player_id,
                damage,
                weapon,
                body_part,
                take,
            },
        )
    }

    pub(crate) fn submit_typed_rpc<T>(
        self,
        descriptor: events::Rpc<T>,
        value: T,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let Ok(payload) = descriptor.encode(self, value) else {
            return Err(SampClientSdkResult::InvalidArgument);
        };
        self.submit_rpc(
            descriptor.id(),
            payload.as_bytes(),
            payload.len_bits(),
            SampClientSdkSendOptions::default(),
        )
    }

    pub(crate) fn submit_protocol_rpc<D>(
        self,
        _descriptor: D,
        value: D::Value,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult>
    where
        D: samp_protocol::WireDescriptor,
    {
        if D::KIND != samp_protocol::WireKind::Rpc {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let Ok(payload) = D::encode_bits(&value) else {
            return Err(SampClientSdkResult::InvalidArgument);
        };
        self.submit_rpc(
            D::ID,
            payload.as_bytes(),
            payload.len_bits(),
            SampClientSdkSendOptions::default(),
        )
    }

    pub(crate) fn submit_typed_packet<T>(
        self,
        descriptor: events::Packet<T>,
        value: T,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let Ok(payload) = descriptor.encode(self, value) else {
            return Err(SampClientSdkResult::InvalidArgument);
        };
        self.submit_packet(
            descriptor.id(),
            payload.as_bytes(),
            payload.len_bits(),
            SampClientSdkSendOptions::default(),
        )
    }

    fn send_typed_packet<T>(self, descriptor: events::Packet<T>, value: T) -> SampClientSdkResult {
        self.submit_typed_packet(descriptor, value)
            .map_or_else(|error| error, |_| SampClientSdkResult::Ok)
    }
}
