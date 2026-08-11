use super::{HOST, HostState, STATUS_WAITING};
use crate::Runtime;
use crate::runtime::DirectClientError;
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, AtomicU64},
    },
};

pub(super) fn direct_client_result(error: DirectClientError) -> SampClientSdkResult {
    match error {
        DirectClientError::NotReady => SampClientSdkResult::NotReady,
        DirectClientError::Busy => SampClientSdkResult::Busy,
        DirectClientError::UnsupportedVersion => SampClientSdkResult::UnsupportedVersion,
        DirectClientError::QueueFull => SampClientSdkResult::QueueFull,
    }
}

/// Submits a validated receipt-bearing direct-client command through the
/// initialized runtime and writes its ID only after successful queueing.
///
/// # Safety
///
/// `receipt` must be non-null and writable. Callers validate it before reading
/// any other ABI pointer, preserving each export's input-validation order.
pub(super) unsafe fn submit_direct_command(
    receipt: *mut SampClientSdkCommandReceipt,
    submit: impl FnOnce(&Runtime) -> Result<u64, DirectClientError>,
) -> SampClientSdkResult {
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    unsafe { finish_direct_command(receipt, submit(&runtime)) }
}

/// # Safety
///
/// `receipt` must be non-null and writable.
pub(super) unsafe fn finish_direct_command(
    receipt: *mut SampClientSdkCommandReceipt,
    result: Result<u64, DirectClientError>,
) -> SampClientSdkResult {
    match result {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe fn copied_nul_free_string(
    value: *const u8,
    value_len: usize,
    maximum: usize,
) -> Result<Vec<u8>, ()> {
    if value_len > maximum || (value.is_null() && value_len != 0) {
        return Err(());
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len) }
    };
    if value.contains(&0) {
        return Err(());
    }
    Ok(value.to_vec())
}

pub(super) fn host() -> &'static HostState {
    HOST.get_or_init(|| HostState {
        status: AtomicU32::new(STATUS_WAITING),
        bootstrap_started: AtomicBool::new(false),
        runtime: OnceLock::new(),
        subscriptions: Mutex::new(HashMap::new()),
        next_subscription: AtomicU64::new(1),
    })
}

pub(super) fn clone_initialized<T>(slot: &OnceLock<Arc<T>>) -> Option<Arc<T>> {
    slot.get().cloned()
}
