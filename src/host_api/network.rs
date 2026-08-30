//! Packet/RPC send and incoming-emulation ABI entry points.

use super::{ListenerKind, clone_initialized, host, is_shutting_down};
use crate::{BitStream, BitStreamError, PacketPriority, PacketReliability, SendError, SendOptions};
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult, SampClientSdkSendOptions};

/// Reports whether the host has captured the native receiver required to emulate an incoming
/// packet. This copies no native address across the ABI.
pub(super) extern "system" fn incoming_emulation_ready() -> u8 {
    u8::from(
        clone_initialized(&host().runtime)
            .is_some_and(|runtime| runtime.incoming_emulation_ready()),
    )
}

pub(super) unsafe extern "system" fn send_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
) -> SampClientSdkResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Packet)
}

pub(super) unsafe extern "system" fn send_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
) -> SampClientSdkResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Rpc)
}

pub(super) unsafe extern "system" fn emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Packet)
}

pub(super) unsafe extern "system" fn emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Rpc)
}

pub(super) unsafe extern "system" fn submit_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_send(
        id,
        data,
        byte_len,
        bit_len,
        options,
        ListenerKind::Packet,
        receipt,
    )
}

pub(super) unsafe extern "system" fn submit_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_send(
        id,
        data,
        byte_len,
        bit_len,
        options,
        ListenerKind::Rpc,
        receipt,
    )
}

pub(super) unsafe extern "system" fn submit_emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Packet, receipt)
}

pub(super) unsafe extern "system" fn submit_emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Rpc, receipt)
}

fn send(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    kind: ListenerKind,
) -> SampClientSdkResult {
    if is_shutting_down() {
        return SampClientSdkResult::ShuttingDown;
    }
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(options) = send_options(options) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.send_packet_with_options(id, &payload, options),
        ListenerKind::Rpc => runtime.send_rpc_with_options(id, &payload, options),
    };
    result.map_or_else(send_result, |sent| {
        if sent {
            SampClientSdkResult::Ok
        } else {
            SampClientSdkResult::NativeCallFailed
        }
    })
}

fn submit_send(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    kind: ListenerKind,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if is_shutting_down() {
        return SampClientSdkResult::ShuttingDown;
    }
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(options) = send_options(options) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.submit_packet_with_options(id, &payload, options),
        ListenerKind::Rpc => runtime.submit_rpc_with_options(id, &payload, options),
    };
    match result {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => send_result(error),
    }
}

fn emulate_incoming(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    kind: ListenerKind,
) -> SampClientSdkResult {
    if is_shutting_down() {
        return SampClientSdkResult::ShuttingDown;
    }
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.emulate_incoming_packet(id, payload),
        ListenerKind::Rpc => runtime.emulate_incoming_rpc(id, payload),
    };
    result.map_or_else(send_result, |_| SampClientSdkResult::Ok)
}

fn submit_emulate_incoming(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    kind: ListenerKind,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if is_shutting_down() {
        return SampClientSdkResult::ShuttingDown;
    }
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.submit_emulate_incoming_packet(id, payload),
        ListenerKind::Rpc => runtime.submit_emulate_incoming_rpc(id, payload),
    };
    match result {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => send_result(error),
    }
}

unsafe fn stream_from_abi(
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> Result<BitStream, BitStreamError> {
    if data.is_null() && byte_len != 0 {
        return Err(BitStreamError::InvalidOffset {
            offset_bits: bit_len,
            length_bits: 0,
        });
    }
    let bytes = if byte_len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data, byte_len) }.to_vec()
    };
    BitStream::from_bytes_with_bits(bytes, bit_len)
}

fn send_result(error: SendError) -> SampClientSdkResult {
    match error {
        SendError::ClientNotReady => SampClientSdkResult::NotReady,
        SendError::QueueFull => SampClientSdkResult::QueueFull,
        SendError::PayloadTooLarge => SampClientSdkResult::PayloadTooLarge,
        SendError::NativeCallFailed => SampClientSdkResult::NativeCallFailed,
        SendError::TimestampedPacketUnsupported => SampClientSdkResult::InvalidArgument,
    }
}

fn send_options(options: SampClientSdkSendOptions) -> Result<SendOptions, ()> {
    let priority = match options.priority {
        0 => PacketPriority::System,
        1 => PacketPriority::High,
        2 => PacketPriority::Medium,
        3 => PacketPriority::Low,
        _ => return Err(()),
    };
    let reliability = match options.reliability {
        6 => PacketReliability::Unreliable,
        7 => PacketReliability::UnreliableSequenced,
        8 => PacketReliability::Reliable,
        9 => PacketReliability::ReliableOrdered,
        10 => PacketReliability::ReliableSequenced,
        _ => return Err(()),
    };
    Ok(SendOptions {
        priority,
        reliability,
        ordering_channel: options.ordering_channel,
        timestamp: options.timestamp,
    })
}
