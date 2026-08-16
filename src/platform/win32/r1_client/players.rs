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

    /// Determines whether a connected remote player currently lacks a GTA ped.
    /// The host-owned marker packet cache supplies its last streamed-out
    /// coordinates separately.
    pub(in super::super) fn remote_player_is_streamed_out(
        self,
        id: u16,
    ) -> Result<Option<bool>, DirectClientError> {
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
        if remote.is_null() || !readable_range(remote.cast(), mem::size_of::<usize>()) {
            return Err(DirectClientError::NotReady);
        }
        let ped = unsafe { read_pointer(remote as usize) }.ok_or(DirectClientError::NotReady)?;
        if ped.is_null() {
            return Ok(Some(true));
        }
        if !readable_range(
            ped.cast(),
            SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let game_ped = unsafe { read_pointer(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        Ok(Some(game_ped.is_null()))
    }

    /// Copies one fixed R1 `SOnfootData` record on the game thread. The local
    /// player uses its local record; other IDs use a defined remote record.
    pub(in super::super) fn onfoot_sync(
        self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), PLAYER_POOL_LOCAL_ID_OFFSET + 2) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id);
        if local_id == Some(id) {
            let get_local_player: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local_player(pool) };
            if local.is_null() {
                return Ok(None);
            }
            return self
                .onfoot_sync_from_address(id, local as usize + LOCAL_PLAYER_ONFOOT_OFFSET)
                .map(Some);
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
        if remote.is_null()
            || !readable_range(
                remote.cast(),
                REMOTE_PLAYER_ONFOOT_OFFSET + ONFOOT_SYNC_SIZE,
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => self
                .onfoot_sync_from_address(id, remote as usize + REMOTE_PLAYER_ONFOOT_OFFSET)
                .map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn onfoot_sync_from_address(
        self,
        id: u16,
        address: usize,
    ) -> Result<OnFootSyncSnapshot, DirectClientError> {
        if !readable_range(address as *const u8, ONFOOT_SYNC_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let controller_left_stick_x =
            unsafe { read_unaligned::<i16>(address + ONFOOT_CONTROLLER_LEFT_STICK_X_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let controller_left_stick_y =
            unsafe { read_unaligned::<i16>(address + ONFOOT_CONTROLLER_LEFT_STICK_Y_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let controller_buttons =
            unsafe { read_unaligned::<i16>(address + ONFOOT_CONTROLLER_BUTTONS_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(address + ONFOOT_POSITION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let quaternion = [
            unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET + 4) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET + 8) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET + 12) }
                .ok_or(DirectClientError::NotReady)?,
        ];
        if !quaternion.iter().all(|value| value.is_finite()) {
            return Err(DirectClientError::NotReady);
        }
        let health = unsafe { read_unaligned::<u8>(address + ONFOOT_HEALTH_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let armour = unsafe { read_unaligned::<u8>(address + ONFOOT_ARMOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let weapon = unsafe { read_unaligned::<u8>(address + ONFOOT_WEAPON_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let special_action =
            unsafe { read_unaligned::<u8>(address + ONFOOT_SPECIAL_ACTION_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let speed = unsafe { read_vector3(address + ONFOOT_SPEED_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let surfing_offset = unsafe { read_vector3(address + ONFOOT_SURFING_OFFSET_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let surfing_vehicle_id =
            unsafe { read_unaligned::<u16>(address + ONFOOT_SURFING_VEHICLE_ID_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let animation = unsafe { read_unaligned::<u32>(address + ONFOOT_ANIMATION_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        Ok(OnFootSyncSnapshot {
            id,
            controller_left_stick_x,
            controller_left_stick_y,
            controller_buttons,
            position,
            quaternion,
            health,
            armour,
            weapon,
            special_action,
            speed,
            surfing_offset,
            surfing_vehicle_id,
            animation,
        })
    }

    /// Copies one fixed R1 `SIncarData` record on the game thread. The local
    /// player uses its local record; other IDs use a defined remote record.
    pub(in super::super) fn incar_sync(
        self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), PLAYER_POOL_LOCAL_ID_OFFSET + 2) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id);
        if local_id == Some(id) {
            let get_local_player: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local_player(pool) };
            if local.is_null() {
                return Ok(None);
            }
            return self
                .incar_sync_from_address(id, local as usize + LOCAL_PLAYER_INCAR_OFFSET)
                .map(Some);
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
        if remote.is_null()
            || !readable_range(remote.cast(), REMOTE_PLAYER_INCAR_OFFSET + INCAR_SYNC_SIZE)
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => self
                .incar_sync_from_address(id, remote as usize + REMOTE_PLAYER_INCAR_OFFSET)
                .map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn incar_sync_from_address(
        self,
        id: u16,
        address: usize,
    ) -> Result<InCarSyncSnapshot, DirectClientError> {
        if !readable_range(address as *const u8, INCAR_SYNC_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let vehicle_id = unsafe { read_unaligned::<u16>(address + INCAR_VEHICLE_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let controller_left_stick_x =
            unsafe { read_unaligned::<i16>(address + INCAR_CONTROLLER_LEFT_STICK_X_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let controller_left_stick_y =
            unsafe { read_unaligned::<i16>(address + INCAR_CONTROLLER_LEFT_STICK_Y_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let controller_buttons =
            unsafe { read_unaligned::<i16>(address + INCAR_CONTROLLER_BUTTONS_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let quaternion = [
            unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET + 4) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET + 8) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET + 12) }
                .ok_or(DirectClientError::NotReady)?,
        ];
        if !quaternion.iter().all(|value| value.is_finite()) {
            return Err(DirectClientError::NotReady);
        }
        let position = unsafe { read_vector3(address + INCAR_POSITION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let speed = unsafe { read_vector3(address + INCAR_SPEED_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let vehicle_health =
            unsafe { read_unaligned::<f32>(address + INCAR_VEHICLE_HEALTH_OFFSET) }
                .filter(|value| value.is_finite())
                .ok_or(DirectClientError::NotReady)?;
        let driver_health = unsafe { read_unaligned::<u8>(address + INCAR_DRIVER_HEALTH_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let driver_armour = unsafe { read_unaligned::<u8>(address + INCAR_DRIVER_ARMOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let weapon = unsafe { read_unaligned::<u8>(address + INCAR_WEAPON_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let siren = read_u8_bool(address + INCAR_SIREN_OFFSET)?;
        let landing_gear = read_u8_bool(address + INCAR_LANDING_GEAR_OFFSET)?;
        let trailer_id = unsafe { read_unaligned::<u16>(address + INCAR_TRAILER_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let vehicle_specific =
            unsafe { read_unaligned::<[u8; 4]>(address + INCAR_VEHICLE_SPECIFIC_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(InCarSyncSnapshot {
            id,
            vehicle_id,
            controller_left_stick_x,
            controller_left_stick_y,
            controller_buttons,
            quaternion,
            position,
            speed,
            vehicle_health,
            driver_health,
            driver_armour,
            weapon,
            siren,
            landing_gear,
            trailer_id,
            vehicle_specific,
        })
    }

    /// Copies one fixed R1 `SPassengerData` record on the game thread. The
    /// local player uses its local record; other IDs use a defined remote record.
    pub(in super::super) fn passenger_sync(
        self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), PLAYER_POOL_LOCAL_ID_OFFSET + 2) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id);
        if local_id == Some(id) {
            let get_local_player: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local_player(pool) };
            if local.is_null() {
                return Ok(None);
            }
            return self
                .passenger_sync_from_address(id, local as usize + LOCAL_PLAYER_PASSENGER_OFFSET)
                .map(Some);
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
        if remote.is_null()
            || !readable_range(
                remote.cast(),
                REMOTE_PLAYER_PASSENGER_OFFSET + PASSENGER_SYNC_SIZE,
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => self
                .passenger_sync_from_address(id, remote as usize + REMOTE_PLAYER_PASSENGER_OFFSET)
                .map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn passenger_sync_from_address(
        self,
        id: u16,
        address: usize,
    ) -> Result<PassengerSyncSnapshot, DirectClientError> {
        if !readable_range(address as *const u8, PASSENGER_SYNC_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let vehicle_id = unsafe { read_unaligned::<u16>(address + PASSENGER_VEHICLE_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let seat_id = unsafe { read_unaligned::<u8>(address + PASSENGER_SEAT_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let weapon = unsafe { read_unaligned::<u8>(address + PASSENGER_WEAPON_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let health = unsafe { read_unaligned::<u8>(address + PASSENGER_HEALTH_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let armour = unsafe { read_unaligned::<u8>(address + PASSENGER_ARMOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let controller_left_stick_x =
            unsafe { read_unaligned::<i16>(address + PASSENGER_CONTROLLER_LEFT_STICK_X_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let controller_left_stick_y =
            unsafe { read_unaligned::<i16>(address + PASSENGER_CONTROLLER_LEFT_STICK_Y_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let controller_buttons =
            unsafe { read_unaligned::<i16>(address + PASSENGER_CONTROLLER_BUTTONS_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(address + PASSENGER_POSITION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        Ok(PassengerSyncSnapshot {
            id,
            vehicle_id,
            seat_id,
            weapon,
            health,
            armour,
            controller_left_stick_x,
            controller_left_stick_y,
            controller_buttons,
            position,
        })
    }

    /// Copies one fixed R1 `STrailerData` record on the game thread. The local
    /// player uses its local record; other IDs use a defined remote record.
    pub(in super::super) fn trailer_sync(
        self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), PLAYER_POOL_LOCAL_ID_OFFSET + 2) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id);
        if local_id == Some(id) {
            let get_local_player: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local_player(pool) };
            if local.is_null() {
                return Ok(None);
            }
            return self
                .trailer_sync_from_address(id, local as usize + LOCAL_PLAYER_TRAILER_OFFSET)
                .map(Some);
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
        if remote.is_null()
            || !readable_range(
                remote.cast(),
                REMOTE_PLAYER_TRAILER_OFFSET + TRAILER_SYNC_SIZE,
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => self
                .trailer_sync_from_address(id, remote as usize + REMOTE_PLAYER_TRAILER_OFFSET)
                .map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn trailer_sync_from_address(
        self,
        id: u16,
        address: usize,
    ) -> Result<TrailerSyncSnapshot, DirectClientError> {
        if !readable_range(address as *const u8, TRAILER_SYNC_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let trailer_id = unsafe { read_unaligned::<u16>(address + TRAILER_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(address + TRAILER_POSITION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let quaternion = [
            unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET + 4) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET + 8) }
                .ok_or(DirectClientError::NotReady)?,
            unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET + 12) }
                .ok_or(DirectClientError::NotReady)?,
        ];
        if !quaternion.iter().all(|value| value.is_finite()) {
            return Err(DirectClientError::NotReady);
        }
        let speed = unsafe { read_vector3(address + TRAILER_SPEED_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let turn_speed = unsafe { read_vector3(address + TRAILER_TURN_SPEED_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        Ok(TrailerSyncSnapshot {
            id,
            trailer_id,
            position,
            quaternion,
            speed,
            turn_speed,
        })
    }

    pub(in super::super) fn aim_sync(
        self,
        id: u16,
    ) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), PLAYER_POOL_LOCAL_ID_OFFSET + 2) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id);
        if local_id == Some(id) {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            if local.is_null() {
                return Ok(None);
            }
            return self
                .aim_sync_from_address(id, local as usize + LOCAL_PLAYER_AIM_OFFSET)
                .map(Some);
        }
        let connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let get_remote: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_remote(pool, id) };
        if remote.is_null()
            || !readable_range(remote.cast(), REMOTE_PLAYER_AIM_OFFSET + AIM_SYNC_SIZE)
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => self
                .aim_sync_from_address(id, remote as usize + REMOTE_PLAYER_AIM_OFFSET)
                .map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }
    fn aim_sync_from_address(
        self,
        id: u16,
        address: usize,
    ) -> Result<AimSyncSnapshot, DirectClientError> {
        if !readable_range(address as *const u8, AIM_SYNC_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let camera_mode = unsafe { read_unaligned::<u8>(address + AIM_CAMERA_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let aim_first = unsafe { read_vector3(address + AIM_FIRST_OFFSET) }
            .filter(|v| v.x.is_finite() && v.y.is_finite() && v.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let aim_position = unsafe { read_vector3(address + AIM_POSITION_OFFSET) }
            .filter(|v| v.x.is_finite() && v.y.is_finite() && v.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let aim_z = unsafe { read_unaligned::<f32>(address + AIM_Z_OFFSET) }
            .filter(|v| v.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let zoom_and_weapon_state =
            unsafe { read_unaligned::<u8>(address + AIM_ZOOM_WEAPON_STATE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let aspect_ratio = unsafe { read_unaligned::<u8>(address + AIM_ASPECT_RATIO_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        Ok(AimSyncSnapshot {
            id,
            camera_mode,
            aim_first,
            aim_position,
            aim_z,
            zoom_and_weapon_state,
            aspect_ratio,
        })
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
