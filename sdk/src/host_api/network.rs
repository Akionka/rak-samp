//! Low-level packet/RPC send and incoming-emulation `HostApi` wrappers.

use crate::events;
use crate::raknet::BitStream;
use crate::{
    CommandReceipt, HostApi, SampClientSdkCommandReceipt, SampClientSdkResult,
    SampClientSdkSendOptions,
};

impl HostApi {
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
}
