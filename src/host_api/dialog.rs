//! Cached local-dialog detail ABI reads.

use super::{clone_initialized, conversions, direct_client_result, host};
use sdk_abi::{SampClientSdkDialogSnapshotV1, SampClientSdkResult};

pub(super) unsafe extern "system" fn local_dialog_selected_item(
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_dialog_selected_item() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_dialog_list_item_count(
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_dialog_list_item_count() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_dialog_snapshot(
    output: *mut SampClientSdkDialogSnapshotV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_dialog_state() {
        Ok(Some(snapshot)) => match conversions::local_dialog_snapshot_to_abi(snapshot) {
            Ok(snapshot) => snapshot,
            Err(()) => return SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => SampClientSdkDialogSnapshotV1::default(),
        Err(error) => return direct_client_result(error),
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}
