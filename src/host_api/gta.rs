//! GTA SA service callbacks and native operation adapters.

use super::{clone_initialized, host, is_shutting_down, next_subscription_id};
use crate::host_api::reclamation::PluginRelease;
use gta_sa::{PedSnapshot, Vector3};
use modkit_abi::{
    CommandReceiptId, GameContextTokenV1, GtaPedSnapshotV1, GtaReleaseCallbackV1,
    GtaTickCallbackV1, GtaVector3V1, MOD_BUSY, MOD_INVALID_ARGUMENT, MOD_NOT_FOUND, MOD_NOT_READY,
    MOD_OK, MOD_SHUTTING_DOWN, ModResult, SubscriptionId,
};
use modkit_runtime::{CallbackContext, CallbackGate, ScopeToken};
use sdk_abi::SampClientSdkResult;
use std::{
    collections::BTreeMap,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex},
    time::Duration,
};

pub(super) struct GtaTickRegistry {
    entries: Mutex<BTreeMap<u64, Arc<GtaTickEntry>>>,
}

struct GtaTickEntry {
    callback: GtaTickCallbackV1,
    user_data: usize,
    release: Mutex<Option<PluginRelease>>,
    gate: CallbackGate,
}

impl GtaTickRegistry {
    pub(super) fn new() -> Self {
        Self {
            entries: Mutex::new(BTreeMap::new()),
        }
    }

    fn register(
        &self,
        callback: GtaTickCallbackV1,
        user_data: usize,
        release: GtaReleaseCallbackV1,
    ) -> Option<u64> {
        let id = next_subscription_id()?;
        self.entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(
                id,
                Arc::new(GtaTickEntry {
                    callback,
                    user_data,
                    release: Mutex::new(Some(PluginRelease::new(user_data, release))),
                    gate: CallbackGate::new(),
                }),
            );
        Some(id)
    }

    fn remove(&self, id: u64) -> Option<Arc<GtaTickEntry>> {
        let entry = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&id)?;
        entry.gate.set_allowed(false);
        Some(entry)
    }

    fn dispatch(&self, token: ScopeToken) {
        let entries: Vec<_> = self
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .cloned()
            .collect();
        let context = unsafe { GameContextTokenV1::from_raw(token.raw()) };
        for entry in entries {
            let Some(_flight) = entry.gate.enter() else {
                continue;
            };
            let _callback = CallbackContext::enter();
            let result = catch_unwind(AssertUnwindSafe(|| unsafe {
                (entry.callback)(entry.user_data as *mut c_void, context)
            }));
            if result.is_err() {
                log::error!("GTA tick callback panicked across its adapter boundary");
            }
        }
    }
    fn shutdown(&self) {
        let entries = {
            let mut entries = self
                .entries
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            for entry in entries.values() {
                entry.gate.set_allowed(false);
            }
            std::mem::take(&mut *entries)
        };
        for entry in entries.into_values() {
            entry.release_deferred();
        }
    }
}

impl GtaTickEntry {
    fn release(self: Arc<Self>) {
        if let Some(release) = self
            .release
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
        {
            release.release();
        }
    }

    fn release_deferred(self: Arc<Self>) {
        super::reclamation::defer(move || {
            self.gate.wait_until_drained();
            self.release();
        });
    }
}

pub(crate) fn dispatch_tick(token: ScopeToken) {
    host().gta_ticks.dispatch(token);
}
pub(crate) fn shutdown() {
    host().gta_ticks.shutdown();
}

pub(super) fn unregister(id: u64) -> Option<SampClientSdkResult> {
    let entry = host().gta_ticks.remove(id)?;
    entry.release_deferred();
    Some(SampClientSdkResult::Ok)
}

pub(super) fn unregister_and_wait(
    id: u64,
    timeout: Option<Duration>,
) -> Option<SampClientSdkResult> {
    let entry = host().gta_ticks.remove(id)?;
    let drained = timeout.map_or_else(
        || {
            entry.gate.wait_until_drained();
            true
        },
        |duration| entry.gate.wait_until_drained_timeout(duration),
    );
    if !drained {
        entry.gate.set_allowed(true);
        host()
            .gta_ticks
            .entries
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, entry);
        return Some(SampClientSdkResult::TimedOut);
    }
    entry.release();
    Some(SampClientSdkResult::Ok)
}

pub(super) unsafe extern "system" fn register_tick(
    callback: Option<GtaTickCallbackV1>,
    user_data: *mut c_void,
    release: Option<GtaReleaseCallbackV1>,
    out_subscription: *mut SubscriptionId,
) -> ModResult {
    if out_subscription.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    if is_shutting_down() {
        return MOD_SHUTTING_DOWN;
    }
    let (Some(callback), Some(release)) = (callback, release) else {
        return MOD_INVALID_ARGUMENT;
    };
    if clone_initialized(&host().runtime).is_none() {
        return MOD_NOT_READY;
    }
    let Some(id) = host()
        .gta_ticks
        .register(callback, user_data as usize, release)
    else {
        return MOD_BUSY;
    };
    unsafe { out_subscription.write(SubscriptionId(id)) };
    MOD_OK
}

pub(super) unsafe extern "system" fn local_ped_snapshot(
    context: GameContextTokenV1,
    out: *mut GtaPedSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaPedSnapshotV1::default();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_local_ped_snapshot(ScopeToken::from_raw(context.raw())) {
        Ok(Some(snapshot)) => {
            *out = snapshot_to_abi(snapshot);
            MOD_OK
        }
        Ok(None) => MOD_NOT_FOUND,
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn teleport_local_ped(
    context: GameContextTokenV1,
    destination: GtaVector3V1,
) -> ModResult {
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_teleport_local_ped(
        ScopeToken::from_raw(context.raw()),
        vector_from_abi(destination),
    ) {
        Ok(()) => MOD_OK,
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn submit_local_ped_snapshot(
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    submit_receipt(out_receipt, |runtime| {
        runtime.submit_gta_local_ped_snapshot()
    })
}

pub(super) unsafe extern "system" fn take_local_ped_snapshot(
    receipt: CommandReceiptId,
    out: *mut GtaPedSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaPedSnapshotV1::default();
    if receipt.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.take_gta_local_ped_snapshot(receipt.0) {
        Some(Some(snapshot)) => {
            *out = snapshot_to_abi(snapshot);
            MOD_OK
        }
        Some(None) => MOD_NOT_FOUND,
        None => MOD_NOT_READY,
    }
}

pub(super) unsafe extern "system" fn submit_teleport_local_ped(
    destination: GtaVector3V1,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    submit_receipt(out_receipt, |runtime| {
        runtime.submit_gta_teleport_local_ped(vector_from_abi(destination))
    })
}

fn submit_receipt(
    out: *mut CommandReceiptId,
    submit: impl FnOnce(&crate::Runtime) -> Result<u64, crate::runtime::DirectClientError>,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    if is_shutting_down() {
        return MOD_SHUTTING_DOWN;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match submit(&runtime) {
        Ok(id) => {
            *out = CommandReceiptId(id);
            MOD_OK
        }
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

fn vector_from_abi(value: GtaVector3V1) -> Vector3 {
    Vector3::new(value.x, value.y, value.z)
}

fn snapshot_to_abi(snapshot: PedSnapshot) -> GtaPedSnapshotV1 {
    GtaPedSnapshotV1 {
        handle: snapshot.handle.get(),
        reserved: 0,
        entity: modkit_abi::GtaEntitySnapshotV1 {
            position: GtaVector3V1 {
                x: snapshot.entity.position.x,
                y: snapshot.entity.position.y,
                z: snapshot.entity.position.z,
            },
        },
        health: snapshot.health,
        armour: snapshot.armour,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    unsafe extern "system" fn count_tick(user_data: *mut c_void, context: GameContextTokenV1) {
        assert_eq!(context.raw(), 9);
        let count = unsafe { user_data.cast::<AtomicUsize>().as_ref() }.unwrap();
        count.fetch_add(1, Ordering::Relaxed);
    }

    unsafe extern "system" fn ignore_release(_user_data: *mut c_void) {}

    #[test]
    fn registry_dispatches_in_registration_order_and_stops_removed_entries() {
        let first = AtomicUsize::new(0);
        let second = AtomicUsize::new(0);
        let registry = GtaTickRegistry::new();
        registry.entries.lock().unwrap().insert(
            2,
            Arc::new(GtaTickEntry {
                callback: count_tick,
                user_data: (&second as *const AtomicUsize) as usize,
                release: Mutex::new(Some(PluginRelease::new(0, ignore_release))),
                gate: CallbackGate::new(),
            }),
        );
        registry.entries.lock().unwrap().insert(
            1,
            Arc::new(GtaTickEntry {
                callback: count_tick,
                user_data: (&first as *const AtomicUsize) as usize,
                release: Mutex::new(Some(PluginRelease::new(0, ignore_release))),
                gate: CallbackGate::new(),
            }),
        );

        registry.dispatch(ScopeToken::from_raw(9));
        assert_eq!(first.load(Ordering::Relaxed), 1);
        assert_eq!(second.load(Ordering::Relaxed), 1);

        let removed = registry.remove(1).unwrap();
        registry.dispatch(ScopeToken::from_raw(9));
        assert_eq!(first.load(Ordering::Relaxed), 1);
        assert_eq!(second.load(Ordering::Relaxed), 2);
        removed.release();
    }
}
