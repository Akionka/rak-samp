//! Cache publication and connection-lifecycle invalidation.

use super::*;

impl BackendState {
    pub(super) fn cache_local_player_snapshot(&self, snapshot: Option<LocalPlayerSnapshot>) {
        let Ok(mut candidate) = self.local_player_candidate.try_lock() else {
            return;
        };
        let Ok(mut cached) = self.local_player_snapshot.try_lock() else {
            return;
        };

        let Some(snapshot) = snapshot else {
            *candidate = None;
            *cached = None;
            return;
        };

        match cached.as_ref() {
            Some(current) if current.id == snapshot.id => {
                *cached = Some(snapshot);
                *candidate = None;
            }
            Some(_) => {
                *cached = None;
                *candidate = Some(snapshot);
            }
            None if candidate
                .as_ref()
                .is_some_and(|prior| prior.id == snapshot.id) =>
            {
                *cached = Some(snapshot);
                *candidate = None;
            }
            None => *candidate = Some(snapshot),
        }
    }

    pub(super) fn clear_player_info_cache(&self) {
        if let Ok(mut cache) = self.player_info_cache.try_lock() {
            cache.fill(PlayerInfoCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_remote_player_state_cache(&self) {
        if let Ok(mut cache) = self.remote_player_state_cache.try_lock() {
            cache.fill(RemotePlayerStateCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_streamed_out_player_position_cache(&self) {
        if let Ok(mut cache) = self.streamed_out_player_position_cache.try_lock() {
            cache.fill(StreamedOutPlayerPositionCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_marker_sync_positions(&self) {
        if let Ok(mut positions) = self.marker_sync_positions.try_lock() {
            positions.fill(None);
        }
    }

    pub(super) fn clear_onfoot_sync_cache(&self) {
        if let Ok(mut cache) = self.onfoot_sync_cache.try_lock() {
            cache.fill(OnFootSyncCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_incar_sync_cache(&self) {
        if let Ok(mut cache) = self.incar_sync_cache.try_lock() {
            cache.fill(InCarSyncCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_passenger_sync_cache(&self) {
        if let Ok(mut cache) = self.passenger_sync_cache.try_lock() {
            cache.fill(PassengerSyncCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_trailer_sync_cache(&self) {
        if let Ok(mut cache) = self.trailer_sync_cache.try_lock() {
            cache.fill(TrailerSyncCacheEntry::Unknown);
        }
    }
    pub(super) fn clear_aim_sync_cache(&self) {
        if let Ok(mut cache) = self.aim_sync_cache.try_lock() {
            cache.fill(AimSyncCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_vehicle_exists_cache(&self) {
        if let Ok(mut cache) = self.vehicle_exists_cache.try_lock() {
            cache.fill(VehicleExistsCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_text_label_exists_cache(&self) {
        if let Ok(mut cache) = self.text_label_exists_cache.try_lock() {
            cache.fill(TextLabelExistsCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_text_label_cache(&self) {
        if let Ok(mut cache) = self.text_label_cache.try_lock() {
            cache.fill(TextLabelCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_textdraw_exists_cache(&self) {
        if let Ok(mut cache) = self.textdraw_exists_cache.try_lock() {
            cache.fill(TextdrawExistsCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_textdraw_cache(&self) {
        if let Ok(mut cache) = self.textdraw_cache.try_lock() {
            cache.fill(TextdrawCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_chat_entry_cache(&self) {
        if let Ok(mut cache) = self.chat_entry_cache.try_lock() {
            cache.fill(ChatEntryCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_object_exists_cache(&self) {
        if let Ok(mut cache) = self.object_exists_cache.try_lock() {
            cache.fill(ObjectExistsCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_gangzone_cache(&self) {
        if let Ok(mut cache) = self.gangzone_cache.try_lock() {
            cache.fill(GangzoneCacheEntry::Unknown);
        }
    }

    pub(super) fn clear_handle_cache(&self, cache: &Mutex<Vec<HandleCacheEntry>>) {
        if let Ok(mut cache) = cache.try_lock() {
            cache.fill(HandleCacheEntry::Unknown);
        }
    }

    pub(super) fn invalidate_after_disconnect(&self) {
        self.rpc_receiver.store(0, Ordering::Release);
        self.player_address.store(0, Ordering::Release);
        self.player_port.store(0, Ordering::Release);
        self.invalidate_connection_state();
    }

    /// Invalidates every cache tied to one server connection. This runs on the
    /// game thread at a connection boundary and intentionally acquires each
    /// host cache lock: serving a prior server's entity data is worse than a
    /// short first-read `NotReady` while a plugin finishes copying a snapshot.
    pub(super) fn invalidate_connection_state(&self) {
        self.raw_player_pool.store(0, Ordering::Release);
        self.raw_vehicle_pool.store(0, Ordering::Release);
        self.raw_local_player.store(0, Ordering::Release);
        *self
            .local_player_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
        *self
            .local_player_candidate
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;

        self.player_info_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(PlayerInfoCacheEntry::Unknown);
        self.player_info_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.remote_player_state_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(RemotePlayerStateCacheEntry::Unknown);
        self.remote_player_state_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.streamed_out_player_position_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(StreamedOutPlayerPositionCacheEntry::Unknown);
        self.marker_sync_positions
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(None);
        self.streamed_out_player_position_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.onfoot_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(OnFootSyncCacheEntry::Unknown);
        self.onfoot_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.incar_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(InCarSyncCacheEntry::Unknown);
        self.incar_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.passenger_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(PassengerSyncCacheEntry::Unknown);
        self.passenger_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.trailer_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TrailerSyncCacheEntry::Unknown);
        self.trailer_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.aim_sync_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(AimSyncCacheEntry::Unknown);
        self.aim_sync_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.vehicle_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(VehicleExistsCacheEntry::Unknown);
        self.vehicle_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.text_label_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextLabelExistsCacheEntry::Unknown);
        self.text_label_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.text_label_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextLabelCacheEntry::Unknown);
        self.text_label_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.textdraw_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextdrawExistsCacheEntry::Unknown);
        self.textdraw_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.textdraw_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(TextdrawCacheEntry::Unknown);
        self.textdraw_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.chat_entry_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(ChatEntryCacheEntry::Unknown);
        self.chat_entry_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.object_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(ObjectExistsCacheEntry::Unknown);
        self.object_exists_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.gangzone_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .fill(GangzoneCacheEntry::Unknown);
        self.gangzone_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.object_handle_cache);
        self.object_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.object_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.object_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.pickup_handle_cache);
        self.pickup_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.pickup_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.pickup_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.vehicle_handle_cache);
        self.vehicle_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.vehicle_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.vehicle_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.clear_handle_cache(&self.player_handle_cache);
        self.player_handle_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.player_handle_reverse_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();
        self.player_handle_reverse_requests
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clear();

        self.player_count_ready.store(false, Ordering::Release);
        self.player_max_id_ready.store(false, Ordering::Release);
        *self
            .server_info_snapshot
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = None;
    }

    pub(super) fn ready_client(&self) -> Result<*mut c_void, SendError> {
        let client = self.rak_client.load(Ordering::Acquire) as *mut c_void;
        if client.is_null() {
            Err(SendError::ClientNotReady)
        } else {
            Ok(client)
        }
    }

    pub(super) fn ready_rpc_receiver(&self) -> Result<*mut c_void, SendError> {
        let receiver = self.rpc_receiver.load(Ordering::Acquire) as *mut c_void;
        if receiver.is_null() {
            Err(SendError::ClientNotReady)
        } else {
            Ok(receiver)
        }
    }

    pub(super) fn incoming_emulation_ready(&self) -> bool {
        self.rpc_receiver.load(Ordering::Acquire) != 0
            && self.incoming_rpc_trampoline.load(Ordering::Acquire) != 0
    }

    pub(super) fn cache_is_published(&self) -> bool {
        let generation = self.cache_generation.load(Ordering::Acquire);
        generation != 0 && generation.is_multiple_of(2)
    }
}

pub(super) fn player_info_from_local(player: &LocalPlayerSnapshot) -> PlayerInfoSnapshot {
    PlayerInfoSnapshot {
        id: player.id,
        defined: true,
        paused: false,
        nickname: player.nickname.clone(),
        is_local: true,
        is_npc: false,
        colour: player.colour,
        score: player.score,
        ping: player.ping,
    }
}

pub(super) fn is_connected_game_state(game_state: i32) -> bool {
    game_state == R1_CONNECTED_GAME_STATE
}

pub(super) fn crosses_connection_boundary(was_ready: bool, previous: i32, current: i32) -> bool {
    was_ready
        && previous != current
        && (is_connected_game_state(previous) || is_connected_game_state(current))
}
