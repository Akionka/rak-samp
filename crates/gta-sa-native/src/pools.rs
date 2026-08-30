//! GTA `CPools` reference-conversion calls.
//!
//! The `CPools::GetPedRef` / `CPools::GetVehicleRef` targets are GTA-owned and
//! live in the GTA profile. SA-MP backends call through this wrapper instead of
//! holding their own GTA absolute addresses.

use crate::{
    GtaProfile, call::NativeCallTarget, layout::read_entity_position, profile::AbsoluteAddress,
};
use gta_sa::{EntitySnapshot, ObjectHandle, PedHandle, VehicleHandle, VehicleSnapshot};
use modkit_win32::ReadableRegion;
use std::ffi::c_void;

/// Calling convention of the selected `CPools` reference getter.
///
/// The R1 and classic SA-MP builds expose the same GTA function with different
/// calling conventions; the SA-MP backend selects the matching ABI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CpoolRefAbi {
    R1,
    Classic,
}

/// Failure to invoke a verified `CPools` reference getter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpoolRefError;

/// GTA pool selected for a live handle lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolKind {
    Ped,
    Vehicle,
    Object,
}

/// Failure to validate a GTA handle against its current native pool slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolReadError {
    InvalidHandle,
    NativeCall,
    UnreadableEntity,
}

/// Converts one guarded GTA game-object pointer to its GTA handle.
///
/// Returns `Ok(None)` when the getter reports a null handle (the entity is not
/// currently registered in the pool) and `Err` when the target is not readable.
///
/// # Safety
///
/// `target` must be a verified `CPools` reference getter for the active GTA
/// profile, `game_object` must be a valid, readable GTA game-object pointer for
/// the duration of the call, and `abi` must match the calling convention of the
/// selected getter.
pub unsafe fn cpool_ref(
    target: AbsoluteAddress,
    abi: CpoolRefAbi,
    game_object: *mut c_void,
) -> Result<Option<i32>, CpoolRefError> {
    let function = NativeCallTarget::resolve(target).map_err(|_| CpoolRefError)?;
    let handle = match abi {
        CpoolRefAbi::R1 | CpoolRefAbi::Classic => unsafe {
            function.call_cdecl_ptr_to_i32(game_object)
        },
    };
    Ok((handle != 0).then_some(handle))
}

/// Reports whether a ped handle currently resolves to a readable pool slot.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope. The selected
/// profile must match the loaded image.
pub unsafe fn ped_exists(profile: GtaProfile, handle: PedHandle) -> Result<bool, PoolReadError> {
    unsafe { pool_pointer(profile, PoolKind::Ped, handle.get()) }.map(|value| value.is_some())
}

/// Reports whether a vehicle handle currently resolves to a readable pool slot.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope. The selected
/// profile must match the loaded image.
pub unsafe fn vehicle_exists(
    profile: GtaProfile,
    handle: VehicleHandle,
) -> Result<bool, PoolReadError> {
    unsafe { pool_pointer(profile, PoolKind::Vehicle, handle.get()) }.map(|value| value.is_some())
}

/// Reports whether an object handle currently resolves to a readable pool slot.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope. The selected
/// profile must match the loaded image.
pub unsafe fn object_exists(
    profile: GtaProfile,
    handle: ObjectHandle,
) -> Result<bool, PoolReadError> {
    unsafe { pool_pointer(profile, PoolKind::Object, handle.get()) }.map(|value| value.is_some())
}

/// Copies the verified position and health fields from one live vehicle.
///
/// Returns `Ok(None)` when the handle no longer resolves to a current slot.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope. The selected
/// profile must match the loaded image.
pub unsafe fn vehicle_snapshot(
    profile: GtaProfile,
    handle: VehicleHandle,
) -> Result<Option<VehicleSnapshot>, PoolReadError> {
    let Some(vehicle) = (unsafe { pool_pointer(profile, PoolKind::Vehicle, handle.get())? }) else {
        return Ok(None);
    };
    let region = ReadableRegion::validate(vehicle as usize, profile.spec.vehicle.size.get())
        .ok_or(PoolReadError::UnreadableEntity)?;
    let position = read_entity_position(&region, profile.spec.entity)
        .map_err(|_| PoolReadError::UnreadableEntity)?;
    let health = unsafe { region.read_unaligned::<f32>(profile.spec.vehicle.health.get()) }
        .ok_or(PoolReadError::UnreadableEntity)?;
    Ok(Some(VehicleSnapshot {
        handle,
        entity: EntitySnapshot { position },
        health,
    }))
}

unsafe fn pool_pointer(
    profile: GtaProfile,
    kind: PoolKind,
    handle: i32,
) -> Result<Option<*mut c_void>, PoolReadError> {
    if handle <= 0 {
        return Err(PoolReadError::InvalidHandle);
    }
    let (pool_root, slot_size, object_size, getter) = match kind {
        PoolKind::Ped => (
            profile.spec.pools.ped_pool,
            profile.spec.pools.ped_slot_size,
            profile.spec.ped.size,
            profile.spec.pools.get_ped,
        ),
        PoolKind::Vehicle => (
            profile.spec.pools.vehicle_pool,
            profile.spec.pools.vehicle_slot_size,
            profile.spec.vehicle.size,
            profile.spec.pools.get_vehicle,
        ),
        PoolKind::Object => (
            profile.spec.pools.object_pool,
            profile.spec.pools.object_slot_size,
            profile.spec.object.size,
            profile.spec.pools.get_object,
        ),
    };
    let pool = ReadableRegion::validate(pool_root.get(), core::mem::size_of::<usize>())
        .and_then(|root| unsafe { root.read_unaligned::<usize>(0) })
        .filter(|pool| *pool != 0)
        .and_then(|pool| ReadableRegion::validate(pool, profile.spec.pool_layout.size.get()))
        .ok_or(PoolReadError::UnreadableEntity)?;
    let capacity = unsafe { pool.read_unaligned::<i32>(profile.spec.pool_layout.capacity.get()) }
        .filter(|capacity| *capacity > 0)
        .ok_or(PoolReadError::UnreadableEntity)? as usize;
    let index = ((handle as u32) >> 8) as usize;
    if index >= capacity {
        return Ok(None);
    }
    let objects = unsafe { pool.read_unaligned::<usize>(profile.spec.pool_layout.objects.get()) }
        .filter(|objects| *objects != 0)
        .ok_or(PoolReadError::UnreadableEntity)?;
    let flags = unsafe { pool.read_unaligned::<usize>(profile.spec.pool_layout.flags.get()) }
        .filter(|flags| *flags != 0)
        .ok_or(PoolReadError::UnreadableEntity)?;
    let expected = objects
        .checked_add(
            index
                .checked_mul(slot_size.get())
                .ok_or(PoolReadError::UnreadableEntity)?,
        )
        .ok_or(PoolReadError::UnreadableEntity)?;
    ReadableRegion::validate(
        flags
            .checked_add(index)
            .ok_or(PoolReadError::UnreadableEntity)?,
        1,
    )
    .ok_or(PoolReadError::UnreadableEntity)?;
    ReadableRegion::validate(expected, object_size.get()).ok_or(PoolReadError::UnreadableEntity)?;

    let target = NativeCallTarget::resolve(getter).map_err(|_| PoolReadError::NativeCall)?;
    let pointer = unsafe { target.call_cdecl_i32_to_ptr(handle) };
    if pointer.is_null() {
        return Ok(None);
    }
    if pointer as usize != expected {
        return Err(PoolReadError::UnreadableEntity);
    }
    Ok(Some(pointer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpool_ref_abi_variants_are_distinct() {
        assert_ne!(CpoolRefAbi::R1, CpoolRefAbi::Classic);
    }

    #[test]
    fn cpool_ref_rejects_an_unreadable_target() {
        let result = unsafe {
            cpool_ref(
                AbsoluteAddress::new(usize::MAX),
                CpoolRefAbi::R1,
                std::ptr::null_mut(),
            )
        };
        assert_eq!(result, Err(CpoolRefError));
    }

    #[test]
    fn pool_lookup_rejects_nonpositive_handles_before_native_access() {
        let profile = GtaProfile::select(0x0040_0000, crate::GTA_SA_10_US_SHA256).unwrap();
        assert_eq!(
            unsafe { pool_pointer(profile, PoolKind::Vehicle, 0) },
            Err(PoolReadError::InvalidHandle)
        );
        assert_eq!(
            unsafe { pool_pointer(profile, PoolKind::Object, -1) },
            Err(PoolReadError::InvalidHandle)
        );
    }
}
