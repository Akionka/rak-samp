//! Opaque native-address ABI entry points.

use super::{clone_initialized, host};
use crate::Runtime;
use core::ffi::c_void;
use sdk_abi::SampClientSdkResult;
use std::ptr;

pub(super) unsafe extern "system" fn raw_rakclient(
    output: *mut *mut c_void,
) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_rakclient)
}

pub(super) unsafe extern "system" fn raw_rakpeer(output: *mut *mut c_void) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_rakpeer)
}

pub(super) unsafe extern "system" fn raw_player_pool(
    output: *mut *mut c_void,
) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_player_pool)
}

pub(super) unsafe extern "system" fn raw_vehicle_pool(
    output: *mut *mut c_void,
) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_vehicle_pool)
}

pub(super) unsafe extern "system" fn raw_local_player(
    output: *mut *mut c_void,
) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_local_player)
}

fn raw_native_address(
    output: *mut *mut c_void,
    lookup: fn(&Runtime) -> Option<*mut c_void>,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = ptr::null_mut();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let Some(address) = lookup(&runtime) else {
        return SampClientSdkResult::NotReady;
    };
    *output = address;
    SampClientSdkResult::Ok
}
