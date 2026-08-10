//! Connection lifecycle command ABI entry points.

use super::{clone_initialized, copied_nul_free_string, direct_client_result, host};
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_connect_to_server(address, port) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn submit_disconnect_with_reason(
    block_duration: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_disconnect_with_reason(block_duration) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
