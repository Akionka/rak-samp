//! Published local and remote player reads.

use super::{
    AimSyncCacheEntry, BackendState, InCarSyncCacheEntry, MAX_SAMP_PLAYERS, OnFootSyncCacheEntry,
    PassengerSyncCacheEntry, PlayerInfoCacheEntry, RemotePlayerStateCacheEntry,
    StreamedOutPlayerPositionCacheEntry, TrailerSyncCacheEntry, player_info_from_local,
    try_lock_direct,
};
use crate::{
    BitStream,
    runtime::{
        AimSyncSnapshot, DirectClientError, InCarSyncSnapshot, LocalPlayerSnapshot,
        OnFootSyncSnapshot, PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot,
        TrailerSyncSnapshot, Vector3,
    },
};
use std::sync::atomic::Ordering;

pub(super) const MARKERS_SYNC_PACKET_ID: u8 = 208;

impl BackendState {
    /// Captures active R1 marker coordinates from an accepted markers-sync packet.
    /// Inactive records intentionally preserve the last active coordinate, matching
    /// SAMPFUNCS' private cache behavior.
    pub(super) fn capture_marker_sync(&self, packet_id: u8, payload: &BitStream) {
        if packet_id != MARKERS_SYNC_PACKET_ID {
            return;
        }
        let mut stream = payload.clone();
        let Ok(count) = stream.read_i32() else {
            return;
        };
        let Ok(count) = usize::try_from(count) else {
            return;
        };
        if count == 0 || count >= MAX_SAMP_PLAYERS {
            return;
        }

        let mut updates = Vec::with_capacity(count);
        for _ in 0..count {
            let (Ok(id), Ok(active)) = (stream.read_u16(), stream.read_bool()) else {
                return;
            };
            let position = if active {
                let (Ok(x), Ok(y), Ok(z)) =
                    (stream.read_i16(), stream.read_i16(), stream.read_i16())
                else {
                    return;
                };
                Some(Vector3 {
                    x: f32::from(x),
                    y: f32::from(y),
                    z: f32::from(z),
                })
            } else {
                None
            };
            updates.push((id, position));
        }
        if stream.remaining_bits() >= u8::BITS as usize {
            return;
        }

        let Ok(mut positions) = self.marker_sync_positions.try_lock() else {
            return;
        };
        for (id, position) in updates {
            if let Some(position) = position
                && let Some(slot) = positions.get_mut(usize::from(id))
            {
                *slot = Some(position);
            }
        }
    }

    pub(super) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        try_lock_direct(&self.local_player_snapshot)?
            .clone()
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn player_info(
        &self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(id) >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }

        if let Some(local) = try_lock_direct(&self.local_player_snapshot)?
            .as_ref()
            .filter(|player| player.id == id)
            .map(player_info_from_local)
        {
            return Ok(Some(local));
        }

        let cached = try_lock_direct(&self.player_info_cache)?
            .get(usize::from(id))
            .cloned()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            PlayerInfoCacheEntry::Known(snapshot) => {
                // The cached result is immediately usable, including a recent
                // disconnected result. Queue an opportunistic refresh without
                // making the nonblocking read fail if that queue is busy.
                let _ = self.queue_player_info_request(id);
                Ok(snapshot)
            }
            PlayerInfoCacheEntry::Unknown => {
                self.queue_player_info_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn player_defined(&self, id: u16) -> Result<bool, DirectClientError> {
        self.player_info(id)
            .map(|player| player.is_some_and(|player| player.defined))
    }

    pub(super) fn remote_player_state(
        &self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.remote_player_state_cache)?
            .get(usize::from(id))
            .cloned()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            RemotePlayerStateCacheEntry::Known(snapshot) => {
                let _ = self.queue_remote_player_state_request(id);
                Ok(snapshot)
            }
            RemotePlayerStateCacheEntry::Unknown => {
                self.queue_remote_player_state_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn streamed_out_player_position(
        &self,
        id: u16,
    ) -> Result<Option<Vector3>, DirectClientError> {
        if self.r1_client().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.streamed_out_player_position_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            StreamedOutPlayerPositionCacheEntry::Known(position) => {
                let _ = self.queue_streamed_out_player_position_request(id);
                Ok(position)
            }
            StreamedOutPlayerPositionCacheEntry::Unknown => {
                self.queue_streamed_out_player_position_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn onfoot_sync(
        &self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        if self.r1_client().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.onfoot_sync_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            OnFootSyncCacheEntry::Known(snapshot) => {
                let _ = self.queue_onfoot_sync_request(id);
                Ok(snapshot)
            }
            OnFootSyncCacheEntry::Unknown => {
                self.queue_onfoot_sync_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn vehicle_sync(
        &self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        if self.r1_client().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.incar_sync_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            InCarSyncCacheEntry::Known(snapshot) => {
                let _ = self.queue_incar_sync_request(id);
                Ok(snapshot)
            }
            InCarSyncCacheEntry::Unknown => {
                self.queue_incar_sync_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn passenger_sync(
        &self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        if self.r1_client().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.passenger_sync_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            PassengerSyncCacheEntry::Known(snapshot) => {
                let _ = self.queue_passenger_sync_request(id);
                Ok(snapshot)
            }
            PassengerSyncCacheEntry::Unknown => {
                self.queue_passenger_sync_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn trailer_sync(
        &self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        if self.r1_client().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.trailer_sync_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            TrailerSyncCacheEntry::Known(snapshot) => {
                let _ = self.queue_trailer_sync_request(id);
                Ok(snapshot)
            }
            TrailerSyncCacheEntry::Unknown => {
                self.queue_trailer_sync_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
    pub(super) fn aim_sync(&self, id: u16) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        if self.r1_client().is_none()
            || self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || usize::from(id) >= MAX_SAMP_PLAYERS
        {
            return Err(DirectClientError::NotReady);
        }
        let cached = try_lock_direct(&self.aim_sync_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?;
        match cached {
            AimSyncCacheEntry::Known(snapshot) => {
                let _ = self.queue_aim_sync_request(id);
                Ok(snapshot)
            }
            AimSyncCacheEntry::Unknown => {
                self.queue_aim_sync_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn player_paused(&self, id: u16) -> Result<bool, DirectClientError> {
        self.player_info(id)
            .map(|player| player.is_some_and(|player| player.paused))
    }

    pub(super) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.player_count_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        let count = if include_npcs {
            self.player_count_including_npcs.load(Ordering::Acquire)
        } else {
            self.player_count_excluding_npcs.load(Ordering::Acquire)
        };
        u16::try_from(count).map_err(|_| DirectClientError::NotReady)
    }

    pub(super) fn player_max_id(&self) -> Result<u16, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.player_max_id_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        u16::try_from(self.player_max_id.load(Ordering::Acquire))
            .map_err(|_| DirectClientError::NotReady)
    }
}
