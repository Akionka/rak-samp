//! Host module resolution and connection.

use crate::host::{Host, HostStatus};
use core::{fmt, mem};
use std::time::{Duration, Instant};

/// The default host module name, NUL-terminated.
pub const DEFAULT_HOST_MODULE: &[u8] = b"gta_mod_host.asi\0";

/// A failure while resolving or connecting to the modkit host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConnectError {
    /// The current platform cannot load the host module.
    UnsupportedPlatform,
    /// The host module is not loaded in the process.
    HostNotLoaded,
    /// The host module does not export `GtaModHost_GetApiV1`.
    MissingApi,
    /// The host reported an unsupported bootstrap ABI version.
    UnsupportedAbi,
    /// The host failed to initialize.
    HostFailed,
    /// Timed out waiting for host readiness.
    TimedOut,
}

impl fmt::Display for ConnectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => formatter.write_str("modkit plugins require Windows x86"),
            Self::HostNotLoaded => formatter.write_str("modkit host module is not loaded"),
            Self::MissingApi => {
                formatter.write_str("modkit host does not export GtaModHost_GetApiV1")
            }
            Self::UnsupportedAbi => {
                formatter.write_str("modkit host bootstrap ABI v1 is unavailable")
            }
            Self::HostFailed => formatter.write_str("modkit host failed to initialize"),
            Self::TimedOut => formatter.write_str("timed out waiting for modkit host readiness"),
        }
    }
}

impl std::error::Error for ConnectError {}

/// Waits for the default `gta_mod_host.asi` host to expose a ready bootstrap.
///
/// Call this from a plugin worker thread, never from `DllMain`.
pub(crate) fn wait_for_default_host(timeout: Duration) -> Result<Host, ConnectError> {
    wait_for_host(DEFAULT_HOST_MODULE, timeout)
}

/// Waits for a named host module to expose a ready bootstrap.
///
/// `module_name` must be NUL-terminated, for example `b"gta_mod_host.asi\\0"`.
pub(crate) fn wait_for_host(module_name: &[u8], timeout: Duration) -> Result<Host, ConnectError> {
    if module_name.last() != Some(&0) {
        return Err(ConnectError::HostNotLoaded);
    }
    wait_for_ready_host(timeout, || resolve_host(module_name), |host| host.status())
}

fn wait_for_ready_host<T>(
    timeout: Duration,
    mut resolve: impl FnMut() -> Result<T, ConnectError>,
    status: impl Fn(&T) -> HostStatus,
) -> Result<T, ConnectError> {
    let started = Instant::now();
    loop {
        match resolve() {
            Ok(host) => match status(&host) {
                HostStatus::Ready => return Ok(host),
                HostStatus::Failed | HostStatus::ShuttingDown => {
                    return Err(ConnectError::HostFailed);
                }
                HostStatus::Waiting => {}
            },
            Err(ConnectError::HostNotLoaded | ConnectError::MissingApi) => {}
            Err(error) => return Err(error),
        }
        if started.elapsed() >= timeout {
            return Err(ConnectError::TimedOut);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[cfg(all(windows, target_arch = "x86"))]
fn resolve_host(module_name: &[u8]) -> Result<Host, ConnectError> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };
    if module.is_null() {
        return Err(ConnectError::HostNotLoaded);
    }
    let symbol = unsafe { GetProcAddress(module, c"GtaModHost_GetApiV1".as_ptr().cast()) };
    let Some(symbol) = symbol else {
        return Err(ConnectError::MissingApi);
    };
    let get_api: modkit_abi::GetModHostApiV1 = unsafe { mem::transmute(symbol) };
    let mut out: *const modkit_abi::ModHostApiV1 = core::ptr::null();
    let result = unsafe { get_api(&mut out) };
    if !result.is_ok() {
        return Err(ConnectError::HostFailed);
    }
    let Some(api) = (unsafe { out.as_ref() }) else {
        return Err(ConnectError::HostFailed);
    };
    if api.abi_version != modkit_abi::MOD_HOST_ABI_VERSION_V1
        || api.size < mem::size_of::<modkit_abi::ModHostApiV1>() as u32
    {
        return Err(ConnectError::UnsupportedAbi);
    }
    Ok(unsafe { Host::from_raw(api) })
}

#[cfg(not(all(windows, target_arch = "x86")))]
fn resolve_host(_module_name: &[u8]) -> Result<Host, ConnectError> {
    Err(ConnectError::UnsupportedPlatform)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_waiter_retries_until_the_host_reports_ready() {
        use std::collections::VecDeque;
        let mut outcomes = VecDeque::from([
            Err(ConnectError::HostNotLoaded),
            Ok(HostStatus::Waiting),
            Ok(HostStatus::Ready),
        ]);
        let status = wait_for_ready_host(
            Duration::from_millis(50),
            || outcomes.pop_front().expect("fixture has an outcome"),
            |status| *status,
        )
        .expect("ready status resolves the host");
        assert_eq!(status, HostStatus::Ready);
        assert!(outcomes.is_empty());
    }

    #[test]
    fn host_waiter_returns_the_host_failure() {
        assert_eq!(
            wait_for_ready_host(Duration::ZERO, || Ok(HostStatus::Failed), |status| *status,),
            Err(ConnectError::HostFailed)
        );
    }

    #[test]
    fn host_waiter_times_out_while_the_host_is_pending() {
        assert_eq!(
            wait_for_ready_host(Duration::ZERO, || Ok(HostStatus::Waiting), |status| *status,),
            Err(ConnectError::TimedOut)
        );
    }

    #[test]
    fn default_host_module_is_nul_terminated() {
        assert_eq!(DEFAULT_HOST_MODULE.last(), Some(&0));
    }
}
