//! Cached local chat-input ABI reads.

use super::{
    clone_initialized, copied_nul_free_string, direct_client_result, host, submit_direct_command,
};
use sdk_abi::{SampClientSdkChatInputTextV1, SampClientSdkCommandReceipt, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_local_chat_input_text(
    text: *const u8,
    text_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 128) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_chat_input_text(text)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_chat_input_enabled(
    enabled: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || enabled > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_chat_input_enabled(enabled != 0)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_chat_input_process(
    text: *const u8,
    text_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 128) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_chat_input_process(text)
        })
    }
}

pub(super) unsafe extern "system" fn local_chat_input_text(
    output: *mut SampClientSdkChatInputTextV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_input_text() {
        Ok(text) => {
            if text.len() > output.bytes.len() {
                return SampClientSdkResult::NativeCallFailed;
            }
            *output = SampClientSdkChatInputTextV1::default();
            output.len = text.len() as u8;
            output.bytes[..text.len()].copy_from_slice(&text);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_chat_command_defined(
    name: *const u8,
    name_len: usize,
    output: *mut u8,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, 32) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if name.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_command_defined(&name) {
        Ok(defined) => {
            *output = u8::from(defined);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
