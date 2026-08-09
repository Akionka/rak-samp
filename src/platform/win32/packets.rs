//! Outbound native packet and RPC sends.

use super::{
    AllocatePacketFn, BackendState, NativeBitStream, OutgoingPacketFn, OutgoingRpcFn,
    PEER_PACKET_QUEUE_OFFSET, QueueWriteLockFn, QueueWriteUnlockFn, packet_stream, priority_value,
    reliability_value,
};
use crate::{BitStream, SendError, SendOptions};
use std::{mem, ptr, sync::atomic::Ordering};

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
}
