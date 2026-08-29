//! Player-pool reads and lookups.

use super::*;

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
pub(super) type R1PlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
pub(super) type ClassicPlayerPoolPlayerBooleanFn =
    unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
pub(super) type R1PlayerPoolGetRemotePlayerFn =
    unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
pub(super) type ClassicPlayerPoolGetRemotePlayerFn =
    unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
type R1PlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type ClassicPlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type R1RemotePlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type ClassicRemotePlayerGetColourFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type R1RemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicRemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type R1RemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type ClassicRemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
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

    pub(super) fn remote_player_targets(self) -> Result<RemotePlayerTargets, DirectClientError> {
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

    pub(super) fn connected_remote_player(
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

    pub(super) fn remote_player_defined(
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

    pub(super) fn remote_player_is_npc(
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

    pub(super) fn local_player_ped(
        self,
        local: *mut c_void,
    ) -> Result<*mut c_void, DirectClientError> {
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
pub(super) struct RemotePlayerTargets {
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

pub(super) fn validate_player_counts(
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

pub(super) fn validate_player_max_id(
    id: i32,
    player_limit: usize,
) -> Result<u16, DirectClientError> {
    u16::try_from(id)
        .ok()
        .filter(|id| usize::from(*id) < player_limit)
        .ok_or(DirectClientError::NotReady)
}
