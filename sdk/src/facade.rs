//! The public, owned-value facade over the host ABI.

use crate::{
    ChatEntry, CommandReceipt, Gangzone, HostApi, LocalAnimation, LocalChatDisplayMode,
    LocalChatMessage, LocalCursorMode, LocalDeathMessage, LocalDialog, LocalDialogState,
    LocalPlayer, PlayerInfo, RemotePlayerState, ResolveError, SampClientSdkClientVersion,
    SampClientSdkDirection, SampClientSdkEncodedString, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkResult, SampClientSdkSendOptions, SampGameState,
    SendRateKind, ServerInfo, SpecialAction, Subscription, TextDraw, TextLabel,
    limits::{
        MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES, MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS,
        MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES,
    },
};
use std::time::Duration;

macro_rules! bounded_id {
    ($name:ident, $maximum:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u16);

        impl $name {
            /// Returns `None` when `raw` is outside the R1 pool range.
            #[must_use]
            pub const fn new(raw: u16) -> Option<Self> {
                if raw < $maximum {
                    Some(Self(raw))
                } else {
                    None
                }
            }

            /// Returns the bounded raw SA-MP pool index.
            #[must_use]
            pub const fn get(self) -> u16 {
                self.0
            }
        }
    };
}

bounded_id!(
    PlayerId,
    MAX_SAMP_PLAYERS,
    "A checked SA-MP player-pool ID."
);
bounded_id!(
    VehicleId,
    MAX_SAMP_VEHICLES,
    "A checked SA-MP vehicle-pool ID."
);
bounded_id!(
    TextLabelId,
    MAX_SAMP_TEXT_LABELS,
    "A checked SA-MP 3D text-label ID."
);
bounded_id!(
    TextdrawId,
    MAX_SAMP_TEXTDRAWS,
    "A checked SA-MP textdraw-pool index."
);
bounded_id!(
    ObjectId,
    MAX_SAMP_OBJECTS,
    "A checked SA-MP object-pool ID."
);
bounded_id!(
    GangzoneId,
    MAX_SAMP_GANGZONES,
    "A checked SA-MP gangzone-pool ID."
);

macro_rules! gta_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            /// Returns `None` for the null GTA handle.
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                if raw == 0 { None } else { Some(Self(raw)) }
            }

            /// Returns the raw non-null GTA handle.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

gta_handle!(
    ObjectHandle,
    "A typed non-null GTA SA object handle (GTAREF)."
);
gta_handle!(
    PickupHandle,
    "A typed non-null GTA SA pickup handle (GTAREF)."
);
gta_handle!(VehicleHandle, "A typed non-null GTA SA vehicle handle.");
gta_handle!(PedHandle, "A typed non-null GTA SA ped handle.");

/// Entry point for safe, copied SA-MP client operations.
#[derive(Clone, Copy)]
pub struct Samp {
    api: HostApi,
}

impl Samp {
    /// Connects to the default `samp_client_sdk.asi` host.
    pub fn connect(timeout: Duration) -> Result<Self, ResolveError> {
        crate::wait_for_default_host(timeout).map(|api| Self { api })
    }

    /// Connects to a named host module. `module_name` must be NUL-terminated.
    pub fn connect_to(module_name: &[u8], timeout: Duration) -> Result<Self, ResolveError> {
        crate::wait_for_host(module_name, timeout).map(|api| Self { api })
    }

    /// Returns the host lifecycle state without accessing client memory.
    #[must_use]
    pub fn status(self) -> SampClientSdkHostStatus {
        self.api.status()
    }

    /// Returns lifecycle and recognized-build predicates without reading
    /// client memory. This groups SF.lua's three historical probe helpers
    /// under one explicit host-status view.
    #[must_use]
    pub const fn probe(self) -> Probe {
        Probe { api: self.api }
    }

    /// Returns the recognized SA-MP client version identity.
    pub fn version(self) -> Result<SampClientSdkClientVersion, SampClientSdkResult> {
        self.api.samp_version()
    }

    /// Returns the cached native R1 game-state scalar.
    pub fn game_state(self) -> Result<i32, SampClientSdkResult> {
        self.api.samp_game_state()
    }

    /// Queues one validated R1 CNetGame-state write on the game thread.
    pub fn set_game_state(
        self,
        state: SampGameState,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_samp_game_state(state)
    }

    #[must_use]
    pub fn net(self) -> Net {
        Net { api: self.api }
    }

    #[must_use]
    pub fn server(self) -> Server {
        Server { api: self.api }
    }

    #[must_use]
    pub fn local(self) -> Local {
        Local { api: self.api }
    }

    #[must_use]
    pub fn players(self) -> Players {
        Players { api: self.api }
    }

    #[must_use]
    pub fn textdraws(self) -> Textdraws {
        Textdraws { api: self.api }
    }

    #[must_use]
    pub fn labels(self) -> Labels {
        Labels { api: self.api }
    }

    #[must_use]
    pub fn objects(self) -> Objects {
        Objects { api: self.api }
    }

    #[must_use]
    pub fn pickups(self) -> Pickups {
        Pickups { api: self.api }
    }

    #[must_use]
    pub fn vehicles(self) -> Vehicles {
        Vehicles { api: self.api }
    }

    #[must_use]
    pub fn gangzones(self) -> Gangzones {
        Gangzones { api: self.api }
    }

    #[must_use]
    pub fn dialogs(self) -> Dialogs {
        Dialogs { api: self.api }
    }

    #[must_use]
    pub fn chat(self) -> Chat {
        Chat { api: self.api }
    }

    #[must_use]
    pub fn chat_input(self) -> ChatInput {
        ChatInput { api: self.api }
    }

    #[must_use]
    pub fn cursor(self) -> Cursor {
        Cursor { api: self.api }
    }

    #[must_use]
    pub fn scoreboard(self) -> Scoreboard {
        Scoreboard { api: self.api }
    }

    #[must_use]
    pub fn anim(self) -> Anim {
        Anim { api: self.api }
    }

    pub(crate) const fn api(self) -> HostApi {
        self.api
    }

    #[cfg(test)]
    pub(crate) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }
}

/// Safe host and recognized-build probes.
#[derive(Clone, Copy)]
pub struct Probe {
    api: HostApi,
}

impl Probe {
    /// Returns whether the host has attached to a recognized `samp.dll`.
    #[must_use]
    pub fn is_samp_loaded(self) -> bool {
        self.api.is_samp_loaded()
    }

    /// Returns whether the SDK recognizes the loaded SA-MP build.
    #[must_use]
    pub fn is_sampfuncs_lua_loaded(self) -> bool {
        self.api.samp_version().is_ok()
    }

    /// Returns whether the recognized client and its RakClient hooks are ready.
    #[must_use]
    pub fn is_samp_available(self) -> bool {
        self.api.is_samp_available()
    }
}

/// Safe networking and subscription operations.
#[derive(Clone, Copy)]
pub struct Net {
    api: HostApi,
}

impl Net {
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
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_REQUEST_SPAWN, ())
    }

    /// Queues the protocol-level request-class RPC without changing local class state.
    pub fn send_request_class(
        self,
        class_id: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_REQUEST_CLASS, class_id)
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
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_SPAWN, ())
    }

    /// Queues the protocol-level enter-vehicle RPC without changing the local ped.
    pub fn send_enter_vehicle(
        self,
        vehicle_id: u16,
        passenger: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::SEND_ENTER_VEHICLE,
            crate::events::rpc::outgoing::EnterVehicle {
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
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_EXIT_VEHICLE, vehicle_id)
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
            crate::events::rpc::outgoing::SEND_DIALOG_RESPONSE,
            crate::events::rpc::outgoing::DialogResponse {
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
            crate::events::rpc::outgoing::SEND_CLICK_PLAYER,
            crate::events::rpc::outgoing::ClickPlayer { player_id, source },
        )
    }

    /// Queues a server-bound textdraw-click RPC.
    pub fn send_click_textdraw(
        self,
        textdraw_id: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::SEND_CLICK_TEXT_DRAW,
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
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_QUIT_MENU, ())
    }

    /// Queues a server-bound menu-row selection.
    pub fn send_menu_select_row(self, row: u8) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_MENU_SELECT, row)
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
            crate::events::rpc::outgoing::SEND_VEHICLE_DESTROYED,
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
            crate::events::rpc::outgoing::SEND_VEHICLE_DAMAGED,
            crate::events::rpc::outgoing::VehicleDamage {
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
            crate::events::rpc::outgoing::SEND_VEHICLE_TUNING,
            crate::events::rpc::outgoing::VehicleTuning {
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
        edit: crate::events::rpc::outgoing::EditAttachedObject,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_typed_rpc(
            crate::events::rpc::outgoing::SEND_EDIT_ATTACHED_OBJECT,
            edit,
        )
    }

    /// Queues a complete global or player-object edit action.
    pub fn send_edit_object(
        self,
        edit: crate::events::rpc::outgoing::EditObject,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_typed_rpc(crate::events::rpc::outgoing::SEND_EDIT_OBJECT, edit)
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
            crate::events::rpc::outgoing::SEND_DAMAGE,
            crate::events::rpc::outgoing::Damage {
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

#[derive(Clone, Copy)]
pub struct Server {
    api: HostApi,
}

impl Server {
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

#[derive(Clone, Copy)]
pub struct Local {
    api: HostApi,
}

impl Local {
    pub fn player(self) -> Result<LocalPlayer, SampClientSdkResult> {
        self.api.local_player()
    }

    /// Queues the R1 local-player spawn path.
    pub fn spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_player_spawn()
    }

    /// Queues one established R1 local-player special action.
    pub fn set_special_action(
        self,
        action: SpecialAction,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_player_special_action(action)
    }

    /// Queues a documented R1 local-player nickname update.
    pub fn set_nickname(self, name: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_player_name(name)
    }

    /// Queues the documented R1 unoccupied-vehicle synchronization send.
    pub fn force_unoccupied_sync(
        self,
        vehicle: VehicleId,
        seat: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_force_unoccupied_sync(vehicle.get(), seat)
    }

    /// Queues the protocol-level class request without changing local class state.
    pub fn request_class(self, class_id: i32) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net { api: self.api }.send_request_class(class_id)
    }

    /// Queues the protocol-level interior-change RPC without changing GTA state.
    pub fn send_interior_change(
        self,
        interior_id: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net { api: self.api }.send_interior_change(interior_id)
    }

    /// Queues the protocol-level spawn RPC without invoking native spawn code.
    pub fn send_spawn(self) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net { api: self.api }.send_spawn()
    }

    /// Queues the protocol-level enter-vehicle RPC without changing the local ped.
    pub fn send_enter_vehicle(
        self,
        vehicle: VehicleId,
        passenger: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net { api: self.api }.send_enter_vehicle(vehicle.get(), passenger)
    }

    /// Queues the protocol-level exit-vehicle RPC without changing the local ped.
    pub fn send_exit_vehicle(
        self,
        vehicle: VehicleId,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        Net { api: self.api }.send_exit_vehicle(vehicle.get())
    }
}

#[derive(Clone, Copy)]
pub struct Players {
    api: HostApi,
}

impl Players {
    #[must_use]
    pub fn player(self, id: PlayerId) -> Player {
        Player { api: self.api, id }
    }

    pub fn get(self, id: PlayerId) -> Result<Option<PlayerInfo>, SampClientSdkResult> {
        self.api.player_info(id.get())
    }

    pub fn remote_state(
        self,
        id: PlayerId,
    ) -> Result<Option<RemotePlayerState>, SampClientSdkResult> {
        self.api.remote_player_state(id.get())
    }

    pub fn is_defined(self, id: PlayerId) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_defined(id.get())
    }

    pub fn is_paused(self, id: PlayerId) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_paused(id.get())
    }

    pub fn count(self, include_npcs: bool) -> Result<u16, SampClientSdkResult> {
        self.api.player_count(include_npcs)
    }

    pub fn max_id(self) -> Result<Option<PlayerId>, SampClientSdkResult> {
        self.api.player_max_id().map(PlayerId::new)
    }
}

/// Safe, nonblocking view of one checked SA-MP player-pool entry.
#[derive(Clone, Copy)]
pub struct Player {
    api: HostApi,
    id: PlayerId,
}

impl Player {
    #[must_use]
    pub const fn id(self) -> PlayerId {
        self.id
    }

    pub fn is_connected(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_connected(self.id.get())
    }

    pub fn nickname(self) -> Result<Option<Vec<u8>>, SampClientSdkResult> {
        self.api.player_nickname(self.id.get())
    }

    pub fn is_npc(self) -> Result<Option<bool>, SampClientSdkResult> {
        self.api.is_player_npc(self.id.get())
    }

    pub fn score(self) -> Result<Option<i32>, SampClientSdkResult> {
        self.api.player_score(self.id.get())
    }

    pub fn ping(self) -> Result<Option<u32>, SampClientSdkResult> {
        self.api.player_ping(self.id.get())
    }

    pub fn armour(self) -> Result<Option<f32>, SampClientSdkResult> {
        self.api.player_armour(self.id.get())
    }

    pub fn health(self) -> Result<Option<f32>, SampClientSdkResult> {
        self.api.player_health(self.id.get())
    }

    pub fn is_paused(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_paused(self.id.get())
    }

    pub fn special_action(self) -> Result<Option<u8>, SampClientSdkResult> {
        self.api.player_special_action(self.id.get())
    }

    pub fn animation_id(self) -> Result<Option<u16>, SampClientSdkResult> {
        self.api.player_animation_id(self.id.get())
    }

    pub fn is_defined(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_player_defined(self.id.get())
    }

    pub fn colour(self) -> Result<Option<u32>, SampClientSdkResult> {
        self.api.player_colour(self.id.get())
    }

    /// Queues a documented R1 local- or remote-player colour change.
    pub fn set_colour(self, colour: u32) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_player_colour(self.id.get(), colour)
    }

    /// Returns the cached GTA SA ped handle for this player.
    pub fn ped_handle(self) -> Result<Option<PedHandle>, SampClientSdkResult> {
        self.api
            .player_ped_handle(self.id.get())
            .map(|handle| handle.and_then(|handle| PedHandle::new(handle as u32)))
    }
}

impl PedHandle {
    /// Resolves this GTA SA ped handle back to a checked player-pool ID.
    pub fn to_id(self, samp: Samp) -> Result<Option<PlayerId>, SampClientSdkResult> {
        samp.api()
            .player_id_by_ped_handle(self.get() as i32)
            .map(|id| id.and_then(PlayerId::new))
    }
}

#[derive(Clone, Copy)]
pub struct Textdraws {
    api: HostApi,
}

impl Textdraws {
    pub fn exists(self, id: TextdrawId) -> Result<bool, SampClientSdkResult> {
        self.api.is_textdraw_defined(id.get())
    }

    pub fn get(self, id: TextdrawId) -> Result<Option<TextDraw>, SampClientSdkResult> {
        self.api.textdraw(id.get())
    }

    /// Queues the documented R1 textdraw-pool deletion.
    pub fn delete(self, id: TextdrawId) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_delete_textdraw(id.get())
    }

    /// Queues a finite R1 textdraw screen-position update.
    pub fn set_position(
        self,
        id: TextdrawId,
        x: f32,
        y: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_position(id.get(), x, y)
    }

    /// Queues finite R1 textdraw letter dimensions and a native colour value.
    pub fn set_letter_style(
        self,
        id: TextdrawId,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_letter_style(id.get(), width, height, colour)
    }

    /// Queues an R1 textdraw proportional-flag update.
    pub fn set_proportional(
        self,
        id: TextdrawId,
        proportional: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_proportional(id.get(), proportional)
    }

    /// Queues an R1 textdraw shadow and background-colour update.
    pub fn set_shadow(
        self,
        id: TextdrawId,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_shadow(id.get(), shadow, colour)
    }

    /// Queues an R1 textdraw outline and background-colour update.
    pub fn set_outline(
        self,
        id: TextdrawId,
        outline: u8,
        colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_outline(id.get(), outline, colour)
    }

    /// Queues a finite R1 textdraw box update.
    pub fn set_box(
        self,
        id: TextdrawId,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_box(id.get(), enabled, colour, width, height)
    }

    /// Queues a validated R1 textdraw alignment update (1 left, 2 centre, 3 right).
    pub fn set_alignment(
        self,
        id: TextdrawId,
        alignment: u8,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_alignment(id.get(), alignment)
    }

    /// Queues a bounded R1 textdraw display-string update.
    pub fn set_text(
        self,
        id: TextdrawId,
        text: &[u8],
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_set_textdraw_string(id.get(), text)
    }

    /// Queues a finite R1 textdraw model rotation, zoom, and vehicle-colour update.
    pub fn set_model_style(
        self,
        id: TextdrawId,
        rotation: crate::Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_set_textdraw_model_style(id.get(), rotation, zoom, colour1, colour2)
    }
}

#[derive(Clone, Copy)]
pub struct Labels {
    api: HostApi,
}

impl Labels {
    pub fn exists(self, id: TextLabelId) -> Result<bool, SampClientSdkResult> {
        self.api.is_text_label_defined(id.get())
    }

    pub fn get(self, id: TextLabelId) -> Result<Option<TextLabel>, SampClientSdkResult> {
        self.api.text_label(id.get())
    }

    /// Queues deletion of this documented R1 3D text-label-pool entry.
    pub fn delete(self, id: TextLabelId) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_delete_text_label(id.get())
    }

    /// Queues creation of one R1 3D text label at a caller-selected pool ID.
    #[allow(clippy::too_many_arguments)]
    pub fn create_at(
        self,
        id: TextLabelId,
        text: &[u8],
        colour: u32,
        position: crate::Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: Option<PlayerId>,
        attached_vehicle_id: Option<VehicleId>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_create_text_label(
            id.get(),
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id.map(PlayerId::get),
            attached_vehicle_id.map(VehicleId::get),
        )
    }
}

#[derive(Clone, Copy)]
pub struct Objects {
    api: HostApi,
}

impl Objects {
    pub fn exists(self, id: ObjectId) -> Result<bool, SampClientSdkResult> {
        self.api.is_object_defined(id.get())
    }

    /// Returns the cached GTA SA object handle for a checked object ID.
    pub fn handle(self, id: ObjectId) -> Result<Option<ObjectHandle>, SampClientSdkResult> {
        self.api
            .object_handle(id.get())
            .map(|handle| handle.and_then(|handle| ObjectHandle::new(handle as u32)))
    }
}

impl ObjectHandle {
    /// Resolves this GTA SA object handle back to a checked object-pool ID.
    pub fn to_id(self, samp: Samp) -> Result<Option<ObjectId>, SampClientSdkResult> {
        samp.api()
            .object_id_by_handle(self.get() as i32)
            .map(|id| id.and_then(ObjectId::new))
    }
}

/// Placeholder for the pickup facade. No pickup read or mutation has crossed
/// the fixed R1 native boundary yet.
#[derive(Clone, Copy)]
pub struct Pickups {
    api: HostApi,
}

impl Pickups {
    /// Returns the cached GTA SA pickup handle for a raw pickup-pool index.
    pub fn handle(self, id: u16) -> Result<Option<PickupHandle>, SampClientSdkResult> {
        self.api
            .pickup_handle(id)
            .map(|handle| handle.and_then(|handle| PickupHandle::new(handle as u32)))
    }
}

impl PickupHandle {
    /// Resolves this GTA SA pickup handle back to a pickup-pool index.
    pub fn to_id(self, samp: Samp) -> Result<Option<u16>, SampClientSdkResult> {
        samp.api().pickup_id_by_handle(self.get() as i32)
    }
}

#[derive(Clone, Copy)]
pub struct Vehicles {
    api: HostApi,
}

impl Vehicles {
    pub fn exists(self, id: VehicleId) -> Result<bool, SampClientSdkResult> {
        self.api.is_vehicle_defined(id.get())
    }

    /// Returns the cached GTA SA vehicle handle for a checked vehicle ID.
    pub fn handle(self, id: VehicleId) -> Result<Option<VehicleHandle>, SampClientSdkResult> {
        self.api
            .vehicle_handle(id.get())
            .map(|handle| handle.and_then(|handle| VehicleHandle::new(handle as u32)))
    }
}

impl VehicleHandle {
    /// Resolves this GTA SA vehicle handle back to a checked vehicle-pool ID.
    pub fn to_id(self, samp: Samp) -> Result<Option<VehicleId>, SampClientSdkResult> {
        samp.api()
            .vehicle_id_by_handle(self.get() as i32)
            .map(|id| id.and_then(VehicleId::new))
    }
}

#[derive(Clone, Copy)]
pub struct Gangzones {
    api: HostApi,
}

impl Gangzones {
    pub fn get(self, id: GangzoneId) -> Result<Option<Gangzone>, SampClientSdkResult> {
        self.api.gangzone(id.get())
    }
}

#[derive(Clone, Copy)]
pub struct Dialogs {
    api: HostApi,
}

impl Dialogs {
    pub fn active(self) -> Result<Option<LocalDialogState>, SampClientSdkResult> {
        self.api.active_local_dialog()
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_dialog_active()
    }

    /// Returns the copied selected index for an active R1 list dialog.
    pub fn selected_item(self) -> Result<i32, SampClientSdkResult> {
        self.api.local_dialog_selected_item()
    }

    /// Returns the copied count of items in the active R1 dialog list.
    pub fn list_item_count(self) -> Result<i32, SampClientSdkResult> {
        self.api.local_dialog_list_item_count()
    }

    /// Queues selection of an item in the active R1 list dialog.
    pub fn set_selected_item(
        self,
        selected: i32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog_selected_item(selected)
    }

    pub fn show(self, dialog: LocalDialog<'_>) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog(dialog)
    }

    /// Queues an R1 write that marks the current dialog as client-side or
    /// server-side on the game thread.
    pub fn set_client_side(
        self,
        client_side: bool,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_dialog_client_side(client_side)
    }

    /// Queues closure of the active R1 dialog with its first (`0`) or second
    /// (`1`) response button.
    pub fn close_with_button(self, button: u8) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if button > 1 {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        self.api.submit_local_dialog_close(button)
    }

    /// Queues a bounded R1 dialog editbox text replacement on the game thread.
    pub fn set_editbox_text(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        if text.len() > MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES || text.contains(&0) {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        self.api.submit_local_dialog_editbox_text(text)
    }
}

#[derive(Clone, Copy)]
pub struct Chat {
    api: HostApi,
}

impl Chat {
    pub fn display_mode(self) -> Result<LocalChatDisplayMode, SampClientSdkResult> {
        self.api.local_chat_display_mode()
    }

    /// Returns one copied fixed R1 chat-history entry.
    pub fn entry(self, id: u16) -> Result<ChatEntry, SampClientSdkResult> {
        self.api.chat_entry(id)
    }

    /// Queues one R1 chat display-mode write.
    pub fn set_display_mode(
        self,
        mode: LocalChatDisplayMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_display_mode(mode)
    }

    /// Queues one bounded R1 chat-history entry replacement.
    pub fn set_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api
            .submit_local_chat_entry(id, text, prefix, text_colour, prefix_colour)
    }

    pub fn is_visible(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_chat_visible()
    }

    #[allow(clippy::should_implement_trait)] // Mirrors the documented `Chat::add` SDK verb.
    pub fn add(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_message(message)
    }

    /// Alias for [`Self::add`] that emphasizes the request's explicit native style.
    #[allow(clippy::should_implement_trait)] // Mirrors the documented `Chat::add_with_style` SDK verb.
    pub fn add_with_style(
        self,
        message: LocalChatMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.add(message)
    }

    pub fn death_window(self) -> DeathWindow {
        DeathWindow { api: self.api }
    }
}

#[derive(Clone, Copy)]
pub struct DeathWindow {
    api: HostApi,
}

/// Safe cached state for SA-MP's local chat-input UI.
#[derive(Clone, Copy)]
pub struct ChatInput {
    api: HostApi,
}

impl ChatInput {
    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_chat_input_active()
    }

    /// Returns the owned game-thread-cached R1 chat-input text.
    pub fn text(self) -> Result<Vec<u8>, SampClientSdkResult> {
        self.api.local_chat_input_text()
    }

    /// Queues a copied R1 chat-input text update. Text is limited to 128 bytes
    /// and cannot contain an interior NUL.
    pub fn set_text(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_text(text)
    }

    /// Queues R1's native chat-input open or close transition.
    pub fn set_enabled(self, enabled: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_enabled(enabled)
    }

    /// Queues a copied R1 chat-input text update followed by native command
    /// processing. Text is limited to 128 bytes and cannot contain an interior
    /// NUL.
    pub fn process(self, text: &[u8]) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_chat_input_process(text)
    }
}

impl DeathWindow {
    #[allow(clippy::should_implement_trait)] // Mirrors the documented death-window `add` verb.
    pub fn add(
        self,
        message: LocalDeathMessage<'_>,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_death_message(message)
    }
}

#[derive(Clone, Copy)]
pub struct Cursor {
    api: HostApi,
}

impl Cursor {
    pub fn mode(self) -> Result<LocalCursorMode, SampClientSdkResult> {
        self.api.local_cursor_mode()
    }

    pub fn is_active(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_cursor_active()
    }

    /// Queues one validated R1 cursor-mode change on the game thread.
    pub fn set_mode(
        self,
        mode: LocalCursorMode,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_cursor_mode(mode)
    }

    /// Queues SF.lua-compatible R1 cursor visibility behavior, including input
    /// re-enabling when hiding the cursor.
    pub fn toggle(self, show: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_cursor_toggle(show)
    }
}

#[derive(Clone, Copy)]
pub struct Scoreboard {
    api: HostApi,
}

impl Scoreboard {
    pub fn is_open(self) -> Result<bool, SampClientSdkResult> {
        self.api.is_local_scoreboard_open()
    }

    /// Queues one R1 scoreboard visibility change on the game thread.
    pub fn toggle(self, open: bool) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_local_scoreboard_open(open)
    }
}

#[derive(Clone, Copy)]
pub struct Anim {
    api: HostApi,
}

/// Compatibility spelling for the `samp.anim()` facade.
pub type Animations = Anim;

impl Anim {
    pub fn get(self, id: u16) -> Result<LocalAnimation, SampClientSdkResult> {
        self.api.local_animation(id)
    }

    pub fn find(self, name: &[u8], file: &[u8]) -> Result<Option<u16>, SampClientSdkResult> {
        self.api.local_animation_id(name, file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_ids_reject_pool_bounds() {
        assert_eq!(
            PlayerId::new(MAX_SAMP_PLAYERS - 1).map(PlayerId::get),
            Some(1003)
        );
        assert_eq!(PlayerId::new(MAX_SAMP_PLAYERS), None);
        assert_eq!(VehicleId::new(MAX_SAMP_VEHICLES), None);
        assert_eq!(TextLabelId::new(MAX_SAMP_TEXT_LABELS), None);
        assert_eq!(TextdrawId::new(MAX_SAMP_TEXTDRAWS), None);
        assert_eq!(ObjectId::new(MAX_SAMP_OBJECTS), None);
        assert_eq!(GangzoneId::new(MAX_SAMP_GANGZONES), None);
    }

    #[test]
    fn gta_handles_reject_the_null_value() {
        assert_eq!(ObjectHandle::new(0), None);
        assert_eq!(PickupHandle::new(0), None);
        assert_eq!(VehicleHandle::new(0), None);
        assert_eq!(PedHandle::new(0), None);
        assert_eq!(ObjectHandle::new(42).map(ObjectHandle::get), Some(42));
        assert_eq!(PickupHandle::new(42).map(PickupHandle::get), Some(42));
        assert_eq!(VehicleHandle::new(42).map(VehicleHandle::get), Some(42));
        assert_eq!(PedHandle::new(42).map(PedHandle::get), Some(42));
    }

    #[test]
    fn handle_lookups_route_through_the_mock_abi_and_round_trip() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let object_id = ObjectId::new(7).unwrap();
        let object_handle = samp.objects().handle(object_id).unwrap().unwrap();
        assert_eq!(object_handle.get(), 0x1007);
        assert_eq!(object_handle.to_id(samp).unwrap(), Some(object_id));

        let pickup_handle = samp.pickups().handle(7).unwrap().unwrap();
        assert_eq!(pickup_handle.get(), 0x2007);
        assert_eq!(pickup_handle.to_id(samp).unwrap(), Some(7));

        let vehicle_id = VehicleId::new(7).unwrap();
        let vehicle_handle = samp.vehicles().handle(vehicle_id).unwrap().unwrap();
        assert_eq!(vehicle_handle.get(), 0x3007);
        assert_eq!(vehicle_handle.to_id(samp).unwrap(), Some(vehicle_id));

        let player_id = PlayerId::new(7).unwrap();
        let ped_handle = samp
            .players()
            .player(player_id)
            .ped_handle()
            .unwrap()
            .unwrap();
        assert_eq!(ped_handle.get(), 0x4007);
        assert_eq!(ped_handle.to_id(samp).unwrap(), Some(player_id));
    }

    #[test]
    fn facade_reads_route_to_the_mock_abi() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        assert!(samp.probe().is_samp_loaded());
        assert!(samp.probe().is_sampfuncs_lua_loaded());
        assert!(samp.probe().is_samp_available());
        assert_eq!(samp.version(), Ok(SampClientSdkClientVersion::R1));
        assert_eq!(samp.game_state(), Ok(14));
        assert_eq!(samp.server().info().map(|info| info.port), Ok(7777));
        assert_eq!(samp.local().player().map(|player| player.id()), Ok(42));
        assert_eq!(samp.players().count(true), Ok(3));
        assert_eq!(
            samp.players().player(PlayerId::new(7).unwrap()).nickname(),
            Ok(Some(b"remote".to_vec()))
        );
        assert_eq!(
            samp.textdraws().exists(TextdrawId::new(7).unwrap()),
            Ok(true)
        );
        assert_eq!(
            samp.textdraws()
                .get(TextdrawId::new(7).unwrap())
                .map(|value| value.map(|value| (value.letter_style(), value.position()))),
            Ok(Some(((1.0, 2.0, 0xFF11_2233), (3.0, 4.0))))
        );
        assert_eq!(samp.labels().exists(TextLabelId::new(7).unwrap()), Ok(true));
        assert_eq!(
            samp.labels()
                .delete(TextLabelId::new(7).unwrap())
                .map(|receipt| receipt.id()),
            Ok(36)
        );
        assert_eq!(
            samp.labels()
                .create_at(
                    TextLabelId::new(7).unwrap(),
                    b"fixture",
                    0xFF11_2233,
                    crate::Vector3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0
                    },
                    25.0,
                    true,
                    Some(PlayerId::new(8).unwrap()),
                    None,
                )
                .map(|receipt| receipt.id()),
            Ok(39)
        );
        assert_eq!(samp.dialogs().list_item_count(), Ok(3));
        assert_eq!(
            samp.chat().entry(7).map(|entry| (entry.text, entry.prefix)),
            Ok((b"fixture".to_vec(), b"prefix".to_vec()))
        );
        assert_eq!(samp.objects().exists(ObjectId::new(7).unwrap()), Ok(true));
        assert_eq!(samp.vehicles().exists(VehicleId::new(7).unwrap()), Ok(true));
        assert_eq!(samp.chat_input().is_active(), Ok(false));
        assert_eq!(
            samp.dialogs().active().map(|dialog| dialog.map(|dialog| (
                dialog.id(),
                dialog.style(),
                dialog.caption().to_vec(),
                dialog.is_client_side(),
                dialog.text().to_vec(),
                dialog.editbox_text().map(<[u8]>::to_vec),
                dialog.items().to_vec()
            ))),
            Ok(Some((
                7,
                crate::LocalDialogStyle::Input,
                b"fixture".to_vec(),
                true,
                b"fixture".to_vec(),
                Some(b"fixture".to_vec()),
                vec![b"fixture".to_vec(); 3]
            )))
        );
        assert_eq!(
            samp.anim().get(0).map(|animation| animation.name),
            Ok(b"AIRPORT".to_vec())
        );
        assert_eq!(samp.anim().find(b"AIRPORT", b"THRW_BARL_THRW"), Ok(Some(0)));
        assert_eq!(samp.net().rpc_name(61), Some("ShowDialog"));
        assert_eq!(samp.net().packet_name(207), Some("PLAYER_SYNC"));
        assert_eq!(
            samp.net()
                .encode_string(b"ok")
                .map(|value| value.len_bits()),
            Ok(32)
        );
        let mut stream = crate::raknet::BitStream::from_bits([0b1010_0000], 3).unwrap();
        assert_eq!(
            samp.net().decode_string(&mut stream),
            Ok(b"fixture".to_vec())
        );
        let _ = samp.pickups();
    }

    #[test]
    fn network_commands_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut chat = samp.net().send_chat(b"fixture").unwrap();
        assert_eq!(chat.id(), 4);
        assert_eq!(chat.try_take(), Ok(Some(())));

        let mut packet = samp.net().send_packet(207, &[1, 2], 16).unwrap();
        assert_eq!(packet.id(), 4);
        assert_eq!(packet.try_take(), Ok(Some(())));

        let mut rpc = samp
            .net()
            .send_rpc_with_options(61, &[3], 8, SampClientSdkSendOptions::default())
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

    #[test]
    fn local_protocol_actions_delegate_to_the_receipt_bearing_network_path() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let vehicle = VehicleId::new(7).unwrap();
        for mut receipt in [
            samp.local().request_class(3).unwrap(),
            samp.local().send_interior_change(1).unwrap(),
            samp.local().send_spawn().unwrap(),
            samp.local().send_enter_vehicle(vehicle, false).unwrap(),
            samp.local().send_exit_vehicle(vehicle).unwrap(),
        ] {
            assert_eq!(receipt.id(), 4);
            assert_eq!(receipt.try_take(), Ok(Some(())));
        }
    }

    #[test]
    fn cursor_mode_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.cursor().set_mode(LocalCursorMode::LockCamera).unwrap();
        assert_eq!(receipt.id(), 6);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn cursor_toggle_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.cursor().toggle(false).unwrap();
        assert_eq!(receipt.id(), 14);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn chat_display_mode_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .chat()
            .set_display_mode(LocalChatDisplayMode::NoShadow)
            .unwrap();
        assert_eq!(receipt.id(), 15);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn chat_input_mutations_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());

        let mut text = samp.chat_input().set_text(b"/sdk").unwrap();
        assert_eq!(text.id(), 17);
        assert_eq!(text.try_take(), Ok(Some(())));

        let mut enabled = samp.chat_input().set_enabled(true).unwrap();
        assert_eq!(enabled.id(), 18);
        assert_eq!(enabled.try_take(), Ok(Some(())));

        let mut processed = samp.chat_input().process(b"/sdk").unwrap();
        assert_eq!(processed.id(), 19);
        assert_eq!(processed.try_take(), Ok(Some(())));
    }

    #[test]
    fn chat_input_text_is_an_owned_cached_value() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        assert_eq!(samp.chat_input().text(), Ok(b"/sdk".to_vec()));
    }

    #[test]
    fn textdraw_delete_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .textdraws()
            .delete(TextdrawId::new(7).unwrap())
            .unwrap();
        assert_eq!(receipt.id(), 26);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_position_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .textdraws()
            .set_position(TextdrawId::new(7).unwrap(), 12.5, 34.0)
            .unwrap();
        assert_eq!(receipt.id(), 27);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn textdraw_letter_style_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp
            .textdraws()
            .set_letter_style(TextdrawId::new(7).unwrap(), 1.25, 2.5, 0xFF11_2233)
            .unwrap();
        assert_eq!(receipt.id(), 28);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn scoreboard_toggle_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.scoreboard().toggle(true).unwrap();
        assert_eq!(receipt.id(), 7);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn dialog_client_side_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.dialogs().set_client_side(true).unwrap();
        assert_eq!(receipt.id(), 8);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn dialog_close_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.dialogs().close_with_button(1).unwrap();
        assert_eq!(receipt.id(), 16);
        assert_eq!(receipt.try_take(), Ok(Some(())));
        assert!(matches!(
            samp.dialogs().close_with_button(2),
            Err(SampClientSdkResult::InvalidArgument)
        ));
    }

    #[test]
    fn dialog_editbox_mutation_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.dialogs().set_editbox_text(b"fixture").unwrap();
        assert_eq!(receipt.id(), 40);
        assert_eq!(receipt.try_take(), Ok(Some(())));
        assert!(matches!(
            samp.dialogs().set_editbox_text(&[0]),
            Err(SampClientSdkResult::InvalidArgument)
        ));
    }

    #[test]
    fn game_state_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.set_game_state(SampGameState::Connected).unwrap();
        assert_eq!(receipt.id(), 10);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn local_player_mutations_and_send_rate_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());

        let mut spawn = samp.local().spawn().unwrap();
        assert_eq!(spawn.id(), 11);
        assert_eq!(spawn.try_take(), Ok(Some(())));

        let mut special_action = samp
            .local()
            .set_special_action(SpecialAction::HandsUp)
            .unwrap();
        assert_eq!(special_action.id(), 12);
        assert_eq!(special_action.try_take(), Ok(Some(())));

        let mut send_rate = samp.net().set_send_rate(SendRateKind::Aim, 25).unwrap();
        assert_eq!(send_rate.id(), 13);
        assert_eq!(send_rate.try_take(), Ok(Some(())));

        let mut colour = samp
            .players()
            .player(PlayerId::new(7).unwrap())
            .set_colour(0xFF00_00FF)
            .unwrap();
        assert_eq!(colour.id(), 21);
        assert_eq!(colour.try_take(), Ok(Some(())));

        let mut nickname = samp.local().set_nickname(b"fixture").unwrap();
        assert_eq!(nickname.id(), 22);
        assert_eq!(nickname.try_take(), Ok(Some(())));

        let mut unoccupied = samp
            .local()
            .force_unoccupied_sync(VehicleId::new(7).unwrap(), 1)
            .unwrap();
        assert_eq!(unoccupied.id(), 23);
        assert_eq!(unoccupied.try_take(), Ok(Some(())));
    }
}
