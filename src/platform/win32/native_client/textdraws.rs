//! Guarded textdraw operations shared by immutable client profiles.

use super::{
    memory::{
        bounded_c_string, read_i32_bool, read_pointer, read_u8_bool, read_unaligned,
        readable_range, writable_range,
    },
    profile::{NativeClientProfile, NativeRva, PoolGetterAbi},
};
use crate::runtime::{DirectClientError, TextdrawSnapshot, Vector3};
use std::{ffi::c_void, mem, ptr};

type R1TextdrawPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *mut c_void, *const u8) -> *mut c_void;
type ClassicTextdrawPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *mut c_void, *const u8) -> *mut c_void;
type R1TextdrawPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type ClassicTextdrawPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type R1TextdrawSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const u8);
type ClassicTextdrawSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const u8);

impl NativeClientProfile {
    fn textdraw_pool(self) -> Result<*mut u8, DirectClientError> {
        let required = self
            .spec
            .net_game
            .pools_offset
            .get()
            .checked_add(self.spec.net_game.pools.textdraw_offset.get())
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
        unsafe {
            read_pointer(
                (pools as usize)
                    .checked_add(self.spec.net_game.pools.textdraw_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, 1))
        .ok_or(DirectClientError::NotReady)
    }

    fn textdraw_flag_address(self, pool: *mut u8, id: u16) -> Result<usize, DirectClientError> {
        self.textdraw_id(id)?;
        (pool as usize)
            .checked_add(self.spec.pools.textdraw.not_empty_offset.get())
            .and_then(|address| address.checked_add(usize::from(id) * mem::size_of::<i32>()))
            .ok_or(DirectClientError::NotReady)
    }

    fn textdraw_slot_address(self, pool: *mut u8, id: u16) -> Result<usize, DirectClientError> {
        self.textdraw_id(id)?;
        (pool as usize)
            .checked_add(self.spec.pools.textdraw.objects_offset.get())
            .and_then(|address| address.checked_add(usize::from(id) * mem::size_of::<usize>()))
            .ok_or(DirectClientError::NotReady)
    }

    fn textdraw_id(self, id: u16) -> Result<(), DirectClientError> {
        (usize::from(id) < self.spec.pools.limits.textdraws.get())
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    fn textdraw_exists_from_pool(self, pool: *mut u8, id: u16) -> Result<bool, DirectClientError> {
        read_i32_bool(self.textdraw_flag_address(pool, id)?)
    }

    fn textdraw_object(self, id: u16) -> Result<usize, DirectClientError> {
        let pool = self.textdraw_pool()?;
        if !self.textdraw_exists_from_pool(pool, id)? {
            return Err(DirectClientError::NotReady);
        }
        let object = unsafe { read_pointer(self.textdraw_slot_address(pool, id)?) }
            .filter(|pointer| !pointer.is_null())
            .ok_or(DirectClientError::NotReady)?;
        readable_range(object, self.spec.textdraws.native_size.get())
            .then_some(object as usize)
            .ok_or(DirectClientError::NotReady)
    }

    fn textdraw_target(self, rva: NativeRva) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }

    fn textdraw_data_address(
        self,
        object: usize,
        offset: usize,
    ) -> Result<usize, DirectClientError> {
        object
            .checked_add(self.spec.textdraws.data_offset.get())
            .and_then(|address| address.checked_add(offset))
            .ok_or(DirectClientError::NotReady)
    }

    fn write_textdraw<T: Copy>(
        self,
        id: u16,
        offset: usize,
        value: T,
    ) -> Result<(), DirectClientError> {
        let address = self.textdraw_data_address(self.textdraw_object(id)?, offset)?;
        if !writable_range(address as *const u8, mem::size_of::<T>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(address as *mut T, value) };
        Ok(())
    }

    pub(crate) fn textdraw_exists(self, id: u16) -> Result<bool, DirectClientError> {
        self.textdraw_exists_from_pool(self.textdraw_pool()?, id)
    }

    pub(crate) fn delete_textdraw(self, id: u16) -> Result<(), DirectClientError> {
        let pool = self.textdraw_pool()?;
        self.textdraw_id(id)?;
        let target = self.textdraw_target(self.spec.textdraws.delete_rva)?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let delete: R1TextdrawPoolDeleteFn = mem::transmute(target);
                    delete(pool.cast(), id);
                }
                PoolGetterAbi::Classic => {
                    let delete: ClassicTextdrawPoolDeleteFn = mem::transmute(target);
                    delete(pool.cast(), id);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn create_textdraw(
        self,
        id: u16,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if text.len() > self.spec.textdraws.create_text_capacity.get()
            || text.contains(&0)
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.textdraw_pool()?;
        if self.textdraw_exists_from_pool(pool, id)?
            || unsafe { read_pointer(self.textdraw_slot_address(pool, id)?) }
                .filter(|pointer| !pointer.is_null())
                .is_some()
        {
            return Err(DirectClientError::NotReady);
        }
        let mut transmit = vec![0_u8; self.spec.textdraws.transmit.size.get()];
        let x_end = self.spec.textdraws.transmit.x.get() + mem::size_of::<f32>();
        let y_end = self.spec.textdraws.transmit.y.get() + mem::size_of::<f32>();
        if x_end > transmit.len() || y_end > transmit.len() {
            return Err(DirectClientError::NotReady);
        }
        transmit[self.spec.textdraws.transmit.x.get()..x_end].copy_from_slice(&x.to_le_bytes());
        transmit[self.spec.textdraws.transmit.y.get()..y_end].copy_from_slice(&y.to_le_bytes());
        let mut native_text = text.to_vec();
        native_text.push(0);
        let target = self.textdraw_target(self.spec.textdraws.create_rva)?;
        let created = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let create: R1TextdrawPoolCreateFn = mem::transmute(target);
                    create(
                        pool.cast(),
                        i32::from(id),
                        transmit.as_mut_ptr().cast(),
                        native_text.as_ptr(),
                    )
                }
                PoolGetterAbi::Classic => {
                    let create: ClassicTextdrawPoolCreateFn = mem::transmute(target);
                    create(
                        pool.cast(),
                        i32::from(id),
                        transmit.as_mut_ptr().cast(),
                        native_text.as_ptr(),
                    )
                }
            }
        };
        (!created.is_null())
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    pub(crate) fn set_textdraw_position(
        self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.x.get(), x)?;
        self.write_textdraw(id, data.y.get(), y)
    }

    pub(crate) fn set_textdraw_style(self, id: u16, style: i32) -> Result<(), DirectClientError> {
        if !(0..=5).contains(&style) {
            return Err(DirectClientError::NotReady);
        }
        self.write_textdraw(id, self.spec.textdraws.data.style.get(), style)
    }

    pub(crate) fn set_textdraw_letter_style(
        self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.width.get(), width)?;
        self.write_textdraw(id, data.height.get(), height)?;
        self.write_textdraw(id, data.colour.get(), colour)
    }

    pub(crate) fn set_textdraw_proportional(
        self,
        id: u16,
        proportional: bool,
    ) -> Result<(), DirectClientError> {
        self.write_textdraw(
            id,
            self.spec.textdraws.data.proportional.get(),
            u8::from(proportional),
        )
    }

    pub(crate) fn set_textdraw_shadow(
        self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.background_colour.get(), colour)?;
        self.write_textdraw(id, data.shadow.get(), shadow)
    }

    pub(crate) fn set_textdraw_outline(
        self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.background_colour.get(), colour)?;
        self.write_textdraw(id, data.outline.get(), outline)
    }

    pub(crate) fn set_textdraw_box(
        self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<(), DirectClientError> {
        if !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.box_enabled.get(), u8::from(enabled))?;
        self.write_textdraw(id, data.box_width.get(), width)?;
        self.write_textdraw(id, data.box_height.get(), height)?;
        self.write_textdraw(id, data.box_colour.get(), colour)
    }

    pub(crate) fn set_textdraw_alignment(
        self,
        id: u16,
        alignment: u8,
    ) -> Result<(), DirectClientError> {
        if !(1..=3).contains(&alignment) {
            return Err(DirectClientError::NotReady);
        }
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.align_center.get(), u8::from(alignment == 2))?;
        self.write_textdraw(id, data.align_left.get(), u8::from(alignment == 1))?;
        self.write_textdraw(id, data.align_right.get(), u8::from(alignment == 3))
    }

    pub(crate) fn set_textdraw_model_style(
        self,
        id: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<(), DirectClientError> {
        if !rotation.x.is_finite()
            || !rotation.y.is_finite()
            || !rotation.z.is_finite()
            || !zoom.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let data = self.spec.textdraws.data;
        self.write_textdraw(id, data.rotation.get(), rotation.x)?;
        self.write_textdraw(id, data.rotation.get() + 4, rotation.y)?;
        self.write_textdraw(id, data.rotation.get() + 8, rotation.z)?;
        self.write_textdraw(id, data.zoom.get(), zoom)?;
        self.write_textdraw(id, data.model_colour1.get(), colour1)?;
        self.write_textdraw(id, data.model_colour2.get(), colour2)
    }

    pub(crate) fn set_textdraw_string(self, id: u16, text: &[u8]) -> Result<(), DirectClientError> {
        if text.len() >= self.spec.textdraws.create_text_capacity.get() || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let object = self.textdraw_object(id)? as *mut c_void;
        let target = self.textdraw_target(self.spec.textdraws.text_setter_rva)?;
        let mut native_text = text.to_vec();
        native_text.push(0);
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let setter: R1TextdrawSetTextFn = mem::transmute(target);
                    setter(object, native_text.as_ptr());
                }
                PoolGetterAbi::Classic => {
                    let setter: ClassicTextdrawSetTextFn = mem::transmute(target);
                    setter(object, native_text.as_ptr());
                }
            }
        }
        Ok(())
    }

    pub(crate) fn textdraw(self, id: u16) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        if !self.textdraw_exists(id)? {
            return Ok(None);
        }
        let object = self.textdraw_object(id)?;
        let data = self.spec.textdraws.data;
        let field = |offset| self.textdraw_data_address(object, offset);
        let finite = |offset| {
            unsafe { read_unaligned::<f32>(field(offset)?) }
                .filter(|value| value.is_finite())
                .ok_or(DirectClientError::NotReady)
        };
        let flag = |offset| read_u8_bool(field(offset)?);
        let text = unsafe {
            bounded_c_string(
                object
                    .checked_add(self.spec.textdraws.string_offset.get())
                    .ok_or(DirectClientError::NotReady)? as *const u8,
                self.spec
                    .textdraws
                    .stored_string_capacity
                    .get()
                    .saturating_add(1),
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextdrawSnapshot {
            pool_index: id,
            text,
            letter_width: finite(data.width.get())?,
            letter_height: finite(data.height.get())?,
            letter_colour: unsafe { read_unaligned(field(data.colour.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            x: finite(data.x.get())?,
            y: finite(data.y.get())?,
            shadow: unsafe { read_unaligned(field(data.shadow.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            outline: unsafe { read_unaligned(field(data.outline.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            background_colour: unsafe { read_unaligned(field(data.background_colour.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            style: unsafe { read_unaligned(field(data.style.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            proportional: flag(data.proportional.get())?,
            align_left: flag(data.align_left.get())?,
            align_center: flag(data.align_center.get())?,
            align_right: flag(data.align_right.get())?,
            box_enabled: flag(data.box_enabled.get())?,
            box_width: finite(data.box_width.get())?,
            box_height: finite(data.box_height.get())?,
            box_colour: unsafe { read_unaligned(field(data.box_colour.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            model_id: unsafe { read_unaligned(field(data.model_id.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            rotation: Vector3 {
                x: finite(data.rotation.get())?,
                y: finite(data.rotation.get() + 4)?,
                z: finite(data.rotation.get() + 8)?,
            },
            zoom: finite(data.zoom.get())?,
            model_colour1: unsafe { read_unaligned(field(data.model_colour1.get())?) }
                .ok_or(DirectClientError::NotReady)?,
            model_colour2: unsafe { read_unaligned(field(data.model_colour2.get())?) }
                .ok_or(DirectClientError::NotReady)?,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    #[test]
    fn textdraw_id_bounds_are_identical_for_every_profile() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.textdraw_exists(u16::MAX),
                Err(DirectClientError::NotReady)
            );
        }
    }

    #[test]
    fn textdraw_specs_pin_all_profile_layouts_and_native_calls() {
        let expected = [
            (SampVersion::R1, 0x1AE20, 0x1AD00, 0xAC870),
            (SampVersion::R3_1, 0x1E1C0, 0x1E0A0, 0xB26D0),
            (SampVersion::R5_1, 0x1E910, 0x1E7F0, 0xB2F60),
            (SampVersion::Dl, 0x1E3D0, 0x1E2B0, 0xB2B60),
        ];
        for (version, create_rva, delete_rva, setter_rva) in expected {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            let textdraw = profile.spec.textdraws;
            assert_eq!(profile.spec.pools.limits.textdraws.get(), 2304);
            assert_eq!(profile.spec.pools.textdraw.not_empty_offset.get(), 0);
            assert_eq!(profile.spec.pools.textdraw.objects_offset.get(), 0x2400);
            assert_eq!(textdraw.create_rva.get(), create_rva);
            assert_eq!(textdraw.delete_rva.get(), delete_rva);
            assert_eq!(textdraw.text_setter_rva.get(), setter_rva);
            assert_eq!(textdraw.native_size.get(), 0x9D6);
            assert_eq!(textdraw.string_offset.get(), 801);
            assert_eq!(textdraw.create_text_capacity.get(), 800);
            assert_eq!(textdraw.stored_string_capacity.get(), 1601);
            assert_eq!(textdraw.transmit.size.get(), 0x3F);
            assert_eq!(textdraw.transmit.x.get(), 0x21);
            assert_eq!(textdraw.transmit.y.get(), 0x25);
        }
    }
}
