//! Safe wrappers over the modkit host bootstrap and service tables.

use modkit_abi::{
    CommandCompletionV1, CommandReceiptId, CoreServiceV1, HostStatusV1, LegacySampServiceV1,
    ModHostApiV1, ModResult, SERVICE_ID_CORE, SERVICE_ID_LEGACY_SAMP_ABI, ServiceHeader,
    SubscriptionId,
};
use std::time::Duration;

use crate::resolve::{ConnectError, wait_for_default_host, wait_for_host};

/// Host lifecycle status reported by the Core service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostStatus {
    /// The host is still initializing.
    Waiting,
    /// The host is ready to serve plugins.
    Ready,
    /// The host failed to initialize.
    Failed,
    /// The host has begun shutdown.
    ShuttingDown,
}

impl HostStatus {
    fn from_raw(state: u32) -> Self {
        match state {
            HostStatusV1::STATE_READY => Self::Ready,
            HostStatusV1::STATE_FAILED => Self::Failed,
            HostStatusV1::STATE_SHUTTING_DOWN => Self::ShuttingDown,
            _ => Self::Waiting,
        }
    }
}

/// A connected modkit host.
///
/// The bootstrap table is host-owned, immutable, and valid for the process
/// lifetime. Host hot-unload is not supported.
#[derive(Clone, Copy)]
pub struct Host {
    api: &'static ModHostApiV1,
}

impl Host {
    /// Connects to the default `gta_mod_host.asi` host.
    ///
    /// Call this from a plugin worker thread, never from `DllMain`.
    pub fn connect(timeout: Duration) -> Result<Self, ConnectError> {
        wait_for_default_host(timeout)
    }

    /// Connects to a named host module. `module_name` must be NUL-terminated.
    ///
    /// Call this from a plugin worker thread, never from `DllMain`.
    pub fn connect_to(module_name: &[u8], timeout: Duration) -> Result<Self, ConnectError> {
        wait_for_host(module_name, timeout)
    }

    /// Wraps a validated bootstrap table.
    ///
    /// # Safety
    ///
    /// `api` must point to a live, immutable bootstrap table exported by a
    /// compatible host and remain valid for the process lifetime.
    pub(crate) unsafe fn from_raw(api: &'static ModHostApiV1) -> Self {
        Self { api }
    }

    /// Returns the host lifecycle status through the Core service.
    pub fn status(self) -> HostStatus {
        match self.query_service(SERVICE_ID_CORE, 1) {
            Ok(Service::Core(core)) => {
                let mut out = HostStatusV1 {
                    state: u32::MAX,
                    reserved: [u32::MAX; 3],
                };
                let result = unsafe { (core.host_status)(&mut out) };
                if result.is_ok() {
                    HostStatus::from_raw(out.state)
                } else {
                    HostStatus::Failed
                }
            }
            Ok(Service::LegacySamp(_)) => HostStatus::Failed,
            Err(_) => HostStatus::Failed,
        }
    }

    /// Queries an exact service ID + version pair.
    ///
    /// Returns `Ok` with the service table when the exact pair is published,
    /// regardless of native backend readiness. Unknown services return
    /// [`ServiceError::NotFound`]; known-but-unavailable versions return
    /// [`ServiceError::UnsupportedVersion`].
    pub fn query_service(self, service_id: u32, version: u32) -> Result<Service, ServiceError> {
        let mut out: *const ServiceHeader = core::ptr::null();
        let result = unsafe { (self.api.query_service)(service_id, version, &mut out) };
        match result {
            r if r.is_ok() => {
                let Some(header) = (unsafe { out.as_ref() }) else {
                    return Err(ServiceError::HostFailed);
                };
                if !header.matches(service_id, version, 0) {
                    return Err(ServiceError::HostFailed);
                }
                Service::from_header(header).ok_or(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED))
            }
            r if r == modkit_abi::MOD_NOT_FOUND => Err(ServiceError::NotFound),
            r if r == modkit_abi::MOD_UNSUPPORTED_VERSION => Err(ServiceError::UnsupportedVersion),
            r if r == modkit_abi::MOD_NOT_READY => Err(ServiceError::NotReady),
            r if r == modkit_abi::MOD_SHUTTING_DOWN => Err(ServiceError::ShuttingDown),
            r => Err(ServiceError::Host(r)),
        }
    }

    /// Returns the Core service v1.
    pub fn core(self) -> Result<Core, ServiceError> {
        match self.query_service(SERVICE_ID_CORE, 1)? {
            Service::Core(core) => Ok(Core { core }),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the migration-only Legacy SA-MP service v1.
    pub fn legacy_samp(self) -> Result<&'static LegacySampServiceV1, ServiceError> {
        match self.query_service(SERVICE_ID_LEGACY_SAMP_ABI, 1)? {
            Service::LegacySamp(legacy) => Ok(legacy),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }
}

/// A resolved service table.
#[derive(Clone, Copy)]
pub enum Service {
    /// The Core service v1.
    Core(&'static CoreServiceV1),
    /// The migration-only Legacy SA-MP service v1.
    LegacySamp(&'static LegacySampServiceV1),
}

impl Service {
    fn from_header(header: &'static ServiceHeader) -> Option<Self> {
        match header.service_id {
            SERVICE_ID_CORE => {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<CoreServiceV1>()
                        .as_ref()
                };
                table.map(Service::Core)
            }
            SERVICE_ID_LEGACY_SAMP_ABI => {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<LegacySampServiceV1>()
                        .as_ref()
                };
                table.map(Service::LegacySamp)
            }
            _ => None,
        }
    }
}

/// A failure while querying a service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceError {
    /// The service ID is unknown to the host.
    NotFound,
    /// The requested version of a known service is unavailable.
    UnsupportedVersion,
    /// The host registry is not yet published.
    NotReady,
    /// The host is shutting down.
    ShuttingDown,
    /// The host returned an unexpected result code.
    Host(ModResult),
    /// The host returned a malformed or unexpected service table.
    HostFailed,
}

/// The Core service v1 facade.
#[derive(Clone, Copy)]
pub struct Core {
    core: &'static CoreServiceV1,
}

impl Core {
    /// Returns the host lifecycle status.
    pub fn host_status(self) -> Result<HostStatus, ModResult> {
        let mut out = HostStatusV1 {
            state: u32::MAX,
            reserved: [u32::MAX; 3],
        };
        let result = unsafe { (self.core.host_status)(&mut out) };
        if result.is_ok() {
            Ok(HostStatus::from_raw(out.state))
        } else {
            Err(result)
        }
    }

    /// Removes a subscription without waiting for in-flight callbacks.
    pub fn unregister(self, id: SubscriptionId) -> Result<(), ModResult> {
        let result = unsafe { (self.core.unregister)(id) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }

    /// Removes a subscription and waits for in-flight callbacks to drain.
    ///
    /// This may block; it is rejected from the game thread, `DllMain`, and host
    /// callbacks with [`modkit_abi::MOD_WAIT_REJECTED`].
    pub fn unregister_and_wait(
        self,
        id: SubscriptionId,
        timeout: Duration,
    ) -> Result<(), ModResult> {
        let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
        let result = unsafe { (self.core.unregister_and_wait)(id, timeout_ms) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }

    /// Polls a command receipt without blocking.
    pub fn receipt_poll(
        self,
        id: CommandReceiptId,
    ) -> Result<Option<CommandCompletionV1>, ModResult> {
        let mut out = CommandCompletionV1::default();
        let result = unsafe { (self.core.receipt_poll)(id, &mut out) };
        match result {
            r if r.is_ok() => Ok(Some(out)),
            r if r == modkit_abi::MOD_PENDING => Ok(None),
            r => Err(r),
        }
    }

    /// Waits for a command receipt completion.
    ///
    /// This may block; it is rejected from the game thread, `DllMain`, and host
    /// callbacks with [`modkit_abi::MOD_WAIT_REJECTED`].
    pub fn receipt_wait(
        self,
        id: CommandReceiptId,
        timeout: Duration,
    ) -> Result<CommandCompletionV1, ModResult> {
        let timeout_ms = timeout.as_millis().min(u128::from(u32::MAX)) as u32;
        let mut out = CommandCompletionV1::default();
        let result = unsafe { (self.core.receipt_wait)(id, timeout_ms, &mut out) };
        if result.is_ok() { Ok(out) } else { Err(result) }
    }

    /// Detaches a command receipt without cancelling its owned command.
    pub fn receipt_release(self, id: CommandReceiptId) -> Result<(), ModResult> {
        let result = unsafe { (self.core.receipt_release)(id) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }

    /// Logs a UTF-8 message through the host logger.
    ///
    /// `level` is `0`=error, `1`=warn, `2`=info, `3`=debug.
    pub fn log_utf8(self, level: u32, message: &[u8]) -> Result<(), ModResult> {
        let result = unsafe { (self.core.log_utf8)(level, message.as_ptr(), message.len() as u32) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }
}
