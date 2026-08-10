//! Outbound native packet and RPC sends.

use super::{
    AllocatePacketFn, BackendState, ID_RPC, ID_TIMESTAMP, IncomingRpcFn, NativeBitStream,
    OutgoingPacketFn, OutgoingRpcFn, PEER_PACKET_QUEUE_OFFSET, QueueWriteLockFn,
    QueueWriteUnlockFn, RpcPlayerId, packet_stream, priority_value, reliability_value,
    remaining_stream,
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

pub(super) fn parse_rpc_envelope(
    input: &[u8],
) -> Result<(u8, BitStream, Option<[u8; 4]>), SendError> {
    let mut stream = BitStream::from_bytes(input.to_vec());
    let first = stream.read_u8().map_err(|_| SendError::NativeCallFailed)?;
    let timestamp = if first == ID_TIMESTAMP {
        let bytes = stream
            .read_bytes(4)
            .map_err(|_| SendError::NativeCallFailed)?;
        let mut timestamp = [0; 4];
        timestamp.copy_from_slice(&bytes);
        if stream.read_u8().map_err(|_| SendError::NativeCallFailed)? != ID_RPC {
            return Err(SendError::NativeCallFailed);
        }
        Some(timestamp)
    } else if first == ID_RPC {
        None
    } else {
        return Err(SendError::NativeCallFailed);
    };
    let id = stream.read_u8().map_err(|_| SendError::NativeCallFailed)?;
    let payload_bits = stream
        .read_compressed_u32()
        .map_err(|_| SendError::NativeCallFailed)? as usize;
    if payload_bits > stream.remaining_bits() {
        return Err(SendError::NativeCallFailed);
    }
    Ok((id, remaining_stream(&mut stream, payload_bits), timestamp))
}

pub(super) fn build_rpc_envelope(
    id: u8,
    payload: &BitStream,
    timestamp: Option<[u8; 4]>,
) -> Result<Vec<u8>, SendError> {
    let payload_bits = u32::try_from(payload.len_bits()).map_err(|_| SendError::PayloadTooLarge)?;
    let mut stream = BitStream::new();
    if let Some(timestamp) = timestamp {
        stream
            .write_u8(ID_TIMESTAMP)
            .map_err(|_| SendError::PayloadTooLarge)?;
        stream
            .write_bytes(&timestamp)
            .map_err(|_| SendError::PayloadTooLarge)?;
    }
    stream
        .write_u8(ID_RPC)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_u8(id)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_compressed_u32(payload_bits)
        .map_err(|_| SendError::PayloadTooLarge)?;
    stream
        .write_stream(payload)
        .map_err(|_| SendError::PayloadTooLarge)?;
    Ok(stream.as_bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_envelopes_round_trip_with_and_without_timestamps() {
        let payload = BitStream::from_bytes(vec![0x12, 0x34, 0x56]);

        for timestamp in [None, Some([1, 2, 3, 4])] {
            let encoded = build_rpc_envelope(42, &payload, timestamp).unwrap();
            let (id, decoded, decoded_timestamp) = parse_rpc_envelope(&encoded).unwrap();

            assert_eq!(id, 42);
            assert_eq!(decoded_timestamp, timestamp);
            assert_eq!(decoded.len_bits(), payload.len_bits());
            assert_eq!(decoded.as_bytes(), payload.as_bytes());
        }
    }

    #[test]
    fn rpc_envelopes_preserve_partial_payload_bits() {
        let payload = BitStream::from_bytes_with_bits(vec![0b1010_0000], 3).unwrap();
        let encoded = build_rpc_envelope(7, &payload, None).unwrap();
        let (id, decoded, timestamp) = parse_rpc_envelope(&encoded).unwrap();

        assert_eq!(id, 7);
        assert_eq!(timestamp, None);
        assert_eq!(decoded.len_bits(), 3);
        assert_eq!(decoded.as_bytes(), &[0b1010_0000]);
    }

    #[test]
    fn rpc_envelopes_reject_malformed_and_truncated_inputs() {
        assert!(matches!(
            parse_rpc_envelope(&[0]),
            Err(SendError::NativeCallFailed)
        ));
        assert!(matches!(
            parse_rpc_envelope(&[ID_TIMESTAMP, 1, 2, 3, 4, 0]),
            Err(SendError::NativeCallFailed)
        ));

        let mut truncated =
            build_rpc_envelope(7, &BitStream::from_bytes(vec![0xAA]), None).unwrap();
        truncated.pop();
        assert!(matches!(
            parse_rpc_envelope(&truncated),
            Err(SendError::NativeCallFailed)
        ));
    }
}
