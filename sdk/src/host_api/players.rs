use crate::{
    AimSync, HostApi, InCarSync, LocalPlayer, MAX_SAMP_PLAYERS, OnFootSync, PassengerSync,
    PlayerInfo, RemotePlayerState, SampClientSdkAimSyncV1, SampClientSdkInCarSyncV1,
    SampClientSdkLocalPlayerV1, SampClientSdkOnFootSyncV1, SampClientSdkPassengerSyncV1,
    SampClientSdkPlayerInfoV1, SampClientSdkRemotePlayerStateV1, SampClientSdkResult,
    SampClientSdkTrailerSyncV1, TrailerSync, aim_sync_from_abi, onfoot_sync_from_abi,
    passenger_sync_from_abi, player_info_from_abi, remote_player_state_from_abi,
    trailer_sync_from_abi, vehicle_sync_from_abi,
};

impl HostApi {
    /// Returns whether the latest cached R1 player record is defined in the client world.
    pub fn is_player_defined(self, id: u16) -> Result<bool, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut defined = 0;
        match unsafe { (self.raw.player_defined)(id, &mut defined) } {
            SampClientSdkResult::Ok => match defined {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
    /// Returns whether the latest cached R1 player status is `PLAYER_STATE_NONE`.
    pub fn is_player_paused(self, id: u16) -> Result<bool, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut paused = 0;
        match unsafe { (self.raw.player_paused)(id, &mut paused) } {
            SampClientSdkResult::Ok => match paused {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(SampClientSdkResult::NativeCallFailed),
            },
            result => Err(result),
        }
    }
    /// Returns the latest cached R1 player-pool count.
    pub fn player_count(self, include_npcs: bool) -> Result<u16, SampClientSdkResult> {
        let mut count = 0;
        match unsafe { (self.raw.player_count)(u8::from(include_npcs), &mut count) } {
            SampClientSdkResult::Ok => Ok(count),
            result => Err(result),
        }
    }
    /// Returns the latest cached R1 non-streamed player maximum ID.
    pub fn player_max_id(self) -> Result<u16, SampClientSdkResult> {
        let mut id = 0;
        match unsafe { (self.raw.player_max_id)(&mut id) } {
            SampClientSdkResult::Ok => Ok(id),
            result => Err(result),
        }
    }
    /// Returns an owned player-directory entry for `id`.
    ///
    /// A result of `Ok(None)` means the host's latest completed R1 query found
    /// the ID disconnected. The first remote query returns `NotReady` while it
    /// waits for the verified game-thread pump; retry it from normal plugin
    /// work rather than blocking a callback.
    pub fn player_info(self, id: u16) -> Result<Option<PlayerInfo>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkPlayerInfoV1::default();
        match unsafe { (self.raw.player_info)(id, &mut raw) } {
            SampClientSdkResult::Ok => player_info_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns whether the latest cached player-directory result has `id` connected.
    pub fn is_player_connected(self, id: u16) -> Result<bool, SampClientSdkResult> {
        self.player_info(id).map(|player| player.is_some())
    }
    /// Returns health, armour, special action, and animation ID copied from a defined remote R1 player. `Ok(None)` means the latest completed query found that ID disconnected or not world-defined.
    pub fn remote_player_state(
        self,
        id: u16,
    ) -> Result<Option<RemotePlayerState>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkRemotePlayerStateV1::default();
        match unsafe { (self.raw.remote_player_state)(id, &mut raw) } {
            SampClientSdkResult::Ok => remote_player_state_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns an owned on-foot synchronization snapshot for the local or a
    /// defined remote player. `Ok(None)` means the latest completed query found
    /// no matching player.
    pub fn onfoot_sync(self, id: u16) -> Result<Option<OnFootSync>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkOnFootSyncV1::default();
        match unsafe { (self.raw.onfoot_sync)(id, &mut raw) } {
            SampClientSdkResult::Ok => onfoot_sync_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns an owned in-car synchronization snapshot for the local or a
    /// defined remote player. `Ok(None)` means the latest completed query found
    /// no matching player.
    pub fn vehicle_sync(self, id: u16) -> Result<Option<InCarSync>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkInCarSyncV1::default();
        match unsafe { (self.raw.vehicle_sync)(id, &mut raw) } {
            SampClientSdkResult::Ok => vehicle_sync_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns an owned passenger synchronization snapshot for the local or a
    /// defined remote player. `Ok(None)` means the latest completed query found
    /// no matching player.
    pub fn passenger_sync(self, id: u16) -> Result<Option<PassengerSync>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkPassengerSyncV1::default();
        match unsafe { (self.raw.passenger_sync)(id, &mut raw) } {
            SampClientSdkResult::Ok => passenger_sync_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns an owned trailer synchronization snapshot for the local or a
    /// defined remote player. `Ok(None)` means the latest completed query found
    /// no matching player.
    pub fn trailer_sync(self, id: u16) -> Result<Option<TrailerSync>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkTrailerSyncV1::default();
        match unsafe { (self.raw.trailer_sync)(id, &mut raw) } {
            SampClientSdkResult::Ok => trailer_sync_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns an owned aim synchronization snapshot for the local or a defined
    /// remote player. `Ok(None)` means the latest completed query found no player.
    pub fn aim_sync(self, id: u16) -> Result<Option<AimSync>, SampClientSdkResult> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(SampClientSdkResult::InvalidArgument);
        }
        let mut raw = SampClientSdkAimSyncV1::default();
        match unsafe { (self.raw.aim_sync)(id, &mut raw) } {
            SampClientSdkResult::Ok => aim_sync_from_abi(raw),
            result => Err(result),
        }
    }
    /// Returns the cached remote-player health for `id`.
    pub fn player_health(self, id: u16) -> Result<Option<f32>, SampClientSdkResult> {
        self.remote_player_state(id)
            .map(|state| state.map(|state| state.health))
    }
    /// Returns the cached remote-player armour for `id`.
    pub fn player_armour(self, id: u16) -> Result<Option<f32>, SampClientSdkResult> {
        self.remote_player_state(id)
            .map(|state| state.map(|state| state.armour))
    }
    /// Returns the cached remote-player special action for `id`.
    pub fn player_special_action(self, id: u16) -> Result<Option<u8>, SampClientSdkResult> {
        self.remote_player_state(id)
            .map(|state| state.map(|state| state.special_action))
    }
    /// Returns the cached remote-player animation ID for `id`.
    pub fn player_animation_id(self, id: u16) -> Result<Option<u16>, SampClientSdkResult> {
        self.remote_player_state(id)
            .map(|state| state.map(|state| state.animation_id))
    }
    /// Returns copied player nickname bytes without assuming a text encoding.
    pub fn player_nickname(self, id: u16) -> Result<Option<Vec<u8>>, SampClientSdkResult> {
        self.player_info(id)
            .map(|player| player.map(|player| player.nickname))
    }
    /// Returns the cached player NPC state when the ID is connected.
    pub fn is_player_npc(self, id: u16) -> Result<Option<bool>, SampClientSdkResult> {
        self.player_info(id)
            .map(|player| player.map(|player| player.is_npc))
    }
    /// Returns the cached player ARGB colour when the ID is connected.
    pub fn player_colour(self, id: u16) -> Result<Option<u32>, SampClientSdkResult> {
        self.player_info(id)
            .map(|player| player.map(|player| player.colour))
    }
    /// Returns the cached player score when the ID is connected.
    pub fn player_score(self, id: u16) -> Result<Option<i32>, SampClientSdkResult> {
        self.player_info(id)
            .map(|player| player.map(|player| player.score))
    }
    /// Returns the cached player ping in milliseconds when the ID is connected.
    pub fn player_ping(self, id: u16) -> Result<Option<u32>, SampClientSdkResult> {
        self.player_info(id)
            .map(|player| player.map(|player| player.ping))
    }
    /// Returns a cloned, nonblocking local-player snapshot.
    ///
    /// This returns [`SampClientSdkResult::NotReady`] until the verified R1 game
    /// thread has published its first complete, server-assigned snapshot.
    pub fn local_player(self) -> Result<LocalPlayer, SampClientSdkResult> {
        let mut raw = SampClientSdkLocalPlayerV1::default();
        match unsafe { (self.raw.local_player)(&mut raw) } {
            SampClientSdkResult::Ok => {}
            result => return Err(result),
        }
        let nickname_len = usize::from(raw.nickname_len);
        if nickname_len > raw.nickname.len() {
            return Err(SampClientSdkResult::NativeCallFailed);
        }
        Ok(LocalPlayer {
            id: raw.id,
            nickname: raw.nickname[..nickname_len].to_vec(),
            colour: raw.colour,
            spawned: raw.spawned != 0,
            health: raw.health,
            armour: raw.armour,
            position: raw.position,
            velocity: raw.velocity,
            special_action: raw.special_action,
            animation_id: raw.animation_id,
            vehicle_id: (raw.has_vehicle != 0).then_some(raw.vehicle_id),
            score: raw.score,
            ping: raw.ping,
        })
    }

    /// Returns the cached local-player ID.
    pub fn local_player_id(self) -> Result<u16, SampClientSdkResult> {
        self.local_player().map(|player| player.id)
    }
    /// Returns owned local-player nickname bytes without assuming text encoding.
    pub fn local_player_nickname(self) -> Result<Vec<u8>, SampClientSdkResult> {
        self.local_player().map(|player| player.nickname)
    }
    /// Returns the cached local-player ARGB colour.
    pub fn local_player_colour(self) -> Result<u32, SampClientSdkResult> {
        self.local_player().map(|player| player.colour)
    }
    /// Returns whether the cached local player is spawned.
    pub fn is_local_player_spawned(self) -> Result<bool, SampClientSdkResult> {
        self.local_player().map(|player| player.spawned)
    }
    /// Returns the cached local-player health.
    pub fn local_player_health(self) -> Result<f32, SampClientSdkResult> {
        self.local_player().map(|player| player.health)
    }
    /// Returns the cached local-player armour.
    pub fn local_player_armour(self) -> Result<f32, SampClientSdkResult> {
        self.local_player().map(|player| player.armour)
    }
    /// Returns the cached local-player special action.
    pub fn local_player_special_action(self) -> Result<u8, SampClientSdkResult> {
        self.local_player().map(|player| player.special_action)
    }
    /// Returns the cached local-player animation ID.
    pub fn local_player_animation_id(self) -> Result<u16, SampClientSdkResult> {
        self.local_player().map(|player| player.animation_id)
    }
    /// Returns the cached local-player score.
    pub fn local_player_score(self) -> Result<i32, SampClientSdkResult> {
        self.local_player().map(|player| player.score)
    }
    /// Returns the cached local-player ping in milliseconds.
    pub fn local_player_ping(self) -> Result<u32, SampClientSdkResult> {
        self.local_player().map(|player| player.ping)
    }
}
