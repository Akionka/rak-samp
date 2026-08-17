//! Guarded text-label operations shared by immutable client profiles.

use super::{
    colours::{ArgbColour, NativeRgbaColour},
    memory::{
        bounded_c_string, read_i32_bool, read_pointer, read_u8_bool, read_unaligned, readable_range,
    },
    profile::{NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{DirectClientError, TextLabelSnapshot, Vector3};
use std::{ffi::c_void, mem};

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeVector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vector3> for NativeVector3 {
    fn from(value: Vector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

type R1LabelPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, u16, *const u8, u32, NativeVector3, f32, u8, u16, u16);
type ClassicLabelPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, u16, *const u8, u32, NativeVector3, f32, u8, u16, u16);
type R1LabelPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type ClassicLabelPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;

impl NativeClientProfile {
    fn label_pool(self) -> Result<*mut u8, DirectClientError> {
        let required = self
            .spec
            .net_game
            .pools_offset
            .get()
            .checked_add(self.spec.net_game.pools.text_label_offset.get())
            .and_then(|offset| offset.checked_add(mem::size_of::<usize>()))
            .ok_or(DirectClientError::NotReady)?;
        let net_game = self
            .net_game_with_range(required)
            .ok_or(DirectClientError::NotReady)?;
        let pools = unsafe {
            read_pointer(
                (net_game as usize)
                    .checked_add(self.spec.net_game.pools_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let pool = unsafe {
            read_pointer(
                (pools as usize)
                    .checked_add(self.spec.net_game.pools.text_label_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, 1))
        .ok_or(DirectClientError::NotReady)?;
        Ok(pool)
    }

    fn label_flag_address(self, pool: *mut u8, id: u16) -> Result<usize, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.text_labels.get() {
            return Err(DirectClientError::NotReady);
        }
        (pool as usize)
            .checked_add(self.spec.pools.text_label.not_empty_offset.get())
            .and_then(|address| address.checked_add(usize::from(id) * mem::size_of::<i32>()))
            .ok_or(DirectClientError::NotReady)
    }

    fn label_target(self, rva: super::profile::NativeRva) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn text_label_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.text_labels.get() {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.label_pool()?;
        read_i32_bool(self.label_flag_address(pool, id)?)
    }

    pub(crate) fn first_free_text_label_id(self) -> Result<u16, DirectClientError> {
        let pool = self.label_pool()?;
        for id in 0..self.spec.pools.limits.text_labels.get() {
            let id = u16::try_from(id).map_err(|_| DirectClientError::NotReady)?;
            if !self.text_label_exists_from_pool(pool, id)? {
                return Ok(id);
            }
        }
        Err(DirectClientError::NotReady)
    }

    fn text_label_exists_from_pool(
        self,
        pool: *mut u8,
        id: u16,
    ) -> Result<bool, DirectClientError> {
        read_i32_bool(self.label_flag_address(pool, id)?)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_text_label(
        self,
        id: u16,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<(), DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.text_labels.get()
            || text.len() > self.spec.text_labels.text_capacity.get()
            || text.contains(&0)
            || !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !draw_distance.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.label_pool()?;
        let target = self.label_target(self.spec.text_labels.create_rva)?;
        let mut text = text.to_vec();
        text.push(0);
        let native_colour: NativeRgbaColour = ArgbColour::new(colour).into();
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let create: R1LabelPoolCreateFn = mem::transmute(target);
                    create(
                        pool.cast(),
                        id,
                        text.as_ptr(),
                        native_colour.get(),
                        position.into(),
                        draw_distance,
                        u8::from(behind_walls),
                        attached_player_id,
                        attached_vehicle_id,
                    );
                }
                PoolGetterAbi::Classic => {
                    let create: ClassicLabelPoolCreateFn = mem::transmute(target);
                    create(
                        pool.cast(),
                        id,
                        text.as_ptr(),
                        native_colour.get(),
                        position.into(),
                        draw_distance,
                        u8::from(behind_walls),
                        attached_player_id,
                        attached_vehicle_id,
                    );
                }
            }
        }
        Ok(())
    }

    pub(crate) fn delete_text_label(self, id: u16) -> Result<(), DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.text_labels.get() {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.label_pool()?;
        let target = self.label_target(self.spec.text_labels.delete_rva)?;
        let result = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let delete: R1LabelPoolDeleteFn = mem::transmute(target);
                    delete(pool.cast(), id)
                }
                PoolGetterAbi::Classic => {
                    let delete: ClassicLabelPoolDeleteFn = mem::transmute(target);
                    delete(pool.cast(), id)
                }
            }
        };
        (result != 0)
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn text_label(
        self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        let pool = self.label_pool()?;
        if !self.text_label_exists_from_pool(pool, id)? {
            return Ok(None);
        }
        let layout = self.spec.text_labels;
        let label = (pool as usize)
            .checked_add(
                usize::from(id)
                    .checked_mul(layout.size.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(label as *const u8, layout.size.get()) {
            return Err(DirectClientError::NotReady);
        }
        let text = unsafe {
            read_pointer(
                label
                    .checked_add(layout.text_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .and_then(|pointer| unsafe {
            bounded_c_string(pointer, layout.text_capacity.get().saturating_add(1))
        })
        .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe {
            read_unaligned::<u32>(
                label
                    .checked_add(layout.colour_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .map(NativeRgbaColour::new)
        .map(ArgbColour::from)
        .map(ArgbColour::get)
        .ok_or(DirectClientError::NotReady)?;
        let position = Vector3 {
            x: unsafe {
                read_unaligned(
                    label
                        .checked_add(layout.position_offset.get())
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
            .filter(|value: &f32| value.is_finite())
            .ok_or(DirectClientError::NotReady)?,
            y: unsafe {
                read_unaligned(
                    label
                        .checked_add(layout.position_offset.get() + 4)
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
            .filter(|value: &f32| value.is_finite())
            .ok_or(DirectClientError::NotReady)?,
            z: unsafe {
                read_unaligned(
                    label
                        .checked_add(layout.position_offset.get() + 8)
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
            .filter(|value: &f32| value.is_finite())
            .ok_or(DirectClientError::NotReady)?,
        };
        let draw_distance = unsafe {
            read_unaligned::<f32>(
                label
                    .checked_add(layout.draw_distance_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)?;
        let behind_walls = read_u8_bool(
            label
                .checked_add(layout.behind_walls_offset.get())
                .ok_or(DirectClientError::NotReady)?,
        )?;
        let attached_player = unsafe {
            read_unaligned::<u16>(
                label
                    .checked_add(layout.attached_player_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let attached_vehicle = unsafe {
            read_unaligned::<u16>(
                label
                    .checked_add(layout.attached_vehicle_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextLabelSnapshot {
            id,
            text,
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id: (attached_player != u16::MAX).then_some(attached_player),
            attached_vehicle_id: (attached_vehicle != u16::MAX).then_some(attached_vehicle),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    #[test]
    fn label_id_bounds_are_identical_for_every_profile() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.text_label_exists(u16::MAX),
                Err(DirectClientError::NotReady)
            );
        }
    }
}
