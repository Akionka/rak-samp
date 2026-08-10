//! Cached pooled-record ABI reads.

use super::{clone_initialized, conversions, direct_client_result, host};
use sdk_abi::limits::{
    MAX_SAMP_CHAT_ENTRIES, MAX_SAMP_GANGZONES, MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS,
};
use sdk_abi::{
    SampClientSdkChatEntryV1, SampClientSdkGangzoneV1, SampClientSdkResult,
    SampClientSdkTextDrawV1, SampClientSdkTextLabelV1,
};

pub(super) unsafe extern "system" fn gangzone_info(
    id: u16,
    output: *mut SampClientSdkGangzoneV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_GANGZONES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.gangzone(id) {
        Ok(Some(snapshot)) => match conversions::gangzone_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkGangzoneV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn text_label_info(
    id: u16,
    output: *mut SampClientSdkTextLabelV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.text_label(id) {
        Ok(Some(snapshot)) => match conversions::text_label_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkTextLabelV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn textdraw_info(
    pool_index: u16,
    output: *mut SampClientSdkTextDrawV1,
) -> SampClientSdkResult {
    if pool_index >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.textdraw(pool_index) {
        Ok(Some(snapshot)) => match conversions::textdraw_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkTextDrawV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn chat_entry_info(
    id: u16,
    output: *mut SampClientSdkChatEntryV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_CHAT_ENTRIES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.chat_entry(id) {
        Ok(snapshot) => match conversions::chat_entry_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Err(error) => direct_client_result(error),
    }
}
