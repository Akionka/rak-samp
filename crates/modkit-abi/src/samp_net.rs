//! SA-MP Packet/RPC service v1 ABI.

use core::ffi::c_void;

use crate::{CommandReceiptId, ModResult, SampReleaseCallbackV1, ServiceHeader, SubscriptionId};

pub const SAMP_NET_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_NET_DIRECTION_INCOMING: u32 = 0;
pub const SAMP_NET_DIRECTION_OUTGOING: u32 = 1;
pub const SAMP_NET_ACTION_CONTINUE: u32 = 0;
pub const SAMP_NET_ACTION_BLOCK: u32 = 1;

pub const SAMP_NET_PRIORITY_SYSTEM: u32 = 0;
pub const SAMP_NET_PRIORITY_HIGH: u32 = 1;
pub const SAMP_NET_PRIORITY_MEDIUM: u32 = 2;
pub const SAMP_NET_PRIORITY_LOW: u32 = 3;
pub const SAMP_NET_RELIABILITY_UNRELIABLE: u32 = 6;
pub const SAMP_NET_RELIABILITY_UNRELIABLE_SEQUENCED: u32 = 7;
pub const SAMP_NET_RELIABILITY_RELIABLE: u32 = 8;
pub const SAMP_NET_RELIABILITY_RELIABLE_ORDERED: u32 = 9;
pub const SAMP_NET_RELIABILITY_RELIABLE_SEQUENCED: u32 = 10;

/// Opaque callback-local event. It is invalid after the callback returns.
#[repr(C)]
pub struct SampNetEventV1 {
    private: [u8; 0],
}

pub type SampNetEventCallbackV1 =
    unsafe extern "system" fn(user_data: *mut c_void, event: *mut SampNetEventV1) -> u32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampNetSendOptionsV1 {
    pub priority: u32,
    pub reliability: u32,
    pub ordering_channel: u8,
    pub timestamp: u8,
    pub reserved: [u8; 2],
}

impl Default for SampNetSendOptionsV1 {
    fn default() -> Self {
        Self {
            priority: SAMP_NET_PRIORITY_HIGH,
            reliability: SAMP_NET_RELIABILITY_RELIABLE_ORDERED,
            ordering_channel: 0,
            timestamp: 0,
            reserved: [0; 2],
        }
    }
}

/// Small immutable SA-MP Packet/RPC service table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SampNetServiceV1 {
    pub header: ServiceHeader,
    pub register_packet: unsafe extern "system" fn(
        direction: u32,
        callback: Option<SampNetEventCallbackV1>,
        user_data: *mut c_void,
        release: Option<SampReleaseCallbackV1>,
        out_subscription: *mut SubscriptionId,
    ) -> ModResult,
    pub register_rpc: unsafe extern "system" fn(
        direction: u32,
        callback: Option<SampNetEventCallbackV1>,
        user_data: *mut c_void,
        release: Option<SampReleaseCallbackV1>,
        out_subscription: *mut SubscriptionId,
    ) -> ModResult,
    pub event_id:
        unsafe extern "system" fn(event: *const SampNetEventV1, out: *mut u8) -> ModResult,
    pub event_reset: unsafe extern "system" fn(event: *mut SampNetEventV1) -> ModResult,
    pub event_remaining_bits:
        unsafe extern "system" fn(event: *const SampNetEventV1, out: *mut u32) -> ModResult,
    pub event_read_bits: unsafe extern "system" fn(
        event: *mut SampNetEventV1,
        out: *mut u8,
        out_capacity: u32,
        bit_len: u32,
    ) -> ModResult,
    pub event_replace_bits: unsafe extern "system" fn(
        event: *mut SampNetEventV1,
        data: *const u8,
        byte_len: u32,
        bit_len: u32,
    ) -> ModResult,
    pub encode_string: unsafe extern "system" fn(
        value: *const u8,
        value_len: u32,
        out: *mut u8,
        out_capacity: u32,
        out_byte_len: *mut u32,
        out_bit_len: *mut u32,
    ) -> ModResult,
    pub event_read_encoded_string: unsafe extern "system" fn(
        event: *mut SampNetEventV1,
        out: *mut u8,
        out_capacity: u32,
        out_len: *mut u32,
    ) -> ModResult,
    pub submit_packet: unsafe extern "system" fn(
        id: u8,
        data: *const u8,
        byte_len: u32,
        bit_len: u32,
        options: SampNetSendOptionsV1,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_rpc: unsafe extern "system" fn(
        id: u8,
        data: *const u8,
        byte_len: u32,
        bit_len: u32,
        options: SampNetSendOptionsV1,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_emulate_incoming_packet: unsafe extern "system" fn(
        id: u8,
        data: *const u8,
        byte_len: u32,
        bit_len: u32,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_emulate_incoming_rpc: unsafe extern "system" fn(
        id: u8,
        data: *const u8,
        byte_len: u32,
        bit_len: u32,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub incoming_emulation_ready: unsafe extern "system" fn(out: *mut u8) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_options_layout_is_fixed() {
        assert_eq!(core::mem::size_of::<SampNetSendOptionsV1>(), 12);
        assert_eq!(core::mem::align_of::<SampNetSendOptionsV1>(), 4);
        assert_eq!(core::mem::offset_of!(SampNetSendOptionsV1, timestamp), 9);
        assert_eq!(SampNetSendOptionsV1::default().reserved, [0; 2]);
    }

    #[test]
    fn samp_net_service_layout_is_fixed() {
        let pointer_size = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(SampNetServiceV1, header), 0);
        assert_eq!(core::mem::offset_of!(SampNetServiceV1, register_packet), 16);
        assert_eq!(
            core::mem::offset_of!(SampNetServiceV1, incoming_emulation_ready),
            16 + 13 * pointer_size
        );
        assert_eq!(
            core::mem::size_of::<SampNetServiceV1>(),
            16 + 14 * pointer_size
        );
    }
}
