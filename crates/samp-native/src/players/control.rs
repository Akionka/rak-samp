//! Local-player and player control operations.

use super::pool::{
    ClassicPlayerPoolGetRemotePlayerFn, ClassicPlayerPoolPlayerBooleanFn,
    R1PlayerPoolGetRemotePlayerFn, R1PlayerPoolPlayerBooleanFn,
};
use super::*;

type R1LocalPlayerSpawnFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicLocalPlayerSpawnFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type R1LocalPlayerSetSpecialActionFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type ClassicLocalPlayerSetSpecialActionFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type R1PlayerPoolSetLocalPlayerNameFn = unsafe extern "thiscall" fn(*mut c_void, *const i8);
type ClassicPlayerPoolSetLocalPlayerNameFn = unsafe extern "thiscall" fn(*mut c_void, *const i8);
type R1LocalPlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type ClassicLocalPlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type R1RemotePlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type ClassicRemotePlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);

impl NativeProfile {
    /// Updates a validated synchronization send rate.
    pub fn set_send_rate(self, kind: u8, milliseconds: u32) -> Result<(), DirectClientError> {
        let rate = i32::try_from(milliseconds).map_err(|_| DirectClientError::NotReady)?;
        let rva = match kind {
            0 => self.spec.sync.send_rates.onfoot,
            1 => self.spec.sync.send_rates.incar,
            2 => self.spec.sync.send_rates.aim,
            _ => return Err(DirectClientError::NotReady),
        };
        let address = self
            .module_base
            .checked_add(rva.get())
            .ok_or(DirectClientError::NotReady)?;
        unsafe { write_unaligned(address, rate) }
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    /// Invokes the selected local-player spawn method on the game thread.
    pub fn spawn_local_player(self) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        let target = self.player_function_target(self.spec.players.local_rvas.spawn.get())?;
        let spawned = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let spawn: R1LocalPlayerSpawnFn = mem::transmute(target);
                    spawn(local)
                }
                PoolGetterAbi::Classic => {
                    let spawn: ClassicLocalPlayerSpawnFn = mem::transmute(target);
                    spawn(local)
                }
            }
        };
        (spawned != 0)
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    /// Changes the selected local-player special action on the game thread.
    pub fn set_local_player_special_action(self, action: u8) -> Result<(), DirectClientError> {
        if !matches!(action, 0..=12 | 20..=25 | 68) {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let target =
            self.player_function_target(self.spec.players.local_rvas.set_special_action.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set: R1LocalPlayerSetSpecialActionFn = mem::transmute(target);
                    set(local, action);
                }
                PoolGetterAbi::Classic => {
                    let set: ClassicLocalPlayerSetSpecialActionFn = mem::transmute(target);
                    set(local, action);
                }
            }
        }
        Ok(())
    }

    /// Changes the selected local-player name through its guarded pool method.
    pub fn set_local_player_name(self, name: &[u8]) -> Result<(), DirectClientError> {
        if name.len() > self.spec.players.local_player_name_capacity.get() || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let target =
            self.player_function_target(self.spec.players.pool_rvas.set_local_player_name.get())?;
        let mut native_name = Vec::with_capacity(name.len() + 1);
        native_name.extend_from_slice(name);
        native_name.push(0);
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set: R1PlayerPoolSetLocalPlayerNameFn = mem::transmute(target);
                    set(pool, native_name.as_ptr().cast());
                }
                PoolGetterAbi::Classic => {
                    let set: ClassicPlayerPoolSetLocalPlayerNameFn = mem::transmute(target);
                    set(pool, native_name.as_ptr().cast());
                }
            }
        }
        Ok(())
    }

    /// Changes a local or connected remote player's ARGB colour on the game thread.
    pub fn set_player_colour(self, id: u16, colour: u32) -> Result<(), DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.players.get() {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id = unsafe {
            read_unaligned::<u16>(
                (pool as usize)
                    .checked_add(self.spec.pools.player.local_id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|local_id| usize::from(*local_id) < self.spec.pools.limits.players.get())
        .ok_or(DirectClientError::NotReady)?;
        if local_id == id {
            let local = self.local_player_address()?;
            let target =
                self.player_function_target(self.spec.players.local_rvas.set_colour.get())?;
            unsafe {
                match self.spec.strategies.pool_getter_abi {
                    PoolGetterAbi::R1 => {
                        let set: R1LocalPlayerSetColourFn = mem::transmute(target);
                        set(local, colour);
                    }
                    PoolGetterAbi::Classic => {
                        let set: ClassicLocalPlayerSetColourFn = mem::transmute(target);
                        set(local, colour);
                    }
                }
            }
            return Ok(());
        }
        let connected_target =
            self.player_function_target(self.spec.players.pool_rvas.is_connected.get())?;
        let connected = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let connected: R1PlayerPoolPlayerBooleanFn = mem::transmute(connected_target);
                    connected(pool, id)
                }
                PoolGetterAbi::Classic => {
                    let connected: ClassicPlayerPoolPlayerBooleanFn =
                        mem::transmute(connected_target);
                    connected(pool, id)
                }
            }
        };
        if connected != 1 {
            return Err(DirectClientError::NotReady);
        }
        let remote_target =
            self.player_function_target(self.spec.players.pool_rvas.get_remote_player.get())?;
        let remote = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let get: R1PlayerPoolGetRemotePlayerFn = mem::transmute(remote_target);
                    get(pool, id)
                }
                PoolGetterAbi::Classic => {
                    let get: ClassicPlayerPoolGetRemotePlayerFn = mem::transmute(remote_target);
                    get(pool, id)
                }
            }
        };
        if remote.is_null() || !readable_range(remote.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let set_target =
            self.player_function_target(self.spec.players.remote_rvas.set_colour.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let set: R1RemotePlayerSetColourFn = mem::transmute(set_target);
                    set(remote, colour);
                }
                PoolGetterAbi::Classic => {
                    let set: ClassicRemotePlayerSetColourFn = mem::transmute(set_target);
                    set(remote, colour);
                }
            }
        }
        Ok(())
    }
}
