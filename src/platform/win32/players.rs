//! Published local and remote player reads.

use super::{
    BackendState, MAX_SAMP_PLAYERS, OnFootSyncCacheEntry, PlayerInfoCacheEntry,
    RemotePlayerStateCacheEntry, player_info_from_local, try_lock_direct,
};
use crate::runtime::{
    DirectClientError, LocalPlayerSnapshot, OnFootSyncSnapshot, PlayerInfoSnapshot,
    RemotePlayerStateSnapshot,
};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn local_player(&self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        if self.r1_client.is_none() {
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
        if self.r1_client.is_none() {
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
        if self.r1_client.is_none() {
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

    pub(super) fn onfoot_sync(
        &self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        if self.r1_client.is_none()
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

    pub(super) fn player_paused(&self, id: u16) -> Result<bool, DirectClientError> {
        self.player_info(id)
            .map(|player| player.is_some_and(|player| player.paused))
    }

    pub(super) fn player_count(&self, include_npcs: bool) -> Result<u16, DirectClientError> {
        if self.r1_client.is_none() {
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
        if self.r1_client.is_none() {
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
