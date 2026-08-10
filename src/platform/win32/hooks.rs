//! Native packet and RPC listener dispatch helpers.

use super::{BackendState, RawBitStream, RawPacket, packet_stream, remaining_stream_bounded};
use crate::{BitStream, Direction, event::HookAction};
use std::{ptr, slice, sync::atomic::Ordering};

pub(super) const MAX_INCOMING_PACKET_BYTES: usize = 16 * 1024 * 1024;

pub(super) unsafe fn dispatch_packet_stream(
    state: &BackendState,
    direction: Direction,
    native: *mut RawBitStream,
) -> HookAction {
    if native.is_null() {
        return HookAction::Continue;
    }
    let Ok(mut stream) = (unsafe { (&*native).copy_to_owned() }) else {
        return HookAction::Continue;
    };
    let Ok(id) = stream.read_u8() else {
        return HookAction::Continue;
    };
    let remaining_bits = stream.remaining_bits();
    let capacity_bits = stream.capacity_bits().unwrap_or(remaining_bits);
    let mut payload = remaining_stream_bounded(
        &mut stream,
        remaining_bits,
        capacity_bits.saturating_sub(u8::BITS as usize),
    );
    let action = state.registry.dispatch_packet(direction, id, &mut payload);
    if action == HookAction::Continue
        && let Ok(combined) = packet_stream(id, &payload)
    {
        let _ = unsafe { (&mut *native).replace_from(&combined) };
    }
    action
}

pub(super) unsafe fn dispatch_rpc_stream(
    state: &BackendState,
    direction: Direction,
    id: u8,
    native: *mut RawBitStream,
) -> HookAction {
    if native.is_null() {
        return HookAction::Continue;
    }
    let Ok(mut payload) = (unsafe { (&*native).copy_to_owned() }) else {
        return HookAction::Continue;
    };
    let action = state.registry.dispatch_rpc(direction, id, &mut payload);
    if action == HookAction::Continue {
        let _ = unsafe { (&mut *native).replace_from(&payload) };
    }
    action
}

pub(super) unsafe fn dispatch_raw_packet(
    state: &BackendState,
    packet: *mut RawPacket,
) -> HookAction {
    if !state.registry.has_packet_listener(Direction::Incoming) {
        return HookAction::Continue;
    }
    if packet.is_null() {
        return HookAction::Continue;
    }
    let length = unsafe { ptr::addr_of!((*packet).length).read_unaligned() };
    let bit_size = unsafe { ptr::addr_of!((*packet).bit_size).read_unaligned() } as usize;
    let data = unsafe { ptr::addr_of!((*packet).data).read_unaligned() };
    let byte_len = validated_packet_byte_len(length, bit_size);
    let metadata_is_valid = !data.is_null() && byte_len.is_some();
    if !state
        .incoming_packet_diagnostic_logged
        .swap(true, Ordering::AcqRel)
    {
        if metadata_is_valid {
            log::debug!(
                "first incoming packet metadata is valid: length={length}, bit_size={bit_size}"
            );
        } else {
            log::warn!(
                "rejected invalid incoming packet metadata: length={length}, bit_size={bit_size}, data_is_null={} (traffic passed through unchanged)",
                data.is_null()
            );
        }
    }
    let Some(byte_len) = byte_len else {
        return HookAction::Continue;
    };
    if data.is_null() {
        return HookAction::Continue;
    }
    let bytes = unsafe { slice::from_raw_parts(data, byte_len) }.to_vec();
    let Ok(mut stream) = BitStream::from_bytes_with_capacity(bytes, bit_size, bit_size) else {
        return HookAction::Continue;
    };
    let Ok(id) = stream.read_u8() else {
        return HookAction::Continue;
    };
    let remaining_bits = stream.remaining_bits();
    let mut payload = remaining_stream_bounded(
        &mut stream,
        remaining_bits,
        bit_size.saturating_sub(u8::BITS as usize),
    );
    let action = state
        .registry
        .dispatch_packet(Direction::Incoming, id, &mut payload);
    if action == HookAction::Continue
        && let Ok(combined) = packet_stream(id, &payload)
        && combined.len_bits() <= bit_size
    {
        unsafe {
            ptr::copy_nonoverlapping(combined.as_bytes().as_ptr(), data, combined.len_bytes())
        };
        unsafe {
            ptr::addr_of_mut!((*packet).bit_size).write_unaligned(combined.len_bits() as u32)
        };
        unsafe { ptr::addr_of_mut!((*packet).length).write_unaligned(combined.len_bytes() as u32) };
    }
    action
}

pub(super) fn validated_packet_byte_len(length: u32, bit_size: usize) -> Option<usize> {
    if bit_size < u8::BITS as usize {
        return None;
    }
    let byte_len = bit_size.checked_add(u8::BITS as usize - 1)? / u8::BITS as usize;
    if byte_len > length as usize
        || byte_len > MAX_INCOMING_PACKET_BYTES
        || byte_len > isize::MAX as usize
    {
        return None;
    }
    Some(byte_len)
}
