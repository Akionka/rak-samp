//! Native packet and RPC listener dispatch helpers.

use super::players::MARKERS_SYNC_PACKET_ID;
use super::{
    BackendState, ClientHookInstallState, DEALLOCATE_PACKET_SLOT, GameProcessFn,
    INCOMING_PACKET_SLOT, IncomingRpcFn, OUTGOING_PACKET_SLOT, OUTGOING_RPC_SLOT, OutgoingPacketFn,
    OutgoingRpcFn, RawBitStream, RawPacket, RpcPlayerId, active_state, packet_stream, packets,
    remaining_stream_bounded,
};
use crate::{AttachError, BitStream, Direction, event::HookAction};
use std::{ffi::c_void, mem, ptr, slice, sync::atomic::Ordering};

pub(super) const MAX_INCOMING_PACKET_BYTES: usize = 16 * 1024 * 1024;

type IncomingPacketFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut RawPacket;
type DeallocatePacketFn = unsafe extern "thiscall" fn(*mut c_void, *mut RawPacket);

type RakClientConstructorFn = unsafe extern "C" fn() -> *mut c_void;
type DialogCloseFn = unsafe extern "thiscall" fn(*mut c_void, u8);

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
    let has_listener = state.registry.has_packet_listener(Direction::Incoming);
    if !has_listener && unsafe { data.read() } != MARKERS_SYNC_PACKET_ID {
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
    if !has_listener {
        state.capture_marker_sync(id, &payload);
        return HookAction::Continue;
    }
    let original_marker_payload = (id == MARKERS_SYNC_PACKET_ID).then(|| payload.clone());
    let action = state
        .registry
        .dispatch_packet(Direction::Incoming, id, &mut payload);
    let mut replacement_applied = false;
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
        replacement_applied = true;
    }
    if action == HookAction::Continue && id == MARKERS_SYNC_PACKET_ID {
        if replacement_applied {
            state.capture_marker_sync(id, &payload);
        } else if let Some(original_payload) = original_marker_payload.as_ref() {
            state.capture_marker_sync(id, original_payload);
        }
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
    let previous_receiver = state.rpc_receiver.swap(receiver as usize, Ordering::AcqRel);
    if previous_receiver == 0 && !receiver.is_null() {
        // This establishes the native receive queue needed by packet emulation.
        // Log only metadata; packet and RPC payloads must never reach diagnostics.
        log::debug!("captured incoming-RPC receiver for packet emulation: length={length}");
    }
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

pub(super) unsafe extern "C" fn game_process_detour() {
    let Some(state) = active_state() else {
        return;
    };
    let trampoline = state.game_process_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return;
    }
    let original: GameProcessFn = unsafe { mem::transmute(trampoline) };
    if !state
        .game_process_diagnostic_logged
        .swap(true, Ordering::AcqRel)
    {
        log::debug!("entered CGame::Process detour for the first time");
    }
    unsafe { state.run_game_process_tick(original) };
}

pub(super) unsafe extern "thiscall" fn dialog_close_detour(dialog: *mut c_void, button: u8) {
    let Some(state) = active_state() else {
        return;
    };
    let trampoline = state.dialog_close_trampoline.load(Ordering::Acquire);
    if trampoline == 0 {
        return;
    }
    state.capture_dialog_response(dialog, button);
    let original: DialogCloseFn = unsafe { mem::transmute(trampoline) };
    unsafe { original(dialog, button) };
}

impl BackendState {
    fn capture_dialog_response(&self, dialog: *mut c_void, button: u8) {
        let Some(profile) = self.connection_profile() else {
            return;
        };
        let Ok(Some(response)) = profile.dialog_response_on_close(dialog, button) else {
            return;
        };
        let mut cached = self
            .local_dialog_response
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *cached = Some(response);
    }
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

#[derive(Default)]
pub(super) struct HookStorage {
    pub(super) constructor: Option<modkit_win32::InlineHook>,
    pub(super) incoming_rpc: Option<modkit_win32::InlineHook>,
    pub(super) game_process: Option<modkit_win32::InlineHook>,
    pub(super) dialog_close: Option<modkit_win32::InlineHook>,
    pub(super) vtable: Option<VtableHook>,
}

pub(super) struct VtableHook {
    vtable: usize,
    entries: [VtableEntry; 3],
}

#[derive(Clone, Copy)]
struct VtableEntry {
    slot: usize,
    original: usize,
    detour: usize,
}

impl VtableHook {
    pub(super) unsafe fn install(
        client: *mut c_void,
        state: &BackendState,
    ) -> Result<Self, AttachError> {
        let Some(vtable) = (unsafe { modkit_win32::read_unaligned::<usize>(client as usize) })
        else {
            return Err(AttachError::ClientNotReady);
        };
        if vtable == 0 {
            return Err(AttachError::ClientNotReady);
        }

        let replacements = [
            (
                OUTGOING_PACKET_SLOT,
                outgoing_packet_detour as *const () as usize,
            ),
            (
                INCOMING_PACKET_SLOT,
                incoming_packet_detour as *const () as usize,
            ),
            (OUTGOING_RPC_SLOT, outgoing_rpc_detour as *const () as usize),
        ];
        let maximum_slot = replacements
            .iter()
            .map(|(slot, _)| *slot)
            .chain(std::iter::once(DEALLOCATE_PACKET_SLOT))
            .max()
            .ok_or(AttachError::ClientNotReady)?;
        let required_bytes = maximum_slot
            .checked_add(1)
            .and_then(|count| count.checked_mul(mem::size_of::<usize>()))
            .ok_or(AttachError::ClientNotReady)?;
        if !modkit_win32::readable_range(vtable as *const u8, required_bytes) {
            return Err(AttachError::ClientNotReady);
        }
        let mut entries = [VtableEntry {
            slot: 0,
            original: 0,
            detour: 0,
        }; 3];
        for (index, (slot, detour)) in replacements.into_iter().enumerate() {
            let slot_address =
                vtable_slot_address(vtable, slot).ok_or(AttachError::ClientNotReady)?;
            let original = unsafe { modkit_win32::read_unaligned::<usize>(slot_address) }
                .ok_or(AttachError::ClientNotReady)?;
            if original == 0 {
                return Err(AttachError::ClientNotReady);
            }
            entries[index] = VtableEntry {
                slot,
                original,
                detour,
            };
        }

        state
            .outgoing_packet_original
            .store(entries[0].original, Ordering::Release);
        state
            .incoming_packet_original
            .store(entries[1].original, Ordering::Release);
        let deallocate_packet = unsafe {
            modkit_win32::read_unaligned::<usize>(
                vtable_slot_address(vtable, DEALLOCATE_PACKET_SLOT)
                    .ok_or(AttachError::ClientNotReady)?,
            )
        }
        .ok_or(AttachError::ClientNotReady)?;
        state
            .deallocate_packet_original
            .store(deallocate_packet, Ordering::Release);
        state
            .outgoing_rpc_original
            .store(entries[2].original, Ordering::Release);

        for (index, entry) in entries.iter().enumerate() {
            let slot = vtable_slot_address(vtable, entry.slot).ok_or(AttachError::ClientNotReady)?
                as *mut usize;
            if unsafe { modkit_win32::write_protected(slot, entry.detour) }.is_err() {
                for restore in entries[..index].iter().rev() {
                    if let Some(address) = vtable_slot_address(vtable, restore.slot)
                        && let Err(error) = unsafe {
                            modkit_win32::write_protected(address as *mut usize, restore.original)
                        }
                    {
                        log::warn!(
                            "failed to roll back RakClient vtable slot {}: {error:?}",
                            restore.slot
                        );
                    }
                }
                return Err(AttachError::HookInstallFailed("patching RakClient vtable"));
            }
        }

        Ok(Self { vtable, entries })
    }
}

impl Drop for VtableHook {
    fn drop(&mut self) {
        for entry in self.entries.iter().rev() {
            let Some(slot) = vtable_slot_address(self.vtable, entry.slot) else {
                continue;
            };
            if unsafe { modkit_win32::read_unaligned::<usize>(slot) } == Some(entry.detour)
                && let Err(error) =
                    unsafe { modkit_win32::write_protected(slot as *mut usize, entry.original) }
            {
                log::warn!(
                    "failed to restore RakClient vtable slot {}: {error:?}",
                    entry.slot
                );
            }
        }
    }
}

fn vtable_slot_address(vtable: usize, slot: usize) -> Option<usize> {
    slot.checked_mul(mem::size_of::<usize>())
        .and_then(|offset| vtable.checked_add(offset))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vtable_slot_address_rejects_arithmetic_overflow() {
        assert_eq!(vtable_slot_address(usize::MAX, 1), None);
        assert_eq!(vtable_slot_address(0, usize::MAX), None);
    }

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
