//! Safe wrappers over the modkit host bootstrap and service tables.

use modkit_abi::{
    CommandCompletionV1, CommandReceiptId, CoreServiceV1, GTA_SA_SERVICE_VERSION_V1,
    GTA_SA_SERVICE_VERSION_V2, GtaCameraSnapshotV1, GtaPedSnapshotV1, GtaPoolKindV1,
    GtaReleaseCallbackV1, GtaSaServiceV1, GtaSaServiceV2, GtaTickCallbackV1, GtaTimerSnapshotV1,
    GtaVector3V1, GtaVehicleSnapshotV1, HostStatusV1, LegacySampServiceV1, ModHostApiV1, ModResult,
    SAMP_CODEC_SERVICE_VERSION_V1, SAMP_CONTROL_SERVICE_VERSION_V1, SAMP_NET_SERVICE_VERSION_V1,
    SAMP_PLAYER_SERVICE_VERSION_V1, SAMP_POOL_SERVICE_VERSION_V1, SAMP_SERVICE_VERSION_V1,
    SAMP_TEXT_LABEL_SERVICE_VERSION_V1, SAMP_TEXTDRAW_SERVICE_VERSION_V1,
    SAMP_UI_SERVICE_VERSION_V1, SERVICE_ID_CORE, SERVICE_ID_GTA_SA, SERVICE_ID_LEGACY_SAMP_ABI,
    SERVICE_ID_SAMP, SERVICE_ID_SAMP_CODEC, SERVICE_ID_SAMP_CONTROL, SERVICE_ID_SAMP_NETWORK,
    SERVICE_ID_SAMP_PLAYER, SERVICE_ID_SAMP_POOL, SERVICE_ID_SAMP_TEXT_LABEL,
    SERVICE_ID_SAMP_TEXTDRAW, SERVICE_ID_SAMP_UI, SampAimSyncV1, SampAnimationV1,
    SampChatCommandCallbackV1, SampChatEntryV1, SampChatInputTextV1, SampCodecServiceV1,
    SampControlServiceV1, SampDialogRequestV1, SampDialogResponseV1, SampDialogSnapshotV1,
    SampGangzoneV1, SampInCarSyncV1, SampLocalPlayerV1, SampNetEventCallbackV1, SampNetEventV1,
    SampNetSendOptionsV1, SampNetServiceV1, SampOnFootSyncV1, SampPassengerSyncV1,
    SampPlayerInfoV1, SampPlayerServiceV1, SampPoolServiceV1, SampReleaseCallbackV1,
    SampRemotePlayerStateV1, SampServerInfoV1, SampServiceV1, SampStreamedOutPlayerPositionV1,
    SampTextLabelCreateV1, SampTextLabelServiceV1, SampTextLabelV1, SampTextdrawServiceV1,
    SampTextdrawV1, SampTrailerSyncV1, SampUiServiceV1, ServiceHeader, ServiceId, SubscriptionId,
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

    /// Returns the SA-MP text-label service v1.
    pub fn samp_text_labels(self) -> Result<SampTextLabelService, ServiceError> {
        match self.query_service(
            SERVICE_ID_SAMP_TEXT_LABEL,
            SAMP_TEXT_LABEL_SERVICE_VERSION_V1,
        )? {
            Service::SampTextLabel(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP connection and replication-control service v1.
    pub fn samp_control(self) -> Result<SampControlService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_CONTROL, SAMP_CONTROL_SERVICE_VERSION_V1)? {
            Service::SampControl(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP local UI service v1.
    pub fn samp_ui(self) -> Result<SampUiService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_UI, SAMP_UI_SERVICE_VERSION_V1)? {
            Service::SampUi(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP player, synchronization, and animation service v1.
    pub fn samp_players(self) -> Result<SampPlayerService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_PLAYER, SAMP_PLAYER_SERVICE_VERSION_V1)? {
            Service::SampPlayer(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP pool mappings and gangzone service v1.
    pub fn samp_pools(self) -> Result<SampPoolService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_POOL, SAMP_POOL_SERVICE_VERSION_V1)? {
            Service::SampPool(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP textdraw service v1.
    pub fn samp_textdraws(self) -> Result<SampTextdrawService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_TEXTDRAW, SAMP_TEXTDRAW_SERVICE_VERSION_V1)? {
            Service::SampTextdraw(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the SA-MP native string codec service v1.
    pub fn samp_codec(self) -> Result<SampCodecService, ServiceError> {
        match self.query_service(SERVICE_ID_SAMP_CODEC, SAMP_CODEC_SERVICE_VERSION_V1)? {
            Service::SampCodec(service) => Ok(service),
            _ => Err(ServiceError::Host(modkit_abi::MOD_UNSUPPORTED)),
        }
    }

    /// Returns the latest GTA San Andreas service understood by this SDK.
    pub fn gta_sa_service(self) -> Result<GtaSaService, ServiceError> {
        match self.query_service(SERVICE_ID_GTA_SA, GTA_SA_SERVICE_VERSION_V2)? {
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
    /// The SA-MP text-label service v1.
    SampTextLabel(SampTextLabelService),
    /// The SA-MP connection and replication-control service v1.
    SampControl(SampControlService),
    /// The SA-MP local UI service v1.
    SampUi(SampUiService),
    /// The SA-MP player, synchronization, and animation service v1.
    SampPlayer(SampPlayerService),
    /// The SA-MP pool mappings and gangzone service v1.
    SampPool(SampPoolService),
    /// The SA-MP textdraw service v1.
    SampTextdraw(SampTextdrawService),
    /// The SA-MP native string codec service v1.
    SampCodec(SampCodecService),
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
            SERVICE_ID_GTA_SA => match version {
                GTA_SA_SERVICE_VERSION_V1
                    if header.matches(
                        service_id,
                        version,
                        core::mem::size_of::<GtaSaServiceV1>() as u32,
                    ) =>
                {
                    unsafe {
                        (header as *const ServiceHeader)
                            .cast::<GtaSaServiceV1>()
                            .as_ref()
                    }
                    .map(|table| {
                        Service::GtaSa(GtaSaService {
                            table: GtaSaServiceTable::V1(table),
                        })
                    })
                }
                GTA_SA_SERVICE_VERSION_V2
                    if header.matches(
                        service_id,
                        version,
                        core::mem::size_of::<GtaSaServiceV2>() as u32,
                    ) =>
                {
                    unsafe {
                        (header as *const ServiceHeader)
                            .cast::<GtaSaServiceV2>()
                            .as_ref()
                    }
                    .map(|table| {
                        Service::GtaSa(GtaSaService {
                            table: GtaSaServiceTable::V2(table),
                        })
                    })
                }
                _ => None,
            },
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
            SERVICE_ID_SAMP_TEXT_LABEL
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampTextLabelServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampTextLabelServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampTextLabel(SampTextLabelService { table }))
            }
            SERVICE_ID_SAMP_CONTROL
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampControlServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampControlServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampControl(SampControlService { table }))
            }
            SERVICE_ID_SAMP_UI
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampUiServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampUiServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampUi(SampUiService { table }))
            }
            SERVICE_ID_SAMP_PLAYER
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampPlayerServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampPlayerServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampPlayer(SampPlayerService { table }))
            }
            SERVICE_ID_SAMP_POOL
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampPoolServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampPoolServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampPool(SampPoolService { table }))
            }
            SERVICE_ID_SAMP_TEXTDRAW
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampTextdrawServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampTextdrawServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampTextdraw(SampTextdrawService { table }))
            }
            SERVICE_ID_SAMP_CODEC
                if header.matches(
                    service_id,
                    version,
                    core::mem::size_of::<SampCodecServiceV1>() as u32,
                ) =>
            {
                let table = unsafe {
                    (header as *const ServiceHeader)
                        .cast::<SampCodecServiceV1>()
                        .as_ref()
                };
                table.map(|table| Service::SampCodec(SampCodecService { table }))
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

/// Validated low-level view of a GTA San Andreas service.
#[derive(Clone, Copy)]
pub struct GtaSaService {
    table: GtaSaServiceTable,
}

#[derive(Clone, Copy)]
enum GtaSaServiceTable {
    V1(&'static GtaSaServiceV1),
    V2(&'static GtaSaServiceV2),
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

/// Validated low-level view of the SA-MP text-label service v1.
#[derive(Clone, Copy)]
pub struct SampTextLabelService {
    table: &'static SampTextLabelServiceV1,
}

/// Validated low-level view of the SA-MP control service v1.
#[derive(Clone, Copy)]
pub struct SampControlService {
    table: &'static SampControlServiceV1,
}

/// Validated low-level view of the SA-MP local UI service v1.
#[derive(Clone, Copy)]
pub struct SampUiService {
    table: &'static SampUiServiceV1,
}

/// Validated low-level view of the SA-MP player service v1.
#[derive(Clone, Copy)]
pub struct SampPlayerService {
    table: &'static SampPlayerServiceV1,
}

/// Validated low-level view of the SA-MP pool service v1.
#[derive(Clone, Copy)]
pub struct SampPoolService {
    table: &'static SampPoolServiceV1,
}

/// Validated low-level view of the SA-MP textdraw service v1.
#[derive(Clone, Copy)]
pub struct SampTextdrawService {
    table: &'static SampTextdrawServiceV1,
}

/// Validated low-level view of the SA-MP codec service v1.
#[derive(Clone, Copy)]
pub struct SampCodecService {
    table: &'static SampCodecServiceV1,
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
        let result = match self.table {
            GtaSaServiceTable::V1(table) => unsafe {
                (table.register_tick)(Some(callback), user_data, Some(release), &mut out)
            },
            GtaSaServiceTable::V2(table) => unsafe {
                (table.register_tick)(Some(callback), user_data, Some(release), &mut out)
            },
        };
        result_with_out(result, out)
    }

    pub fn local_ped_snapshot(
        self,
        context: &crate::GameContext<'_>,
    ) -> Result<GtaPedSnapshotV1, ModResult> {
        let mut out = GtaPedSnapshotV1::default();
        let result = match self.table {
            GtaSaServiceTable::V1(table) => unsafe {
                (table.local_ped_snapshot)(context.token(), &mut out)
            },
            GtaSaServiceTable::V2(table) => unsafe {
                (table.local_ped_snapshot)(context.token(), &mut out)
            },
        };
        result_with_out(result, out)
    }

    pub fn teleport_local_ped(
        self,
        context: &crate::GameContext<'_>,
        destination: GtaVector3V1,
    ) -> Result<(), ModResult> {
        let result = match self.table {
            GtaSaServiceTable::V1(table) => unsafe {
                (table.teleport_local_ped)(context.token(), destination)
            },
            GtaSaServiceTable::V2(table) => unsafe {
                (table.teleport_local_ped)(context.token(), destination)
            },
        };
        result_unit(result)
    }

    pub fn submit_local_ped_snapshot(self) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        let result = match self.table {
            GtaSaServiceTable::V1(table) => unsafe { (table.submit_local_ped_snapshot)(&mut out) },
            GtaSaServiceTable::V2(table) => unsafe { (table.submit_local_ped_snapshot)(&mut out) },
        };
        result_with_out(result, out)
    }

    pub fn take_local_ped_snapshot(
        self,
        receipt: CommandReceiptId,
    ) -> Result<GtaPedSnapshotV1, ModResult> {
        let mut out = GtaPedSnapshotV1::default();
        let result = match self.table {
            GtaSaServiceTable::V1(table) => unsafe {
                (table.take_local_ped_snapshot)(receipt, &mut out)
            },
            GtaSaServiceTable::V2(table) => unsafe {
                (table.take_local_ped_snapshot)(receipt, &mut out)
            },
        };
        result_with_out(result, out)
    }

    pub fn submit_teleport_local_ped(
        self,
        destination: GtaVector3V1,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        let result = match self.table {
            GtaSaServiceTable::V1(table) => unsafe {
                (table.submit_teleport_local_ped)(destination, &mut out)
            },
            GtaSaServiceTable::V2(table) => unsafe {
                (table.submit_teleport_local_ped)(destination, &mut out)
            },
        };
        result_with_out(result, out)
    }

    pub fn entity_exists(
        self,
        context: &crate::GameContext<'_>,
        kind: GtaPoolKindV1,
        handle: i32,
    ) -> Result<bool, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = 0;
        result_with_out(
            unsafe { (table.entity_exists)(context.token(), kind, handle, &mut out) },
            out != 0,
        )
    }

    pub fn submit_entity_exists(
        self,
        kind: GtaPoolKindV1,
        handle: i32,
    ) -> Result<CommandReceiptId, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (table.submit_entity_exists)(kind, handle, &mut out) },
            out,
        )
    }

    pub fn take_entity_exists(self, receipt: CommandReceiptId) -> Result<bool, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = 0;
        result_with_out(
            unsafe { (table.take_entity_exists)(receipt, &mut out) },
            out != 0,
        )
    }

    pub fn vehicle_snapshot(
        self,
        context: &crate::GameContext<'_>,
        handle: i32,
    ) -> Result<GtaVehicleSnapshotV1, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = GtaVehicleSnapshotV1::default();
        result_with_out(
            unsafe { (table.vehicle_snapshot)(context.token(), handle, &mut out) },
            out,
        )
    }

    pub fn submit_vehicle_snapshot(self, handle: i32) -> Result<CommandReceiptId, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (table.submit_vehicle_snapshot)(handle, &mut out) },
            out,
        )
    }

    pub fn take_vehicle_snapshot(
        self,
        receipt: CommandReceiptId,
    ) -> Result<GtaVehicleSnapshotV1, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = GtaVehicleSnapshotV1::default();
        result_with_out(
            unsafe { (table.take_vehicle_snapshot)(receipt, &mut out) },
            out,
        )
    }

    pub fn find_ground_z(
        self,
        context: &crate::GameContext<'_>,
        x: f32,
        y: f32,
    ) -> Result<f32, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = 0.0;
        result_with_out(
            unsafe { (table.find_ground_z)(context.token(), x, y, &mut out) },
            out,
        )
    }

    pub fn submit_find_ground_z(self, x: f32, y: f32) -> Result<CommandReceiptId, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = CommandReceiptId(0);
        result_with_out(unsafe { (table.submit_find_ground_z)(x, y, &mut out) }, out)
    }

    pub fn take_find_ground_z(self, receipt: CommandReceiptId) -> Result<f32, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = 0.0;
        result_with_out(
            unsafe { (table.take_find_ground_z)(receipt, &mut out) },
            out,
        )
    }

    pub fn timer_snapshot(
        self,
        context: &crate::GameContext<'_>,
    ) -> Result<GtaTimerSnapshotV1, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = GtaTimerSnapshotV1::default();
        result_with_out(
            unsafe { (table.timer_snapshot)(context.token(), &mut out) },
            out,
        )
    }

    pub fn submit_timer_snapshot(self) -> Result<CommandReceiptId, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = CommandReceiptId(0);
        result_with_out(unsafe { (table.submit_timer_snapshot)(&mut out) }, out)
    }

    pub fn take_timer_snapshot(
        self,
        receipt: CommandReceiptId,
    ) -> Result<GtaTimerSnapshotV1, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = GtaTimerSnapshotV1::default();
        result_with_out(
            unsafe { (table.take_timer_snapshot)(receipt, &mut out) },
            out,
        )
    }

    pub fn camera_snapshot(
        self,
        context: &crate::GameContext<'_>,
    ) -> Result<GtaCameraSnapshotV1, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = GtaCameraSnapshotV1::default();
        result_with_out(
            unsafe { (table.camera_snapshot)(context.token(), &mut out) },
            out,
        )
    }

    pub fn submit_camera_snapshot(self) -> Result<CommandReceiptId, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = CommandReceiptId(0);
        result_with_out(unsafe { (table.submit_camera_snapshot)(&mut out) }, out)
    }

    pub fn take_camera_snapshot(
        self,
        receipt: CommandReceiptId,
    ) -> Result<GtaCameraSnapshotV1, ModResult> {
        let GtaSaServiceTable::V2(table) = self.table else {
            return Err(modkit_abi::MOD_UNSUPPORTED_VERSION);
        };
        let mut out = GtaCameraSnapshotV1::default();
        result_with_out(
            unsafe { (table.take_camera_snapshot)(receipt, &mut out) },
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

impl SampTextLabelService {
    pub fn snapshot(self, id: u16) -> Result<SampTextLabelV1, ModResult> {
        let mut out = SampTextLabelV1::default();
        result_with_out(unsafe { (self.table.snapshot)(id, &mut out) }, out)
    }

    pub fn submit_delete(self, id: u16) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(unsafe { (self.table.submit_delete)(id, &mut out) }, out)
    }

    pub fn submit_set_text(self, id: u16, text: &[u8]) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_set_text)(id, text.as_ptr(), text_len, &mut out) },
            out,
        )
    }

    pub fn submit_create_at(
        self,
        id: u16,
        request: &SampTextLabelCreateV1,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_create_at)(id, request, &mut out) },
            out,
        )
    }

    pub fn submit_create(
        self,
        request: &SampTextLabelCreateV1,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_create)(request, &mut out) },
            out,
        )
    }
}

impl SampCodecService {
    pub fn decode_string(
        self,
        input: &[u8],
        input_bit_len: usize,
        input_read_offset: usize,
        output: &mut [u8],
    ) -> Result<(usize, usize), ModResult> {
        let input_byte_len = checked_len(input.len())?;
        let input_bit_len = checked_len(input_bit_len)?;
        let input_read_offset = checked_len(input_read_offset)?;
        let output_capacity = checked_len(output.len())?;
        let mut output_len = 0;
        let mut output_read_offset = 0;
        let result = unsafe {
            (self.table.decode_string)(
                input.as_ptr(),
                input_byte_len,
                input_bit_len,
                input_read_offset,
                output.as_mut_ptr(),
                output_capacity,
                &mut output_len,
                &mut output_read_offset,
            )
        };
        if result.is_ok() {
            Ok((output_len as usize, output_read_offset as usize))
        } else {
            Err(result)
        }
    }
}

impl SampControlService {
    pub fn submit_game_state(self, state: i32) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_game_state)(state, &mut out) },
            out,
        )
    }

    pub fn submit_send_rate(
        self,
        kind: u32,
        milliseconds: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_send_rate)(kind, milliseconds, &mut out) },
            out,
        )
    }

    pub fn submit_connect(self, address: &[u8], port: u16) -> Result<CommandReceiptId, ModResult> {
        let address_len = checked_len(address.len())?;
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_connect)(address.as_ptr(), address_len, port, &mut out) },
            out,
        )
    }

    pub fn submit_disconnect(self, block_duration: u32) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(
            unsafe { (self.table.submit_disconnect)(block_duration, &mut out) },
            out,
        )
    }
}

impl SampTextdrawService {
    pub fn exists(self, id: u16) -> Result<bool, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.exists)(id, &mut out) }, out != 0)
    }

    pub fn snapshot(self, id: u16) -> Result<SampTextdrawV1, ModResult> {
        let mut out = SampTextdrawV1::default();
        result_with_out(unsafe { (self.table.snapshot)(id, &mut out) }, out)
    }

    pub fn submit_create(
        self,
        id: u16,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_create)(id, text.as_ptr(), text_len, x, y, out)
        })
    }

    pub fn submit_delete(self, id: u16) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_delete)(id, out) })
    }

    pub fn submit_set_position(
        self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_set_position)(id, x, y, out) })
    }

    pub fn submit_set_style(self, id: u16, style: i32) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_set_style)(id, style, out) })
    }

    pub fn submit_set_letter_style(
        self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe {
            (self.table.submit_set_letter_style)(id, width, height, colour, out)
        })
    }

    pub fn submit_set_proportional(
        self,
        id: u16,
        proportional: bool,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe {
            (self.table.submit_set_proportional)(id, u8::from(proportional), out)
        })
    }

    pub fn submit_set_shadow(
        self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_set_shadow)(id, shadow, colour, out) })
    }

    pub fn submit_set_outline(
        self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_set_outline)(id, outline, colour, out) })
    }

    pub fn submit_set_box(
        self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe {
            (self.table.submit_set_box)(id, u8::from(enabled), colour, width, height, out)
        })
    }

    pub fn submit_set_alignment(
        self,
        id: u16,
        alignment: u8,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_set_alignment)(id, alignment, out) })
    }

    pub fn submit_set_text(self, id: u16, text: &[u8]) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_set_text)(id, text.as_ptr(), text_len, out)
        })
    }

    pub fn submit_set_model_style(
        self,
        id: u16,
        rotation: [f32; 3],
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe {
            (self.table.submit_set_model_style)(
                id,
                rotation[0],
                rotation[1],
                rotation[2],
                zoom,
                colour1,
                colour2,
                out,
            )
        })
    }

    fn receipt(
        self,
        submit: impl FnOnce(*mut CommandReceiptId) -> ModResult,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(submit(&mut out), out)
    }
}

impl SampPoolService {
    pub fn object_exists(self, id: u16) -> Result<bool, ModResult> {
        self.exists(id, self.table.object_exists)
    }

    pub fn vehicle_exists(self, id: u16) -> Result<bool, ModResult> {
        self.exists(id, self.table.vehicle_exists)
    }

    pub fn object_handle(self, id: u16) -> Result<Option<i32>, ModResult> {
        self.forward(id, self.table.object_handle)
    }

    pub fn object_id_by_handle(self, handle: i32) -> Result<Option<u16>, ModResult> {
        self.reverse(handle, self.table.object_id_by_handle)
    }

    pub fn pickup_handle(self, id: u16) -> Result<Option<i32>, ModResult> {
        self.forward(id, self.table.pickup_handle)
    }

    pub fn pickup_id_by_handle(self, handle: i32) -> Result<Option<u16>, ModResult> {
        self.reverse(handle, self.table.pickup_id_by_handle)
    }

    pub fn vehicle_handle(self, id: u16) -> Result<Option<i32>, ModResult> {
        self.forward(id, self.table.vehicle_handle)
    }

    pub fn vehicle_id_by_handle(self, handle: i32) -> Result<Option<u16>, ModResult> {
        self.reverse(handle, self.table.vehicle_id_by_handle)
    }

    pub fn player_ped_handle(self, id: u16) -> Result<Option<i32>, ModResult> {
        self.forward(id, self.table.player_ped_handle)
    }

    pub fn player_id_by_ped_handle(self, handle: i32) -> Result<Option<u16>, ModResult> {
        self.reverse(handle, self.table.player_id_by_ped_handle)
    }

    pub fn gangzone(self, id: u16) -> Result<SampGangzoneV1, ModResult> {
        let mut out = SampGangzoneV1::default();
        result_with_out(unsafe { (self.table.gangzone)(id, &mut out) }, out)
    }

    fn exists(
        self,
        id: u16,
        function: unsafe extern "system" fn(u16, *mut u8) -> ModResult,
    ) -> Result<bool, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { function(id, &mut out) }, out != 0)
    }

    fn forward(
        self,
        id: u16,
        function: unsafe extern "system" fn(u16, *mut i32) -> ModResult,
    ) -> Result<Option<i32>, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { function(id, &mut out) }, (out != 0).then_some(out))
    }

    fn reverse(
        self,
        handle: i32,
        function: unsafe extern "system" fn(i32, *mut u16) -> ModResult,
    ) -> Result<Option<u16>, ModResult> {
        let mut out = u16::MAX;
        result_with_out(
            unsafe { function(handle, &mut out) },
            (out != u16::MAX).then_some(out),
        )
    }
}

impl SampPlayerService {
    pub fn remote_state(self, id: u16) -> Result<SampRemotePlayerStateV1, ModResult> {
        let mut out = SampRemotePlayerStateV1::default();
        result_with_out(unsafe { (self.table.remote_state)(id, &mut out) }, out)
    }

    pub fn streamed_out_position(
        self,
        id: u16,
    ) -> Result<SampStreamedOutPlayerPositionV1, ModResult> {
        let mut out = SampStreamedOutPlayerPositionV1::default();
        result_with_out(
            unsafe { (self.table.streamed_out_position)(id, &mut out) },
            out,
        )
    }

    pub fn onfoot_sync(self, id: u16) -> Result<SampOnFootSyncV1, ModResult> {
        let mut out = SampOnFootSyncV1::default();
        result_with_out(unsafe { (self.table.onfoot_sync)(id, &mut out) }, out)
    }

    pub fn vehicle_sync(self, id: u16) -> Result<SampInCarSyncV1, ModResult> {
        let mut out = SampInCarSyncV1::default();
        result_with_out(unsafe { (self.table.vehicle_sync)(id, &mut out) }, out)
    }

    pub fn passenger_sync(self, id: u16) -> Result<SampPassengerSyncV1, ModResult> {
        let mut out = SampPassengerSyncV1::default();
        result_with_out(unsafe { (self.table.passenger_sync)(id, &mut out) }, out)
    }

    pub fn trailer_sync(self, id: u16) -> Result<SampTrailerSyncV1, ModResult> {
        let mut out = SampTrailerSyncV1::default();
        result_with_out(unsafe { (self.table.trailer_sync)(id, &mut out) }, out)
    }

    pub fn aim_sync(self, id: u16) -> Result<SampAimSyncV1, ModResult> {
        let mut out = SampAimSyncV1::default();
        result_with_out(unsafe { (self.table.aim_sync)(id, &mut out) }, out)
    }

    pub fn player_defined(self, id: u16) -> Result<bool, ModResult> {
        self.player_bool(id, self.table.player_defined)
    }

    pub fn player_paused(self, id: u16) -> Result<bool, ModResult> {
        self.player_bool(id, self.table.player_paused)
    }

    pub fn player_count(self, include_npcs: bool) -> Result<u16, ModResult> {
        let mut out = 0;
        result_with_out(
            unsafe { (self.table.player_count)(u8::from(include_npcs), &mut out) },
            out,
        )
    }

    pub fn player_max_id(self) -> Result<Option<u16>, ModResult> {
        let mut out = u16::MAX;
        result_with_out(
            unsafe { (self.table.player_max_id)(&mut out) },
            (out != u16::MAX).then_some(out),
        )
    }

    pub fn animation(self, id: u16) -> Result<SampAnimationV1, ModResult> {
        let mut out = SampAnimationV1::default();
        result_with_out(unsafe { (self.table.animation)(id, &mut out) }, out)
    }

    pub fn animation_id(self, name: &[u8], file: &[u8]) -> Result<Option<u16>, ModResult> {
        let name_len = checked_len(name.len())?;
        let file_len = checked_len(file.len())?;
        let mut out = -1;
        let result = unsafe {
            (self.table.animation_id)(name.as_ptr(), name_len, file.as_ptr(), file_len, &mut out)
        };
        if !result.is_ok() {
            return Err(result);
        }
        if out < 0 {
            Ok(None)
        } else {
            u16::try_from(out)
                .map(Some)
                .map_err(|_| modkit_abi::MOD_NATIVE_CALL_FAILED)
        }
    }

    pub fn submit_spawn(self) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_spawn)(out) })
    }

    pub fn submit_special_action(self, action: u8) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_special_action)(action, out) })
    }

    pub fn submit_name(self, name: &[u8]) -> Result<CommandReceiptId, ModResult> {
        let name_len = checked_len(name.len())?;
        self.receipt(|out| unsafe { (self.table.submit_name)(name.as_ptr(), name_len, out) })
    }

    pub fn submit_colour(self, id: u16, colour: u32) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_colour)(id, colour, out) })
    }

    pub fn submit_force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_unoccupied_sync)(vehicle, seat, out) })
    }

    pub fn submit_force_aim_sync(self) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_aim_sync)(out) })
    }

    pub fn submit_force_onfoot_sync(self) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_onfoot_sync)(out) })
    }

    pub fn submit_force_stats_sync(self) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_stats_sync)(out) })
    }

    pub fn submit_force_trailer_sync(self, trailer: u16) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_trailer_sync)(trailer, out) })
    }

    pub fn submit_force_vehicle_sync(self, vehicle: u16) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_vehicle_sync)(vehicle, out) })
    }

    pub fn submit_force_passenger_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_passenger_sync)(vehicle, seat, out) })
    }

    pub fn submit_force_weapons_sync(self) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_force_weapons_sync)(out) })
    }

    fn player_bool(
        self,
        id: u16,
        function: unsafe extern "system" fn(u16, *mut u8) -> ModResult,
    ) -> Result<bool, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { function(id, &mut out) }, out != 0)
    }

    fn receipt(
        self,
        submit: impl FnOnce(*mut CommandReceiptId) -> ModResult,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(submit(&mut out), out)
    }
}

impl SampUiService {
    pub fn chat_display_mode(self) -> Result<i32, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.chat_display_mode)(&mut out) }, out)
    }

    pub fn chat_entry(self, id: u16) -> Result<SampChatEntryV1, ModResult> {
        let mut out = SampChatEntryV1::default();
        result_with_out(unsafe { (self.table.chat_entry)(id, &mut out) }, out)
    }

    pub fn chat_input_active(self) -> Result<bool, ModResult> {
        self.bool_value(self.table.chat_input_active)
    }

    pub fn chat_input_text(self) -> Result<SampChatInputTextV1, ModResult> {
        let mut out = SampChatInputTextV1::default();
        result_with_out(unsafe { (self.table.chat_input_text)(&mut out) }, out)
    }

    pub fn chat_command_defined(self, name: &[u8]) -> Result<bool, ModResult> {
        let name_len = checked_len(name.len())?;
        let mut out = 0;
        let result =
            unsafe { (self.table.chat_command_defined)(name.as_ptr(), name_len, &mut out) };
        result_with_out(result, out != 0)
    }

    pub fn cursor_mode(self) -> Result<i32, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.cursor_mode)(&mut out) }, out)
    }

    pub fn scoreboard_open(self) -> Result<bool, ModResult> {
        self.bool_value(self.table.scoreboard_open)
    }

    pub fn dialog_active(self) -> Result<bool, ModResult> {
        self.bool_value(self.table.dialog_active)
    }

    pub fn dialog_snapshot(self) -> Result<SampDialogSnapshotV1, ModResult> {
        let mut out = SampDialogSnapshotV1::default();
        result_with_out(unsafe { (self.table.dialog_snapshot)(&mut out) }, out)
    }

    pub fn take_dialog_response(self) -> Result<SampDialogResponseV1, ModResult> {
        let mut out = SampDialogResponseV1::default();
        result_with_out(unsafe { (self.table.take_dialog_response)(&mut out) }, out)
    }

    pub fn dialog_selected_item(self) -> Result<i32, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { (self.table.dialog_selected_item)(&mut out) }, out)
    }

    pub fn dialog_list_item_count(self) -> Result<i32, ModResult> {
        let mut out = 0;
        result_with_out(
            unsafe { (self.table.dialog_list_item_count)(&mut out) },
            out,
        )
    }

    pub fn submit_chat_message(
        self,
        style: u32,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        let prefix_len = checked_len(prefix.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_chat_message)(
                style,
                text.as_ptr(),
                text_len,
                prefix.as_ptr(),
                prefix_len,
                text_colour,
                prefix_colour,
                out,
            )
        })
    }

    pub fn submit_death_message(
        self,
        killer: &[u8],
        victim: &[u8],
        killer_colour: u32,
        victim_colour: u32,
        weapon: u8,
    ) -> Result<CommandReceiptId, ModResult> {
        let killer_len = checked_len(killer.len())?;
        let victim_len = checked_len(victim.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_death_message)(
                killer.as_ptr(),
                killer_len,
                victim.as_ptr(),
                victim_len,
                killer_colour,
                victim_colour,
                weapon,
                out,
            )
        })
    }

    pub fn submit_chat_display_mode(self, mode: i32) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_chat_display_mode)(mode, out) })
    }

    pub fn submit_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        let prefix_len = checked_len(prefix.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_chat_entry)(
                id,
                text.as_ptr(),
                text_len,
                prefix.as_ptr(),
                prefix_len,
                text_colour,
                prefix_colour,
                out,
            )
        })
    }

    pub fn submit_chat_input_text(self, text: &[u8]) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_chat_input_text)(text.as_ptr(), text_len, out)
        })
    }

    pub fn submit_chat_input_enabled(self, enabled: bool) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe {
            (self.table.submit_chat_input_enabled)(u8::from(enabled), out)
        })
    }

    pub fn submit_chat_input_process(self, text: &[u8]) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_chat_input_process)(text.as_ptr(), text_len, out)
        })
    }

    pub fn submit_cursor_mode(self, mode: i32) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_cursor_mode)(mode, out) })
    }

    pub fn submit_cursor_toggle(self, show: bool) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_cursor_toggle)(u8::from(show), out) })
    }

    pub fn submit_scoreboard_open(self, open: bool) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_scoreboard_open)(u8::from(open), out) })
    }

    pub fn submit_dialog(
        self,
        request: &SampDialogRequestV1,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_dialog)(request, out) })
    }

    pub fn submit_dialog_client_side(
        self,
        client_side: bool,
    ) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe {
            (self.table.submit_dialog_client_side)(u8::from(client_side), out)
        })
    }

    pub fn submit_dialog_selected_item(self, selected: i32) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_dialog_selected_item)(selected, out) })
    }

    pub fn submit_dialog_editbox_text(self, text: &[u8]) -> Result<CommandReceiptId, ModResult> {
        let text_len = checked_len(text.len())?;
        self.receipt(|out| unsafe {
            (self.table.submit_dialog_editbox_text)(text.as_ptr(), text_len, out)
        })
    }

    pub fn submit_dialog_close(self, button: u8) -> Result<CommandReceiptId, ModResult> {
        self.receipt(|out| unsafe { (self.table.submit_dialog_close)(button, out) })
    }

    fn bool_value(
        self,
        function: unsafe extern "system" fn(*mut u8) -> ModResult,
    ) -> Result<bool, ModResult> {
        let mut out = 0;
        result_with_out(unsafe { function(&mut out) }, out != 0)
    }

    fn receipt(
        self,
        submit: impl FnOnce(*mut CommandReceiptId) -> ModResult,
    ) -> Result<CommandReceiptId, ModResult> {
        let mut out = CommandReceiptId(0);
        result_with_out(submit(&mut out), out)
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
