//! Verified local-ped resolution and copied snapshot reads.

use crate::{
    CpoolRefAbi, GtaProfile, NativeCallTarget, RawVector3, cpool_ref,
    layout::{EntityReadError, read_entity_position},
    profile::{EntityLayoutSpec, PedLayoutSpec},
};
use gta_sa::{EntitySnapshot, PedHandle, PedSnapshot, Vector3};
use modkit_win32::ReadableRegion;
use std::ffi::c_void;

/// Failure to resolve or copy the verified local-ped slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PedReadError {
    NativeCall,
    UnreadablePed,
    UnreadableMatrix,
    InvalidHandle,
    InvalidPosition,
    InvalidVtable,
}

/// Resolves the current GTA local player ped and converts it to a live handle.
///
/// Returns `Ok(None)` when GTA has no local ped.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope compatible with
/// `GAME_THREAD_ANY_PHASE`. The selected profile must match the loaded image.
pub unsafe fn local_ped_handle(profile: GtaProfile) -> Result<Option<PedHandle>, PedReadError> {
    let ped = unsafe { find_local_ped(profile)? };
    let Some(ped) = ped else {
        return Ok(None);
    };
    ReadableRegion::validate(ped as usize, profile.spec.ped.size.get())
        .ok_or(PedReadError::UnreadablePed)?;
    let raw = unsafe { cpool_ref(profile.spec.pools.get_ped_ref, CpoolRefAbi::Classic, ped) }
        .map_err(|_| PedReadError::NativeCall)?;
    raw.map(|value| PedHandle::new(value).ok_or(PedReadError::InvalidHandle))
        .transpose()
}

/// Copies the verified position, health, and armour fields for the local ped.
///
/// Returns `Ok(None)` when GTA has no local ped or its pool slot is absent.
///
/// # Safety
///
/// The caller must hold a runtime-validated game-thread scope compatible with
/// `GAME_THREAD_ANY_PHASE`. The selected profile must match the loaded image.
pub unsafe fn local_ped_snapshot(profile: GtaProfile) -> Result<Option<PedSnapshot>, PedReadError> {
    let ped = unsafe { find_local_ped(profile)? };
    let Some(ped) = ped else {
        return Ok(None);
    };
    let region = ReadableRegion::validate(ped as usize, profile.spec.ped.size.get())
        .ok_or(PedReadError::UnreadablePed)?;
    let raw_handle =
        unsafe { cpool_ref(profile.spec.pools.get_ped_ref, CpoolRefAbi::Classic, ped) }
            .map_err(|_| PedReadError::NativeCall)?;
    let Some(raw_handle) = raw_handle else {
        return Ok(None);
    };
    let handle = PedHandle::new(raw_handle).ok_or(PedReadError::InvalidHandle)?;
    let (position, health, armour) =
        read_snapshot_values(&region, profile.spec.entity, profile.spec.ped)?;
    Ok(Some(PedSnapshot {
        handle,
        entity: EntitySnapshot { position },
        health,
        armour,
    }))
}

/// Relocates the current local ped through the verified virtual `CPed::Teleport`.
///
/// The exact implementation removes and re-adds the ped to `CWorld`, clears
/// velocity, and updates either the attached matrix or embedded placement.
///
/// # Safety
///
/// The caller must hold a runtime-validated `POST_GAME_PROCESS_ONLY` scope.
/// The selected profile must match the loaded image.
pub unsafe fn teleport_local_ped(
    profile: GtaProfile,
    destination: Vector3,
) -> Result<(), PedReadError> {
    if !destination.x.is_finite() || !destination.y.is_finite() || !destination.z.is_finite() {
        return Err(PedReadError::InvalidPosition);
    }
    let ped = unsafe { find_local_ped(profile)? }.ok_or(PedReadError::InvalidHandle)?;
    let region = ReadableRegion::validate(ped as usize, profile.spec.ped.size.get())
        .ok_or(PedReadError::UnreadablePed)?;
    let vtable = unsafe { region.read_unaligned::<usize>(0) }.ok_or(PedReadError::UnreadablePed)?;
    if vtable != profile.spec.ped_vtable.player_ped.get()
        && vtable != profile.spec.ped_vtable.ped.get()
    {
        return Err(PedReadError::InvalidVtable);
    }
    let target =
        unsafe { NativeCallTarget::from_vtable(ped, profile.spec.ped_vtable.teleport_slot) }
            .map_err(|_| PedReadError::InvalidVtable)?;
    if target.address() != profile.spec.ped_vtable.teleport_target {
        return Err(PedReadError::InvalidVtable);
    }
    unsafe {
        target.call_thiscall_vector3_bool(
            ped,
            RawVector3 {
                x: destination.x,
                y: destination.y,
                z: destination.z,
            },
            0,
        );
    }
    Ok(())
}

unsafe fn find_local_ped(profile: GtaProfile) -> Result<Option<*mut c_void>, PedReadError> {
    let target = NativeCallTarget::resolve(profile.spec.player.find_player_ped)
        .map_err(|_| PedReadError::NativeCall)?;
    let ped = unsafe { target.call_cdecl_i32_to_ptr(-1) };
    Ok((!ped.is_null()).then_some(ped))
}

fn read_snapshot_values(
    ped: &ReadableRegion,
    entity: EntityLayoutSpec,
    ped_layout: PedLayoutSpec,
) -> Result<(Vector3, f32, f32), PedReadError> {
    let position = read_entity_position(ped, entity).map_err(|error| match error {
        EntityReadError::UnreadableEntity => PedReadError::UnreadablePed,
        EntityReadError::UnreadableMatrix => PedReadError::UnreadableMatrix,
    })?;
    let health = unsafe { ped.read_unaligned::<f32>(ped_layout.health.get()) }
        .ok_or(PedReadError::UnreadablePed)?;
    let armour = unsafe { ped.read_unaligned::<f32>(ped_layout.armour.get()) }
        .ok_or(PedReadError::UnreadablePed)?;
    Ok((position, health, armour))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GTA_SA_10_US_SHA256, RawMatrix, profile::GtaProfile};

    #[test]
    fn snapshot_reads_simple_transform_when_matrix_is_absent() {
        let profile = GtaProfile::select(0x0040_0000, GTA_SA_10_US_SHA256).unwrap();
        let mut ped = vec![0_u8; profile.spec.ped.size.get()];
        unsafe {
            (ped.as_mut_ptr()
                .add(profile.spec.entity.placeable_position.get()) as *mut RawVector3)
                .write_unaligned(RawVector3 {
                    x: 10.0,
                    y: -20.0,
                    z: 3.5,
                });
            (ped.as_mut_ptr().add(profile.spec.ped.health.get()) as *mut f32).write_unaligned(87.5);
            (ped.as_mut_ptr().add(profile.spec.ped.armour.get()) as *mut f32).write_unaligned(42.0);
        }
        let region = ReadableRegion::validate(ped.as_ptr() as usize, ped.len()).unwrap();
        assert_eq!(
            read_snapshot_values(&region, profile.spec.entity, profile.spec.ped),
            Ok((Vector3::new(10.0, -20.0, 3.5), 87.5, 42.0))
        );
    }

    #[test]
    fn snapshot_reads_attached_matrix_position() {
        let profile = GtaProfile::select(0x0040_0000, GTA_SA_10_US_SHA256).unwrap();
        let mut ped = vec![0_u8; profile.spec.ped.size.get()];
        let mut matrix = RawMatrix {
            position: RawVector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            ..RawMatrix::default()
        };
        unsafe {
            (ped.as_mut_ptr()
                .add(profile.spec.entity.matrix_pointer.get()) as *mut usize)
                .write_unaligned((&mut matrix as *mut RawMatrix) as usize);
        }
        let region = ReadableRegion::validate(ped.as_ptr() as usize, ped.len()).unwrap();
        let (position, _, _) =
            read_snapshot_values(&region, profile.spec.entity, profile.spec.ped).unwrap();
        assert_eq!(position, Vector3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn teleport_rejects_non_finite_coordinates_before_native_access() {
        let profile = GtaProfile::select(0x0040_0000, GTA_SA_10_US_SHA256).unwrap();
        assert_eq!(
            unsafe { teleport_local_ped(profile, Vector3::new(f32::NAN, 0.0, 0.0)) },
            Err(PedReadError::InvalidPosition)
        );
    }
}
