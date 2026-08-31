//! Exact-version SA-MP connection and replication-control service ABI.

use crate::{CommandReceiptId, ModResult, ServiceHeader};

pub const SAMP_CONTROL_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_SEND_RATE_ON_FOOT: u32 = 0;
pub const SAMP_SEND_RATE_IN_CAR: u32 = 1;
pub const SAMP_SEND_RATE_AIM: u32 = 2;

/// `ANY_THREAD + CALLBACK_SAFE`; every operation queues copied work.
#[repr(C)]
pub struct SampControlServiceV1 {
    pub header: ServiceHeader,
    pub submit_game_state:
        unsafe extern "system" fn(state: i32, out_receipt: *mut CommandReceiptId) -> ModResult,
    pub submit_send_rate: unsafe extern "system" fn(
        kind: u32,
        milliseconds: u32,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_connect: unsafe extern "system" fn(
        address: *const u8,
        address_len: u32,
        port: u16,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_disconnect: unsafe extern "system" fn(
        block_duration: u32,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_layout_is_header_plus_four_functions() {
        assert_eq!(
            core::mem::offset_of!(SampControlServiceV1, submit_game_state),
            16
        );
        assert_eq!(
            core::mem::size_of::<SampControlServiceV1>(),
            16 + 4 * core::mem::size_of::<usize>()
        );
    }
}
