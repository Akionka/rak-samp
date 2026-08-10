//! Cached SAMP environment and metadata ABI reads.

use super::{clone_initialized, conversions, direct_client_result, host};
use crate::SampVersion;
use sdk_abi::{SampClientSdkResult, SampClientSdkServerInfoV1};

pub(super) unsafe extern "system" fn samp_game_state(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.samp_game_state() {
        Ok(game_state) => {
            *output = game_state;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn server_info(
    output: *mut SampClientSdkServerInfoV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.server_info() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = conversions::server_info_to_abi(snapshot) else {
        return SampClientSdkResult::NativeCallFailed;
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn samp_version(output: *mut u32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    *output = samp_version_to_abi(runtime.samp_version());
    SampClientSdkResult::Ok
}

pub(super) const fn samp_version_to_abi(version: SampVersion) -> u32 {
    match version {
        SampVersion::R1 => 1,
        SampVersion::R2 => 2,
        SampVersion::R3_1 => 3,
        SampVersion::R4_2 => 4,
        SampVersion::R5_1 => 5,
        SampVersion::Dl => 6,
    }
}
