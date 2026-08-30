//! Native RakNet ABI declarations and wire-value mappings.

use super::*;

pub(super) use samp_native::hooks::RpcPlayerId;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub(super) struct PacketPlayerId {
    pub(super) binary_address: u32,
    pub(super) port: u16,
}

#[repr(C, packed)]
pub(super) struct RawPacket {
    pub(super) player_index: u16,
    pub(super) player_id: PacketPlayerId,
    pub(super) length: u32,
    pub(super) bit_size: u32,
    pub(super) data: *mut u8,
    pub(super) delete_data: bool,
}

pub(super) type StringWriteEncoderFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, i32, *mut RawBitStream, i32);
pub(super) type StringReadDecoderFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut i8, i32, *mut RawBitStream, i32) -> bool;
pub(super) type OutgoingPacketFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut RawBitStream, i32, i32, i8) -> bool;
pub(super) type OutgoingRpcFn = unsafe extern "thiscall" fn(
    *mut c_void,
    *mut i32,
    *mut RawBitStream,
    i32,
    i32,
    i8,
    bool,
) -> bool;
pub(super) type IncomingRpcFn =
    unsafe extern "thiscall" fn(*mut c_void, *mut u8, i32, RpcPlayerId) -> bool;
pub(super) type AllocatePacketFn = unsafe extern "C" fn(i32) -> *mut RawPacket;
pub(super) type QueueWriteLockFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut *mut RawPacket;
pub(super) type QueueWriteUnlockFn = unsafe extern "thiscall" fn(*mut c_void);

pub(super) const fn priority_value(priority: PacketPriority) -> i32 {
    match priority {
        PacketPriority::System => 0,
        PacketPriority::High => 1,
        PacketPriority::Medium => 2,
        PacketPriority::Low => 3,
    }
}

pub(super) const fn reliability_value(reliability: PacketReliability) -> i32 {
    match reliability {
        PacketReliability::Unreliable => 6,
        PacketReliability::UnreliableSequenced => 7,
        PacketReliability::Reliable => 8,
        PacketReliability::ReliableOrdered => 9,
        PacketReliability::ReliableSequenced => 10,
    }
}
