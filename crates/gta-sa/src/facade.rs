//! Safe plugin-side GTA service facade.

use crate::{
    EntitySnapshot, ObjectHandle, PedHandle, PedSnapshot, TimerSnapshot, Vector3, VehicleHandle,
    VehicleSnapshot,
};
use modkit_abi::{
    CommandReceiptId, GTA_POOL_OBJECT_V1, GTA_POOL_PED_V1, GTA_POOL_VEHICLE_V1, GameContextTokenV1,
    GtaPedSnapshotV1, GtaPoolKindV1, GtaTimerSnapshotV1, GtaVector3V1, GtaVehicleSnapshotV1,
    ModResult, SubscriptionId,
};
use modkit_sdk::{Core, GameContext, GtaSaService, Host, ServiceError};
use std::{
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Mutex,
    time::Duration,
};

#[derive(Clone, Copy)]
pub struct Gta {
    core: Core,
    service: GtaSaService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    ReceiptConsumed,
    Service(ServiceError),
    Host(ModResult),
    NoLocalPed,
    InvalidHandle,
}

impl From<ServiceError> for Error {
    fn from(value: ServiceError) -> Self {
        Self::Service(value)
    }
}

impl Gta {
    pub fn from_host(host: Host) -> Result<Self, Error> {
        Ok(Self {
            core: host.core()?,
            service: host.gta_sa_service()?,
        })
    }

    /// Registers one serialized post-game-process callback.
    pub fn on_tick<F>(self, callback: F) -> Result<TickSubscription, Error>
    where
        F: for<'scope> FnMut(TickContext<'scope>) -> Result<(), Error> + Send + 'static,
    {
        let state = Box::new(TickHandler {
            gta: self,
            callback: Mutex::new(Box::new(callback)),
        });
        let user_data = Box::into_raw(state).cast::<c_void>();
        let registration = unsafe {
            self.service
                .register_tick(tick_trampoline, user_data, release_tick_handler)
        };
        match registration {
            Ok(id) => Ok(TickSubscription {
                core: self.core,
                id: Some(id),
            }),
            Err(error) => {
                unsafe { drop(Box::from_raw(user_data.cast::<TickHandler>())) };
                Err(Error::Host(error))
            }
        }
    }

    #[must_use]
    pub const fn player(self) -> QueuedPlayer {
        QueuedPlayer { gta: self }
    }

    #[must_use]
    pub const fn peds(self) -> QueuedPedPool {
        QueuedPedPool { gta: self }
    }

    #[must_use]
    pub const fn vehicles(self) -> QueuedVehiclePool {
        QueuedVehiclePool { gta: self }
    }

    #[must_use]
    pub const fn objects(self) -> QueuedObjectPool {
        QueuedObjectPool { gta: self }
    }

    #[must_use]
    pub const fn world(self) -> QueuedWorld {
        QueuedWorld { gta: self }
    }
    #[must_use]
    pub const fn timer(self) -> QueuedTimer {
        QueuedTimer { gta: self }
    }

    pub fn submit_local_ped_snapshot(self) -> Result<SnapshotReceipt, Error> {
        let id = self
            .service
            .submit_local_ped_snapshot()
            .map_err(Error::Host)?;
        Ok(SnapshotReceipt {
            core: self.core,
            service: self.service,
            id: Some(id),
        })
    }

    pub fn submit_teleport_local_ped(self, destination: Vector3) -> Result<CommandReceipt, Error> {
        let id = self
            .service
            .submit_teleport_local_ped(vector_to_abi(destination))
            .map_err(Error::Host)?;
        Ok(CommandReceipt {
            core: self.core,
            id: Some(id),
        })
    }
}

pub trait HostGtaSaExt {
    fn gta_sa(self) -> Result<Gta, Error>;
}

impl HostGtaSaExt for Host {
    fn gta_sa(self) -> Result<Gta, Error> {
        Gta::from_host(self)
    }
}

#[derive(Clone, Copy)]
pub struct QueuedPlayer {
    gta: Gta,
}

impl QueuedPlayer {
    pub fn snapshot(self) -> Result<SnapshotReceipt, Error> {
        self.gta.submit_local_ped_snapshot()
    }

    pub fn teleport(self, destination: Vector3) -> Result<CommandReceipt, Error> {
        self.gta.submit_teleport_local_ped(destination)
    }
}

#[derive(Clone, Copy)]
pub struct QueuedPedPool {
    gta: Gta,
}

impl QueuedPedPool {
    pub fn exists(self, handle: PedHandle) -> Result<ExistenceReceipt, Error> {
        submit_exists(self.gta, GTA_POOL_PED_V1, handle.get())
    }
}

#[derive(Clone, Copy)]
pub struct QueuedVehiclePool {
    gta: Gta,
}

impl QueuedVehiclePool {
    pub fn exists(self, handle: VehicleHandle) -> Result<ExistenceReceipt, Error> {
        submit_exists(self.gta, GTA_POOL_VEHICLE_V1, handle.get())
    }

    pub fn snapshot(self, handle: VehicleHandle) -> Result<VehicleSnapshotReceipt, Error> {
        let id = self
            .gta
            .service
            .submit_vehicle_snapshot(handle.get())
            .map_err(Error::Host)?;
        Ok(VehicleSnapshotReceipt {
            core: self.gta.core,
            service: self.gta.service,
            id: Some(id),
        })
    }
}

#[derive(Clone, Copy)]
pub struct QueuedObjectPool {
    gta: Gta,
}

impl QueuedObjectPool {
    pub fn exists(self, handle: ObjectHandle) -> Result<ExistenceReceipt, Error> {
        submit_exists(self.gta, GTA_POOL_OBJECT_V1, handle.get())
    }
}

#[derive(Clone, Copy)]
pub struct QueuedWorld {
    gta: Gta,
}

impl QueuedWorld {
    pub fn ground_z(self, x: f32, y: f32) -> Result<GroundZReceipt, Error> {
        let id = self
            .gta
            .service
            .submit_find_ground_z(x, y)
            .map_err(Error::Host)?;
        Ok(GroundZReceipt {
            core: self.gta.core,
            service: self.gta.service,
            id: Some(id),
        })
    }
}
#[derive(Clone, Copy)]
pub struct QueuedTimer {
    gta: Gta,
}

impl QueuedTimer {
    pub fn snapshot(self) -> Result<TimerSnapshotReceipt, Error> {
        let id = self
            .gta
            .service
            .submit_timer_snapshot()
            .map_err(Error::Host)?;
        Ok(TimerSnapshotReceipt {
            core: self.gta.core,
            service: self.gta.service,
            id: Some(id),
        })
    }
}

fn submit_exists(gta: Gta, kind: GtaPoolKindV1, handle: i32) -> Result<ExistenceReceipt, Error> {
    let id = gta
        .service
        .submit_entity_exists(kind, handle)
        .map_err(Error::Host)?;
    Ok(ExistenceReceipt {
        core: gta.core,
        service: gta.service,
        id: Some(id),
    })
}

pub struct TickContext<'scope> {
    gta: Gta,
    context: GameContext<'scope>,
}

impl<'scope> TickContext<'scope> {
    pub fn player(&'scope self) -> Result<Player<'scope>, Error> {
        let snapshot = self
            .gta
            .service
            .local_ped_snapshot(&self.context)
            .map_err(|error| {
                if error == modkit_abi::MOD_NOT_FOUND {
                    Error::NoLocalPed
                } else {
                    Error::Host(error)
                }
            })?;
        Ok(Player {
            service: self.gta.service,
            context: &self.context,
            snapshot: snapshot_from_abi(snapshot)?,
        })
    }

    #[must_use]
    pub fn peds(&'scope self) -> PedPool<'scope> {
        PedPool {
            service: self.gta.service,
            context: &self.context,
        }
    }

    #[must_use]
    pub fn vehicles(&'scope self) -> VehiclePool<'scope> {
        VehiclePool {
            service: self.gta.service,
            context: &self.context,
        }
    }

    #[must_use]
    pub fn objects(&'scope self) -> ObjectPool<'scope> {
        ObjectPool {
            service: self.gta.service,
            context: &self.context,
        }
    }

    #[must_use]
    pub fn world(&'scope self) -> World<'scope> {
        World {
            service: self.gta.service,
            context: &self.context,
        }
    }
    #[must_use]
    pub fn timer(&'scope self) -> Timer<'scope> {
        Timer {
            service: self.gta.service,
            context: &self.context,
        }
    }
}

pub struct PedPool<'scope> {
    service: GtaSaService,
    context: &'scope GameContext<'scope>,
}

impl PedPool<'_> {
    pub fn exists(&self, handle: PedHandle) -> Result<bool, Error> {
        self.service
            .entity_exists(self.context, GTA_POOL_PED_V1, handle.get())
            .map_err(Error::Host)
    }
}

pub struct VehiclePool<'scope> {
    service: GtaSaService,
    context: &'scope GameContext<'scope>,
}

impl VehiclePool<'_> {
    pub fn exists(&self, handle: VehicleHandle) -> Result<bool, Error> {
        self.service
            .entity_exists(self.context, GTA_POOL_VEHICLE_V1, handle.get())
            .map_err(Error::Host)
    }

    pub fn snapshot(&self, handle: VehicleHandle) -> Result<Option<VehicleSnapshot>, Error> {
        match self.service.vehicle_snapshot(self.context, handle.get()) {
            Ok(snapshot) => vehicle_snapshot_from_abi(snapshot).map(Some),
            Err(error) if error == modkit_abi::MOD_NOT_FOUND => Ok(None),
            Err(error) => Err(Error::Host(error)),
        }
    }
}

pub struct ObjectPool<'scope> {
    service: GtaSaService,
    context: &'scope GameContext<'scope>,
}

impl ObjectPool<'_> {
    pub fn exists(&self, handle: ObjectHandle) -> Result<bool, Error> {
        self.service
            .entity_exists(self.context, GTA_POOL_OBJECT_V1, handle.get())
            .map_err(Error::Host)
    }
}

pub struct World<'scope> {
    service: GtaSaService,
    context: &'scope GameContext<'scope>,
}

impl World<'_> {
    pub fn ground_z(&self, x: f32, y: f32) -> Result<f32, Error> {
        self.service
            .find_ground_z(self.context, x, y)
            .map_err(Error::Host)
    }
}
pub struct Timer<'scope> {
    service: GtaSaService,
    context: &'scope GameContext<'scope>,
}

impl Timer<'_> {
    pub fn snapshot(&self) -> Result<TimerSnapshot, Error> {
        self.service
            .timer_snapshot(self.context)
            .map(timer_snapshot_from_abi)
            .map_err(Error::Host)
    }
}

pub struct Player<'scope> {
    service: GtaSaService,
    context: &'scope GameContext<'scope>,
    snapshot: PedSnapshot,
}

impl Player<'_> {
    pub const fn snapshot(&self) -> Result<PedSnapshot, Error> {
        Ok(self.snapshot)
    }

    #[must_use]
    pub const fn position(&self) -> Vector3 {
        self.snapshot.position()
    }

    pub fn teleport(&self, destination: Vector3) -> Result<(), Error> {
        self.service
            .teleport_local_ped(self.context, vector_to_abi(destination))
            .map_err(Error::Host)
    }
}

type TickCallback =
    dyn for<'scope> FnMut(TickContext<'scope>) -> Result<(), Error> + Send + 'static;

struct TickHandler {
    gta: Gta,
    callback: Mutex<Box<TickCallback>>,
}

unsafe extern "system" fn tick_trampoline(user_data: *mut c_void, token: GameContextTokenV1) {
    let Some(handler) = (unsafe { user_data.cast::<TickHandler>().as_ref() }) else {
        return;
    };
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let context = unsafe { GameContext::from_raw(token) };
        let mut callback = handler
            .callback
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let _ = callback(TickContext {
            gta: handler.gta,
            context,
        });
    }));
}

unsafe extern "system" fn release_tick_handler(user_data: *mut c_void) {
    if !user_data.is_null() {
        unsafe { drop(Box::from_raw(user_data.cast::<TickHandler>())) };
    }
}

pub struct TickSubscription {
    core: Core,
    id: Option<SubscriptionId>,
}

impl TickSubscription {
    pub fn unregister_and_wait(mut self, timeout: Duration) -> Result<(), Error> {
        let Some(id) = self.id.take() else {
            return Ok(());
        };
        self.core
            .unregister_and_wait(id, timeout)
            .map_err(Error::Host)
    }
}

impl Drop for TickSubscription {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.unregister(id);
        }
    }
}

pub struct CommandReceipt {
    core: Core,
    id: Option<CommandReceiptId>,
}

impl CommandReceipt {
    pub fn wait(mut self, timeout: Duration) -> Result<(), Error> {
        let id = self.id.take().ok_or(Error::ReceiptConsumed)?;
        let completion = self.core.receipt_wait(id, timeout).map_err(Error::Host)?;
        if completion.status.is_ok() {
            Ok(())
        } else {
            Err(Error::Host(completion.status))
        }
    }
}

impl Drop for CommandReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

pub struct SnapshotReceipt {
    core: Core,
    service: GtaSaService,
    id: Option<CommandReceiptId>,
}

impl SnapshotReceipt {
    pub fn wait(mut self, timeout: Duration) -> Result<PedSnapshot, Error> {
        let id = self.id.take().ok_or(Error::ReceiptConsumed)?;
        let completion = self.core.receipt_wait(id, timeout).map_err(Error::Host)?;
        if !completion.status.is_ok() {
            return Err(Error::Host(completion.status));
        }
        self.service
            .take_local_ped_snapshot(id)
            .map_err(|error| {
                if error == modkit_abi::MOD_NOT_FOUND {
                    Error::NoLocalPed
                } else {
                    Error::Host(error)
                }
            })
            .and_then(snapshot_from_abi)
    }
}

impl Drop for SnapshotReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

pub struct ExistenceReceipt {
    core: Core,
    service: GtaSaService,
    id: Option<CommandReceiptId>,
}

impl ExistenceReceipt {
    pub fn wait(mut self, timeout: Duration) -> Result<bool, Error> {
        let id = self.id.take().ok_or(Error::ReceiptConsumed)?;
        let completion = self.core.receipt_wait(id, timeout).map_err(Error::Host)?;
        if !completion.status.is_ok() {
            return Err(Error::Host(completion.status));
        }
        self.service.take_entity_exists(id).map_err(Error::Host)
    }
}

impl Drop for ExistenceReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

pub struct VehicleSnapshotReceipt {
    core: Core,
    service: GtaSaService,
    id: Option<CommandReceiptId>,
}

impl VehicleSnapshotReceipt {
    pub fn wait(mut self, timeout: Duration) -> Result<Option<VehicleSnapshot>, Error> {
        let id = self.id.take().ok_or(Error::ReceiptConsumed)?;
        let completion = self.core.receipt_wait(id, timeout).map_err(Error::Host)?;
        if !completion.status.is_ok() {
            return Err(Error::Host(completion.status));
        }
        match self.service.take_vehicle_snapshot(id) {
            Ok(snapshot) => vehicle_snapshot_from_abi(snapshot).map(Some),
            Err(error) if error == modkit_abi::MOD_NOT_FOUND => Ok(None),
            Err(error) => Err(Error::Host(error)),
        }
    }
}

impl Drop for VehicleSnapshotReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

pub struct GroundZReceipt {
    core: Core,
    service: GtaSaService,
    id: Option<CommandReceiptId>,
}

impl GroundZReceipt {
    pub fn wait(mut self, timeout: Duration) -> Result<f32, Error> {
        let id = self.id.take().ok_or(Error::ReceiptConsumed)?;
        let completion = self.core.receipt_wait(id, timeout).map_err(Error::Host)?;
        if !completion.status.is_ok() {
            return Err(Error::Host(completion.status));
        }
        self.service.take_find_ground_z(id).map_err(Error::Host)
    }
}

impl Drop for GroundZReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}
pub struct TimerSnapshotReceipt {
    core: Core,
    service: GtaSaService,
    id: Option<CommandReceiptId>,
}

impl TimerSnapshotReceipt {
    pub fn wait(mut self, timeout: Duration) -> Result<TimerSnapshot, Error> {
        let id = self.id.take().ok_or(Error::ReceiptConsumed)?;
        let completion = self.core.receipt_wait(id, timeout).map_err(Error::Host)?;
        if !completion.status.is_ok() {
            return Err(Error::Host(completion.status));
        }
        self.service
            .take_timer_snapshot(id)
            .map(timer_snapshot_from_abi)
            .map_err(Error::Host)
    }
}

impl Drop for TimerSnapshotReceipt {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            let _ = self.core.receipt_release(id);
        }
    }
}

fn vector_to_abi(value: Vector3) -> GtaVector3V1 {
    GtaVector3V1 {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

fn snapshot_from_abi(value: GtaPedSnapshotV1) -> Result<PedSnapshot, Error> {
    let handle = PedHandle::new(value.handle).ok_or(Error::InvalidHandle)?;
    Ok(PedSnapshot {
        handle,
        entity: EntitySnapshot {
            position: Vector3::new(
                value.entity.position.x,
                value.entity.position.y,
                value.entity.position.z,
            ),
        },
        health: value.health,
        armour: value.armour,
    })
}

fn vehicle_snapshot_from_abi(value: GtaVehicleSnapshotV1) -> Result<VehicleSnapshot, Error> {
    let handle = VehicleHandle::new(value.handle).ok_or(Error::InvalidHandle)?;
    Ok(VehicleSnapshot {
        handle,
        entity: EntitySnapshot {
            position: Vector3::new(
                value.entity.position.x,
                value.entity.position.y,
                value.entity.position.z,
            ),
        },
        health: value.health,
    })
}
fn timer_snapshot_from_abi(value: GtaTimerSnapshotV1) -> TimerSnapshot {
    TimerSnapshot {
        frame_counter: value.frame_counter,
        game_time_ms: value.game_time_ms,
        time_step: value.time_step,
        time_step_non_clipped: value.time_step_non_clipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vehicle_snapshot_conversion_owns_values_and_validates_handle() {
        let snapshot = vehicle_snapshot_from_abi(GtaVehicleSnapshotV1 {
            handle: 19,
            reserved: 0,
            entity: modkit_abi::GtaEntitySnapshotV1 {
                position: GtaVector3V1 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
            },
            health: 875.0,
        })
        .unwrap();
        assert_eq!(snapshot.handle, VehicleHandle::new(19).unwrap());
        assert_eq!(snapshot.position(), Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(snapshot.health, 875.0);

        assert_eq!(
            vehicle_snapshot_from_abi(GtaVehicleSnapshotV1::default()),
            Err(Error::InvalidHandle)
        );
    }

    #[test]
    fn timer_snapshot_conversion_preserves_native_units() {
        let snapshot = timer_snapshot_from_abi(GtaTimerSnapshotV1 {
            frame_counter: 42,
            game_time_ms: 1_250,
            time_step: 1.0,
            time_step_non_clipped: 1.25,
        });
        assert_eq!(snapshot.frame_counter, 42);
        assert_eq!(snapshot.game_time_ms, 1_250);
        assert_eq!(snapshot.time_step, 1.0);
        assert_eq!(snapshot.time_step_non_clipped, 1.25);
    }
}
