//! Cached local chat-input ABI reads.

use super::{clone_initialized, direct_client_result, host};
use sdk_abi::{SampClientSdkChatInputTextV1, SampClientSdkResult};

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
