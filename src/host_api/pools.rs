//! Cached pool-existence ABI reads.

use super::{clone_initialized, direct_client_result, host};
use sdk_abi::SampClientSdkResult;
use sdk_abi::limits::{
    MAX_SAMP_OBJECTS, MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES,
};

pub(super) unsafe extern "system" fn vehicle_exists(
    id: u16,
    output: *mut u8,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.vehicle_exists(id) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn text_label_exists(
    id: u16,
    output: *mut u8,
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
    match runtime.text_label_exists(id) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn textdraw_exists(
    pool_index: u16,
    output: *mut u8,
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
    match runtime.textdraw_exists(pool_index) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn object_exists(
    id: u16,
    output: *mut u8,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_OBJECTS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.object_exists(id) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
