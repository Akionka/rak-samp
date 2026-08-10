//! Native packet and RPC listener dispatch helpers.

use super::{
    BackendState, ClientHookInstallState, GameProcessFn, IncomingRpcFn, OutgoingPacketFn,
    OutgoingRpcFn, RawBitStream, RawPacket, RpcPlayerId, active_state, packet_stream, packets,
    remaining_stream_bounded,
};
use crate::{BitStream, Direction, event::HookAction};
use std::{ffi::c_void, mem, ptr, slice, sync::atomic::Ordering};

pub(super) const MAX_INCOMING_PACKET_BYTES: usize = 16 * 1024 * 1024;

type IncomingPacketFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut RawPacket;
type DeallocatePacketFn = unsafe extern "thiscall" fn(*mut c_void, *mut RawPacket);

type RakClientConstructorFn = unsafe extern "C" fn() -> *mut c_void;

#[derive(Clone, Copy)]
struct OutgoingRpcCall {
    client: *mut c_void,
    id: *mut i32,
    stream: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
    timestamp: bool,
}

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

pub(super) fn call_incoming_packet(state: &BackendState, client: *mut c_void) -> *mut RawPacket {
    let original = state.incoming_packet_original.load(Ordering::Acquire);
    if original == 0 {
        return ptr::null_mut();
    }
    let original: IncomingPacketFn = unsafe { mem::transmute(original) };
    unsafe { original(client) }
}

pub(super) fn deallocate_packet(state: &BackendState, client: *mut c_void, packet: *mut RawPacket) {
    let original = state.deallocate_packet_original.load(Ordering::Acquire);
    if original != 0 {
        let original: DeallocatePacketFn = unsafe { mem::transmute(original) };
        unsafe { original(client, packet) };
    }
}

pub(super) fn call_outgoing_packet(
    state: &BackendState,
    client: *mut c_void,
    stream: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
) -> bool {
    let original = state.outgoing_packet_original.load(Ordering::Acquire);
    if original == 0 {
        return false;
    }
    let original: OutgoingPacketFn = unsafe { mem::transmute(original) };
    unsafe { original(client, stream, priority, reliability, channel) }
}

fn call_outgoing_rpc(state: &BackendState, call: OutgoingRpcCall) -> bool {
    let original = state.outgoing_rpc_original.load(Ordering::Acquire);
    if original == 0 {
        return false;
    }
    let original: OutgoingRpcFn = unsafe { mem::transmute(original) };
    unsafe {
        original(
            call.client,
            call.id,
            call.stream,
            call.priority,
            call.reliability,
            call.channel,
            call.timestamp,
        )
    }
}

pub(super) unsafe extern "thiscall" fn outgoing_packet_detour(
    client: *mut c_void,
    native: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
) -> bool {
    let Some(state) = active_state() else {
        return false;
    };
    if !state.registry.has_packet_listener(Direction::Outgoing) {
        return call_outgoing_packet(&state, client, native, priority, reliability, channel);
    }
    let action = unsafe { dispatch_packet_stream(&state, Direction::Outgoing, native) };
    if action == HookAction::Block {
        return false;
    }
    call_outgoing_packet(&state, client, native, priority, reliability, channel)
}

pub(super) unsafe extern "thiscall" fn outgoing_rpc_detour(
    client: *mut c_void,
    id: *mut i32,
    native: *mut RawBitStream,
    priority: i32,
    reliability: i32,
    channel: i8,
    timestamp: bool,
) -> bool {
    let Some(state) = active_state() else {
        return false;
    };
    let original_call = OutgoingRpcCall {
        client,
        id,
        stream: native,
        priority,
        reliability,
        channel,
        timestamp,
    };
    if id.is_null() {
        return call_outgoing_rpc(&state, original_call);
    }
    if !state.registry.has_rpc_listener(Direction::Outgoing) {
        return call_outgoing_rpc(&state, original_call);
    }
    let action = unsafe { dispatch_rpc_stream(&state, Direction::Outgoing, *id as u8, native) };
    if action == HookAction::Block {
        return false;
    }
    call_outgoing_rpc(&state, original_call)
}

pub(super) unsafe extern "thiscall" fn incoming_packet_detour(
    client: *mut c_void,
) -> *mut RawPacket {
    let Some(state) = active_state() else {
        return ptr::null_mut();
    };
    loop {
        let packet = call_incoming_packet(&state, client);
        if packet.is_null() {
            return packet;
        }
        let action = unsafe { dispatch_raw_packet(&state, packet) };
        if action == HookAction::Continue {
            return packet;
        }
        deallocate_packet(&state, client, packet);
    }
}

pub(super) unsafe extern "thiscall" fn incoming_rpc_detour(
    receiver: *mut c_void,
    data: *mut u8,
    length: i32,
    player: RpcPlayerId,
) -> bool {
    let Some(state) = active_state() else {
        return false;
    };
    state
        .rpc_receiver
        .store(receiver as usize, Ordering::Release);
    state
        .player_address
        .store(player.binary_address, Ordering::Release);
    state.player_port.store(player.port, Ordering::Release);
    let original = state.incoming_rpc_trampoline.load(Ordering::Acquire);
    if original == 0 || data.is_null() || length < 0 {
        return false;
    }
    let original: IncomingRpcFn = unsafe { mem::transmute(original) };
    let input = unsafe { slice::from_raw_parts(data, length as usize) };
    if !state.registry.has_rpc_listener(Direction::Incoming) {
        return unsafe { original(receiver, data, length, player) };
    }
    let Ok((rpc_id, mut payload, timestamp)) = packets::parse_rpc_envelope(input) else {
        return unsafe { original(receiver, data, length, player) };
    };
    if state
        .registry
        .dispatch_rpc(Direction::Incoming, rpc_id, &mut payload)
        == HookAction::Block
    {
        return false;
    }
    let Ok(mut output) = packets::build_rpc_envelope(rpc_id, &payload, timestamp) else {
        return unsafe { original(receiver, data, length, player) };
    };
    unsafe { original(receiver, output.as_mut_ptr(), output.len() as i32, player) }
}

pub(super) unsafe extern "thiscall" fn game_process_detour(game: *mut c_void) {
    let Some(state) = active_state() else {
        return;
    };
    let trampoline = state.game_process_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return;
    }
    let original: GameProcessFn = unsafe { mem::transmute(trampoline) };
    unsafe { state.run_game_process_tick(game, original) };
}

pub(super) unsafe extern "C" fn rak_client_constructor_detour() -> *mut c_void {
    let Some(state) = active_state() else {
        return ptr::null_mut();
    };
    let trampoline = state.constructor_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return ptr::null_mut();
    }
    let original: RakClientConstructorFn = unsafe { mem::transmute(trampoline) };
    let client = unsafe { original() };
    if !client.is_null()
        && let Err(error) = state.install_client_hooks(client)
    {
        state
            .client_hook_status
            .store(ClientHookInstallState::Failed.as_raw(), Ordering::Release);
        log::error!("RakClient hook installation failed: {error}");
    }
    client
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_byte_aligned_and_partial_byte_packets() {
        assert_eq!(validated_packet_byte_len(2, 16), Some(2));
        assert_eq!(validated_packet_byte_len(2, 9), Some(2));
    }

    #[test]
    fn rejects_metadata_that_cannot_describe_the_buffer() {
        assert_eq!(validated_packet_byte_len(1, 7), None);
        assert_eq!(validated_packet_byte_len(1, 9), None);
        assert_eq!(
            validated_packet_byte_len(
                (MAX_INCOMING_PACKET_BYTES + 1) as u32,
                (MAX_INCOMING_PACKET_BYTES + 1) * 8
            ),
            None
        );
    }
}
