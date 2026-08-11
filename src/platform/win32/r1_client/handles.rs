//! R1 GTA-handle lookup operations.

use super::*;

impl R1ClientProfile {
    pub(in super::super) fn object_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_OBJECT_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            OBJECT_POOL_OBJECTS_OFFSET + (usize::from(id) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, checked_len)
            || !read_r1_bool(
                pool + OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
            )?
        {
            return Ok(None);
        }
        let object = unsafe {
            read_unaligned::<usize>(
                pool + OBJECT_POOL_OBJECTS_OFFSET + usize::from(id) * mem::size_of::<usize>(),
            )
        }
        .filter(|object| *object != 0)
        .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            object as *const u8,
            ENTITY_HANDLE_OFFSET + mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let handle = unsafe { read_unaligned::<i32>(object + ENTITY_HANDLE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 object pool for a matching GTAREF on the game thread.
    pub(in super::super) fn object_id_by_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_OBJECTS {
            if self.object_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one R1 pickup-pool handle (GTAREF) on the game thread.
    pub(in super::super) fn pickup_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_PICKUPS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_PICKUP_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_PICKUP_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            PICKUP_POOL_HANDLES_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        let handle = unsafe {
            read_unaligned::<i32>(
                pool + PICKUP_POOL_HANDLES_OFFSET + usize::from(id) * mem::size_of::<i32>(),
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 pickup pool for a matching GTAREF on the game thread.
    pub(in super::super) fn pickup_id_by_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_PICKUPS {
            if self.pickup_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one R1 vehicle GTA handle (GTAREF) on the game thread by
    /// converting the validated `m_pGameObject` pointer through the fixed
    /// GTA SA `CPools::GetVehicleRef` target.
    pub(in super::super) fn vehicle_handle(
        self,
        id: u16,
    ) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.vehicle_pool()?;
        let checked_len =
            VEHICLE_POOL_GAME_OBJECTS_OFFSET + (usize::from(id) + 1) * mem::size_of::<usize>();
        if !readable_range(pool.cast(), checked_len)
            || !read_r1_bool(
                pool as usize
                    + VEHICLE_POOL_NOT_EMPTY_OFFSET
                    + usize::from(id) * mem::size_of::<i32>(),
            )?
        {
            return Ok(None);
        }
        let game_object = unsafe {
            read_unaligned::<usize>(
                pool as usize
                    + VEHICLE_POOL_GAME_OBJECTS_OFFSET
                    + usize::from(id) * mem::size_of::<usize>(),
            )
        }
        .filter(|game_object| *game_object != 0)
        .ok_or(DirectClientError::NotReady)?;
        let get_vehicle_ref: CpoolRefFn = unsafe { mem::transmute(CPOOLS_GET_VEHICLE_REF) };
        let handle = unsafe { get_vehicle_ref(game_object as *mut c_void) };
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 vehicle pool for a matching GTA handle on the game thread.
    pub(in super::super) fn vehicle_id_by_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_VEHICLES {
            if self.vehicle_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one R1 player-pool GTA ped handle (GTAREF) on the game thread.
    ///
    /// The local player resolves through `CLocalPlayer::GetPed` → `m_pGamePed`
    /// and the fixed GTA SA `CPools::GetPedRef`; remote players resolve through
    /// `CRemotePlayer.m_pPed` → `m_pGamePed` → `GetPedRef`.
    pub(in super::super) fn player_ped_handle(
        self,
        id: u16,
    ) -> Result<Option<i32>, DirectClientError> {
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
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id)
                .ok_or(DirectClientError::NotReady)?;
        let game_ped = if id == local_id {
            let get_local_player: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local_player(pool) };
            if local.is_null() || !readable_range(local.cast(), 1) {
                return Err(DirectClientError::NotReady);
            }
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
            unsafe { read_unaligned::<usize>(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
        } else {
            let is_connected: PlayerPoolPlayerBooleanFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
            if unsafe { is_connected(pool, id) } != 1 {
                return Ok(None);
            }
            let get_player: PlayerPoolGetRemotePlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
            let remote = unsafe { get_player(pool, id) };
            if remote.is_null() || !readable_range(remote.cast(), mem::size_of::<usize>()) {
                return Err(DirectClientError::NotReady);
            }
            let ped = unsafe { read_unaligned::<usize>(remote as usize) }
                .filter(|ped| *ped != 0)
                .ok_or(DirectClientError::NotReady)?;
            if !readable_range(
                ped as *const u8,
                SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
            ) {
                return Err(DirectClientError::NotReady);
            }
            unsafe { read_unaligned::<usize>(ped + SAMP_PED_GAME_PED_OFFSET) }
        };
        let game_ped = game_ped
            .filter(|game_ped| *game_ped != 0)
            .ok_or(DirectClientError::NotReady)?;
        let get_ped_ref: CpoolRefFn = unsafe { mem::transmute(CPOOLS_GET_PED_REF) };
        let handle = unsafe { get_ped_ref(game_ped as *mut c_void) };
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 player pool for a matching GTA ped handle on the game
    /// thread. The local player is checked first, matching SF.lua's
    /// `sampGetPlayerIdByCharHandle`.
    pub(in super::super) fn player_id_by_ped_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id)
                .ok_or(DirectClientError::NotReady)?;
        if self.player_ped_handle(local_id)? == Some(handle) {
            return Ok(Some(local_id));
        }
        for id in 0..MAX_SAMP_PLAYERS {
            if id != local_id && self.player_ped_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }
}
