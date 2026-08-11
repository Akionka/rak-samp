//! R1 player-directory and local-player snapshot operations.

use super::*;

impl R1ClientProfile {
    /// Copies one remote player through bounded fixed-offset R1 accessors.
    /// It is invoked only by the host's game-thread pump; no client pointer
    /// survives this method.
    pub(in super::super) fn player_info(
        self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { is_connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }

        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null() || !readable_range(remote.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let is_npc: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_NPC_RVA) };
        let get_name: PlayerPoolGetPlayerNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
        let get_score: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_SCORE_RVA) };
        let get_ping: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_PING_RVA) };
        let get_colour: RemotePlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_COLOUR_ARGB_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        let get_status: RemotePlayerGetStatusFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_STATUS_RVA) };
        let is_npc = match unsafe { is_npc(pool, id) } {
            0 => false,
            1 => true,
            _ => return Err(DirectClientError::NotReady),
        };
        let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
            .filter(|name| !name.is_empty())
            .ok_or(DirectClientError::NotReady)?;

        Ok(Some(PlayerInfoSnapshot {
            id,
            defined: match unsafe { does_exist(remote) } {
                0 => false,
                1 => true,
                _ => return Err(DirectClientError::NotReady),
            },
            paused: unsafe { get_status(remote) } == 0,
            nickname,
            is_local: false,
            is_npc,
            colour: unsafe { get_colour(remote) },
            score: unsafe { get_score(pool, id) },
            ping: (unsafe { get_ping(pool, id) }).max(0) as u32,
        }))
    }

    /// Copies the volatile fields maintained by R1's remote-player update and
    /// process paths. This runs only on the host game-thread pump.
    pub(in super::super) fn remote_player_state(
        self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { is_connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null() || !readable_range(remote.cast(), REMOTE_PLAYER_STATE_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let health = unsafe {
            read_unaligned::<f32>(remote as usize + REMOTE_PLAYER_REPORTED_HEALTH_OFFSET)
        }
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)?;
        let armour = unsafe {
            read_unaligned::<f32>(remote as usize + REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET)
        }
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)?;
        let special_action =
            unsafe { read_unaligned::<u8>(remote as usize + REMOTE_PLAYER_SPECIAL_ACTION_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let animation =
            unsafe { read_unaligned::<u32>(remote as usize + REMOTE_PLAYER_ANIMATION_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(RemotePlayerStateSnapshot {
            id,
            health,
            armour,
            special_action,
            animation_id: animation as u16,
        }))
    }

    /// Reads both R1 `CPlayerPool::GetCount` modes on the game-thread pump.
    /// The resulting scalar pair is published by the host; no pool layout or
    /// pointer crosses this private profile boundary.
    pub(in super::super) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_count: PlayerPoolGetCountFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_COUNT_RVA) };
        let including_npcs = unsafe { get_count(pool, 1) };
        let excluding_npcs = unsafe { get_count(pool, 0) };
        let including_npcs = u16::try_from(including_npcs)
            .ok()
            .filter(|count| *count <= MAX_SAMP_PLAYERS)
            .ok_or(DirectClientError::NotReady)?;
        let excluding_npcs = u16::try_from(excluding_npcs)
            .ok()
            .filter(|count| *count <= including_npcs)
            .ok_or(DirectClientError::NotReady)?;
        Ok((including_npcs, excluding_npcs))
    }

    pub(in super::super) fn player_max_id(self) -> Result<u16, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null()
            || !readable_range(
                pool.cast(),
                PLAYER_POOL_LARGEST_ID_OFFSET + mem::size_of::<i32>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let largest_id =
            unsafe { read_unaligned::<i32>(pool as usize + PLAYER_POOL_LARGEST_ID_OFFSET) }
                .and_then(|id| u16::try_from(id).ok())
                .filter(|id| *id < MAX_SAMP_PLAYERS)
                .ok_or(DirectClientError::NotReady)?;
        Ok(largest_id)
    }

    pub(in super::super) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let local = self.local_player_address()?;

        let id = unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
            .and_then(assigned_player_id)
            .ok_or(DirectClientError::NotReady)?;

        let get_ped: LocalPlayerGetPedFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_PED_RVA) };
        let ped = unsafe { get_ped(local) };
        if ped.is_null()
            || !readable_range(
                ped.cast(),
                SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let game_ped = unsafe { read_pointer(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if game_ped.is_null() || !readable_range(game_ped.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let get_name: PlayerPoolGetPlayerNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
        let get_score: PlayerPoolGetLocalScoreFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_SCORE_RVA) };
        let get_ping: PlayerPoolGetLocalPingFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PING_RVA) };
        let get_colour: LocalPlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_COLOUR_ARGB_RVA) };
        let get_health: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_HEALTH_RVA) };
        let get_armour: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_ARMOUR_RVA) };

        let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
            .ok_or(DirectClientError::NotReady)?;
        let current_vehicle =
            unsafe { read_unaligned::<u16>(local as usize + LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let vehicle_id = (current_vehicle != INVALID_ID).then_some(current_vehicle);
        let (position, velocity) = if vehicle_id.is_some() {
            (
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_INCAR_OFFSET
                            + LOCAL_PLAYER_INCAR_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_INCAR_OFFSET
                            + LOCAL_PLAYER_INCAR_SPEED_OFFSET,
                    )
                },
            )
        } else {
            (
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_ONFOOT_OFFSET
                            + LOCAL_PLAYER_ONFOOT_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_ONFOOT_OFFSET
                            + LOCAL_PLAYER_ONFOOT_SPEED_OFFSET,
                    )
                },
            )
        };
        let position = position.ok_or(DirectClientError::NotReady)?;
        let velocity = velocity.ok_or(DirectClientError::NotReady)?;
        let spawned = unsafe { read_unaligned::<u32>(local as usize + LOCAL_PLAYER_ACTIVE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            != 0;
        let special_action = unsafe {
            read_unaligned::<u8>(
                local as usize
                    + LOCAL_PLAYER_ONFOOT_OFFSET
                    + LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let animation = unsafe {
            read_unaligned::<u32>(
                local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET,
            )
        }
        .ok_or(DirectClientError::NotReady)?;

        Ok(LocalPlayerSnapshot {
            id,
            nickname,
            colour: unsafe { get_colour(local) },
            spawned,
            health: unsafe { get_health(ped) },
            armour: unsafe { get_armour(ped) },
            position,
            velocity,
            special_action,
            animation_id: animation as u16,
            vehicle_id,
            score: unsafe { get_score(pool) },
            ping: (unsafe { get_ping(pool) }).max(0) as u32,
        })
    }
}
