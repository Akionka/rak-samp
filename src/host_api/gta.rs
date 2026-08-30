//! GTA SA service callbacks and native operation adapters.

use super::{clone_initialized, host, is_shutting_down, next_subscription_id};
use crate::host_api::reclamation::PluginRelease;
use gta_sa::{
    CameraSnapshot, ObjectHandle, PedHandle, PedSnapshot, TimerSnapshot, Vector3, VehicleHandle,
    VehicleSnapshot,
};
use modkit_abi::{
    CommandReceiptId, GTA_POOL_OBJECT_V1, GTA_POOL_PED_V1, GTA_POOL_VEHICLE_V1, GameContextTokenV1,
    GtaCameraSnapshotV1, GtaMatrixV1, GtaPedSnapshotV1, GtaPoolKindV1, GtaReleaseCallbackV1,
    GtaTickCallbackV1, GtaTimerSnapshotV1, GtaVector3V1, GtaVehicleSnapshotV1, MOD_BUSY,
    MOD_INVALID_ARGUMENT, MOD_NOT_FOUND, MOD_NOT_READY, MOD_OK, MOD_SHUTTING_DOWN, ModResult,
    SubscriptionId,
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

pub(super) unsafe extern "system" fn entity_exists(
    context: GameContextTokenV1,
    kind: GtaPoolKindV1,
    handle: i32,
    out_exists: *mut u8,
) -> ModResult {
    let Some(out_exists) = (unsafe { out_exists.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out_exists = 0;
    let Ok(handle) = entity_handle_from_abi(kind, handle) else {
        return MOD_INVALID_ARGUMENT;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_entity_exists(ScopeToken::from_raw(context.raw()), handle) {
        Ok(exists) => {
            *out_exists = u8::from(exists);
            MOD_OK
        }
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn submit_entity_exists(
    kind: GtaPoolKindV1,
    handle: i32,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    let Ok(handle) = entity_handle_from_abi(kind, handle) else {
        return MOD_INVALID_ARGUMENT;
    };
    submit_receipt(out_receipt, |runtime| {
        runtime.submit_gta_entity_exists(handle)
    })
}

pub(super) unsafe extern "system" fn take_entity_exists(
    receipt: CommandReceiptId,
    out_exists: *mut u8,
) -> ModResult {
    let Some(out_exists) = (unsafe { out_exists.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out_exists = 0;
    if receipt.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.take_gta_entity_exists(receipt.0) {
        Some(exists) => {
            *out_exists = u8::from(exists);
            MOD_OK
        }
        None => MOD_NOT_READY,
    }
}

pub(super) unsafe extern "system" fn vehicle_snapshot(
    context: GameContextTokenV1,
    handle: i32,
    out: *mut GtaVehicleSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaVehicleSnapshotV1::default();
    let Some(handle) = VehicleHandle::new(handle) else {
        return MOD_INVALID_ARGUMENT;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_vehicle_snapshot(ScopeToken::from_raw(context.raw()), handle) {
        Ok(Some(snapshot)) => {
            *out = vehicle_snapshot_to_abi(snapshot);
            MOD_OK
        }
        Ok(None) => MOD_NOT_FOUND,
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn submit_vehicle_snapshot(
    handle: i32,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    let Some(handle) = VehicleHandle::new(handle) else {
        return MOD_INVALID_ARGUMENT;
    };
    submit_receipt(out_receipt, |runtime| {
        runtime.submit_gta_vehicle_snapshot(handle)
    })
}

pub(super) unsafe extern "system" fn take_vehicle_snapshot(
    receipt: CommandReceiptId,
    out: *mut GtaVehicleSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaVehicleSnapshotV1::default();
    if receipt.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.take_gta_vehicle_snapshot(receipt.0) {
        Some(Some(snapshot)) => {
            *out = vehicle_snapshot_to_abi(snapshot);
            MOD_OK
        }
        Some(None) => MOD_NOT_FOUND,
        None => MOD_NOT_READY,
    }
}

pub(super) unsafe extern "system" fn find_ground_z(
    context: GameContextTokenV1,
    x: f32,
    y: f32,
    out_z: *mut f32,
) -> ModResult {
    let Some(out_z) = (unsafe { out_z.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out_z = 0.0;
    if !x.is_finite() || !y.is_finite() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_find_ground_z(ScopeToken::from_raw(context.raw()), x, y) {
        Ok(z) => {
            *out_z = z;
            MOD_OK
        }
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn submit_find_ground_z(
    x: f32,
    y: f32,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    if !x.is_finite() || !y.is_finite() {
        return MOD_INVALID_ARGUMENT;
    }
    submit_receipt(out_receipt, |runtime| {
        runtime.submit_gta_find_ground_z(x, y)
    })
}

pub(super) unsafe extern "system" fn take_find_ground_z(
    receipt: CommandReceiptId,
    out_z: *mut f32,
) -> ModResult {
    let Some(out_z) = (unsafe { out_z.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out_z = 0.0;
    if receipt.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.take_gta_find_ground_z(receipt.0) {
        Some(z) => {
            *out_z = z;
            MOD_OK
        }
        None => MOD_NOT_READY,
    }
}
pub(super) unsafe extern "system" fn timer_snapshot(
    context: GameContextTokenV1,
    out: *mut GtaTimerSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaTimerSnapshotV1::default();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_timer_snapshot(ScopeToken::from_raw(context.raw())) {
        Ok(snapshot) => {
            *out = timer_snapshot_to_abi(snapshot);
            MOD_OK
        }
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn submit_timer_snapshot(
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    submit_receipt(out_receipt, |runtime| runtime.submit_gta_timer_snapshot())
}

pub(super) unsafe extern "system" fn take_timer_snapshot(
    receipt: CommandReceiptId,
    out: *mut GtaTimerSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaTimerSnapshotV1::default();
    if receipt.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.take_gta_timer_snapshot(receipt.0) {
        Some(snapshot) => {
            *out = timer_snapshot_to_abi(snapshot);
            MOD_OK
        }
        None => MOD_NOT_READY,
    }
}
pub(super) unsafe extern "system" fn camera_snapshot(
    context: GameContextTokenV1,
    out: *mut GtaCameraSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaCameraSnapshotV1::default();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.gta_camera_snapshot(ScopeToken::from_raw(context.raw())) {
        Ok(snapshot) => {
            *out = camera_snapshot_to_abi(snapshot);
            MOD_OK
        }
        Err(error) => super::modkit::subscription_result(super::direct_client_result(error)),
    }
}

pub(super) unsafe extern "system" fn submit_camera_snapshot(
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    submit_receipt(out_receipt, |runtime| runtime.submit_gta_camera_snapshot())
}

pub(super) unsafe extern "system" fn take_camera_snapshot(
    receipt: CommandReceiptId,
    out: *mut GtaCameraSnapshotV1,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = GtaCameraSnapshotV1::default();
    if receipt.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    match runtime.take_gta_camera_snapshot(receipt.0) {
        Some(snapshot) => {
            *out = camera_snapshot_to_abi(snapshot);
            MOD_OK
        }
        None => MOD_NOT_READY,
    }
}

fn entity_handle_from_abi(
    kind: GtaPoolKindV1,
    handle: i32,
) -> Result<crate::runtime::GtaEntityHandle, ()> {
    if kind == GTA_POOL_PED_V1 {
        PedHandle::new(handle)
            .map(crate::runtime::GtaEntityHandle::Ped)
            .ok_or(())
    } else if kind == GTA_POOL_VEHICLE_V1 {
        VehicleHandle::new(handle)
            .map(crate::runtime::GtaEntityHandle::Vehicle)
            .ok_or(())
    } else if kind == GTA_POOL_OBJECT_V1 {
        ObjectHandle::new(handle)
            .map(crate::runtime::GtaEntityHandle::Object)
            .ok_or(())
    } else {
        Err(())
    }
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

fn vehicle_snapshot_to_abi(snapshot: VehicleSnapshot) -> GtaVehicleSnapshotV1 {
    GtaVehicleSnapshotV1 {
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
    }
}
fn timer_snapshot_to_abi(snapshot: TimerSnapshot) -> GtaTimerSnapshotV1 {
    GtaTimerSnapshotV1 {
        frame_counter: snapshot.frame_counter,
        game_time_ms: snapshot.game_time_ms,
        time_step: snapshot.time_step,
        time_step_non_clipped: snapshot.time_step_non_clipped,
    }
}
fn camera_snapshot_to_abi(snapshot: CameraSnapshot) -> GtaCameraSnapshotV1 {
    GtaCameraSnapshotV1 {
        game_position: vector_to_abi(snapshot.game_position),
        transform: GtaMatrixV1 {
            right: vector_to_abi(snapshot.transform.right),
            forward: vector_to_abi(snapshot.transform.forward),
            up: vector_to_abi(snapshot.transform.up),
            position: vector_to_abi(snapshot.transform.position),
        },
    }
}

fn vector_to_abi(value: Vector3) -> GtaVector3V1 {
    GtaVector3V1 {
        x: value.x,
        y: value.y,
        z: value.z,
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

    #[test]
    fn pool_abi_rejects_invalid_inputs_before_runtime_access() {
        let mut exists = 7;
        assert_eq!(
            unsafe {
                entity_exists(
                    GameContextTokenV1::from_raw(1),
                    GtaPoolKindV1(99),
                    1,
                    &mut exists,
                )
            },
            MOD_INVALID_ARGUMENT
        );
        assert_eq!(exists, 0);
        assert_eq!(
            unsafe {
                entity_exists(
                    GameContextTokenV1::from_raw(1),
                    GTA_POOL_PED_V1,
                    0,
                    &mut exists,
                )
            },
            MOD_INVALID_ARGUMENT
        );
        assert_eq!(
            unsafe {
                entity_exists(
                    GameContextTokenV1::from_raw(1),
                    GTA_POOL_PED_V1,
                    1,
                    std::ptr::null_mut(),
                )
            },
            MOD_INVALID_ARGUMENT
        );
    }

    #[test]
    fn vehicle_snapshot_abi_zeros_output_before_rejecting_handle() {
        let mut out = GtaVehicleSnapshotV1 {
            handle: 7,
            ..GtaVehicleSnapshotV1::default()
        };
        assert_eq!(
            unsafe { vehicle_snapshot(GameContextTokenV1::from_raw(1), 0, &mut out) },
            MOD_INVALID_ARGUMENT
        );
        assert_eq!(out, GtaVehicleSnapshotV1::default());
    }

    #[test]
    fn ground_query_abi_rejects_non_finite_coordinates_before_runtime_access() {
        let mut out = 7.0;
        assert_eq!(
            unsafe { find_ground_z(GameContextTokenV1::from_raw(1), f32::NAN, 0.0, &mut out,) },
            MOD_INVALID_ARGUMENT
        );
        assert_eq!(out, 0.0);
        assert_eq!(
            unsafe {
                find_ground_z(
                    GameContextTokenV1::from_raw(1),
                    0.0,
                    0.0,
                    std::ptr::null_mut(),
                )
            },
            MOD_INVALID_ARGUMENT
        );
    }

    #[test]
    fn timer_snapshot_abi_rejects_null_output() {
        assert_eq!(
            unsafe { timer_snapshot(GameContextTokenV1::from_raw(1), std::ptr::null_mut(),) },
            MOD_INVALID_ARGUMENT
        );
    }

    #[test]
    fn camera_snapshot_abi_rejects_null_output() {
        assert_eq!(
            unsafe { camera_snapshot(GameContextTokenV1::from_raw(1), std::ptr::null_mut()) },
            MOD_INVALID_ARGUMENT
        );
    }
}
