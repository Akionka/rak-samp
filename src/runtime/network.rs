use super::options::validate_packet_options;
use super::{ClientHookStatus, CodecError, Runtime, SendError, SendOptions};
use crate::{
    BitStream, Direction, ListenerHandle, ListenerRegistrationError, PacketEvent, RpcEvent,
    SampVersion, command::CommandId,
};

impl Runtime {
    /// Registers a synchronous packet listener.
    pub fn on_packet(
        &self,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut PacketEvent<'event>) -> crate::HookAction + Send + 'static,
    ) -> Result<ListenerHandle, ListenerRegistrationError> {
        self.registry.register_packet(direction, callback)
    }

    /// Registers a synchronous RPC listener.
    pub fn on_rpc(
        &self,
        direction: Direction,
        callback: impl for<'event> FnMut(&mut RpcEvent<'event>) -> crate::HookAction + Send + 'static,
    ) -> Result<ListenerHandle, ListenerRegistrationError> {
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
}

#[cfg(test)]
mod tests {
    use super::validate_packet_options;
    use crate::runtime::{PacketPriority, PacketReliability, SendError, SendOptions};

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
}
