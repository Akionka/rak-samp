//! Local-player action command ABI entry points.

use super::{copied_nul_free_string, submit_direct_command};
use sdk_abi::limits::MAX_SAMP_PLAYERS;
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_player_colour(
    id: u16,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_player_colour(id, colour)) }
}

pub(super) unsafe extern "system" fn submit_local_player_name(
    name: *const u8,
    name_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, 255) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_local_player_name(name)) }
}

pub(super) unsafe extern "system" fn submit_local_player_spawn(
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_local_player_spawn()) }
}

pub(super) unsafe extern "system" fn submit_local_player_special_action(
    action: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(action, 0..=12 | 20..=25 | 68) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_player_special_action(action)
        })
    }
}
