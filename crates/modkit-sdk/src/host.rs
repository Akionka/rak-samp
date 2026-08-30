//! Safe wrappers over the modkit host bootstrap and service tables.

use modkit_abi::{
    CommandCompletionV1, CommandReceiptId, CoreServiceV1, GTA_SA_SERVICE_VERSION_V1,
    GtaPedSnapshotV1, GtaReleaseCallbackV1, GtaSaServiceV1, GtaTickCallbackV1, GtaVector3V1,
    HostStatusV1, LegacySampServiceV1, ModHostApiV1, ModResult, SAMP_NET_SERVICE_VERSION_V1,
    SAMP_SERVICE_VERSION_V1, SERVICE_ID_CORE, SERVICE_ID_GTA_SA, SERVICE_ID_LEGACY_SAMP_ABI,
    SERVICE_ID_SAMP, SERVICE_ID_SAMP_NETWORK, SampChatCommandCallbackV1, SampLocalPlayerV1,
    SampNetEventCallbackV1, SampNetEventV1, SampNetSendOptionsV1, SampNetServiceV1,
    SampPlayerInfoV1, SampReleaseCallbackV1, SampServerInfoV1, SampServiceV1, ServiceHeader,
    ServiceId, SubscriptionId,
};
use std::ffi::c_void;
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
    /// Connects to the default `samp_client_sdk.asi` host.
    ///
    /// `MAY_BLOCK` while the host initializes.
    ///
    /// Call this from a plugin worker thread, never from `DllMain`.
    pub fn connect(timeout: Duration) -> Result<Self, ConnectError> {
        wait_for_default_host(timeout)
    }

    /// Connects to a named host module. `module_name` must be NUL-terminated.
    ///
    /// `MAY_BLOCK` while the host initializes.
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
    #[cfg(all(windows, target_arch = "x86"))]
    pub(crate) unsafe fn from_raw(api: &'static ModHostApiV1) -> Self {
        Self { api }
    }

    /// Returns the host lifecycle status through the Core service.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub fn status(self) -> HostStatus {
        match self.core() {
            Ok(core) => core.host_status().unwrap_or(HostStatus::Failed),
            Err(ServiceError::NotReady) => HostStatus::Waiting,
            Err(_) => HostStatus::Failed,
        }
    }

    /// Queries an exact service ID + version pair.
    ///
    /// Returns `Ok` with the service table when the exact pair is published,
    /// regardless of native backend readiness. Unknown services return
    /// [`ServiceError::NotFound`]; known-but-unavailable versions return
    /// [`ServiceError::UnsupportedVersion`].
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub fn query_service(
        self,
        service_id: ServiceId,
        version: u32,
    ) -> Result<Service, ServiceError> {
        let mut out: *const ServiceHeader = core::ptr::null();
        let result = unsafe { (self.api.query_service)(service_id, version, &mut out) };
        match result {
            r if r.is_ok() => {
                let Some(header) = (unsafe { out.as_ref() }) else {
                    return Err(ServiceError::HostFailed);
                };
                Service::from_header(header, service_id, version).ok_or(ServiceError::HostFailed)
            }
            r if r == modkit_abi::MOD_NOT_FOUND => Err(ServiceError::NotFound),
            r if r == modkit_abi::MOD_UNSUPPORTED_VERSION => Err(ServiceError::UnsupportedVersion),
            r if r == modkit_abi::MOD_NOT_READY => Err(ServiceError::NotReady),
            r if r == modkit_abi::MOD_SHUTTING_DOWN => Err(ServiceError::ShuttingDown),
            r => Err(ServiceError::Host(r)),
        }
    }

    /// Returns the Core service v1.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub fn core(self) -> Result<Core, ServiceError> {
        match self.query_service(SERVICE_ID_CORE, 1)? {
            Service::Core(core) => Ok(core),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the migration-only Legacy SA-MP service v1.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub fn legacy_samp(self) -> Result<LegacySamp, ServiceError> {
        match self.query_service(SERVICE_ID_LEGACY_SAMP_ABI, 1)? {
            Service::LegacySamp(legacy) => Ok(legacy),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the general SA-MP service v1.
    pub fn samp(self) -> Result<SampService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP, SAMP_SERVICE_VERSION_V1)? {
            Service::Samp(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP Packet/RPC service v1.
    pub fn samp_net(self) -> Result<SampNetService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_NETWORK, SAMP_NET_SERVICE_VERSION_V1)? {
            Service::SampNet(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the GTA San Andreas service v1.
    pub fn gta_sa_service(self) -> Result<GtaSaService, ServiceError> {
        match self.query_service(SERVICE_ID_GTA_SA, GTA_SA_SERVICE_VERSION_V1)? {
            Service::GtaSa(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }
}

/// A resolved service table.
#[derive(Clone, Copy)]
pub enum Service {
    /// The Core service v1.
    Core(Core),
    /// The GTA San Andreas service v1.
    GtaSa(GtaSaService),
    /// The migration-only Legacy SA-MP service v1.
    LegacySamp(LegacySamp),
    /// The general SA-MP service v1.
    Samp(SampService),
    /// The SA-MP Packet/RPC service v1.
    SampNet(SampNetService),
}

impl Service {
    fn from_header(
        header: &'static ServiceHeader,
        service_id: ServiceId,
        version: u32,
    ) -> Option<Self> {
        match header.service_id {
            SERVICE_ID_CORE
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<CoreServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<CoreServiceV1>()
                        .as_ref()
                };
                table.map(|core| Service::Core(Core { core }))
            }
            SERVICE_ID_GTA_SA
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<GtaSaServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<GtaSaServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::GtaSa(GtaSaService { table }))
            }
            SERVICE_ID_LEGACY_SAMP_ABI
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<LegacySampServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<LegacySampServiceV1>()
                        .as_ref()
                };
                table.map(|legacy| Service::LegacySamp(LegacySamp { legacy }))
            }
            SERVICE_ID_SAMP
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::Samp(SampService { table }))
            }
            SERVICE_ID_SAMP_NETWORK
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampNetServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampNetServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampNet(SampNetService { table }))
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

/// Validated low-level view of the GTA San Andreas service v1.
#[derive(Clone, Copy)]
pub struct GtaSaService {
    table: &'static GtaSaServiceV1,
}

/// Safe availability view of the migration-only Legacy SA-MP service v1.
#[derive(Clone, Copy)]
pub struct LegacySamp {
    legacy: &'static LegacySampServiceV1,
}

/// Validated low-level view of the general SA-MP service v1.
#[derive(Clone, Copy)]
pub struct SampService {
    table: &'static SampServiceV1,
}

/// Validated low-level view of the SA-MP Packet/RPC service v1.
#[derive(Clone, Copy)]
pub struct SampNetService {
    table: &'static SampNetServiceV1,
}

impl GtaSaService {
    /// Registers one raw GTA tick callback.
    ///
    /// # Safety
    ///
    /// `user_data` must remain valid until `release` runs. `callback` and
    /// `release` must remain loaded for the same interval and must not unwind
    /// across the ABI boundary.
    pub unsafe fn register_tick(
        self,
        callback: GtaTickCallbackV1,
        user_data: *mut c_void,
        release: GtaReleaseCallbackV1,
    ) -> Result<SubscriptionId, ModResult> {
        let mut out = SubscriptionId(0);
        let result = unsafe {
            (self.table.register_tick)(Some(callback), user_data, Some(release), &mut out)
        };
        result_with_out(result, out)
    }

    pub fn local_ped_snapshot(
        self,
        context: &crate::GameContext<'_>,
    ) -> Result<GtaPedSnapshotV1, ModResult> {
        let mut out = GtaPedSnapshotV1::default();
        result_with_out(
            unsafe { (self.table.local_ped_snapshot)(context.token(), &mut out) },
            out,
        )
    }

    pub fn teleport_local_ped(
        self,
        context: &crate::GameContext<'_>,
        destination: GtaVector3V1,
    ) -> Result<(), ModResult> {
        result_unit(unsafe { (self.table.teleport_local_ped)(context.token(), destination) })
    }

    pub fn submit_local_ped_snapshot(self) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_local_ped_snapshot)(&mut out) },
            out,
        )
    }

    pub fn take_local_ped_snapshot(
        self,
        receipt: CommandReceiptId,
    ) -> Result<GtaPedSnapshotV1, ModResult> {
        let mut out = GtaPedSnapshotV1::default();
        result_with_out(
            unsafe { (self.table.take_local_ped_snapshot)(receipt, &mut out) },
            out,
        )
    }

    pub fn submit_teleport_local_ped(
        self,
        destination: GtaVector3V1,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_teleport_local_ped)(destination, &mut out) },
            out,
        )
    }
}

impl SampService {
    pub fn version(self) -> Result<u32, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.version)(&mut out) }, out)
    }

    pub fn game_state(self) -> Result<i32, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.game_state)(&mut out) }, out)
    }

    pub fn server_info(self) -> Result<SampServerInfoV1, ModResult> {
        let mut out = SampServerInfoV1::default();
        result_with_out(unsafe { (self.table.server_info)(&mut out) }, out)
    }

    pub fn local_player(self) -> Result<SampLocalPlayerV1, ModResult> {
        let mut out = SampLocalPlayerV1::default();
        result_with_out(unsafe { (self.table.local_player)(&mut out) }, out)
    }

    pub fn player_info(self, id: u16) -> Result<SampPlayerInfoV1, ModResult> {
        let mut out = SampPlayerInfoV1::default();
        result_with_out(unsafe { (self.table.player_info)(id, &mut out) }, out)
    }

    pub fn submit_chat_add(
        self,
        style: u32,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        let prefix_len = checked_len(prefix.len())?;
        let mut receipt = CommandReceiptId(0);
        let result = unsafe {
            (self.table.submit_chat_add)(
                style,
                text.as_ptr(),
                text_len,
                prefix.as_ptr(),
                prefix_len,
                text_colour,
                prefix_colour,
                &mut receipt,
            )
        };
        result_with_out(result, receipt)
    }

    /// Registers a callback whose opaque context remains owned by the host
    /// after successful submission.
    ///
    /// # Safety
    ///
    /// `user_data`, `callback`, and `release` must satisfy the service callback
    /// ownership contract until Core drains the returned subscription.
    pub unsafe fn submit_register_chat_command(
        self,
        name: &[u8],
        callback: SampChatCommandCallbackV1,
        user_data: *mut c_void,
        release: SampReleaseCallbackV1,
    ) -> Result<(SubscriptionId, CommandReceiptId), ModResult> {
        let name_len = checked_len(name.len())?;
        let mut subscription = SubscriptionId(0);
        let mut receipt = CommandReceiptId(0);
        let result = unsafe {
            (self.table.submit_register_chat_command)(
                name.as_ptr(),
                name_len,
                Some(callback),
                user_data,
                Some(release),
                &mut subscription,
                &mut receipt,
            )
        };
        if result.is_ok() {
            Ok((subscription, receipt))
        } else {
            Err(result)
        }
    }
}

impl SampNetService {
    /// Registers a callback whose opaque context remains owned by the host.
    ///
    /// # Safety
    ///
    /// The callback context must follow the service ownership contract until
    /// Core drains the returned subscription.
    pub unsafe fn register_packet(
        self,
        direction: u32,
        callback: SampNetEventCallbackV1,
        user_data: *mut c_void,
        release: SampReleaseCallbackV1,
    ) -> Result<SubscriptionId, ModResult> {
        unsafe { self.register(direction, callback, user_data, release, true) }
    }

    /// Registers an RPC callback.
    ///
    /// # Safety
    ///
    /// The callback context must follow the service ownership contract until
    /// Core drains the returned subscription.
    pub unsafe fn register_rpc(
        self,
        direction: u32,
        callback: SampNetEventCallbackV1,
        user_data: *mut c_void,
        release: SampReleaseCallbackV1,
    ) -> Result<SubscriptionId, ModResult> {
        unsafe { self.register(direction, callback, user_data, release, false) }
    }

    unsafe fn register(
        self,
        direction: u32,
        callback: SampNetEventCallbackV1,
        user_data: *mut c_void,
        release: SampReleaseCallbackV1,
        packet: bool,
    ) -> Result<SubscriptionId, ModResult> {
        let mut out = SubscriptionId(0);
        let result = if packet {
            unsafe {
                (self.table.register_packet)(
                    direction,
                    Some(callback),
                    user_data,
                    Some(release),
                    &mut out,
                )
            }
        } else {
            unsafe {
                (self.table.register_rpc)(
                    direction,
                    Some(callback),
                    user_data,
                    Some(release),
                    &mut out,
                )
            }
        };
        result_with_out(result, out)
    }

    /// # Safety
    ///
    /// `event` must be the live event pointer supplied to this registration's callback.
    pub unsafe fn event_id(self, event: *const SampNetEventV1) -> Result<u8, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.event_id)(event, &mut out) }, out)
    }

    /// # Safety
    ///
    /// `event` must be the live event pointer supplied to this registration's callback.
    pub unsafe fn event_reset(self, event: *mut SampNetEventV1) -> Result<(), ModResult> {
        result_unit(unsafe { (self.table.event_reset)(event) })
    }

    /// # Safety
    ///
    /// `event` must be the live event pointer supplied to this registration's callback.
    pub unsafe fn event_remaining_bits(
        self,
        event: *const SampNetEventV1,
    ) -> Result<u32, ModResult> {
        let mut out = 0;
        result_with_out(
            unsafe { (self.table.event_remaining_bits)(event, &mut out) },
            out,
        )
    }

    /// # Safety
    ///
    /// `event` must be the live event pointer supplied to this registration's callback.
    pub unsafe fn event_read_bits(
        self,
        event: *mut SampNetEventV1,
        out: &mut [u8],
        bit_len: u32,
    ) -> Result<(), ModResult> {
        let capacity = checked_len(out.len())?;
        result_unit(unsafe {
            (self.table.event_read_bits)(event, out.as_mut_ptr(), capacity, bit_len)
        })
    }

    /// # Safety
    ///
    /// `event` must be the live event pointer supplied to this registration's callback.
    pub unsafe fn event_replace_bits(
        self,
        event: *mut SampNetEventV1,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<(), ModResult> {
        let byte_len = checked_len(bytes.len())?;
        result_unit(unsafe {
            (self.table.event_replace_bits)(event, bytes.as_ptr(), byte_len, bit_len)
        })
    }

    pub fn encode_string(self, value: &[u8], out: &mut [u8]) -> Result<(u32, u32), ModResult> {
        let value_len = checked_len(value.len())?;
        let capacity = checked_len(out.len())?;
        let mut byte_len = 0;
        let mut bit_len = 0;
        let result = unsafe {
            (self.table.encode_string)(
                value.as_ptr(),
                value_len,
                out.as_mut_ptr(),
                capacity,
                &mut byte_len,
                &mut bit_len,
            )
        };
        if result.is_ok() {
            Ok((byte_len, bit_len))
        } else {
            Err(result)
        }
    }

    /// # Safety
    ///
    /// `event` must be the live event pointer supplied to this registration's callback.
    pub unsafe fn event_read_encoded_string(
        self,
        event: *mut SampNetEventV1,
        out: &mut [u8],
    ) -> Result<u32, ModResult> {
        let capacity = checked_len(out.len())?;
        let mut len = 0;
        result_with_out(
            unsafe {
                (self.table.event_read_encoded_string)(event, out.as_mut_ptr(), capacity, &mut len)
            },
            len,
        )
    }

    pub fn submit_packet(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
        options: SampNetSendOptionsV1,
    ) -> Result<CommandReceiptId, ModResult> {
        self.submit(id, bytes, bit_len, options, true)
    }

    pub fn submit_rpc(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
        options: SampNetSendOptionsV1,
    ) -> Result<CommandReceiptId, ModResult> {
        self.submit(id, bytes, bit_len, options, false)
    }

    fn submit(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
        options: SampNetSendOptionsV1,
        packet: bool,
    ) -> Result<CommandReceiptId, ModResult> {
        let byte_len = checked_len(bytes.len())?;
        let mut receipt = CommandReceiptId(0);
        let result = unsafe {
            if packet {
                (self.table.submit_packet)(
                    id,
                    bytes.as_ptr(),
                    byte_len,
                    bit_len,
                    options,
                    &mut receipt,
                )
            } else {
                (self.table.submit_rpc)(
                    id,
                    bytes.as_ptr(),
                    byte_len,
                    bit_len,
                    options,
                    &mut receipt,
                )
            }
        };
        result_with_out(result, receipt)
    }

    pub fn submit_emulate_incoming_packet(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.submit_emulate(id, bytes, bit_len, true)
    }

    pub fn submit_emulate_incoming_rpc(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.submit_emulate(id, bytes, bit_len, false)
    }

    fn submit_emulate(
        self,
        id: u8,
        bytes: &[u8],
        bit_len: u32,
        packet: bool,
    ) -> Result<CommandReceiptId, ModResult> {
        let byte_len = checked_len(bytes.len())?;
        let mut receipt = CommandReceiptId(0);
        let result = unsafe {
            if packet {
                (self.table.submit_emulate_incoming_packet)(
                    id,
                    bytes.as_ptr(),
                    byte_len,
                    bit_len,
                    &mut receipt,
                )
            } else {
                (self.table.submit_emulate_incoming_rpc)(
                    id,
                    bytes.as_ptr(),
                    byte_len,
                    bit_len,
                    &mut receipt,
                )
            }
        };
        result_with_out(result, receipt)
    }

    pub fn incoming_emulation_ready(self) -> Result<bool, ModResult> {
        let mut out = 0;
        result_with_out(
            unsafe { (self.table.incoming_emulation_ready)(&mut out) },
            out != 0,
        )
    }
}

fn checked_len(len: usize) -> Result<u32, ModResult> {
    u32::try_from(len).map_err(|_| modkit_abi::MOD_INVALID_ARGUMENT)
}

fn result_unit(result: ModResult) -> Result<(), ModResult> {
    if result.is_ok() { Ok(()) } else { Err(result) }
}

fn result_with_out<T>(result: ModResult, out: T) -> Result<T, ModResult> {
    if result.is_ok() { Ok(out) } else { Err(result) }
}

impl LegacySamp {
    /// Returns whether the host published a non-null legacy API table.
    #[must_use]
    pub fn is_available(self) -> bool {
        !self.legacy.api.is_null()
    }

    /// Returns the opaque legacy V1 table pointer.
    ///
    /// # Safety
    ///
    /// The caller must cast this pointer only to the exact legacy V1 ABI type
    /// used by the loaded host and must follow that table's lifetime contract.
    #[must_use]
    pub unsafe fn api_ptr(self) -> *const core::ffi::c_void {
        self.legacy.api
    }
}

impl Core {
    /// Returns the host lifecycle status.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
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
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub fn unregister(self, id: SubscriptionId) -> Result<(), ModResult> {
        let result = unsafe { (self.core.unregister)(id) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }

    /// Removes a subscription and waits for in-flight callbacks to drain.
    ///
    /// `MAY_BLOCK`.
    /// This may block. Do not call it from `DllMain`. The host rejects calls
    /// from the game thread and host callbacks with
    /// [`modkit_abi::MOD_WAIT_REJECTED`].
    pub fn unregister_and_wait(
        self,
        id: SubscriptionId,
        timeout: Duration,
    ) -> Result<(), ModResult> {
        let timeout_ms = timeout_millis(timeout);
        let result = unsafe { (self.core.unregister_and_wait)(id, timeout_ms) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }

    /// Polls a command receipt without blocking.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`.
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
    /// `MAY_BLOCK`.
    /// This may block. Do not call it from `DllMain`. The host rejects calls
    /// from the game thread and host callbacks with
    /// [`modkit_abi::MOD_WAIT_REJECTED`].
    pub fn receipt_wait(
        self,
        id: CommandReceiptId,
        timeout: Duration,
    ) -> Result<CommandCompletionV1, ModResult> {
        let timeout_ms = timeout_millis(timeout);
        let mut out = CommandCompletionV1::default();
        let result = unsafe { (self.core.receipt_wait)(id, timeout_ms, &mut out) };
        if result.is_ok() { Ok(out) } else { Err(result) }
    }

    /// Detaches a command receipt without cancelling its owned command.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
    pub fn receipt_release(self, id: CommandReceiptId) -> Result<(), ModResult> {
        let result = unsafe { (self.core.receipt_release)(id) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }

    /// Logs a UTF-8 message through the host logger.
    ///
    /// `ANY_THREAD + CALLBACK_SAFE`; performs bounded copying work.
    /// `level` is `0`=error, `1`=warn, `2`=info, `3`=debug.
    pub fn log_utf8(self, level: u32, message: &[u8]) -> Result<(), ModResult> {
        if message.len() > modkit_abi::MAX_LOG_MESSAGE_BYTES as usize {
            return Err(modkit_abi::MOD_INVALID_ARGUMENT);
        }
        let result = unsafe { (self.core.log_utf8)(level, message.as_ptr(), message.len() as u32) };
        if result.is_ok() { Ok(()) } else { Err(result) }
    }
}

fn timeout_millis(timeout: Duration) -> u32 {
    if timeout == Duration::MAX {
        modkit_abi::TIMEOUT_INFINITE
    } else {
        timeout
            .as_millis()
            .min(u128::from(modkit_abi::TIMEOUT_INFINITE - 1)) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "system" fn host_status(_out: *mut HostStatusV1) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn unregister(_id: SubscriptionId) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn unregister_and_wait(
        _id: SubscriptionId,
        _timeout_ms: u32,
    ) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn receipt_poll(
        _id: CommandReceiptId,
        _out: *mut CommandCompletionV1,
    ) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn receipt_wait(
        _id: CommandReceiptId,
        _timeout_ms: u32,
        _out: *mut CommandCompletionV1,
    ) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn receipt_release(_id: CommandReceiptId) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn log_utf8(_level: u32, _message: *const u8, _len: u32) -> ModResult {
        modkit_abi::MOD_OK
    }
    unsafe extern "system" fn query_not_ready(
        _service: ServiceId,
        _version: u32,
        out: *mut *const ServiceHeader,
    ) -> ModResult {
        unsafe { out.write(core::ptr::null()) };
        modkit_abi::MOD_NOT_READY
    }

    static CORE: CoreServiceV1 = CoreServiceV1 {
        header: ServiceHeader {
            service_id: SERVICE_ID_CORE,
            version: 1,
            size: core::mem::size_of::<CoreServiceV1>() as u32,
            reserved: 99,
        },
        host_status,
        unregister,
        unregister_and_wait,
        receipt_poll,
        receipt_wait,
        receipt_release,
        log_utf8,
    };
    static NOT_READY_API: ModHostApiV1 = ModHostApiV1 {
        abi_version: modkit_abi::MOD_HOST_ABI_VERSION_V1,
        size: core::mem::size_of::<ModHostApiV1>() as u32,
        query_service: query_not_ready,
    };
    static SHORT_CORE_HEADER: ServiceHeader = ServiceHeader {
        service_id: SERVICE_ID_CORE,
        version: 1,
        size: core::mem::size_of::<ServiceHeader>() as u32,
        reserved: 0,
    };
    static OVERSIZED_CORE_HEADER: ServiceHeader = ServiceHeader {
        service_id: SERVICE_ID_CORE,
        version: 1,
        size: core::mem::size_of::<CoreServiceV1>() as u32 + 4,
        reserved: 0,
    };

    #[test]
    fn service_cast_requires_the_exact_concrete_table_size() {
        assert!(Service::from_header(&SHORT_CORE_HEADER, SERVICE_ID_CORE, 1).is_none());
        assert!(Service::from_header(&OVERSIZED_CORE_HEADER, SERVICE_ID_CORE, 1).is_none());
        assert!(matches!(
            Service::from_header(&CORE.header, SERVICE_ID_CORE, 1),
            Some(Service::Core(_))
        ));
    }

    #[test]
    fn unpublished_registry_status_remains_waiting() {
        let host = Host {
            api: &NOT_READY_API,
        };
        assert_eq!(host.status(), HostStatus::Waiting);
    }

    #[test]
    fn oversized_log_messages_are_rejected_before_the_ffi_call() {
        let core = Core { core: &CORE };
        let message = vec![0; modkit_abi::MAX_LOG_MESSAGE_BYTES as usize + 1];
        assert_eq!(
            core.log_utf8(2, &message),
            Err(modkit_abi::MOD_INVALID_ARGUMENT)
        );
    }

    #[test]
    fn only_duration_max_maps_to_the_infinite_timeout_sentinel() {
        assert_eq!(timeout_millis(Duration::ZERO), 0);
        assert_eq!(
            timeout_millis(Duration::from_millis(u64::from(u32::MAX))),
            u32::MAX - 1
        );
        assert_eq!(timeout_millis(Duration::MAX), modkit_abi::TIMEOUT_INFINITE);
    }
}
