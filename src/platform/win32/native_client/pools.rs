//! Guarded player and vehicle pool-root resolution.

use super::{
    memory::{read_i32_bool, read_pointer, read_unaligned, readable_range},
    profile::{FieldOffset, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{DirectClientError, GangzoneSnapshot};
use gta_sa_native::{CpoolRefAbi, GtaProfile, cpool_ref};
use std::{ffi::c_void, mem};

type R1PoolGetterFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type ClassicPoolGetterFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type R1VehicleExistsFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type ClassicVehicleExistsFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;

impl NativeClientProfile {
    pub(crate) fn player_pool(self) -> Result<*mut c_void, DirectClientError> {
        self.pool_root(self.spec.net_game.get_player_pool_rva.get())
    }

    pub(crate) fn vehicle_pool(self) -> Result<*mut c_void, DirectClientError> {
        self.pool_root(self.spec.net_game.get_vehicle_pool_rva.get())
    }

    /// Reads one guarded vehicle occupancy value through the selected native API.
    pub(crate) fn vehicle_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.vehicles.get() {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.vehicle_pool()?;
        let required = self
            .spec
            .pools
            .vehicle
            .not_empty_offset
            .get()
            .checked_add(
                (usize::from(id) + 1)
                    .checked_mul(mem::size_of::<i32>())
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(pool.cast(), required) {
            return Err(DirectClientError::NotReady);
        }
        let target = self
            .module_base
            .checked_add(self.spec.pools.vehicle.does_exist_rva.get())
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)?;
        let exists = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let function: R1VehicleExistsFn = mem::transmute(target);
                    function(pool, id)
                }
                PoolGetterAbi::Classic => {
                    let function: ClassicVehicleExistsFn = mem::transmute(target);
                    function(pool, id)
                }
            }
        };
        match exists {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Reads one guarded object-pool occupancy flag.
    pub(crate) fn object_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.objects.get() {
            return Err(DirectClientError::NotReady);
        }
        let offset = self.spec.pools.object.not_empty_offset.get();
        let pool = self.entity_pool(self.spec.net_game.pools.object_offset, offset)?;
        let address = (pool as usize)
            .checked_add(offset)
            .and_then(|address| address.checked_add(usize::from(id) * mem::size_of::<i32>()))
            .ok_or(DirectClientError::NotReady)?;
        read_i32_bool(address)
    }

    /// Copies one object wrapper's GTA handle without exposing its native pointer.
    pub(crate) fn object_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if !self.object_exists(id)? {
            return Ok(None);
        }
        let layout = self.spec.pools.object;
        let required = layout
            .objects_offset
            .get()
            .checked_add(
                (usize::from(id) + 1)
                    .checked_mul(mem::size_of::<usize>())
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)?;
        let pool = self.entity_pool(self.spec.net_game.pools.object_offset, required)?;
        let object = unsafe {
            read_pointer(
                (pool as usize)
                    .checked_add(layout.objects_offset.get())
                    .and_then(|address| {
                        address.checked_add(usize::from(id) * mem::size_of::<usize>())
                    })
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let handle_offset = self.spec.pools.entity_handle_offset.get();
        let handle = unsafe {
            read_unaligned::<i32>(
                (object as usize)
                    .checked_add(handle_offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok((handle != 0).then_some(handle))
    }

    /// Finds an object ID by its copied GTA handle.
    pub(crate) fn object_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        for id in 0..self.spec.pools.limits.objects.get() {
            let id = u16::try_from(id).map_err(|_| DirectClientError::NotReady)?;
            if self.object_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one pickup GTA handle from the selected pickup pool.
    pub(crate) fn pickup_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.pickups.get() {
            return Err(DirectClientError::NotReady);
        }
        let offset = self.spec.pools.pickup.handles_offset.get();
        let required = offset
            .checked_add(
                (usize::from(id) + 1)
                    .checked_mul(mem::size_of::<i32>())
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)?;
        let pool = self.entity_pool(self.spec.net_game.pools.pickup_offset, required)?;
        let handle = unsafe {
            read_unaligned::<i32>(
                (pool as usize)
                    .checked_add(offset)
                    .and_then(|address| {
                        address.checked_add(usize::from(id) * mem::size_of::<i32>())
                    })
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok((handle != 0).then_some(handle))
    }

    /// Finds a pickup ID by its copied GTA handle.
    pub(crate) fn pickup_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        for id in 0..self.spec.pools.limits.pickups.get() {
            let id = u16::try_from(id).map_err(|_| DirectClientError::NotReady)?;
            if self.pickup_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Converts one guarded vehicle game-object pointer to its GTA handle.
    pub(crate) fn vehicle_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if !self.vehicle_exists(id)? {
            return Ok(None);
        }
        let layout = self.spec.pools.vehicle;
        let required = layout
            .game_objects_offset
            .get()
            .checked_add(
                (usize::from(id) + 1)
                    .checked_mul(mem::size_of::<usize>())
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)?;
        let pool = self.vehicle_pool()?;
        if !readable_range(pool.cast(), required) {
            return Err(DirectClientError::NotReady);
        }
        let game_object = unsafe {
            read_pointer(
                (pool as usize)
                    .checked_add(layout.game_objects_offset.get())
                    .and_then(|address| {
                        address.checked_add(usize::from(id) * mem::size_of::<usize>())
                    })
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, 1))
        .ok_or(DirectClientError::NotReady)?;
        let abi = match self.spec.strategies.pool_getter_abi {
            PoolGetterAbi::R1 => CpoolRefAbi::R1,
            PoolGetterAbi::Classic => CpoolRefAbi::Classic,
        };
        let handle = unsafe {
            cpool_ref(
                GtaProfile::gta_sa_10_us().spec.pools.get_vehicle_ref,
                abi,
                game_object.cast(),
            )
        }
        .map_err(|_| DirectClientError::NotReady)?;
        Ok(handle)
    }

    /// Finds a vehicle ID by its GTA handle.
    pub(crate) fn vehicle_id_by_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        for id in 0..self.spec.pools.limits.vehicles.get() {
            let id = u16::try_from(id).map_err(|_| DirectClientError::NotReady)?;
            if self.vehicle_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one guarded gangzone record from the selected pool.
    pub(crate) fn gangzone(self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.gangzones.get() {
            return Err(DirectClientError::NotReady);
        }
        let layout = self.spec.pools.gangzone;
        let pool = self.entity_pool(
            self.spec.net_game.pools.gangzone_offset,
            layout.not_empty_offset.get(),
        )?;
        let occupied = (pool as usize)
            .checked_add(layout.not_empty_offset.get())
            .and_then(|address| address.checked_add(usize::from(id) * mem::size_of::<i32>()))
            .ok_or(DirectClientError::NotReady)?;
        if !read_i32_bool(occupied)? {
            return Ok(None);
        }
        let gangzone = unsafe {
            read_pointer(
                (pool as usize)
                    .checked_add(usize::from(id) * mem::size_of::<usize>())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let required = layout
            .alternate_colour_offset
            .get()
            .checked_add(mem::size_of::<u32>())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(gangzone, required) {
            return Err(DirectClientError::NotReady);
        }
        let scalar = |offset| unsafe {
            read_unaligned::<f32>(
                (gangzone as usize)
                    .checked_add(offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)
        };
        let colour = |offset| unsafe {
            read_unaligned::<u32>(
                (gangzone as usize)
                    .checked_add(offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
            .ok_or(DirectClientError::NotReady)
        };
        Ok(Some(GangzoneSnapshot {
            id,
            left: scalar(layout.left_offset.get())?,
            bottom: scalar(layout.bottom_offset.get())?,
            right: scalar(layout.right_offset.get())?,
            top: scalar(layout.top_offset.get())?,
            colour: colour(layout.colour_offset.get())?,
            alternate_colour: colour(layout.alternate_colour_offset.get())?,
        }))
    }

    fn pool_root(self, getter_rva: usize) -> Result<*mut c_void, DirectClientError> {
        let minimum_net_game = self
            .spec
            .net_game
            .pools_offset
            .get()
            .checked_add(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        let net_game = self
            .net_game_with_range(minimum_net_game)
            .ok_or(DirectClientError::NotReady)?;
        let _pools = unsafe {
            read_pointer(
                (net_game as usize)
                    .checked_add(self.spec.net_game.pools_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, 1))
        .ok_or(DirectClientError::NotReady)?;
        let target = self
            .module_base
            .checked_add(getter_rva)
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)?;
        let pool = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let getter: R1PoolGetterFn = mem::transmute(target);
                    getter(net_game)
                }
                PoolGetterAbi::Classic => {
                    let getter: ClassicPoolGetterFn = mem::transmute(target);
                    getter(net_game)
                }
            }
        };
        (!pool.is_null() && readable_range(pool.cast(), 1))
            .then_some(pool)
            .ok_or(DirectClientError::NotReady)
    }

    fn entity_pool(
        self,
        child_offset: FieldOffset,
        pool_required_size: usize,
    ) -> Result<*mut c_void, DirectClientError> {
        let pools_offset = self.spec.net_game.pools_offset.get();
        let net_game_required = pools_offset
            .checked_add(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        let net_game = self
            .net_game_with_range(net_game_required)
            .ok_or(DirectClientError::NotReady)?;
        let pools = unsafe {
            read_pointer(
                (net_game as usize)
                    .checked_add(pools_offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null())
        .ok_or(DirectClientError::NotReady)?;
        let child_required = child_offset
            .get()
            .checked_add(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(pools, child_required) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe {
            read_pointer(
                (pools as usize)
                    .checked_add(child_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, pool_required_size))
        .ok_or(DirectClientError::NotReady)?;
        Ok(pool.cast())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;
    use std::ptr;

    #[test]
    fn entity_operations_reject_every_profile_limit() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.vehicle_exists(2000),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(
                profile.object_exists(profile.spec.pools.limits.objects.get() as u16),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(
                profile.pickup_handle(4096),
                Err(DirectClientError::NotReady)
            );
            assert_eq!(profile.gangzone(1024), Err(DirectClientError::NotReady));
        }
    }

    #[test]
    fn dl_keeps_its_extended_object_limit() {
        let profile =
            NativeClientProfile::select(0x10000, SampVersion::Dl, SampVersion::Dl.entry_point())
                .expect("the DL identity must select");
        assert_eq!(profile.spec.pools.limits.objects.get(), 2100);
        assert_eq!(
            profile.object_exists(2100),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn pool_getters_do_not_run_before_r3_initializes_its_pool_root() {
        let bootstrap = NativeClientProfile::select(
            0x10000,
            SampVersion::R3_1,
            SampVersion::R3_1.entry_point(),
        )
        .expect("the R3 identity must select");
        let mut module =
            vec![0_u8; bootstrap.spec.net_game.singleton_rva.get() + mem::size_of::<usize>()];
        let mut net_game =
            vec![0_u8; bootstrap.spec.net_game.pools_offset.get() + mem::size_of::<usize>()];
        unsafe {
            ptr::write_unaligned(
                module
                    .as_mut_ptr()
                    .add(bootstrap.spec.net_game.singleton_rva.get())
                    .cast::<usize>(),
                net_game.as_mut_ptr() as usize,
            );
        }
        let profile = NativeClientProfile::select(
            module.as_ptr() as usize,
            SampVersion::R3_1,
            SampVersion::R3_1.entry_point(),
        )
        .expect("the R3 identity must select");

        assert_eq!(profile.player_pool(), Err(DirectClientError::NotReady));
        assert_eq!(profile.vehicle_pool(), Err(DirectClientError::NotReady));
    }
}
