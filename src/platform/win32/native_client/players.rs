//! Guarded player-pool reads shared by immutable client profiles.

use super::{
    memory::{
        bounded_c_string, read_pointer, read_unaligned, read_vector3, readable_range,
        write_unaligned,
    },
    profile::{ForceSyncReset, LocalPlayerSource, NativeClientProfile, PoolGetterAbi},
};
use crate::runtime::{
    AimSyncSnapshot, DirectClientError, InCarSyncSnapshot, LocalPlayerSnapshot, OnFootSyncSnapshot,
    PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot, TrailerSyncSnapshot,
};
use std::{ffi::c_void, mem};

type R1PlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type ClassicPlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type R1PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type ClassicPlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type R1PlayerPoolGetNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type ClassicPlayerPoolGetNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type R1PlayerPoolGetLocalStatFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicPlayerPoolGetLocalStatFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type R1LocalPlayerGetPedFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type R1LocalPlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type ClassicLocalPlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type R1PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
type ClassicPedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
type R1PlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type ClassicPlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type R1PlayerPoolGetRemotePlayerFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
type ClassicPlayerPoolGetRemotePlayerFn =
    unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
type R1PlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type ClassicPlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type R1RemotePlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type ClassicRemotePlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type R1RemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicRemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type R1RemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicRemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
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
type R1LocalPlayerSendUnoccupiedFn = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type ClassicLocalPlayerSendUnoccupiedFn = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type R1LocalPlayerNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type ClassicLocalPlayerNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type R1LocalPlayerSendTrailerFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type ClassicLocalPlayerSendTrailerFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type R1CpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;
type ClassicCpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;

const GTA_CPOOLS_GET_PED_REF: usize = 0x54_FF60;

impl NativeClientProfile {
    /// Copies the count pair from the guarded player pool.
    pub(crate) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        let pool = self.player_pool()?;
        let target = self.player_function_target(self.spec.players.pool_rvas.get_count.get())?;
        let (including_npcs, excluding_npcs) = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let get_count: R1PlayerPoolGetCountFn = mem::transmute(target);
                    (get_count(pool, 1), get_count(pool, 0))
                }
                PoolGetterAbi::Classic => {
                    let get_count: ClassicPlayerPoolGetCountFn = mem::transmute(target);
                    (get_count(pool, 1), get_count(pool, 0))
                }
            }
        };
        validate_player_counts(
            including_npcs,
            excluding_npcs,
            self.spec.pools.limits.players.get(),
        )
    }

    /// Copies and validates the largest assigned player ID from the pool.
    pub(crate) fn player_max_id(self) -> Result<u16, DirectClientError> {
        let pool = self.player_pool()?;
        let offset = self.spec.pools.player.largest_id_offset.get();
        let id = unsafe {
            read_unaligned::<i32>(
                (pool as usize)
                    .checked_add(offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        validate_player_max_id(id, self.spec.pools.limits.players.get())
    }

    /// Resolves the selected profile's local-player object as an opaque address.
    pub(crate) fn local_player_address(self) -> Result<*mut c_void, DirectClientError> {
        match self.spec.strategies.local_player_source {
            LocalPlayerSource::PlayerPoolGetter => {
                let pool = self.player_pool()?;
                let target = self
                    .player_function_target(self.spec.players.pool_rvas.get_local_player.get())?;
                let local = unsafe {
                    match self.spec.strategies.pool_getter_abi {
                        PoolGetterAbi::R1 => {
                            let get_local: R1PlayerPoolGetLocalPlayerFn = mem::transmute(target);
                            get_local(pool)
                        }
                        PoolGetterAbi::Classic => {
                            let get_local: ClassicPlayerPoolGetLocalPlayerFn =
                                mem::transmute(target);
                            get_local(pool)
                        }
                    }
                };
                let minimum_size = self
                    .spec
                    .players
                    .local
                    .readable_size
                    .map_or_else(
                        || {
                            self.spec
                                .players
                                .local
                                .last_any_update_offset
                                .get()
                                .checked_add(mem::size_of::<u32>())
                        },
                        |size| Some(size.get()),
                    )
                    .ok_or(DirectClientError::NotReady)?;
                (!local.is_null() && readable_range(local.cast(), minimum_size))
                    .then_some(local)
                    .ok_or(DirectClientError::NotReady)
            }
            LocalPlayerSource::NetGameField => Err(DirectClientError::NotReady),
        }
    }

    /// Copies the selected profile's local-player snapshot on the game thread.
    pub(crate) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        let pool = self.player_pool()?;
        let id = unsafe {
            read_unaligned::<u16>(
                (pool as usize)
                    .checked_add(self.spec.pools.player.local_id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|id| usize::from(*id) < self.spec.pools.limits.players.get())
        .ok_or(DirectClientError::NotReady)?;
        let local = self.local_player_address()?;
        let ped = self.local_player_ped(local)?;
        let game_ped = unsafe {
            read_pointer(
                (ped as usize)
                    .checked_add(self.spec.players.local.game_ped_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|game_ped| !game_ped.is_null() && readable_range(game_ped.cast(), 1))
        .ok_or(DirectClientError::NotReady)?;
        let _ = game_ped;

        let targets = LocalPlayerTargets {
            name: self.player_function_target(self.spec.players.pool_rvas.get_name.get())?,
            score: self
                .player_function_target(self.spec.players.pool_rvas.get_local_score.get())?,
            ping: self.player_function_target(self.spec.players.pool_rvas.get_local_ping.get())?,
            colour: self
                .player_function_target(self.spec.players.local_rvas.get_colour_argb.get())?,
            health: self.player_function_target(self.spec.players.ped_rvas.get_health.get())?,
            armour: self.player_function_target(self.spec.players.ped_rvas.get_armour.get())?,
        };
        let (name_pointer, score, ping, colour, health, armour) = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let name: R1PlayerPoolGetNameFn = mem::transmute(targets.name);
                    let score: R1PlayerPoolGetLocalStatFn = mem::transmute(targets.score);
                    let ping: R1PlayerPoolGetLocalStatFn = mem::transmute(targets.ping);
                    let colour: R1LocalPlayerGetColourFn = mem::transmute(targets.colour);
                    let health: R1PedGetStatFn = mem::transmute(targets.health);
                    let armour: R1PedGetStatFn = mem::transmute(targets.armour);
                    (
                        name(pool, id),
                        score(pool),
                        ping(pool),
                        colour(local),
                        health(ped),
                        armour(ped),
                    )
                }
                PoolGetterAbi::Classic => {
                    let name: ClassicPlayerPoolGetNameFn = mem::transmute(targets.name);
                    let score: ClassicPlayerPoolGetLocalStatFn = mem::transmute(targets.score);
                    let ping: ClassicPlayerPoolGetLocalStatFn = mem::transmute(targets.ping);
                    let colour: ClassicLocalPlayerGetColourFn = mem::transmute(targets.colour);
                    let health: ClassicPedGetStatFn = mem::transmute(targets.health);
                    let armour: ClassicPedGetStatFn = mem::transmute(targets.armour);
                    (
                        name(pool, id),
                        score(pool),
                        ping(pool),
                        colour(local),
                        health(ped),
                        armour(ped),
                    )
                }
            }
        };
        let nickname = unsafe {
            bounded_c_string(
                name_pointer,
                self.spec
                    .players
                    .local_player_name_capacity
                    .get()
                    .checked_add(1)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|name| !name.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let local_address = local as usize;
        let local_layout = self.spec.players.local;
        let current_vehicle = unsafe {
            read_unaligned::<u16>(
                local_address
                    .checked_add(local_layout.current_vehicle_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let vehicle_id = (current_vehicle != u16::MAX).then_some(current_vehicle);
        let state_offset = if vehicle_id.is_some() {
            local_layout.incar_offset
        } else {
            local_layout.onfoot_offset
        };
        let state_address = local_address
            .checked_add(state_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let (position_offset, speed_offset) = if vehicle_id.is_some() {
            (
                local_layout.incar.position_offset,
                local_layout.incar.speed_offset,
            )
        } else {
            (
                local_layout.onfoot.position_offset,
                local_layout.onfoot.speed_offset,
            )
        };
        let position = unsafe {
            read_vector3(
                state_address
                    .checked_add(position_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let velocity = unsafe {
            read_vector3(
                state_address
                    .checked_add(speed_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let spawned = unsafe {
            read_unaligned::<u32>(
                local_address
                    .checked_add(local_layout.active_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?
            != 0;
        let onfoot_address = local_address
            .checked_add(local_layout.onfoot_offset.get())
            .ok_or(DirectClientError::NotReady)?;
        let special_action = unsafe {
            read_unaligned::<u8>(
                onfoot_address
                    .checked_add(local_layout.onfoot.special_action_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let animation_id = unsafe {
            read_unaligned::<u32>(
                onfoot_address
                    .checked_add(local_layout.onfoot.animation_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)? as u16;
        Ok(LocalPlayerSnapshot {
            id,
            nickname,
            colour,
            spawned,
            health,
            armour,
            position,
            velocity,
            special_action,
            animation_id,
            vehicle_id,
            score,
            ping: ping.max(0) as u32,
        })
    }

    /// Copies one remote-player directory entry on the game thread.
    pub(crate) fn player_info(
        self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        let pool = self.player_pool()?;
        let targets = self.remote_player_targets()?;
        let remote = match self.connected_remote_player(pool, id, targets) {
            Ok(Some(remote)) => remote,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error),
        };
        let defined = self.remote_player_defined(remote, targets)?;
        let is_npc = self.remote_player_is_npc(pool, id, targets)?;
        let (name_pointer, score, ping, colour, status) = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let name: R1PlayerPoolGetNameFn = mem::transmute(targets.name);
                    let score: R1PlayerPoolGetPlayerStatFn = mem::transmute(targets.score);
                    let ping: R1PlayerPoolGetPlayerStatFn = mem::transmute(targets.ping);
                    let colour: R1RemotePlayerGetColourFn = mem::transmute(targets.colour);
                    let status: R1RemotePlayerGetStatusFn = mem::transmute(targets.status);
                    (
                        name(pool, id),
                        score(pool, id),
                        ping(pool, id),
                        colour(remote),
                        status(remote),
                    )
                }
                PoolGetterAbi::Classic => {
                    let name: ClassicPlayerPoolGetNameFn = mem::transmute(targets.name);
                    let score: ClassicPlayerPoolGetPlayerStatFn = mem::transmute(targets.score);
                    let ping: ClassicPlayerPoolGetPlayerStatFn = mem::transmute(targets.ping);
                    let colour: ClassicRemotePlayerGetColourFn = mem::transmute(targets.colour);
                    let status: ClassicRemotePlayerGetStatusFn = mem::transmute(targets.status);
                    (
                        name(pool, id),
                        score(pool, id),
                        ping(pool, id),
                        colour(remote),
                        status(remote),
                    )
                }
            }
        };
        let nickname = unsafe {
            bounded_c_string(
                name_pointer,
                self.spec
                    .players
                    .local_player_name_capacity
                    .get()
                    .checked_add(1)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|name| !name.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(PlayerInfoSnapshot {
            id,
            defined,
            paused: status == 0,
            nickname,
            is_local: false,
            is_npc,
            colour,
            score,
            ping: ping.max(0) as u32,
        }))
    }

    /// Copies volatile remote-player state fields on the game thread.
    pub(crate) fn remote_player_state(
        self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        let pool = self.player_pool()?;
        let targets = self.remote_player_targets()?;
        let remote = match self.connected_remote_player(pool, id, targets) {
            Ok(Some(remote)) => remote,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error),
        };
        if !readable_range(remote.cast(), self.spec.players.remote.state_size.get()) {
            return Err(DirectClientError::NotReady);
        }
        if !self.remote_player_defined(remote, targets)? {
            return Ok(None);
        }
        let layout = self.spec.players.remote;
        let remote_address = remote as usize;
        let health = unsafe {
            read_unaligned::<f32>(
                remote_address
                    .checked_add(layout.reported_health_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)?;
        let armour = unsafe {
            read_unaligned::<f32>(
                remote_address
                    .checked_add(layout.reported_armour_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)?;
        let special_action = unsafe {
            read_unaligned::<u8>(
                remote_address
                    .checked_add(layout.special_action_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let animation_id = unsafe {
            read_unaligned::<u32>(
                remote_address
                    .checked_add(layout.animation_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)? as u16;
        Ok(Some(RemotePlayerStateSnapshot {
            id,
            health,
            armour,
            special_action,
            animation_id,
        }))
    }

    /// Determines whether a connected, defined remote player lacks a GTA ped.
    pub(crate) fn remote_player_is_streamed_out(
        self,
        id: u16,
    ) -> Result<Option<bool>, DirectClientError> {
        let pool = self.player_pool()?;
        let targets = self.remote_player_targets()?;
        let remote = match self.connected_remote_player(pool, id, targets) {
            Ok(Some(remote)) => remote,
            Ok(None) => return Ok(None),
            Err(error) => return Err(error),
        };
        if !self.remote_player_defined(remote, targets)? {
            return Ok(None);
        }
        let ped_offset = self
            .spec
            .players
            .remote
            .ped_offset
            .map_or(0, |offset| offset.get());
        let ped = unsafe {
            read_pointer(
                (remote as usize)
                    .checked_add(ped_offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        if ped.is_null() {
            return Ok(Some(true));
        }
        let game_ped_offset = self.spec.players.local.game_ped_offset.get();
        let required_size = game_ped_offset
            .checked_add(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(ped.cast(), required_size) {
            return Err(DirectClientError::NotReady);
        }
        let game_ped = unsafe {
            read_pointer(
                (ped as usize)
                    .checked_add(game_ped_offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(game_ped.is_null()))
    }

    /// Converts a guarded local or remote GTA ped pointer to its handle.
    pub(crate) fn player_ped_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
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
        .filter(|value| usize::from(*value) < self.spec.pools.limits.players.get())
        .ok_or(DirectClientError::NotReady)?;
        let game_ped = if id == local_id {
            let local = self.local_player_address()?;
            let ped = self.local_player_ped(local)?;
            unsafe {
                read_pointer(
                    (ped as usize)
                        .checked_add(self.spec.players.local.game_ped_offset.get())
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
        } else {
            let targets = self.remote_player_targets()?;
            let Some(remote) = self.connected_remote_player(pool, id, targets)? else {
                return Ok(None);
            };
            if !self.remote_player_defined(remote, targets)? {
                return Ok(None);
            }
            let ped_offset = self
                .spec
                .players
                .remote
                .ped_offset
                .map_or(0, |offset| offset.get());
            let ped = unsafe {
                read_pointer(
                    (remote as usize)
                        .checked_add(ped_offset)
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
            .filter(|pointer| !pointer.is_null())
            .ok_or(DirectClientError::NotReady)?;
            unsafe {
                read_pointer(
                    (ped as usize)
                        .checked_add(self.spec.players.local.game_ped_offset.get())
                        .ok_or(DirectClientError::NotReady)?,
                )
            }
        }
        .filter(|pointer| !pointer.is_null() && readable_range(*pointer, 1))
        .ok_or(DirectClientError::NotReady)?;
        if !readable_range(GTA_CPOOLS_GET_PED_REF as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let handle = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let function: R1CpoolRefFn = mem::transmute(GTA_CPOOLS_GET_PED_REF);
                    function(game_ped.cast())
                }
                PoolGetterAbi::Classic => {
                    let function: ClassicCpoolRefFn = mem::transmute(GTA_CPOOLS_GET_PED_REF);
                    function(game_ped.cast())
                }
            }
        };
        Ok((handle != 0).then_some(handle))
    }

    /// Finds a player ID by its GTA ped handle, checking the local player first.
    pub(crate) fn player_id_by_ped_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        let pool = self.player_pool()?;
        let local_id = unsafe {
            read_unaligned::<u16>(
                (pool as usize)
                    .checked_add(self.spec.pools.player.local_id_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .filter(|value| usize::from(*value) < self.spec.pools.limits.players.get())
        .ok_or(DirectClientError::NotReady)?;
        if self.player_ped_handle(local_id)? == Some(handle) {
            return Ok(Some(local_id));
        }
        for id in 0..self.spec.pools.limits.players.get() {
            let id = u16::try_from(id).map_err(|_| DirectClientError::NotReady)?;
            if id != local_id && self.player_ped_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one on-foot record from the selected local or remote player.
    pub(crate) fn onfoot_sync(
        self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.onfoot_offset,
            self.spec.players.remote.onfoot_offset,
        )?
        else {
            return Ok(None);
        };
        copy_onfoot_sync(id, address, self.spec.sync.onfoot).map(Some)
    }

    /// Copies one in-car record from the selected local or remote player.
    pub(crate) fn incar_sync(
        self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.incar_offset,
            self.spec.players.remote.incar_offset,
        )?
        else {
            return Ok(None);
        };
        copy_incar_sync(id, address, self.spec.sync.incar).map(Some)
    }

    /// Copies one passenger record from the selected local or remote player.
    pub(crate) fn passenger_sync(
        self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.passenger_offset,
            self.spec.players.remote.passenger_offset,
        )?
        else {
            return Ok(None);
        };
        copy_passenger_sync(id, address, self.spec.sync.passenger).map(Some)
    }

    /// Copies one trailer record from the selected local or remote player.
    pub(crate) fn trailer_sync(
        self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.trailer_offset,
            self.spec.players.remote.trailer_offset,
        )?
        else {
            return Ok(None);
        };
        copy_trailer_sync(id, address, self.spec.sync.trailer).map(Some)
    }

    /// Copies one aim record from the selected local or remote player.
    pub(crate) fn aim_sync(self, id: u16) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        let Some(address) = self.sync_record_address(
            id,
            self.spec.players.local.aim_offset,
            self.spec.players.remote.aim_offset,
        )?
        else {
            return Ok(None);
        };
        copy_aim_sync(id, address, self.spec.sync.aim).map(Some)
    }

    /// Sends unoccupied sync with the unified unsigned seat contract.
    pub(crate) fn force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<(), DirectClientError> {
        if usize::from(vehicle) >= self.spec.pools.limits.vehicles.get() {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let target =
            self.player_function_target(self.spec.players.local_rvas.send_unoccupied_data.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let send: R1LocalPlayerSendUnoccupiedFn = mem::transmute(target);
                    send(local, vehicle, i32::from(seat));
                }
                PoolGetterAbi::Classic => {
                    let send: ClassicLocalPlayerSendUnoccupiedFn = mem::transmute(target);
                    send(local, vehicle, i32::from(seat));
                }
            }
        }
        Ok(())
    }

    /// Invokes the local aim sync send after clearing the profile's send gate.
    pub(crate) fn force_aim_sync(self) -> Result<(), DirectClientError> {
        self.reset_force_sync_gate()?;
        self.call_local_no_arg(self.spec.players.local_rvas.send_aim_data)
    }

    /// Invokes the local on-foot sync send after clearing the profile's send gate.
    pub(crate) fn force_onfoot_sync(self) -> Result<(), DirectClientError> {
        self.reset_force_sync_gate()?;
        self.call_local_no_arg(self.spec.players.local_rvas.send_onfoot_data)
    }

    /// Invokes the local stats sync send after clearing the profile's send gate.
    pub(crate) fn force_stats_sync(self) -> Result<(), DirectClientError> {
        self.reset_force_sync_gate()?;
        self.call_local_no_arg(self.spec.players.local_rvas.send_stats)
    }

    /// Updates and sends the local trailer sync record.
    pub(crate) fn force_trailer_sync(self, trailer: u16) -> Result<(), DirectClientError> {
        self.validate_vehicle_id(trailer)?;
        self.reset_force_sync_gate()?;
        self.call_local_trailer(self.spec.players.local_rvas.send_trailer_data, trailer)
    }

    /// Updates and sends the local in-car sync record.
    pub(crate) fn force_vehicle_sync(self, vehicle: u16) -> Result<(), DirectClientError> {
        self.validate_vehicle_id(vehicle)?;
        let local = self.local_player_address()?;
        self.write_local_sync_field(
            local,
            self.spec.players.local.incar_offset.get(),
            self.spec.sync.incar.vehicle_id.get(),
            vehicle,
        )?;
        self.reset_force_sync_gate_for(local)?;
        self.call_local_no_arg_for(local, self.spec.players.local_rvas.send_incar_data)
    }

    /// Updates and sends the local passenger sync record.
    pub(crate) fn force_passenger_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<(), DirectClientError> {
        self.validate_vehicle_id(vehicle)?;
        let local = self.local_player_address()?;
        let parent = self.spec.players.local.passenger_offset.get();
        self.write_local_sync_field(
            local,
            parent,
            self.spec.sync.passenger.vehicle_id.get(),
            vehicle,
        )?;
        self.write_local_sync_field(local, parent, self.spec.sync.passenger.seat_id.get(), seat)?;
        self.reset_force_sync_gate_for(local)?;
        self.call_local_no_arg_for(local, self.spec.players.local_rvas.send_passenger_data)
    }

    /// Invokes the local weapons update without resetting the send gate.
    pub(crate) fn force_weapons_sync(self) -> Result<(), DirectClientError> {
        self.call_local_no_arg(self.spec.players.local_rvas.update_weapons)
    }

    /// Updates a validated synchronization send rate.
    pub(crate) fn set_send_rate(
        self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<(), DirectClientError> {
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
    pub(crate) fn spawn_local_player(self) -> Result<(), DirectClientError> {
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
    pub(crate) fn set_local_player_special_action(
        self,
        action: u8,
    ) -> Result<(), DirectClientError> {
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
    pub(crate) fn set_local_player_name(self, name: &[u8]) -> Result<(), DirectClientError> {
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
    pub(crate) fn set_player_colour(self, id: u16, colour: u32) -> Result<(), DirectClientError> {
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

    fn remote_player_targets(self) -> Result<RemotePlayerTargets, DirectClientError> {
        Ok(RemotePlayerTargets {
            connected: self
                .player_function_target(self.spec.players.pool_rvas.is_connected.get())?,
            remote: self
                .player_function_target(self.spec.players.pool_rvas.get_remote_player.get())?,
            exists: self.player_function_target(self.spec.players.remote_rvas.does_exist.get())?,
            name: self.player_function_target(self.spec.players.pool_rvas.get_name.get())?,
            score: self.player_function_target(self.spec.players.pool_rvas.get_score.get())?,
            ping: self.player_function_target(self.spec.players.pool_rvas.get_ping.get())?,
            colour: self
                .player_function_target(self.spec.players.remote_rvas.get_colour_argb.get())?,
            status: self.player_function_target(self.spec.players.remote_rvas.get_status.get())?,
            is_npc: self
                .spec
                .players
                .pool_rvas
                .is_npc
                .map(|rva| self.player_function_target(rva.get()))
                .transpose()?,
        })
    }

    fn connected_remote_player(
        self,
        pool: *mut c_void,
        id: u16,
        targets: RemotePlayerTargets,
    ) -> Result<Option<*mut c_void>, DirectClientError> {
        if usize::from(id) >= self.spec.pools.limits.players.get() {
            return Err(DirectClientError::NotReady);
        }
        let connected = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let function: R1PlayerPoolPlayerBooleanFn = mem::transmute(targets.connected);
                    function(pool, id)
                }
                PoolGetterAbi::Classic => {
                    let function: ClassicPlayerPoolPlayerBooleanFn =
                        mem::transmute(targets.connected);
                    function(pool, id)
                }
            }
        };
        match connected {
            0 => Ok(None),
            1 => {
                let remote = unsafe {
                    match self.spec.strategies.pool_getter_abi {
                        PoolGetterAbi::R1 => {
                            let function: R1PlayerPoolGetRemotePlayerFn =
                                mem::transmute(targets.remote);
                            function(pool, id)
                        }
                        PoolGetterAbi::Classic => {
                            let function: ClassicPlayerPoolGetRemotePlayerFn =
                                mem::transmute(targets.remote);
                            function(pool, id)
                        }
                    }
                };
                (!remote.is_null() && readable_range(remote.cast(), 1))
                    .then_some(remote)
                    .ok_or(DirectClientError::NotReady)
                    .map(Some)
            }
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn remote_player_defined(
        self,
        remote: *mut c_void,
        targets: RemotePlayerTargets,
    ) -> Result<bool, DirectClientError> {
        let exists = unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let function: R1RemotePlayerDoesExistFn = mem::transmute(targets.exists);
                    function(remote)
                }
                PoolGetterAbi::Classic => {
                    let function: ClassicRemotePlayerDoesExistFn = mem::transmute(targets.exists);
                    function(remote)
                }
            }
        };
        match exists {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn remote_player_is_npc(
        self,
        pool: *mut c_void,
        id: u16,
        targets: RemotePlayerTargets,
    ) -> Result<bool, DirectClientError> {
        if let Some(target) = targets.is_npc {
            let npc = unsafe {
                let function: R1PlayerPoolPlayerBooleanFn = mem::transmute(target);
                function(pool, id)
            };
            return match npc {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(DirectClientError::NotReady),
            };
        }
        let objects_offset = self
            .spec
            .pools
            .player
            .objects_offset
            .ok_or(DirectClientError::NotReady)?
            .get();
        let player_info = self
            .spec
            .pools
            .player
            .player_info
            .ok_or(DirectClientError::NotReady)?;
        let slot_offset = usize::from(id)
            .checked_mul(mem::size_of::<usize>())
            .and_then(|offset| objects_offset.checked_add(offset))
            .ok_or(DirectClientError::NotReady)?;
        let info = unsafe {
            read_pointer(
                (pool as usize)
                    .checked_add(slot_offset)
                    .ok_or(DirectClientError::NotReady)?,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        if info.is_null() || !readable_range(info.cast(), player_info.readable_size.get()) {
            return Err(DirectClientError::NotReady);
        }
        match unsafe {
            read_unaligned::<i32>(
                (info as usize)
                    .checked_add(player_info.npc_offset.get())
                    .ok_or(DirectClientError::NotReady)?,
            )
        } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn local_player_ped(self, local: *mut c_void) -> Result<*mut c_void, DirectClientError> {
        let ped = match self.spec.players.local_rvas.get_ped {
            Some(rva) => {
                let target = self.player_function_target(rva.get())?;
                unsafe {
                    match self.spec.strategies.pool_getter_abi {
                        PoolGetterAbi::R1 => {
                            let get_ped: R1LocalPlayerGetPedFn = mem::transmute(target);
                            get_ped(local)
                        }
                        PoolGetterAbi::Classic => return Err(DirectClientError::NotReady),
                    }
                }
            }
            None => unsafe {
                read_pointer(
                    (local as usize)
                        .checked_add(
                            self.spec
                                .players
                                .local
                                .ped_offset
                                .ok_or(DirectClientError::NotReady)?
                                .get(),
                        )
                        .ok_or(DirectClientError::NotReady)?,
                )
                .map_or(std::ptr::null_mut(), |pointer| pointer.cast())
            },
        };
        (!ped.is_null()
            && readable_range(
                ped.cast(),
                self.spec
                    .players
                    .local
                    .game_ped_offset
                    .get()
                    .checked_add(mem::size_of::<usize>())
                    .ok_or(DirectClientError::NotReady)?,
            ))
        .then_some(ped)
        .ok_or(DirectClientError::NotReady)
    }

    fn validate_vehicle_id(self, vehicle: u16) -> Result<(), DirectClientError> {
        (usize::from(vehicle) < self.spec.pools.limits.vehicles.get())
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    fn sync_record_address(
        self,
        id: u16,
        local_offset: super::profile::FieldOffset,
        remote_offset: super::profile::FieldOffset,
    ) -> Result<Option<usize>, DirectClientError> {
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
        .filter(|value| usize::from(*value) < self.spec.pools.limits.players.get());
        if local_id == Some(id) {
            return (self.local_player_address()? as usize)
                .checked_add(local_offset.get())
                .map(Some)
                .ok_or(DirectClientError::NotReady);
        }
        let targets = self.remote_player_targets()?;
        let Some(remote) = self.connected_remote_player(pool, id, targets)? else {
            return Ok(None);
        };
        if !self.remote_player_defined(remote, targets)? {
            return Ok(None);
        }
        (remote as usize)
            .checked_add(remote_offset.get())
            .map(Some)
            .ok_or(DirectClientError::NotReady)
    }

    fn reset_force_sync_gate(self) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        self.reset_force_sync_gate_for(local)
    }

    fn reset_force_sync_gate_for(self, local: *mut c_void) -> Result<(), DirectClientError> {
        match self.spec.strategies.force_sync_reset {
            ForceSyncReset::ClearLastAnyUpdate => {
                let address = (local as usize)
                    .checked_add(self.spec.players.local.last_any_update_offset.get())
                    .ok_or(DirectClientError::NotReady)?;
                unsafe { write_unaligned(address, 0_u32) }
                    .then_some(())
                    .ok_or(DirectClientError::NotReady)
            }
            ForceSyncReset::ProfileSpecific => Err(DirectClientError::NotReady),
        }
    }

    fn write_local_sync_field<T: Copy>(
        self,
        local: *mut c_void,
        parent_offset: usize,
        field_offset: usize,
        value: T,
    ) -> Result<(), DirectClientError> {
        let address = (local as usize)
            .checked_add(parent_offset)
            .and_then(|address| address.checked_add(field_offset))
            .ok_or(DirectClientError::NotReady)?;
        unsafe { write_unaligned(address, value) }
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    fn call_local_no_arg(self, rva: super::profile::NativeRva) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        self.call_local_no_arg_for(local, rva)
    }

    fn call_local_no_arg_for(
        self,
        local: *mut c_void,
        rva: super::profile::NativeRva,
    ) -> Result<(), DirectClientError> {
        let target = self.player_function_target(rva.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let send: R1LocalPlayerNoArgFn = mem::transmute(target);
                    send(local);
                }
                PoolGetterAbi::Classic => {
                    let send: ClassicLocalPlayerNoArgFn = mem::transmute(target);
                    send(local);
                }
            }
        }
        Ok(())
    }

    fn call_local_trailer(
        self,
        rva: super::profile::NativeRva,
        trailer: u16,
    ) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        let target = self.player_function_target(rva.get())?;
        unsafe {
            match self.spec.strategies.pool_getter_abi {
                PoolGetterAbi::R1 => {
                    let send: R1LocalPlayerSendTrailerFn = mem::transmute(target);
                    send(local, trailer);
                }
                PoolGetterAbi::Classic => {
                    let send: ClassicLocalPlayerSendTrailerFn = mem::transmute(target);
                    send(local, trailer);
                }
            }
        }
        Ok(())
    }

    fn player_function_target(self, rva: usize) -> Result<usize, DirectClientError> {
        self.module_base
            .checked_add(rva)
            .filter(|target| readable_range(*target as *const u8, 1))
            .ok_or(DirectClientError::NotReady)
    }
}

struct LocalPlayerTargets {
    name: usize,
    score: usize,
    ping: usize,
    colour: usize,
    health: usize,
    armour: usize,
}

#[derive(Clone, Copy)]
struct RemotePlayerTargets {
    connected: usize,
    remote: usize,
    exists: usize,
    name: usize,
    score: usize,
    ping: usize,
    colour: usize,
    status: usize,
    is_npc: Option<usize>,
}

fn validate_player_counts(
    including_npcs: i32,
    excluding_npcs: i32,
    player_limit: usize,
) -> Result<(u16, u16), DirectClientError> {
    let including_npcs = u16::try_from(including_npcs)
        .ok()
        .filter(|count| usize::from(*count) <= player_limit)
        .ok_or(DirectClientError::NotReady)?;
    let excluding_npcs = u16::try_from(excluding_npcs)
        .ok()
        .filter(|count| *count <= including_npcs)
        .ok_or(DirectClientError::NotReady)?;
    Ok((including_npcs, excluding_npcs))
}

fn validate_player_max_id(id: i32, player_limit: usize) -> Result<u16, DirectClientError> {
    u16::try_from(id)
        .ok()
        .filter(|id| usize::from(*id) < player_limit)
        .ok_or(DirectClientError::NotReady)
}

fn copy_onfoot_sync(
    id: u16,
    address: usize,
    layout: super::profile::OnFootSyncLayout,
) -> Result<OnFootSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    let scalar = |offset| unsafe {
        read_unaligned::<i16>(
            address
                .checked_add(offset)
                .ok_or(DirectClientError::NotReady)?,
        )
        .ok_or(DirectClientError::NotReady)
    };
    let vector = |offset| unsafe {
        read_vector3(
            address
                .checked_add(offset)
                .ok_or(DirectClientError::NotReady)?,
        )
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)
    };
    let quaternion = [0, 4, 8, 12].map(|delta| unsafe {
        read_unaligned::<f32>(
            address
                .checked_add(layout.quaternion.get() + delta)
                .ok_or(DirectClientError::NotReady)?,
        )
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)
    });
    let [q0, q1, q2, q3] = quaternion;
    Ok(OnFootSyncSnapshot {
        id,
        controller_left_stick_x: scalar(layout.controller_left_stick_x.get())?,
        controller_left_stick_y: scalar(layout.controller_left_stick_y.get())?,
        controller_buttons: scalar(layout.controller_buttons.get())?,
        position: vector(layout.position.get())?,
        quaternion: [q0?, q1?, q2?, q3?],
        health: unsafe { read_unaligned(address + layout.health.get()) }
            .ok_or(DirectClientError::NotReady)?,
        armour: unsafe { read_unaligned(address + layout.armour.get()) }
            .ok_or(DirectClientError::NotReady)?,
        weapon: unsafe { read_unaligned(address + layout.weapon.get()) }
            .ok_or(DirectClientError::NotReady)?,
        special_action: unsafe { read_unaligned(address + layout.special_action.get()) }
            .ok_or(DirectClientError::NotReady)?,
        speed: vector(layout.speed.get())?,
        surfing_offset: vector(layout.surfing_offset.get())?,
        surfing_vehicle_id: unsafe { read_unaligned(address + layout.surfing_vehicle_id.get()) }
            .ok_or(DirectClientError::NotReady)?,
        animation: unsafe { read_unaligned(address + layout.animation.get()) }
            .ok_or(DirectClientError::NotReady)?,
    })
}

fn sync_scalar<T: Copy>(address: usize, offset: usize) -> Result<T, DirectClientError> {
    let address = address
        .checked_add(offset)
        .ok_or(DirectClientError::NotReady)?;
    unsafe { read_unaligned(address) }.ok_or(DirectClientError::NotReady)
}

fn sync_vector(
    address: usize,
    offset: usize,
) -> Result<crate::runtime::Vector3, DirectClientError> {
    let address = address
        .checked_add(offset)
        .ok_or(DirectClientError::NotReady)?;
    unsafe { read_vector3(address) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)
}

fn sync_quaternion(address: usize, offset: usize) -> Result<[f32; 4], DirectClientError> {
    let values = [
        sync_scalar::<f32>(address, offset)?,
        sync_scalar::<f32>(address, offset + 4)?,
        sync_scalar::<f32>(address, offset + 8)?,
        sync_scalar::<f32>(address, offset + 12)?,
    ];
    values
        .iter()
        .all(|value| value.is_finite())
        .then_some(values)
        .ok_or(DirectClientError::NotReady)
}

fn copy_incar_sync(
    id: u16,
    address: usize,
    layout: super::profile::InCarSyncLayout,
) -> Result<InCarSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    let boolean = |offset| match sync_scalar::<u8>(address, offset)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DirectClientError::NotReady),
    };
    Ok(InCarSyncSnapshot {
        id,
        vehicle_id: sync_scalar(address, layout.vehicle_id.get())?,
        controller_left_stick_x: sync_scalar(address, layout.controller_left_stick_x.get())?,
        controller_left_stick_y: sync_scalar(address, layout.controller_left_stick_y.get())?,
        controller_buttons: sync_scalar(address, layout.controller_buttons.get())?,
        quaternion: sync_quaternion(address, layout.quaternion.get())?,
        position: sync_vector(address, layout.position.get())?,
        speed: sync_vector(address, layout.speed.get())?,
        vehicle_health: finite_scalar(address, layout.vehicle_health.get())?,
        driver_health: sync_scalar(address, layout.driver_health.get())?,
        driver_armour: sync_scalar(address, layout.driver_armour.get())?,
        weapon: sync_scalar(address, layout.weapon.get())?,
        siren: boolean(layout.siren.get())?,
        landing_gear: boolean(layout.landing_gear.get())?,
        trailer_id: sync_scalar(address, layout.trailer_id.get())?,
        vehicle_specific: sync_scalar(address, layout.vehicle_specific.get())?,
    })
}

fn finite_scalar(address: usize, offset: usize) -> Result<f32, DirectClientError> {
    let value = sync_scalar::<f32>(address, offset)?;
    value
        .is_finite()
        .then_some(value)
        .ok_or(DirectClientError::NotReady)
}

fn copy_passenger_sync(
    id: u16,
    address: usize,
    layout: super::profile::PassengerSyncLayout,
) -> Result<PassengerSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    Ok(PassengerSyncSnapshot {
        id,
        vehicle_id: sync_scalar(address, layout.vehicle_id.get())?,
        seat_id: sync_scalar(address, layout.seat_id.get())?,
        weapon: sync_scalar(address, layout.weapon.get())?,
        health: sync_scalar(address, layout.health.get())?,
        armour: sync_scalar(address, layout.armour.get())?,
        controller_left_stick_x: sync_scalar(address, layout.controller_left_stick_x.get())?,
        controller_left_stick_y: sync_scalar(address, layout.controller_left_stick_y.get())?,
        controller_buttons: sync_scalar(address, layout.controller_buttons.get())?,
        position: sync_vector(address, layout.position.get())?,
    })
}

fn copy_trailer_sync(
    id: u16,
    address: usize,
    layout: super::profile::TrailerSyncLayout,
) -> Result<TrailerSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    Ok(TrailerSyncSnapshot {
        id,
        trailer_id: sync_scalar(address, layout.id.get())?,
        position: sync_vector(address, layout.position.get())?,
        quaternion: sync_quaternion(address, layout.quaternion.get())?,
        speed: sync_vector(address, layout.speed.get())?,
        turn_speed: sync_vector(address, layout.turn_speed.get())?,
    })
}

fn copy_aim_sync(
    id: u16,
    address: usize,
    layout: super::profile::AimSyncLayout,
) -> Result<AimSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, layout.size.get()) {
        return Err(DirectClientError::NotReady);
    }
    let aim_z = sync_scalar::<f32>(address, layout.z.get())?;
    aim_z
        .is_finite()
        .then_some(())
        .ok_or(DirectClientError::NotReady)?;
    Ok(AimSyncSnapshot {
        id,
        camera_mode: sync_scalar(address, layout.camera_mode.get())?,
        aim_first: sync_vector(address, layout.first.get())?,
        aim_position: sync_vector(address, layout.position.get())?,
        aim_z,
        zoom_and_weapon_state: sync_scalar(address, layout.zoom_weapon_state.get())?,
        aspect_ratio: sync_scalar(address, layout.aspect_ratio.get())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;

    #[test]
    fn player_count_validation_preserves_the_public_bounds() {
        assert_eq!(validate_player_counts(1004, 1000, 1004), Ok((1004, 1000)));
        assert_eq!(
            validate_player_counts(-1, 0, 1004),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            validate_player_counts(1005, 1000, 1004),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            validate_player_counts(4, 5, 1004),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn player_id_validation_preserves_the_public_bounds() {
        assert_eq!(validate_player_max_id(1003, 1004), Ok(1003));
        assert_eq!(
            validate_player_max_id(-1, 1004),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            validate_player_max_id(1004, 1004),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn every_supported_profile_uses_a_verified_player_pool_source() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.spec.strategies.local_player_source,
                LocalPlayerSource::PlayerPoolGetter
            );
            assert_eq!(profile.spec.pools.limits.players.get(), 1004);
            assert!(profile.spec.players.local.last_any_update_offset.get() > 0);
        }
    }

    #[test]
    fn sync_specs_cover_every_supported_profile() {
        for version in [
            SampVersion::R1,
            SampVersion::R3_1,
            SampVersion::R5_1,
            SampVersion::Dl,
        ] {
            let profile = NativeClientProfile::select(0x10000, version, version.entry_point())
                .expect("the supported identity must select");
            assert_eq!(
                profile.spec.strategies.force_sync_reset,
                ForceSyncReset::ClearLastAnyUpdate
            );
            assert_eq!(profile.spec.sync.onfoot.size.get(), 68);
            assert_eq!(profile.spec.sync.incar.size.get(), 63);
            assert_eq!(profile.spec.sync.passenger.size.get(), 24);
            assert_eq!(profile.spec.sync.trailer.size.get(), 54);
            assert_eq!(profile.spec.sync.aim.size.get(), 31);
            assert!(profile.spec.players.local_rvas.send_unoccupied_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_aim_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_onfoot_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_stats.get() > 0);
            assert!(profile.spec.players.local_rvas.send_trailer_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_passenger_data.get() > 0);
            assert!(profile.spec.players.local_rvas.send_incar_data.get() > 0);
            assert!(profile.spec.players.local_rvas.update_weapons.get() > 0);
        }
    }
}
