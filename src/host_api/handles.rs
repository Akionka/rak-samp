//! Forward and reverse native-handle lookup ABI entry points.

use super::{clone_initialized, direct_client_result, host};
use crate::{Runtime, runtime::DirectClientError};
use sdk_abi::SampClientSdkResult;
use std::sync::Arc;

pub(super) unsafe extern "system" fn local_object_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    scalar_option_read(output, |runtime| runtime.object_handle(id))
}

pub(super) unsafe extern "system" fn local_object_id_by_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    id_option_read(output, |runtime| runtime.object_id_by_handle(handle))
}

pub(super) unsafe extern "system" fn local_pickup_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    scalar_option_read(output, |runtime| runtime.pickup_handle(id))
}

pub(super) unsafe extern "system" fn local_pickup_id_by_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    id_option_read(output, |runtime| runtime.pickup_id_by_handle(handle))
}

pub(super) unsafe extern "system" fn local_vehicle_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    scalar_option_read(output, |runtime| runtime.vehicle_handle(id))
}

pub(super) unsafe extern "system" fn local_vehicle_id_by_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    id_option_read(output, |runtime| runtime.vehicle_id_by_handle(handle))
}

pub(super) unsafe extern "system" fn local_player_ped_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    scalar_option_read(output, |runtime| runtime.player_ped_handle(id))
}

pub(super) unsafe extern "system" fn local_player_id_by_ped_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    id_option_read(output, |runtime| runtime.player_id_by_ped_handle(handle))
}

fn scalar_option_read(
    output: *mut i32,
    read: impl FnOnce(&Arc<Runtime>) -> Result<Option<i32>, DirectClientError>,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match read(&runtime) {
        Ok(value) => {
            *output = value.unwrap_or(0);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

fn id_option_read(
    output: *mut u16,
    read: impl FnOnce(&Arc<Runtime>) -> Result<Option<u16>, DirectClientError>,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match read(&runtime) {
        Ok(value) => {
            *output = value.unwrap_or(u16::MAX);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
