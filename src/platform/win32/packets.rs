//! Outbound native packet and RPC sends.

use super::{
    AllocatePacketFn, BackendState, ID_RPC, IncomingRpcFn, NativeBitStream, OutgoingPacketFn,
    OutgoingRpcFn, PEER_PACKET_QUEUE_OFFSET, QueueWriteLockFn, QueueWriteUnlockFn, RpcPlayerId,
    packet_stream, priority_value, reliability_value,
};
use crate::{BitStream, Direction, SendError, SendOptions, event::HookAction};
use std::{ffi::c_void, mem, ptr, sync::atomic::Ordering};

impl BackendState {
    pub(super) fn send_packet_native(
        &self,
        packet_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        let client = self.ready_client()?;
        let original = self.outgoing_packet_original.load(Ordering::Acquire);
        if original == 0 {
            return Err(SendError::ClientNotReady);
        }
        let stream = packet_stream(packet_id, payload)?;
        let mut native = NativeBitStream::new(&stream)?;
        let send: OutgoingPacketFn = unsafe { mem::transmute(original) };
        Ok(unsafe {
            send(
                client,
                native.as_mut_ptr(),
                priority_value(options.priority),
                reliability_value(options.reliability),
                options.ordering_channel as i8,
            )
        })
    }

    pub(super) fn send_rpc_native(
        &self,
        rpc_id: u8,
        payload: &BitStream,
        options: SendOptions,
    ) -> Result<bool, SendError> {
        let client = self.ready_client()?;
        let original = self.outgoing_rpc_original.load(Ordering::Acquire);
        if original == 0 {
            return Err(SendError::ClientNotReady);
        }
        let mut native = NativeBitStream::new(payload)?;
        let send: OutgoingRpcFn = unsafe { mem::transmute(original) };
        let mut id = i32::from(rpc_id);
        Ok(unsafe {
            send(
                client,
                &mut id,
                native.as_mut_ptr(),
                priority_value(options.priority),
                reliability_value(options.reliability),
                options.ordering_channel as i8,
                options.timestamp,
            )
        })
    }

    pub(super) fn emulate_incoming_packet_native(
        &self,
        packet_id: u8,
        payload: BitStream,
    ) -> Result<bool, SendError> {
        let peer = self.ready_rpc_receiver()?;
        let stream = packet_stream(packet_id, &payload)?;
        let byte_len = i32::try_from(stream.len_bytes()).map_err(|_| SendError::PayloadTooLarge)?;
        let bit_size = u32::try_from(stream.len_bits()).map_err(|_| SendError::PayloadTooLarge)?;
        let allocate: AllocatePacketFn =
            unsafe { mem::transmute(self.module_base + self.addresses.allocate_packet as usize) };
        let packet = unsafe { allocate(byte_len) };
        if packet.is_null() {
            return Err(SendError::NativeCallFailed);
        }
        unsafe {
            let packet_data = ptr::addr_of!((*packet).data).read_unaligned();
            if packet_data.is_null() {
                return Err(SendError::NativeCallFailed);
            }
            ptr::copy_nonoverlapping(stream.as_bytes().as_ptr(), packet_data, stream.len_bytes());
            ptr::addr_of_mut!((*packet).length).write_unaligned(stream.len_bytes() as u32);
            ptr::addr_of_mut!((*packet).bit_size).write_unaligned(bit_size);

            let lock: QueueWriteLockFn =
                mem::transmute(self.module_base + self.addresses.write_lock as usize);
            let unlock: QueueWriteUnlockFn =
                mem::transmute(self.module_base + self.addresses.write_unlock as usize);
            let slot = lock(peer.add(PEER_PACKET_QUEUE_OFFSET).cast());
            if slot.is_null() {
                return Err(SendError::NativeCallFailed);
            }
            *slot = packet;
            unlock(peer.add(PEER_PACKET_QUEUE_OFFSET).cast());
        }
        Ok(true)
    }

    pub(super) fn emulate_incoming_rpc_native(
        &self,
        rpc_id: u8,
        mut payload: BitStream,
    ) -> Result<bool, SendError> {
        if self
            .registry
            .dispatch_rpc(Direction::Incoming, rpc_id, &mut payload)
            == HookAction::Block
        {
            return Ok(false);
        }
        let receiver = self.rpc_receiver.load(Ordering::Acquire) as *mut c_void;
        let original = self.incoming_rpc_trampoline.load(Ordering::Acquire);
        if receiver.is_null() || original == 0 {
            return Err(SendError::ClientNotReady);
        }
        let mut envelope = BitStream::new();
        envelope
            .write_u8(ID_RPC)
            .map_err(|_| SendError::PayloadTooLarge)?;
        envelope
            .write_u8(rpc_id)
            .map_err(|_| SendError::PayloadTooLarge)?;
        envelope
            .write_compressed_u32(payload.len_bits() as u32)
            .map_err(|_| SendError::PayloadTooLarge)?;
        envelope
            .write_stream(&payload)
            .map_err(|_| SendError::PayloadTooLarge)?;
        let original: IncomingRpcFn = unsafe { mem::transmute(original) };
        let envelope_len =
            i32::try_from(envelope.len_bytes()).map_err(|_| SendError::PayloadTooLarge)?;
        let player = RpcPlayerId {
            binary_address: self.player_address.load(Ordering::Acquire),
            port: self.player_port.load(Ordering::Acquire),
        };
        Ok(unsafe {
            original(
                receiver,
                envelope.as_bytes().as_ptr().cast_mut(),
                envelope_len,
                player,
            )
        })
    }
}
