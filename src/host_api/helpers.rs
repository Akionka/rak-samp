use super::{HOST, HostState, STATUS_WAITING};
use crate::Runtime;
use crate::runtime::DirectClientError;
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
};

pub(super) fn direct_client_result(error: DirectClientError) -> SampClientSdkResult {
    match error {
        DirectClientError::InvalidArgument => SampClientSdkResult::InvalidArgument,
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
    if host().shutting_down.load(Ordering::Acquire) {
        return SampClientSdkResult::ShuttingDown;
    }
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
        shutting_down: AtomicBool::new(false),
        runtime: OnceLock::new(),
        chat_commands: super::chat_commands::ChatCommandRegistry::new(),
        subscriptions: Mutex::new(HashMap::new()),
        next_subscription: AtomicU64::new(1),
    })
}

pub(super) fn clone_initialized<T>(slot: &OnceLock<Arc<T>>) -> Option<Arc<T>> {
    slot.get().cloned()
}

pub(super) fn next_subscription_id() -> Option<u64> {
    allocate_monotonic_id(&host().next_subscription)
}

pub(super) fn is_shutting_down() -> bool {
    host().shutting_down.load(Ordering::Acquire)
}

fn allocate_monotonic_id(next: &AtomicU64) -> Option<u64> {
    loop {
        let current = next.load(Ordering::Acquire);
        if current == 0 {
            return None;
        }
        let following = current.checked_add(1).unwrap_or(0);
        if next
            .compare_exchange_weak(current, following, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            return Some(current);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{allocate_monotonic_id, direct_client_result};
    use crate::runtime::DirectClientError;
    use sdk_abi::SampClientSdkResult;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn subscription_ids_exhaust_without_zero_or_reuse() {
        let next = AtomicU64::new(u64::MAX);
        assert_eq!(allocate_monotonic_id(&next), Some(u64::MAX));
        assert_eq!(allocate_monotonic_id(&next), None);
    }

    #[test]
    fn invalid_direct_client_argument_maps_to_the_stable_abi_result() {
        assert_eq!(
            direct_client_result(DirectClientError::InvalidArgument),
            SampClientSdkResult::InvalidArgument
        );
    }
}
