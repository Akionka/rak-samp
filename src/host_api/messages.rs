//! Local display-message command ABI entry points.

use super::{clone_initialized, copied_nul_free_string, direct_client_result, host};
use crate::runtime::{LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest};
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_local_chat_message(
    style: u32,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    text_colour: u32,
    prefix_colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(style) = LocalChatMessageStyle::from_raw(style) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 143) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(prefix) = (unsafe { copied_nul_free_string(prefix, prefix_len, 27) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_chat_message(LocalChatMessageRequest {
        style,
        text,
        prefix,
        text_colour,
        prefix_colour,
    }) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn submit_local_death_message(
    killer: *const u8,
    killer_len: usize,
    victim: *const u8,
    victim_len: usize,
    killer_colour: u32,
    victim_colour: u32,
    weapon: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(killer) = (unsafe { copied_nul_free_string(killer, killer_len, 24) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(victim) = (unsafe { copied_nul_free_string(victim, victim_len, 24) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_death_message(LocalDeathMessageRequest {
        killer,
        victim,
        killer_colour,
        victim_colour,
        weapon,
    }) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
