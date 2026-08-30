//! GTA SA game-thread commands.

use super::*;
use gta_sa::{PedSnapshot, TimerSnapshot, Vector3, VehicleHandle, VehicleSnapshot};
use std::sync::{Arc, OnceLock};

#[derive(Clone, Copy, Debug)]
pub(in crate::platform::win32) enum GtaReadRequest {
    LocalPedSnapshot,
    EntityExists(GtaEntityHandle),
    VehicleSnapshot(VehicleHandle),
    GroundZ { x: f32, y: f32 },
    TimerSnapshot,
}
#[derive(Clone, Copy, Debug)]
pub(in crate::platform::win32) enum GtaReadResult {
    LocalPedSnapshot(Option<PedSnapshot>),
    EntityExists(bool),
    VehicleSnapshot(Option<VehicleSnapshot>),
    GroundZ(f32),
    TimerSnapshot(TimerSnapshot),
}

#[derive(Debug)]
pub(in crate::platform::win32) enum GtaCommand {
    Read {
        request: GtaReadRequest,
        result: Arc<OnceLock<GtaReadResult>>,
    },
    TeleportLocalPed(Vector3),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GtaCommandError {
    Ped,
    Pool,
    World,
    Timer,
}

impl BackendState {
    pub(crate) fn submit_gta_local_ped_snapshot(&self) -> Result<CommandId, DirectClientError> {
        self.submit_gta_read(GtaReadRequest::LocalPedSnapshot)
    }

    pub(crate) fn take_gta_local_ped_snapshot(&self, id: CommandId) -> Option<Option<PedSnapshot>> {
        self.take_gta_read(id, |result| match result {
            GtaReadResult::LocalPedSnapshot(snapshot) => Some(snapshot),
            _ => None,
        })
    }

    pub(crate) fn submit_gta_entity_exists(
        &self,
        handle: GtaEntityHandle,
    ) -> Result<CommandId, DirectClientError> {
        self.submit_gta_read(GtaReadRequest::EntityExists(handle))
    }

    pub(crate) fn take_gta_entity_exists(&self, id: CommandId) -> Option<bool> {
        self.take_gta_read(id, |result| match result {
            GtaReadResult::EntityExists(exists) => Some(exists),
            _ => None,
        })
    }

    pub(crate) fn submit_gta_vehicle_snapshot(
        &self,
        handle: VehicleHandle,
    ) -> Result<CommandId, DirectClientError> {
        self.submit_gta_read(GtaReadRequest::VehicleSnapshot(handle))
    }

    pub(crate) fn take_gta_vehicle_snapshot(
        &self,
        id: CommandId,
    ) -> Option<Option<VehicleSnapshot>> {
        self.take_gta_read(id, |result| match result {
            GtaReadResult::VehicleSnapshot(snapshot) => Some(snapshot),
            _ => None,
        })
    }

    pub(crate) fn submit_gta_find_ground_z(
        &self,
        x: f32,
        y: f32,
    ) -> Result<CommandId, DirectClientError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(DirectClientError::InvalidArgument);
        }
        self.submit_gta_read(GtaReadRequest::GroundZ { x, y })
    }

    pub(crate) fn take_gta_find_ground_z(&self, id: CommandId) -> Option<f32> {
        self.take_gta_read(id, |result| match result {
            GtaReadResult::GroundZ(z) => Some(z),
            _ => None,
        })
    }
    pub(crate) fn submit_gta_timer_snapshot(&self) -> Result<CommandId, DirectClientError> {
        self.submit_gta_read(GtaReadRequest::TimerSnapshot)
    }

    pub(crate) fn take_gta_timer_snapshot(&self, id: CommandId) -> Option<TimerSnapshot> {
        self.take_gta_read(id, |result| match result {
            GtaReadResult::TimerSnapshot(snapshot) => Some(snapshot),
            _ => None,
        })
    }

    pub(crate) fn submit_gta_teleport_local_ped(
        &self,
        destination: Vector3,
    ) -> Result<CommandId, DirectClientError> {
        if !destination.x.is_finite() || !destination.y.is_finite() || !destination.z.is_finite() {
            return Err(DirectClientError::InvalidArgument);
        }
        self.queue_game_command(GameCommand::Gta(GtaCommand::TeleportLocalPed(destination)))
    }

    pub(crate) fn gta_local_ped_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<Option<PedSnapshot>, DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::local_ped_snapshot(self.context.gta_profile) }
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(crate) fn gta_entity_exists(
        &self,
        token: modkit_runtime::ScopeToken,
        handle: GtaEntityHandle,
    ) -> Result<bool, DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { self.read_gta_entity_exists(handle) }.map_err(|_| DirectClientError::NotReady)
    }

    pub(crate) fn gta_vehicle_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
        handle: VehicleHandle,
    ) -> Result<Option<VehicleSnapshot>, DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::vehicle_snapshot(self.context.gta_profile, handle) }
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(crate) fn gta_find_ground_z(
        &self,
        token: modkit_runtime::ScopeToken,
        x: f32,
        y: f32,
    ) -> Result<f32, DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::find_ground_z(self.context.gta_profile, x, y) }.map_err(|error| {
            match error {
                gta_sa_native::WorldReadError::InvalidCoordinate => {
                    DirectClientError::InvalidArgument
                }
                _ => DirectClientError::NotReady,
            }
        })
    }
    pub(crate) fn gta_timer_snapshot(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<TimerSnapshot, DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::timer_snapshot(self.context.gta_profile) }
            .map_err(|_| DirectClientError::NotReady)
    }

    pub(crate) fn gta_teleport_local_ped(
        &self,
        token: modkit_runtime::ScopeToken,
        destination: Vector3,
    ) -> Result<(), DirectClientError> {
        self.validate_gta_context(token)?;
        unsafe { gta_sa_native::teleport_local_ped(self.context.gta_profile, destination) }.map_err(
            |error| match error {
                gta_sa_native::PedReadError::InvalidPosition => DirectClientError::InvalidArgument,
                _ => DirectClientError::NotReady,
            },
        )
    }

    fn submit_gta_read(&self, request: GtaReadRequest) -> Result<CommandId, DirectClientError> {
        let result = Arc::new(OnceLock::new());
        let id = self.queue_game_command(GameCommand::Gta(GtaCommand::Read {
            request,
            result: Arc::clone(&result),
        }))?;
        self.gta_read_results
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, result);
        Ok(id)
    }

    fn take_gta_read<T>(
        &self,
        id: CommandId,
        select: impl FnOnce(GtaReadResult) -> Option<T>,
    ) -> Option<T> {
        let mut results = self
            .gta_read_results
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let selected = select(results.get(&id)?.get().copied()?)?;
        results.remove(&id);
        Some(selected)
    }

    fn validate_gta_context(
        &self,
        token: modkit_runtime::ScopeToken,
    ) -> Result<(), DirectClientError> {
        self.game_scope
            .validate(
                token,
                modkit_runtime::NativeExecutionConstraint::PostGameProcessOnly,
            )
            .map_err(|error| match error {
                modkit_runtime::ScopeError::ShuttingDown => DirectClientError::NotReady,
                _ => DirectClientError::InvalidArgument,
            })
    }

    pub(super) fn execute_gta_command(&self, command: GtaCommand) -> Result<(), GtaCommandError> {
        match command {
            GtaCommand::Read { request, result } => {
                let value = match request {
                    GtaReadRequest::LocalPedSnapshot => GtaReadResult::LocalPedSnapshot(unsafe {
                        gta_sa_native::local_ped_snapshot(self.context.gta_profile)
                            .map_err(|_| GtaCommandError::Ped)?
                    }),
                    GtaReadRequest::EntityExists(handle) => GtaReadResult::EntityExists(unsafe {
                        self.read_gta_entity_exists(handle)
                            .map_err(|_| GtaCommandError::Pool)?
                    }),
                    GtaReadRequest::VehicleSnapshot(handle) => {
                        GtaReadResult::VehicleSnapshot(unsafe {
                            gta_sa_native::vehicle_snapshot(self.context.gta_profile, handle)
                                .map_err(|_| GtaCommandError::Pool)?
                        })
                    }
                    GtaReadRequest::GroundZ { x, y } => GtaReadResult::GroundZ(unsafe {
                        gta_sa_native::find_ground_z(self.context.gta_profile, x, y)
                            .map_err(|_| GtaCommandError::World)?
                    }),
                    GtaReadRequest::TimerSnapshot => GtaReadResult::TimerSnapshot(unsafe {
                        gta_sa_native::timer_snapshot(self.context.gta_profile)
                            .map_err(|_| GtaCommandError::Timer)?
                    }),
                };

                let _ = result.set(value);
                Ok(())
            }
            GtaCommand::TeleportLocalPed(destination) => unsafe {
                gta_sa_native::teleport_local_ped(self.context.gta_profile, destination)
                    .map_err(|_| GtaCommandError::Ped)
            },
        }
    }

    unsafe fn read_gta_entity_exists(
        &self,
        handle: GtaEntityHandle,
    ) -> Result<bool, gta_sa_native::PoolReadError> {
        match handle {
            GtaEntityHandle::Ped(handle) => unsafe {
                gta_sa_native::ped_exists(self.context.gta_profile, handle)
            },
            GtaEntityHandle::Vehicle(handle) => unsafe {
                gta_sa_native::vehicle_exists(self.context.gta_profile, handle)
            },
            GtaEntityHandle::Object(handle) => unsafe {
                gta_sa_native::object_exists(self.context.gta_profile, handle)
            },
        }
    }
}
