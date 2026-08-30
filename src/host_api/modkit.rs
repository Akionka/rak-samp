//! Modkit host bootstrap and exact-version service discovery.
//!
//! This module introduces the new `GtaModHost_GetApiV1` export and the
//! `query_service` registry beside the legacy `SampClientSdk_GetApiV1` export.
//! It implements the Core service v1 and the migration-only Legacy SA-MP
//! service wrapper. The legacy export is left unchanged.

use super::{clone_initialized, host, unregister};
use crate::command::CommandError;
use log::{debug, error, info, warn};
use modkit_abi::{
    CommandCompletionV1, CommandReceiptId, CoreServiceV1, HostStatusV1, LegacySampServiceV1,
    MOD_HOST_ABI_VERSION_V1, MOD_INVALID_ARGUMENT, MOD_NOT_FOUND, MOD_NOT_READY, MOD_OK,
    MOD_SHUTTING_DOWN, MOD_UNSUPPORTED_VERSION, ModHostApiV1, ModResult, SERVICE_ID_CORE,
    SERVICE_ID_LEGACY_SAMP_ABI, ServiceHeader, SubscriptionId,
};
use sdk_abi::{SampClientSdkResult, SampClientSdkSubscription};
use std::{ffi::c_void, ptr, sync::atomic::Ordering, time::Duration};

/// The published Core service version.
const CORE_SERVICE_VERSION: u32 = 1;
/// The published Legacy SA-MP service version.
const LEGACY_SERVICE_VERSION: u32 = 1;

/// The host-owned immutable bootstrap table.
static MOD_HOST_API_V1: ModHostApiV1 = ModHostApiV1 {
    abi_version: MOD_HOST_ABI_VERSION_V1,
    size: std::mem::size_of::<ModHostApiV1>() as u32,
    query_service,
};

/// The host-owned immutable Core service table.
static CORE_SERVICE_V1: CoreServiceV1 = CoreServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_CORE,
        version: CORE_SERVICE_VERSION,
        size: std::mem::size_of::<CoreServiceV1>() as u32,
        reserved: 0,
    },
    host_status: core_host_status,
    unregister: core_unregister,
    unregister_and_wait: core_unregister_and_wait,
    receipt_poll: core_receipt_poll,
    receipt_wait: core_receipt_wait,
    receipt_release: core_receipt_release,
    log_utf8: core_log_utf8,
};

/// The host-owned immutable Legacy SA-MP service table.
static LEGACY_SAMP_SERVICE_V1: std::sync::OnceLock<LegacySampServiceV1> =
    std::sync::OnceLock::new();

fn legacy_samp_service() -> &'static LegacySampServiceV1 {
    LEGACY_SAMP_SERVICE_V1.get_or_init(|| LegacySampServiceV1 {
        header: ServiceHeader {
            service_id: SERVICE_ID_LEGACY_SAMP_ABI,
            version: LEGACY_SERVICE_VERSION,
            size: std::mem::size_of::<LegacySampServiceV1>() as u32,
            reserved: 0,
        },
        api: (&super::SAMP_CLIENT_SDK_API_V1 as *const sdk_abi::SampClientSdkApiV1)
            .cast::<c_void>(),
    })
}

/// The new host bootstrap export.
///
/// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
///
/// # Safety
///
/// `out_api` must be null or point to writable storage for one pointer.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn GtaModHost_GetApiV1(out_api: *mut *const ModHostApiV1) -> ModResult {
    if out_api.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    unsafe { out_api.write(&MOD_HOST_API_V1) };
    MOD_OK
}

/// Exact-version service discovery.
///
/// # Safety
///
/// `out_service` must be null or point to writable storage for one pointer.
unsafe extern "system" fn query_service(
    service: modkit_abi::ServiceId,
    requested_version: u32,
    out_service: *mut *const ServiceHeader,
) -> ModResult {
    if out_service.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    unsafe { out_service.write(ptr::null()) };

    if host().shutting_down.load(Ordering::Acquire) {
        return MOD_SHUTTING_DOWN;
    }

    match service {
        SERVICE_ID_CORE => {
            if requested_version != CORE_SERVICE_VERSION {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe { out_service.write((&CORE_SERVICE_V1 as *const CoreServiceV1).cast()) };
            MOD_OK
        }
        SERVICE_ID_LEGACY_SAMP_ABI => {
            if requested_version != LEGACY_SERVICE_VERSION {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service.write((legacy_samp_service() as *const LegacySampServiceV1).cast())
            };
            MOD_OK
        }
        _ => MOD_NOT_FOUND,
    }
}

unsafe extern "system" fn core_host_status(out: *mut HostStatusV1) -> ModResult {
    if out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let state = if host().shutting_down.load(Ordering::Acquire) {
        HostStatusV1::STATE_SHUTTING_DOWN
    } else {
        match host().status.load(Ordering::Acquire) {
            super::STATUS_READY => HostStatusV1::STATE_READY,
            super::STATUS_FAILED => HostStatusV1::STATE_FAILED,
            _ => HostStatusV1::STATE_WAITING,
        }
    };
    unsafe {
        out.write(HostStatusV1 {
            state,
            reserved: [0; 3],
        })
    };
    MOD_OK
}

unsafe extern "system" fn core_unregister(id: SubscriptionId) -> ModResult {
    if id.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let result = unsafe { unregister(SampClientSdkSubscription { id: id.0 }) };
    subscription_result(result)
}

unsafe extern "system" fn core_unregister_and_wait(
    id: SubscriptionId,
    timeout_ms: u32,
) -> ModResult {
    if id.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    if !runtime.command_wait_allowed() {
        return modkit_abi::MOD_WAIT_REJECTED;
    }
    let result = super::listeners::unregister_and_wait_with_timeout(
        SampClientSdkSubscription { id: id.0 },
        Some(timeout_duration(timeout_ms)),
    );
    subscription_result(result)
}

fn subscription_result(result: SampClientSdkResult) -> ModResult {
    match result {
        SampClientSdkResult::Ok => MOD_OK,
        SampClientSdkResult::NotReady => MOD_NOT_READY,
        SampClientSdkResult::InvalidArgument => MOD_INVALID_ARGUMENT,
        SampClientSdkResult::UnsupportedVersion => MOD_UNSUPPORTED_VERSION,
        SampClientSdkResult::SubscriptionNotFound => MOD_NOT_FOUND,
        SampClientSdkResult::ReadOutOfBounds => modkit_abi::MOD_OUT_OF_BOUNDS,
        SampClientSdkResult::PayloadTooLarge => modkit_abi::MOD_PAYLOAD_TOO_LARGE,
        SampClientSdkResult::NativeCallFailed => modkit_abi::MOD_NATIVE_CALL_FAILED,
        SampClientSdkResult::CallbackInProgress => modkit_abi::MOD_CALLBACK_IN_PROGRESS,
        SampClientSdkResult::QueueFull => modkit_abi::MOD_QUEUE_FULL,
        SampClientSdkResult::CommandPending => modkit_abi::MOD_PENDING,
        SampClientSdkResult::TimedOut => modkit_abi::MOD_TIMED_OUT,
        SampClientSdkResult::WaitRejected => modkit_abi::MOD_WAIT_REJECTED,
        SampClientSdkResult::ShuttingDown => MOD_SHUTTING_DOWN,
        SampClientSdkResult::Busy => modkit_abi::MOD_BUSY,
    }
}

unsafe extern "system" fn core_receipt_poll(
    id: CommandReceiptId,
    out: *mut CommandCompletionV1,
) -> ModResult {
    if id.is_zero() || out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.try_take_command(id.0) {
        Ok(Some(result)) => {
            unsafe { out.write(completion(result)) };
            MOD_OK
        }
        Ok(None) => modkit_abi::MOD_PENDING,
        Err(error) => command_error_result(error),
    }
}

unsafe extern "system" fn core_receipt_wait(
    id: CommandReceiptId,
    timeout_ms: u32,
    out: *mut CommandCompletionV1,
) -> ModResult {
    if id.is_zero() || out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    if !runtime.command_wait_allowed() {
        return modkit_abi::MOD_WAIT_REJECTED;
    }
    match runtime.wait_for_command(id.0, timeout_duration(timeout_ms)) {
        Ok(result) => {
            unsafe { out.write(completion(result)) };
            MOD_OK
        }
        Err(error) => command_error_result(error),
    }
}

unsafe extern "system" fn core_receipt_release(id: CommandReceiptId) -> ModResult {
    if id.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    runtime
        .release_command(id.0)
        .map_or_else(command_error_result, |_| MOD_OK)
}

unsafe extern "system" fn core_log_utf8(level: u32, ptr: *const u8, len: u32) -> ModResult {
    if (ptr.is_null() && len != 0) || len > modkit_abi::MAX_LOG_MESSAGE_BYTES {
        return MOD_INVALID_ARGUMENT;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len as usize) }
    };
    let message = String::from_utf8_lossy(bytes);
    match level {
        modkit_abi::LOG_LEVEL_ERROR => error!("{message}"),
        modkit_abi::LOG_LEVEL_WARN => warn!("{message}"),
        modkit_abi::LOG_LEVEL_INFO => info!("{message}"),
        modkit_abi::LOG_LEVEL_DEBUG => debug!("{message}"),
        _ => return MOD_INVALID_ARGUMENT,
    }
    MOD_OK
}

fn timeout_duration(timeout_ms: u32) -> Duration {
    if timeout_ms == modkit_abi::TIMEOUT_INFINITE {
        Duration::MAX
    } else {
        Duration::from_millis(u64::from(timeout_ms))
    }
}

fn completion(result: Result<(), CommandError>) -> CommandCompletionV1 {
    match result {
        Ok(()) => CommandCompletionV1::default(),
        Err(error) => CommandCompletionV1 {
            status: command_error_result(error),
            reserved: 0,
            value0: 0,
            value1: 0,
        },
    }
}

fn command_error_result(error: CommandError) -> ModResult {
    match error {
        CommandError::QueueFull => modkit_abi::MOD_QUEUE_FULL,
        CommandError::IdExhausted => modkit_abi::MOD_BUSY,
        CommandError::ShuttingDown => MOD_SHUTTING_DOWN,
        CommandError::NativeFailure => modkit_abi::MOD_NATIVE_CALL_FAILED,
        CommandError::UnknownReceipt => MOD_INVALID_ARGUMENT,
        CommandError::TimedOut => modkit_abi::MOD_TIMED_OUT,
        CommandError::WaitRejected => modkit_abi::MOD_WAIT_REJECTED,
    }
}

/// Marks the host as shutting down so discovery and new operations fail closed.
pub(crate) fn begin_shutdown() {
    host().shutting_down.store(true, Ordering::Release);
}

#[cfg(test)]
mod tests;
