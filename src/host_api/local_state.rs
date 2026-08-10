//! Cached local UI-state ABI reads.

use super::{clone_initialized, conversions, direct_client_result, host};
use sdk_abi::{SampClientSdkActiveDialogV1, SampClientSdkResult};

pub(super) unsafe extern "system" fn local_chat_display_mode(
    output: *mut i32,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_display_mode() {
        Ok(mode) => {
            *output = mode;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_cursor_mode(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_cursor_mode() {
        Ok(mode) => {
            *output = mode;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_scoreboard_open(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_scoreboard_open() {
        Ok(open) => {
            *output = u8::from(open);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_dialog_active(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_dialog_active() {
        Ok(active) => {
            *output = u8::from(active);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn active_local_dialog(
    output: *mut SampClientSdkActiveDialogV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_dialog_state() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let snapshot = match snapshot {
        Some(snapshot) => match conversions::local_dialog_state_to_abi(&snapshot) {
            Ok(snapshot) => snapshot,
            Err(()) => return SampClientSdkResult::NativeCallFailed,
        },
        None => SampClientSdkActiveDialogV1::default(),
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

pub(super) unsafe extern "system" fn local_chat_input_active(
    output: *mut u8,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_input_active() {
        Ok(active) => {
            *output = u8::from(active);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}
