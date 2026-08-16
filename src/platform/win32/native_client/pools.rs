//! Guarded player and vehicle pool-root resolution.

use super::{
    memory::readable_range,
    profile::{NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::DirectClientError;
use std::{ffi::c_void, mem};

type R1PoolGetterFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type ClassicPoolGetterFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;

impl NativeClientProfile {
    pub(crate) fn player_pool(self) -> Result<*mut c_void, DirectClientError> {
        self.pool_root(self.spec.net_game.get_player_pool_rva.get())
    }

    pub(crate) fn vehicle_pool(self) -> Result<*mut c_void, DirectClientError> {
        self.pool_root(self.spec.net_game.get_vehicle_pool_rva.get())
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
}
