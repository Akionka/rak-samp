//! Safe plugin-side GTA service facade.

use crate::{EntitySnapshot, PedHandle, PedSnapshot, Vector3};
use modkit_abi::{
    CommandReceiptId, GameContextTokenV1, GtaPedSnapshotV1, GtaVector3V1, ModResult, SubscriptionId,
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
        let id = self.id.take().expect("receipt already consumed");
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
        let id = self.id.take().expect("receipt already consumed");
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
