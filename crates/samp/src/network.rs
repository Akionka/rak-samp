use crate::{CommandReceipt, ProtocolSendError, SendRateKind, Subscription};
use modkit_abi::{
    ModResult, SAMP_NET_ACTION_BLOCK, SAMP_NET_ACTION_CONTINUE, SAMP_NET_DIRECTION_INCOMING,
    SAMP_NET_DIRECTION_OUTGOING, SAMP_NET_PRIORITY_HIGH, SAMP_NET_RELIABILITY_RELIABLE,
    SAMP_NET_RELIABILITY_RELIABLE_ORDERED, SAMP_NET_RELIABILITY_UNRELIABLE_SEQUENCED,
    SampNetEventCallbackV1, SampNetEventV1, SampNetSendOptionsV1,
};
use modkit_sdk::{Core, SampCodecService, SampControlService, SampNetService};
use std::{
    ffi::c_void,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Incoming,
    Outgoing,
}

impl Direction {
    const fn raw(self) -> u32 {
        match self {
            Self::Incoming => SAMP_NET_DIRECTION_INCOMING,
            Self::Outgoing => SAMP_NET_DIRECTION_OUTGOING,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Continue,
    Block,
}

impl Action {
    const fn raw(self) -> u32 {
        match self {
            Self::Continue => SAMP_NET_ACTION_CONTINUE,
            Self::Block => SAMP_NET_ACTION_BLOCK,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SendOptions {
    pub priority: u32,
    pub reliability: u32,
    pub ordering_channel: u8,
    pub timestamp: bool,
}

impl Default for SendOptions {
    fn default() -> Self {
        Self {
            priority: SAMP_NET_PRIORITY_HIGH,
            reliability: SAMP_NET_RELIABILITY_RELIABLE_ORDERED,
            ordering_channel: 0,
            timestamp: false,
        }
    }
}

impl From<SendOptions> for SampNetSendOptionsV1 {
    fn from(value: SendOptions) -> Self {
        Self {
            priority: value.priority,
            reliability: value.reliability,
            ordering_channel: value.ordering_channel,
            timestamp: u8::from(value.timestamp),
            reserved: [0; 2],
        }
    }
}

// Mirrors SF.lua's reference `raknetSendRpc` and `raknetSendBitStream`
// policies in `.native-references/SF.lua/SFlua/raknet.lua`.
const TYPED_RPC_SEND_OPTIONS: SendOptions = SendOptions {
    priority: SAMP_NET_PRIORITY_HIGH,
    reliability: SAMP_NET_RELIABILITY_RELIABLE,
    ordering_channel: 0,
    timestamp: false,
};

const TYPED_PACKET_SEND_OPTIONS: SendOptions = SendOptions {
    priority: SAMP_NET_PRIORITY_HIGH,
    reliability: SAMP_NET_RELIABILITY_UNRELIABLE_SEQUENCED,
    ordering_channel: 0,
    timestamp: false,
};

pub struct Event<'callback> {
    service: SampNetService,
    raw: *mut SampNetEventV1,
    callback: PhantomData<&'callback mut SampNetEventV1>,
}

impl Event<'_> {
    #[must_use]
    pub fn id(&self) -> u8 {
        unsafe { self.service.event_id(self.raw) }.unwrap_or(0)
    }

    pub fn reset_read(&mut self) -> Result<(), ModResult> {
        unsafe { self.service.event_reset(self.raw) }
    }

    #[must_use]
    pub fn remaining_bits(&self) -> usize {
        unsafe { self.service.event_remaining_bits(self.raw) }.unwrap_or(0) as usize
    }

    pub fn read_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, ModResult> {
        let bit_len = checked_event_bit_len(bit_len)?;
        let mut out = vec![0; (bit_len as usize).div_ceil(u8::BITS as usize)];
        unsafe { self.service.event_read_bits(self.raw, &mut out, bit_len) }?;
        Ok(out)
    }

    pub(crate) fn read_bits_into(
        &mut self,
        out: &mut [u8],
        bit_len: usize,
    ) -> Result<(), ModResult> {
        if out.len() != bit_len.div_ceil(u8::BITS as usize) {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        }
        let bit_len = checked_event_bit_len(bit_len)?;
        unsafe { self.service.event_read_bits(self.raw, out, bit_len) }
    }

    pub fn replace_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), ModResult> {
        if bit_len > bytes.len().saturating_mul(u8::BITS as usize) {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        }
        let bit_len = u32::try_from(bit_len).map_err(|_| modkit_abi::MOD_INVALID_ARGUMENT)?;
        unsafe { self.service.event_replace_bits(self.raw, bytes, bit_len) }
    }

    pub fn read_encoded_string(&mut self, out: &mut [u8]) -> Result<usize, ModResult> {
        unsafe { self.service.event_read_encoded_string(self.raw, out) }.map(|len| len as usize)
    }

    pub(crate) fn encode_string(
        &self,
        value: &[u8],
        out: &mut [u8],
    ) -> Result<(usize, usize), ModResult> {
        self.service
            .encode_string(value, out)
            .map(|(bytes, bits)| (bytes as usize, bits as usize))
    }
}

fn checked_event_bit_len(bit_len: usize) -> Result<u32, ModResult> {
    u32::try_from(bit_len).map_err(|_| modkit_abi::MOD_INVALID_ARGUMENT)
}

type EventHandler = dyn for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static;

struct EventState {
    service: SampNetService,
    handler: Box<EventHandler>,
}

#[derive(Clone, Copy)]
pub struct Net {
    core: Core,
    service: SampNetService,
    control: SampControlService,
    codec: SampCodecService,
}

impl Net {
    pub(crate) const fn new(
        core: Core,
        service: SampNetService,
        control: SampControlService,
        codec: SampCodecService,
    ) -> Self {
        Self {
            core,
            service,
            control,
            codec,
        }
    }

    pub fn on_packet(
        self,
        direction: Direction,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
    ) -> Result<Subscription, ModResult> {
        self.register(direction, handler, true)
    }

    pub fn set_send_rate(
        self,
        kind: SendRateKind,
        milliseconds: u32,
    ) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(
            self.core,
            self.control.submit_send_rate(kind.raw(), milliseconds)?,
        )
    }

    pub fn connect(self, address: &[u8], port: u16) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.control.submit_connect(address, port)?)
    }

    pub fn disconnect(self, block_duration: u32) -> Result<CommandReceipt, ModResult> {
        CommandReceipt::new(self.core, self.control.submit_disconnect(block_duration)?)
    }

    pub fn on_rpc(
        self,
        direction: Direction,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
    ) -> Result<Subscription, ModResult> {
        self.register(direction, handler, false)
    }

    pub fn on_packet_id(
        self,
        direction: Direction,
        id: u8,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
    ) -> Result<Subscription, ModResult> {
        self.on_packet(direction, move |event| {
            if event.id() == id {
                handler(event)
            } else {
                Action::Continue
            }
        })
    }

    pub fn on_rpc_id(
        self,
        direction: Direction,
        id: u8,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
    ) -> Result<Subscription, ModResult> {
        self.on_rpc(direction, move |event| {
            if event.id() == id {
                handler(event)
            } else {
                Action::Continue
            }
        })
    }

    pub fn on_incoming_typed_packet<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<
                crate::events::Incoming,
                crate::events::PacketKind,
            > + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Incoming,
            crate::events::PacketKind,
        >(descriptor);
        self.on_packet(Direction::Incoming, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Incoming,
                crate::events::PacketKind,
                _,
            >(&state, event, &handler)
        })
    }

    pub fn on_outgoing_typed_packet<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<
                crate::events::Outgoing,
                crate::events::PacketKind,
            > + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Outgoing,
            crate::events::PacketKind,
        >(descriptor);
        self.on_packet(Direction::Outgoing, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Outgoing,
                crate::events::PacketKind,
                _,
            >(&state, event, &handler)
        })
    }

    pub fn on_incoming_typed_rpc<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<crate::events::Incoming, crate::events::RpcKind>
            + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Incoming,
            crate::events::RpcKind,
        >(descriptor);
        self.on_rpc(Direction::Incoming, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Incoming,
                crate::events::RpcKind,
                _,
            >(&state, event, &handler)
        })
    }

    pub fn on_outgoing_typed_rpc<D, F>(
        self,
        descriptor: D,
        handler: F,
    ) -> Result<Subscription, ModResult>
    where
        D: crate::events::TypedCallbackDescriptor<crate::events::Outgoing, crate::events::RpcKind>
            + 'static,
        D::Value: 'static,
        F: Fn(D::Value) -> crate::events::ProtocolAction<D::Value> + Send + Sync + 'static,
    {
        let (_, state) = crate::events::callback_registration::<
            D,
            crate::events::Outgoing,
            crate::events::RpcKind,
        >(descriptor);
        self.on_rpc(Direction::Outgoing, move |event| {
            crate::events::handle_typed_callback::<
                D,
                crate::events::Outgoing,
                crate::events::RpcKind,
                _,
            >(&state, event, &handler)
        })
    }

    fn register(
        self,
        direction: Direction,
        handler: impl for<'event> Fn(&mut Event<'event>) -> Action + Send + Sync + 'static,
        packet: bool,
    ) -> Result<Subscription, ModResult> {
        let state = Box::new(EventState {
            service: self.service,
            handler: Box::new(handler),
        });
        let raw = Box::into_raw(state);
        let result = unsafe {
            if packet {
                self.service.register_packet(
                    direction.raw(),
                    dispatch_event as SampNetEventCallbackV1,
                    raw.cast::<c_void>(),
                    release_event,
                )
            } else {
                self.service.register_rpc(
                    direction.raw(),
                    dispatch_event as SampNetEventCallbackV1,
                    raw.cast::<c_void>(),
                    release_event,
                )
            }
        };
        match result {
            Ok(id) => Subscription::new(self.core, id),
            Err(error) => {
                drop(unsafe { Box::from_raw(raw) });
                Err(error)
            }
        }
    }

    #[must_use]
    pub const fn rpc_name(self, id: u8) -> Option<&'static str> {
        samp_protocol::rpc_name(id)
    }

    #[must_use]
    pub const fn packet_name(self, id: u8) -> Option<&'static str> {
        samp_protocol::packet_name(id)
    }

    fn submit_protocol_rpc<D>(
        self,
        _descriptor: D,
        value: D::Value,
    ) -> Result<CommandReceipt, ProtocolSendError>
    where
        D: samp_protocol::OutgoingRpcDescriptor,
    {
        let payload = D::encode_bits(&value).map_err(ProtocolSendError::Encode)?;
        self.send_rpc_with_options(
            D::ID,
            payload.as_bytes(),
            payload.len_bits(),
            TYPED_RPC_SEND_OPTIONS,
        )
        .map_err(ProtocolSendError::Host)
    }

    fn submit_protocol_packet<D>(
        self,
        _descriptor: D,
        value: D::Value,
    ) -> Result<CommandReceipt, ProtocolSendError>
    where
        D: samp_protocol::OutgoingPacketDescriptor,
    {
        let payload = D::encode_bits(&value).map_err(ProtocolSendError::Encode)?;
        self.send_packet_with_options(
            D::ID,
            payload.as_bytes(),
            payload.len_bits(),
            TYPED_PACKET_SEND_OPTIONS,
        )
        .map_err(ProtocolSendError::Host)
    }

    fn send_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
        take: bool,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_DAMAGE,
            samp_protocol::rpc::outgoing::common::Damage {
                player_id,
                damage,
                weapon,
                body_part,
                take,
            },
        )
    }

    /// Queues one bounded SA-MP chat or slash-command RPC.
    pub fn send_chat(self, text: &[u8]) -> Result<CommandReceipt, ProtocolSendError> {
        if text.first() == Some(&b'/') {
            self.submit_protocol_rpc(
                samp_protocol::rpc::outgoing::chat::SEND_COMMAND,
                text.to_vec(),
            )
        } else {
            self.submit_protocol_rpc(samp_protocol::rpc::outgoing::chat::SEND_CHAT, text.to_vec())
        }
    }

    /// Queues the server-bound request-spawn RPC.
    pub fn send_request_spawn(self) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(samp_protocol::rpc::outgoing::common::SEND_REQUEST_SPAWN, ())
    }

    /// Queues the protocol-level request-class RPC without changing local class state.
    pub fn send_request_class(self, class_id: i32) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_REQUEST_CLASS,
            class_id,
        )
    }

    /// Queues the protocol-level interior-change RPC without changing GTA state.
    pub fn send_interior_change(
        self,
        interior_id: u8,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_INTERIOR_CHANGE,
            interior_id,
        )
    }

    /// Queues the protocol-level spawn RPC without invoking native spawn code.
    pub fn send_spawn(self) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(samp_protocol::rpc::outgoing::common::SEND_SPAWN, ())
    }

    /// Queues the protocol-level enter-vehicle RPC without changing the local ped.
    pub fn send_enter_vehicle(
        self,
        vehicle_id: u16,
        passenger: bool,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_ENTER_VEHICLE,
            samp_protocol::rpc::outgoing::common::EnterVehicle {
                vehicle_id,
                passenger,
            },
        )
    }

    /// Queues the protocol-level exit-vehicle RPC without changing the local ped.
    pub fn send_exit_vehicle(self, vehicle_id: u16) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_EXIT_VEHICLE,
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
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_DIALOG_RESPONSE,
            samp_protocol::rpc::outgoing::common::DialogResponse {
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
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_CLICK_PLAYER,
            samp_protocol::rpc::outgoing::common::ClickPlayer { player_id, source },
        )
    }

    /// Queues a server-bound textdraw-click RPC.
    pub fn send_click_textdraw(
        self,
        textdraw_id: u16,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_CLICK_TEXT_DRAW,
            textdraw_id,
        )
    }

    /// Queues a server-bound death notification.
    pub fn send_death_by_player(
        self,
        player_id: u16,
        reason: u8,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_DEATH_NOTIFICATION,
            samp_protocol::rpc::outgoing::common::DeathNotification {
                reason,
                killer_id: player_id,
            },
        )
    }

    /// Queues the empty server-bound menu-quit RPC.
    pub fn send_menu_quit(self) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(samp_protocol::rpc::outgoing::common::SEND_QUIT_MENU, ())
    }

    /// Queues a server-bound menu-row selection.
    pub fn send_menu_select_row(self, row: u8) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(samp_protocol::rpc::outgoing::common::SEND_MENU_SELECT, row)
    }

    /// Queues a server-bound pickup notification.
    pub fn send_picked_up_pickup(
        self,
        pickup_id: i32,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_PICKED_UP_PICKUP,
            pickup_id,
        )
    }

    /// Queues a server-bound vehicle-destroyed notification.
    pub fn send_vehicle_destroyed(
        self,
        vehicle_id: u16,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_VEHICLE_DESTROYED,
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
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_VEHICLE_DAMAGED,
            samp_protocol::rpc::outgoing::common::VehicleDamage {
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
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_VEHICLE_TUNING,
            samp_protocol::rpc::outgoing::common::VehicleTuning {
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
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.send_damage(player_id, damage, weapon, body_part, false)
    }

    /// Queues a server-bound take-damage notification.
    pub fn send_take_damage(
        self,
        player_id: u16,
        damage: f32,
        weapon: i32,
        body_part: i32,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.send_damage(player_id, damage, weapon, body_part, true)
    }

    /// Queues a complete attached-object edit action.
    pub fn send_edit_attached_object(
        self,
        edit: samp_protocol::rpc::outgoing::common::EditAttachedObject,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(
            samp_protocol::rpc::outgoing::common::SEND_EDIT_ATTACHED_OBJECT,
            edit,
        )
    }

    /// Queues a complete global or player-object edit action.
    pub fn send_edit_object(
        self,
        edit: samp_protocol::rpc::outgoing::common::EditObject,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_rpc(samp_protocol::rpc::outgoing::common::SEND_EDIT_OBJECT, edit)
    }

    /// Queues a bounded server-bound RCON command packet.
    pub fn send_rcon_command(self, command: &[u8]) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(
            samp_protocol::packet::common::SEND_RCON_COMMAND,
            command.to_vec(),
        )
    }

    /// Queues a complete local aim-sync packet.
    pub fn send_aim_sync(
        self,
        sync: samp_protocol::packet::common::AimSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_AIM_SYNC, sync)
    }

    /// Queues a complete local bullet-sync packet.
    pub fn send_bullet_sync(
        self,
        sync: samp_protocol::packet::common::BulletSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_BULLET_SYNC, sync)
    }

    /// Queues a complete local vehicle-sync packet.
    pub fn send_vehicle_sync(
        self,
        sync: samp_protocol::packet::common::VehicleSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_VEHICLE_SYNC, sync)
    }

    /// Queues a complete local on-foot player-sync packet.
    pub fn send_player_sync(
        self,
        sync: samp_protocol::packet::common::PlayerSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_PLAYER_SYNC, sync)
    }

    /// Queues a complete local spectator-sync packet.
    pub fn send_spectator_sync(
        self,
        sync: samp_protocol::packet::common::SpectatorSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_SPECTATOR_SYNC, sync)
    }

    /// Queues a complete local trailer-sync packet.
    pub fn send_trailer_sync(
        self,
        sync: samp_protocol::packet::common::TrailerSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_TRAILER_SYNC, sync)
    }

    /// Queues a complete local passenger-sync packet.
    pub fn send_passenger_sync(
        self,
        sync: samp_protocol::packet::common::PassengerSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_PASSENGER_SYNC, sync)
    }

    /// Queues a complete local unoccupied-vehicle sync packet.
    pub fn send_unoccupied_sync(
        self,
        sync: samp_protocol::packet::common::UnoccupiedSync,
    ) -> Result<CommandReceipt, ProtocolSendError> {
        self.submit_protocol_packet(samp_protocol::packet::common::SEND_UNOCCUPIED_SYNC, sync)
    }

    pub fn send_packet(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt, ModResult> {
        self.send_packet_with_options(id, bytes, bit_len, SendOptions::default())
    }

    pub fn send_packet_with_options(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: usize,
        options: SendOptions,
    ) -> Result<CommandReceipt, ModResult> {
        let bit_len = checked_event_bit_len(bit_len)?;
        let id = self
            .service
            .submit_packet(id, bytes, bit_len, options.into())?;
        CommandReceipt::new(self.core, id)
    }

    pub fn send_rpc(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: usize,
    ) -> Result<CommandReceipt, ModResult> {
        self.send_rpc_with_options(id, bytes, bit_len, SendOptions::default())
    }

    pub fn send_rpc_with_options(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: usize,
        options: SendOptions,
    ) -> Result<CommandReceipt, ModResult> {
        let bit_len = checked_event_bit_len(bit_len)?;
        let id = self
            .service
            .submit_rpc(id, bytes, bit_len, options.into())?;
        CommandReceipt::new(self.core, id)
    }

    pub fn send_packet_stream(
        self,
        id: u8,
        payload: &samp_protocol::BitStream,
    ) -> Result<CommandReceipt, ModResult> {
        self.send_packet(id, payload.as_bytes(), payload.len_bits())
    }

    pub fn send_packet_stream_with_options(
        self,
        id: u8,
        payload: &samp_protocol::BitStream,
        options: SendOptions,
    ) -> Result<CommandReceipt, ModResult> {
        self.send_packet_with_options(id, payload.as_bytes(), payload.len_bits(), options)
    }

    pub fn send_rpc_stream(
        self,
        id: u8,
        payload: &samp_protocol::BitStream,
    ) -> Result<CommandReceipt, ModResult> {
        self.send_rpc(id, payload.as_bytes(), payload.len_bits())
    }

    pub fn send_rpc_stream_with_options(
        self,
        id: u8,
        payload: &samp_protocol::BitStream,
        options: SendOptions,
    ) -> Result<CommandReceipt, ModResult> {
        self.send_rpc_with_options(id, payload.as_bytes(), payload.len_bits(), options)
    }

    pub fn emulate_incoming_packet(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<CommandReceipt, ModResult> {
        let id = self
            .service
            .submit_emulate_incoming_packet(id, bytes, bit_len)?;
        CommandReceipt::new(self.core, id)
    }

    pub fn emulate_incoming_rpc(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<CommandReceipt, ModResult> {
        let id = self
            .service
            .submit_emulate_incoming_rpc(id, bytes, bit_len)?;
        CommandReceipt::new(self.core, id)
    }

    pub fn encode_string(self, value: &[u8]) -> Result<samp_protocol::EncodedBits, ModResult> {
        let capacity_bits = value
            .len()
            .checked_mul(16)
            .and_then(|bits| bits.checked_add(16))
            .ok_or(modkit_abi::MOD_PAYLOAD_TOO_LARGE)?;
        let mut bytes = vec![0; capacity_bits.div_ceil(u8::BITS as usize)];
        let (byte_len, bit_len) = self.encode_string_into(value, &mut bytes)?;
        bytes.truncate(byte_len);
        samp_protocol::EncodedBits::from_bits(bytes, bit_len as usize)
            .map_err(|_| modkit_abi::MOD_NATIVE_CALL_FAILED)
    }

    pub fn encode_string_into(
        self,
        value: &[u8],
        out: &mut [u8],
    ) -> Result<(usize, u32), ModResult> {
        self.service
            .encode_string(value, out)
            .map(|(bytes, bits)| (bytes as usize, bits))
    }

    pub fn decode_string(
        self,
        stream: &mut samp_protocol::BitStream,
    ) -> Result<Vec<u8>, ModResult> {
        const MAX_DECODED_BYTES: usize = 4_095;
        let mut output = vec![0; MAX_DECODED_BYTES + 1];
        let (output_len, read_offset) = self.codec.decode_string(
            stream.as_bytes(),
            stream.len_bits(),
            stream.read_offset_bits(),
            &mut output,
        )?;
        if output_len > MAX_DECODED_BYTES || read_offset > stream.len_bits() {
            return Err(modkit_abi::MOD_NATIVE_CALL_FAILED);
        }
        stream
            .set_read_offset(read_offset)
            .map_err(|_| modkit_abi::MOD_NATIVE_CALL_FAILED)?;
        output.truncate(output_len);
        Ok(output)
    }

    pub fn incoming_emulation_ready(self) -> Result<bool, ModResult> {
        self.service.incoming_emulation_ready()
    }
}

unsafe extern "system" fn dispatch_event(user_data: *mut c_void, raw: *mut SampNetEventV1) -> u32 {
    if user_data.is_null() || raw.is_null() {
        return SAMP_NET_ACTION_CONTINUE;
    }
    let Some(state) = (unsafe { user_data.cast::<EventState>().as_ref() }) else {
        return SAMP_NET_ACTION_CONTINUE;
    };
    let mut event = Event {
        service: state.service,
        raw,
        callback: PhantomData,
    };
    catch_unwind(AssertUnwindSafe(|| (state.handler)(&mut event)))
        .unwrap_or(Action::Continue)
        .raw()
}

unsafe extern "system" fn release_event(user_data: *mut c_void) {
    if !user_data.is_null() {
        drop(unsafe { Box::from_raw(user_data.cast::<EventState>()) });
    }
}

#[cfg(test)]
mod tests {
    use super::{
        SendOptions, TYPED_PACKET_SEND_OPTIONS, TYPED_RPC_SEND_OPTIONS, checked_event_bit_len,
    };

    #[test]
    fn event_read_bit_length_must_fit_the_modkit_abi() {
        assert_eq!(checked_event_bit_len(0), Ok(0));

        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            checked_event_bit_len(u32::MAX as usize + 1),
            Err(modkit_abi::MOD_INVALID_ARGUMENT)
        );
    }

    #[test]
    fn typed_delivery_distinguishes_rpc_from_packet_transport() {
        assert_eq!(
            TYPED_RPC_SEND_OPTIONS,
            SendOptions {
                priority: modkit_abi::SAMP_NET_PRIORITY_HIGH,
                reliability: modkit_abi::SAMP_NET_RELIABILITY_RELIABLE,
                ordering_channel: 0,
                timestamp: false,
            }
        );
        assert_eq!(
            TYPED_PACKET_SEND_OPTIONS,
            SendOptions {
                priority: modkit_abi::SAMP_NET_PRIORITY_HIGH,
                reliability: modkit_abi::SAMP_NET_RELIABILITY_UNRELIABLE_SEQUENCED,
                ordering_channel: 0,
                timestamp: false,
            }
        );
        assert_ne!(TYPED_RPC_SEND_OPTIONS, SendOptions::default());
        assert_ne!(TYPED_PACKET_SEND_OPTIONS, SendOptions::default());
    }
}
