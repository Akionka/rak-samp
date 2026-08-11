//! Cached player ABI reads.

use super::{clone_initialized, conversions, direct_client_result, host};
use sdk_abi::limits::MAX_SAMP_PLAYERS;
use sdk_abi::{
    SampClientSdkAimSyncV1, SampClientSdkInCarSyncV1, SampClientSdkLocalPlayerV1,
    SampClientSdkOnFootSyncV1, SampClientSdkPassengerSyncV1, SampClientSdkPlayerInfoV1,
    SampClientSdkRemotePlayerStateV1, SampClientSdkResult, SampClientSdkTrailerSyncV1,
};

pub(super) unsafe extern "system" fn local_player(
    output: *mut SampClientSdkLocalPlayerV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_player() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = conversions::local_player_to_abi(snapshot) else {
        return SampClientSdkResult::NativeCallFailed;
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn player_info(
    id: u16,
    output: *mut SampClientSdkPlayerInfoV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_info(id) {
        Ok(Some(snapshot)) => match conversions::player_info_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkPlayerInfoV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn remote_player_state(
    id: u16,
    output: *mut SampClientSdkRemotePlayerStateV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.remote_player_state(id) {
        Ok(Some(snapshot)) => match conversions::remote_player_state_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkRemotePlayerStateV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn onfoot_sync(
    id: u16,
    output: *mut SampClientSdkOnFootSyncV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.onfoot_sync(id) {
        Ok(Some(snapshot)) => match conversions::onfoot_sync_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkOnFootSyncV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn vehicle_sync(
    id: u16,
    output: *mut SampClientSdkInCarSyncV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.vehicle_sync(id) {
        Ok(Some(snapshot)) => match conversions::vehicle_sync_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkInCarSyncV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn passenger_sync(
    id: u16,
    output: *mut SampClientSdkPassengerSyncV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.passenger_sync(id) {
        Ok(Some(snapshot)) => match conversions::passenger_sync_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkPassengerSyncV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn trailer_sync(
    id: u16,
    output: *mut SampClientSdkTrailerSyncV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.trailer_sync(id) {
        Ok(Some(snapshot)) => match conversions::trailer_sync_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkTrailerSyncV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn aim_sync(
    id: u16,
    output: *mut SampClientSdkAimSyncV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.aim_sync(id) {
        Ok(Some(snapshot)) => match conversions::aim_sync_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkAimSyncV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn player_defined(
    id: u16,
    output: *mut u8,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_defined(id) {
        Ok(defined) => {
            *output = u8::from(defined);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn player_paused(
    id: u16,
    output: *mut u8,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_paused(id) {
        Ok(paused) => {
            *output = u8::from(paused);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn player_count(
    include_npcs: u8,
    output: *mut u16,
) -> SampClientSdkResult {
    let include_npcs = match include_npcs {
        0 => false,
        1 => true,
        _ => return SampClientSdkResult::InvalidArgument,
    };
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_count(include_npcs) {
        Ok(count) => {
            *output = count;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn player_max_id(output: *mut u16) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_max_id() {
        Ok(id) => {
            *output = id;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
