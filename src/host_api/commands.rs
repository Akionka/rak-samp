//! Command receipt polling, waiting, and release ABI entry points.

use super::{clone_initialized, host};
use crate::command::CommandError;
use sdk_abi::{
    SampClientSdkCommandReceipt, SampClientSdkCommandResultV1, SampClientSdkResult,
    SampClientSdkTextLabelCreateResultV1,
};
use std::time::Duration;

pub(super) unsafe extern "system" fn command_try_take(
    receipt: SampClientSdkCommandReceipt,
    output: *mut SampClientSdkCommandResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.try_take_command(receipt.id) {
        Ok(Some(result)) => {
            unsafe {
                output.write(SampClientSdkCommandResultV1 {
                    status: command_completion_result(result),
                });
            }
            SampClientSdkResult::Ok
        }
        Ok(None) => SampClientSdkResult::CommandPending,
        Err(error) => command_error_result(error),
    }
}

pub(super) unsafe extern "system" fn command_wait(
    receipt: SampClientSdkCommandReceipt,
    timeout_ms: u32,
    output: *mut SampClientSdkCommandResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.wait_for_command(receipt.id, Duration::from_millis(u64::from(timeout_ms))) {
        Ok(result) => {
            unsafe {
                output.write(SampClientSdkCommandResultV1 {
                    status: command_completion_result(result),
                });
            }
            SampClientSdkResult::Ok
        }
        Err(error) => command_error_result(error),
    }
}

pub(super) unsafe extern "system" fn command_release(
    receipt: SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    runtime
        .release_command(receipt.id)
        .map_or_else(command_error_result, |_| SampClientSdkResult::Ok)
}

pub(super) unsafe extern "system" fn text_label_create_try_take(
    receipt: SampClientSdkCommandReceipt,
    output: *mut SampClientSdkTextLabelCreateResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.try_take_created_text_label(receipt.id) {
        Ok(Some(result)) => {
            let (status, id) = text_label_create_completion(result);
            unsafe {
                output.write(SampClientSdkTextLabelCreateResultV1 {
                    status,
                    id,
                    reserved: 0,
                });
            }
            SampClientSdkResult::Ok
        }
        Ok(None) => SampClientSdkResult::CommandPending,
        Err(error) => command_error_result(error),
    }
}

pub(super) unsafe extern "system" fn text_label_create_wait(
    receipt: SampClientSdkCommandReceipt,
    timeout_ms: u32,
    output: *mut SampClientSdkTextLabelCreateResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime
        .wait_for_created_text_label(receipt.id, Duration::from_millis(u64::from(timeout_ms)))
    {
        Ok(result) => {
            let (status, id) = text_label_create_completion(result);
            unsafe {
                output.write(SampClientSdkTextLabelCreateResultV1 {
                    status,
                    id,
                    reserved: 0,
                });
            }
            SampClientSdkResult::Ok
        }
        Err(error) => command_error_result(error),
    }
}

fn command_completion_result(result: Result<(), CommandError>) -> SampClientSdkResult {
    result.map_or_else(command_error_result, |_| SampClientSdkResult::Ok)
}

fn text_label_create_completion(result: Result<u16, CommandError>) -> (SampClientSdkResult, u16) {
    match result {
        Ok(id) => (SampClientSdkResult::Ok, id),
        Err(error) => (command_error_result(error), 0),
    }
}

fn command_error_result(error: CommandError) -> SampClientSdkResult {
    match error {
        CommandError::QueueFull | CommandError::IdExhausted => SampClientSdkResult::QueueFull,
        CommandError::ShuttingDown => SampClientSdkResult::ShuttingDown,
        CommandError::NativeFailure => SampClientSdkResult::NativeCallFailed,
        CommandError::UnknownReceipt => SampClientSdkResult::InvalidArgument,
        CommandError::TimedOut => SampClientSdkResult::TimedOut,
        CommandError::WaitRejected => SampClientSdkResult::WaitRejected,
    }
}
