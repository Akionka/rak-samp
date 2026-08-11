//! Local UI command ABI entry points.

use super::submit_direct_command;
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_local_cursor_toggle(
    show: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || show > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_cursor_toggle(show != 0)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_chat_display_mode(
    mode: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(mode, 0..=2) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_chat_display_mode(mode)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_cursor_mode(
    mode: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(mode, 0..=4) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_local_cursor_mode(mode)) }
}

pub(super) unsafe extern "system" fn submit_local_scoreboard_open(
    open: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(open, 0 | 1) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_scoreboard_open(open != 0)
        })
    }
}
