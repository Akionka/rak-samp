use crate::{
    ABI_VERSION_V1, DEFAULT_HOST_MODULE, HostApi, SampClientSdkGetApiV1, SampClientSdkHostStatus,
};
use core::{fmt, mem};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    UnsupportedPlatform,
    HostNotLoaded,
    MissingApi,
    UnsupportedAbi,
    HostFailed,
    TimedOut,
}

impl fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("samp-client-sdk plugins require Windows")
            }
            Self::HostNotLoaded => formatter.write_str("samp-client-sdk host module is not loaded"),
            Self::MissingApi => {
                formatter.write_str("samp-client-sdk host does not export SampClientSdk_GetApiV1")
            }
            Self::UnsupportedAbi => {
                formatter.write_str("samp-client-sdk host ABI v1 is unavailable")
            }
            Self::HostFailed => formatter.write_str("samp-client-sdk host failed to initialize"),
            Self::TimedOut => {
                formatter.write_str("timed out waiting for samp-client-sdk host readiness")
            }
        }
    }
}

impl std::error::Error for ResolveError {}

/// Waits for the default `samp_client_sdk.asi` host to expose a ready v1 API.
///
/// Call this from a plugin worker thread, never from `DllMain`.
pub fn wait_for_default_host(timeout: Duration) -> Result<HostApi, ResolveError> {
    wait_for_host(DEFAULT_HOST_MODULE, timeout)
}

/// Waits for a named host module to expose a ready v1 API.
///
/// `module_name` must be NUL-terminated, for example `b"samp_client_sdk.asi\\0"`.
pub fn wait_for_host(module_name: &[u8], timeout: Duration) -> Result<HostApi, ResolveError> {
    if module_name.last() != Some(&0) {
        return Err(ResolveError::HostNotLoaded);
    }
    wait_for_ready_host(timeout, || resolve_host(module_name), |api| api.status())
}

fn wait_for_ready_host<T>(
    timeout: Duration,
    mut resolve: impl FnMut() -> Result<T, ResolveError>,
    status: impl Fn(&T) -> SampClientSdkHostStatus,
) -> Result<T, ResolveError> {
    let started = Instant::now();
    loop {
        match resolve() {
            Ok(host) => match status(&host) {
                SampClientSdkHostStatus::Ready => return Ok(host),
                SampClientSdkHostStatus::Failed => return Err(ResolveError::HostFailed),
                SampClientSdkHostStatus::WaitingForSamp => {}
            },
            Err(ResolveError::HostNotLoaded | ResolveError::MissingApi) => {}
            Err(error) => return Err(error),
        }
        if started.elapsed() >= timeout {
            return Err(ResolveError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(all(windows, target_arch = "x86"))]
fn resolve_host(module_name: &[u8]) -> Result<HostApi, ResolveError> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };
    if module.is_null() {
        return Err(ResolveError::HostNotLoaded);
    }
    let symbol = unsafe { GetProcAddress(module, c"SampClientSdk_GetApiV1".as_ptr().cast()) };
    let Some(symbol) = symbol else {
        return Err(ResolveError::MissingApi);
    };
    let get_api: SampClientSdkGetApiV1 = unsafe { mem::transmute(symbol) };
    let raw = unsafe { get_api(ABI_VERSION_V1) };
    unsafe { HostApi::from_raw(raw) }
}

#[cfg(not(all(windows, target_arch = "x86")))]
fn resolve_host(_module_name: &[u8]) -> Result<HostApi, ResolveError> {
    Err(ResolveError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    #[test]
    fn host_waiter_retries_until_the_host_worker_reports_ready() {
        let mut outcomes = VecDeque::from([
            Err(ResolveError::HostNotLoaded),
            Ok(SampClientSdkHostStatus::WaitingForSamp),
            Ok(SampClientSdkHostStatus::Ready),
        ]);

        let status = wait_for_ready_host(
            Duration::from_millis(50),
            || outcomes.pop_front().expect("fixture has an outcome"),
            |status| *status,
        )
        .expect("ready status resolves the host");

        assert_eq!(status, SampClientSdkHostStatus::Ready);
        assert!(outcomes.is_empty());
    }

    #[test]
    fn host_waiter_returns_the_host_worker_failure() {
        assert_eq!(
            wait_for_ready_host(
                Duration::ZERO,
                || Ok(SampClientSdkHostStatus::Failed),
                |status| *status,
            ),
            Err(ResolveError::HostFailed)
        );
    }

    #[test]
    fn host_waiter_times_out_while_the_host_worker_is_pending() {
        assert_eq!(
            wait_for_ready_host(
                Duration::ZERO,
                || Ok(SampClientSdkHostStatus::WaitingForSamp),
                |status| *status,
            ),
            Err(ResolveError::TimedOut)
        );
    }
}
