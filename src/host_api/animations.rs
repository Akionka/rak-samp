//! Cached local animation-table ABI reads.

use super::{clone_initialized, conversions, copied_nul_free_string, direct_client_result, host};
use sdk_abi::{SampClientSdkAnimationV1, SampClientSdkResult};

pub(super) unsafe extern "system" fn local_animation(
    id: u16,
    output: *mut SampClientSdkAnimationV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_animation(id) {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = conversions::animation_to_abi(snapshot) else {
        return SampClientSdkResult::NativeCallFailed;
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn local_animation_id(
    name: *const u8,
    name_len: usize,
    file: *const u8,
    file_len: usize,
    output: *mut i32,
) -> SampClientSdkResult {
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, 35) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(file) = (unsafe { copied_nul_free_string(file, file_len, 35) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if name.is_empty() || file.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_animation_id(&name, &file) {
        Ok(Some(id)) => {
            *output = i32::from(id);
            SampClientSdkResult::Ok
        }
        Ok(None) => {
            *output = -1;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
