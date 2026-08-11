//! Connection lifecycle command ABI entry points.

use super::{copied_nul_free_string, submit_direct_command};
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_connect_to_server(
    address: *const u8,
    address_len: usize,
    port: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || port == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(address) = (unsafe { copied_nul_free_string(address, address_len, 256) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if address.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_connect_to_server(address, port)
        })
    }
}

pub(super) unsafe extern "system" fn submit_disconnect_with_reason(
    block_duration: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_disconnect_with_reason(block_duration)
        })
    }
}
