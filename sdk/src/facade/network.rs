use crate::{
    CommandReceipt, HostApi, SampClientSdkDirection, SampClientSdkEncodedString,
    SampClientSdkHookAction, SampClientSdkResult, SampClientSdkSendOptions, SendRateKind,
    ServerInfo, Subscription,
};

/// Safe networking and subscription operations.
#[derive(Clone, Copy)]
pub struct Net {
    api: HostApi,
}

impl Net {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    /// Returns whether incoming packet emulation has the host-captured receiver it requires.
    ///
    /// The host updates this after its incoming-RPC detour sees real server traffic. The result
    /// is a copied scalar; safe plugins never receive or dereference a native pointer.
    #[must_use]
    pub fn incoming_emulation_ready(self) -> bool {
        self.api.incoming_emulation_ready()
    }

    /// Queues the R1 send interval for one replication stream.
    pub fn set_send_rate(
        self,
        kind: SendRateKind,
        milliseconds: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_send_rate(kind, milliseconds)
    }

    /// Queues the documented R1 reconnect sequence for a bounded server address.
    pub fn connect(
        self,
        address: &[u8],
        port: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_connect_to_server(address, port)
    }

    /// Queues SF.lua's documented R1 RakClient disconnect and restart sequence.
    pub fn disconnect(
        self,
        block_duration: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_disconnect_with_reason(block_duration)
    }

    pub fn encode_string(
        self,
        value: &[u8],
    ) -> Result<SampClientSdkEncodedString, SampClientSdkResult> {
        self.api.encode_string(value)
    }

    pub fn decode_string(
        self,
        stream: &mut crate::raknet::BitStream,
    ) -> Result<Vec<u8>, SampClientSdkResult> {
        self.api.decode_string(stream)
    }

    #[must_use]
    pub const fn rpc_name(self, id: u8) -> Option<&'static str> {
        crate::raknet::rpc_name(id)
    }

    #[must_use]
    pub const fn packet_name(self, id: u8) -> Option<&'static str> {
        crate::raknet::packet_name(id)
    }

    /// Queues one bounded SA-MP chat or slash-command RPC.
    pub fn send_chat(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        let descriptor = if text.first() == Some(&b'/') {
            crate::events::rpc::outgoing::chat::SEND_COMMAND
        } else {
            crate::events::rpc::outgoing::chat::SEND_CHAT
        };
        self.api.submit_typed_rpc(descriptor, text.to_vec())
    }

    /// Queues the server-bound request-spawn RPC.
    pub fn send_request_spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::session::SEND_REQUEST_SPAWN,
            (),
        )
    }

    /// Queues the protocol-level request-class RPC without changing local class state.
    pub fn send_request_class(
        self,
        class_id: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::session::SEND_REQUEST_CLASS,
            class_id,
        )
    }

    /// Queues the protocol-level interior-change RPC without changing GTA state.
    pub fn send_interior_change(
        self,
        interior_id: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::SEND_INTERIOR_CHANGE,
            interior_id,
        )
    }

    /// Queues the protocol-level spawn RPC without invoking native spawn code.
    pub fn send_spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::session::SEND_SPAWN, ())
    }

    /// Queues the protocol-level enter-vehicle RPC without changing the local ped.
    pub fn send_enter_vehicle(
        self,
        vehicle_id: u16,
        passenger: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::vehicle::SEND_ENTER_VEHICLE,
            crate::events::rpc::outgoing::vehicle::EnterVehicle {
                vehicle_id,
                passenger,
            },
        )
    }

    /// Queues the protocol-level exit-vehicle RPC without changing the local ped.
    pub fn send_exit_vehicle(
        self,
        vehicle_id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::vehicle::SEND_EXIT_VEHICLE,
            vehicle_id,
        )
    }

    /// Queues a server-bound dialog response.
    pub fn send_dialog_response(
        self,
        dialog_id: u16,
        button: u8,
        list_item: u16,
        input: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::ui::SEND_DIALOG_RESPONSE,
            crate::events::rpc::outgoing::ui::DialogResponse {
                dialog_id,
                button,
                list_item,
                input: input.to_vec(),
            },
        )
    }

    /// Queues a server-bound player-click RPC.
    pub fn send_click_player(
        self,
        player_id: u16,
        source: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::ui::SEND_CLICK_PLAYER,
            crate::events::rpc::outgoing::ui::ClickPlayer { player_id, source },
        )
    }

    /// Queues a server-bound textdraw-click RPC.
    pub fn send_click_textdraw(
        self,
        textdraw_id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::ui::SEND_CLICK_TEXT_DRAW,
            textdraw_id,
        )
    }

    /// Queues a server-bound death notification.
    pub fn send_death_by_player(
        self,
        player_id: u16,
        reason: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::SEND_DEATH_NOTIFICATION,
            crate::events::rpc::outgoing::DeathNotification {
                reason,
                killer_id: player_id,
            },
        )
    }

    /// Queues the empty server-bound menu-quit RPC.
    pub fn send_menu_quit(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::ui::SEND_QUIT_MENU, ())
    }

    /// Queues a server-bound menu-row selection.
    pub fn send_menu_select_row(self, row: u8) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::ui::SEND_MENU_SELECT, row)
    }

    /// Queues a server-bound pickup notification.
    pub fn send_picked_up_pickup(
        self,
        pickup_id: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::SEND_PICKED_UP_PICKUP,
            pickup_id,
        )
    }

    /// Queues a server-bound vehicle-destroyed notification.
    pub fn send_vehicle_destroyed(
        self,
        vehicle_id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::vehicle::SEND_VEHICLE_DESTROYED,
            vehicle_id,
        )
    }

    /// Queues a server-bound vehicle-damage update.
    pub fn send_vehicle_damage(
        self,
        vehicle_id: u16,
        panel_damage: i32,
        door_damage: i32,
        lights: u8,
        tires: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::damage::SEND_VEHICLE_DAMAGED,
            crate::events::rpc::outgoing::damage::VehicleDamage {
                vehicle_id,
                panel_damage,
                door_damage,
                lights,
                tires,
            },
        )
    }

    /// Queues a server-bound SCM event.
    pub fn send_scm_event(
        self,
        event: i32,
        id: i32,
        param1: i32,
        param2: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::vehicle::SEND_VEHICLE_TUNING,
            crate::events::rpc::outgoing::vehicle::VehicleTuning {
                vehicle_id: id,
                param1,
                param2,
                event,
            },
        )
    }

    /// Queues a server-bound give-damage notification.
    pub fn send_give_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_damage(player_id, damage, weapon, body_part, false)
    }

    /// Queues a server-bound take-damage notification.
    pub fn send_take_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_damage(player_id, damage, weapon, body_part, true)
    }

    /// Queues a complete attached-object edit action.
    pub fn send_edit_attached_object(
        self,
        edit: crate::events::rpc::outgoing::object::EditAttachedObject,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::object::SEND_EDIT_ATTACHED_OBJECT,
            edit,
        )
    }

    /// Queues a complete global or player-object edit action.
    pub fn send_edit_object(
        self,
        edit: crate::events::rpc::outgoing::object::EditObject,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::object::SEND_EDIT_OBJECT, edit)
    }

    /// Queues a bounded server-bound RCON command packet.
    pub fn send_rcon_command(
        self,
        command: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_packet(
            crate::events::packet::outgoing::SEND_RCON_COMMAND,
            command.to_vec(),
        )
    }

    /// Queues a complete local aim-sync packet.
    pub fn send_aim_sync(
        self,
        sync: crate::events::packet::AimSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_AIM_SYNC, sync)
    }

    /// Queues a complete local bullet-sync packet.
    pub fn send_bullet_sync(
        self,
        sync: crate::events::packet::BulletSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_BULLET_SYNC, sync)
    }

    /// Queues a complete local vehicle-sync packet.
    pub fn send_vehicle_sync(
        self,
        sync: crate::events::packet::VehicleSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_VEHICLE_SYNC, sync)
    }

    /// Queues a complete local on-foot player-sync packet.
    pub fn send_player_sync(
        self,
        sync: crate::events::packet::PlayerSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_PLAYER_SYNC, sync)
    }

    /// Queues a complete local spectator-sync packet.
    pub fn send_spectator_sync(
        self,
        sync: crate::events::packet::SpectatorSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_SPECTATOR_SYNC, sync)
    }

    /// Queues a complete local trailer-sync packet.
    pub fn send_trailer_sync(
        self,
        sync: crate::events::packet::TrailerSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_TRAILER_SYNC, sync)
    }

    /// Queues a complete local passenger-sync packet.
    pub fn send_passenger_sync(
        self,
        sync: crate::events::packet::PassengerSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_PASSENGER_SYNC, sync)
    }

    /// Queues a complete local unoccupied-vehicle sync packet.
    pub fn send_unoccupied_sync(
        self,
        sync: crate::events::packet::UnoccupiedSync,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_packet(crate::events::packet::outgoing::SEND_UNOCCUPIED_SYNC, sync)
    }

    pub fn send_packet(
        self,
        id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_packet_with_options(id, payload, bit_len, SampClientSdkSendOptions::default())
    }

    /// Queues a packet with explicit RakNet delivery options.
    pub fn send_packet_with_options(
        self,
        id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_packet(id, payload, bit_len, options)
    }

    pub fn send_rpc(
        self,
        id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_rpc_with_options(id, payload, bit_len, SampClientSdkSendOptions::default())
    }

    /// Queues an RPC with explicit RakNet delivery options.
    pub fn send_rpc_with_options(
        self,
        id: u8,
        payload: &[u8],
        bit_len: usize,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_rpc(id, payload, bit_len, options)
    }

    pub fn send_packet_stream(
        self,
        id: u8,
        payload: &crate::raknet::BitStream,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_packet(id, payload.as_bytes(), payload.len_bits())
    }

    /// Queues a complete owned packet stream with explicit delivery options.
    pub fn send_packet_stream_with_options(
        self,
        id: u8,
        payload: &crate::raknet::BitStream,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_packet_with_options(id, payload.as_bytes(), payload.len_bits(), options)
    }

    pub fn send_rpc_stream(
        self,
        id: u8,
        payload: &crate::raknet::BitStream,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_rpc(id, payload.as_bytes(), payload.len_bits())
    }

    /// Queues a complete owned RPC stream with explicit delivery options.
    pub fn send_rpc_stream_with_options(
        self,
        id: u8,
        payload: &crate::raknet::BitStream,
        options: SampClientSdkSendOptions,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.send_rpc_with_options(id, payload.as_bytes(), payload.len_bits(), options)
    }

    pub fn emulate_incoming_packet(
        self,
        id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_emulate_incoming_packet(id, payload, bit_len)
    }

    pub fn emulate_incoming_rpc(
        self,
        id: u8,
        payload: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_emulate_incoming_rpc(id, payload, bit_len)
    }

    fn send_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
        take: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::damage::SEND_DAMAGE,
            crate::events::rpc::outgoing::damage::Damage {
                player_id,
                damage,
                weapon,
                body_part,
                take,
            },
        )
    }

    pub fn on_packet<F>(
        self,
        direction: SampClientSdkDirection,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut crate::events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.api.on_packet(direction, handler)
    }

    pub fn on_rpc<F>(
        self,
        direction: SampClientSdkDirection,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut crate::events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.api.on_rpc(direction, handler)
    }

    pub fn on_packet_id<F>(
        self,
        direction: SampClientSdkDirection,
        id: u8,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut crate::events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.api.on_packet_id(direction, id, handler)
    }

    pub fn on_rpc_id<F>(
        self,
        direction: SampClientSdkDirection,
        id: u8,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        F: for<'event> Fn(&mut crate::events::Event<'event>) -> SampClientSdkHookAction
            + Send
            + Sync
            + 'static,
    {
        self.api.on_rpc_id(direction, id, handler)
    }

    pub fn on_typed_packet<T, F>(
        self,
        direction: SampClientSdkDirection,
        packet: crate::events::Packet<T>,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        T: 'static,
        F: Fn(T) -> crate::events::RpcAction<T> + Send + Sync + 'static,
    {
        self.api.on_typed_packet(direction, packet, handler)
    }

    pub fn on_typed_rpc<T, F>(
        self,
        direction: SampClientSdkDirection,
        rpc: crate::events::Rpc<T>,
        handler: F,
    ) -> Result<Subscription, SampClientSdkResult>
    where
        T: 'static,
        F: Fn(T) -> crate::events::RpcAction<T> + Send + Sync + 'static,
    {
        self.api.on_typed_rpc(direction, rpc, handler)
    }
}

/// Safe server metadata reads.
#[derive(Clone, Copy)]
pub struct Server {
    api: HostApi,
}

impl Server {
    pub(super) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }

    pub fn info(self) -> Result<ServerInfo, SampClientSdkResult> {
        self.api.server_info()
    }

    pub fn hostname(self) -> Result<Vec<u8>, SampClientSdkResult> {
        self.info().map(|info| info.hostname)
    }

    pub fn address(self) -> Result<Vec<u8>, SampClientSdkResult> {
        self.info().map(|info| info.address)
    }

    pub fn port(self) -> Result<u16, SampClientSdkResult> {
        self.info().map(|info| info.port)
    }
}

#[cfg(test)]
mod tests {
    use crate::Samp;
    use std::time::Duration;

    #[test]
    fn network_commands_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        assert!(samp.net().incoming_emulation_ready());
        let mut chat = samp.net().send_chat(b"fixture").unwrap();
        assert_eq!(chat.id(), 4);
        assert_eq!(chat.try_take(), Ok(Some(())));

        let mut packet = samp.net().send_packet(207, &[1, 2], 16).unwrap();
        assert_eq!(packet.id(), 4);
        assert_eq!(packet.try_take(), Ok(Some(())));

        let mut rpc = samp
            .net()
            .send_rpc_with_options(61, &[3], 8, crate::SampClientSdkSendOptions::default())
            .unwrap();
        assert_eq!(rpc.id(), 4);
        assert_eq!(rpc.wait(Duration::from_millis(0)), Ok(()));

        let mut emulated = samp.net().emulate_incoming_packet(207, &[4], 8).unwrap();
        assert_eq!(emulated.id(), 5);
        assert_eq!(emulated.try_take(), Ok(Some(())));

        let mut connect = samp.net().connect(b"127.0.0.1", 7777).unwrap();
        assert_eq!(connect.id(), 24);
        assert_eq!(connect.try_take(), Ok(Some(())));

        let mut disconnect = samp.net().disconnect(0).unwrap();
        assert_eq!(disconnect.id(), 25);
        assert_eq!(disconnect.try_take(), Ok(Some(())));
    }
}
