//! Game-thread cache refresh and publication helpers.

use super::*;

impl BackendState {
    pub(super) fn refresh_local_player_snapshot(
        &self,
        profile: NativeProfile,
        native_profile: Option<NativeClientProfile>,
    ) {
        let connected = self.samp_game_state_ready.load(Ordering::Acquire)
            && is_connected_game_state(self.samp_game_state.load(Ordering::Acquire));
        let snapshot = profile.local_player_cache_snapshot(connected);
        self.raw_local_player.store(
            native_profile
                .filter(|_| connected)
                .and_then(|profile| profile.local_player_address().ok())
                .map_or(0, |player| player as usize),
            Ordering::Release,
        );
        self.cache_local_player_snapshot(snapshot.snapshot);
    }

    pub(super) fn refresh_player_info(&self, profile: NativeProfile) {
        for id in self.take_player_info_requests() {
            let Ok(snapshot) = profile.player_info(id) else {
                continue;
            };
            let Ok(mut cache) = self.player_info_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = PlayerInfoCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_remote_player_state(&self, profile: NativeProfile) {
        for id in self.take_remote_player_state_requests() {
            let Ok(snapshot) = profile.remote_player_state(id) else {
                continue;
            };
            let Ok(mut cache) = self.remote_player_state_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = RemotePlayerStateCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_streamed_out_player_position(&self, profile: NativeProfile) {
        for id in self.take_streamed_out_player_position_requests() {
            let Ok(streamed_out) = profile.remote_player_is_streamed_out(id) else {
                continue;
            };
            let position = match streamed_out {
                Some(true) => self
                    .marker_sync_positions
                    .try_lock()
                    .ok()
                    .and_then(|positions| positions.get(usize::from(id)).copied().flatten())
                    .filter(|position| position.x != 0.0 && position.y != 0.0),
                Some(false) | None => None,
            };
            let Ok(mut cache) = self.streamed_out_player_position_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = StreamedOutPlayerPositionCacheEntry::Known(position);
            }
        }
    }

    pub(super) fn refresh_onfoot_sync(&self, profile: NativeProfile) {
        for id in self.take_onfoot_sync_requests() {
            let Ok(snapshot) = profile.onfoot_sync(id) else {
                continue;
            };
            let Ok(mut cache) = self.onfoot_sync_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = OnFootSyncCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_incar_sync(&self, profile: NativeProfile) {
        for id in self.take_incar_sync_requests() {
            let Ok(snapshot) = profile.incar_sync(id) else {
                continue;
            };
            let Ok(mut cache) = self.incar_sync_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = InCarSyncCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_passenger_sync(&self, profile: NativeProfile) {
        for id in self.take_passenger_sync_requests() {
            let Ok(snapshot) = profile.passenger_sync(id) else {
                continue;
            };
            let Ok(mut cache) = self.passenger_sync_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = PassengerSyncCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_trailer_sync(&self, profile: NativeProfile) {
        for id in self.take_trailer_sync_requests() {
            let Ok(snapshot) = profile.trailer_sync(id) else {
                continue;
            };
            let Ok(mut cache) = self.trailer_sync_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = TrailerSyncCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_aim_sync(&self, profile: NativeProfile) {
        for id in self.take_aim_sync_requests() {
            let Ok(snapshot) = profile.aim_sync(id) else {
                continue;
            };
            let Ok(mut cache) = self.aim_sync_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = AimSyncCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_player_count(&self, profile: NativeClientProfile) {
        match profile.player_counts() {
            Ok((including_npcs, excluding_npcs)) => {
                self.player_count_including_npcs
                    .store(i32::from(including_npcs), Ordering::Release);
                self.player_count_excluding_npcs
                    .store(i32::from(excluding_npcs), Ordering::Release);
                self.player_count_ready.store(true, Ordering::Release);
            }
            Err(_) => self.player_count_ready.store(false, Ordering::Release),
        }
    }

    pub(super) fn refresh_player_max_id(&self, profile: NativeClientProfile) {
        match profile.player_max_id() {
            Ok(id) => {
                self.player_max_id.store(i32::from(id), Ordering::Release);
                self.player_max_id_ready.store(true, Ordering::Release);
            }
            Err(_) => self.player_max_id_ready.store(false, Ordering::Release),
        }
    }

    pub(super) fn refresh_raw_pool_addresses(&self, profile: NativeClientProfile) {
        let player_pool = profile.player_pool().map_or(0, |pool| pool as usize);
        let vehicle_pool = profile.vehicle_pool().map_or(0, |pool| pool as usize);
        self.raw_player_pool.store(player_pool, Ordering::Release);
        self.raw_vehicle_pool.store(vehicle_pool, Ordering::Release);
    }

    pub(super) fn refresh_vehicle_exists(&self, profile: NativeProfile) {
        for id in self.take_vehicle_exists_requests() {
            let Ok(exists) = profile.vehicle_exists(id) else {
                continue;
            };
            let Ok(mut cache) = self.vehicle_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = VehicleExistsCacheEntry::Known(exists);
            }
        }
    }

    pub(super) fn refresh_text_label_exists(&self, profile: NativeProfile) {
        for id in self.take_text_label_exists_requests() {
            let Ok(exists) = profile.text_label_exists(id) else {
                continue;
            };
            let Ok(mut cache) = self.text_label_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = TextLabelExistsCacheEntry::Known(exists);
            }
        }
    }

    pub(super) fn refresh_text_labels(&self, profile: NativeProfile) {
        for id in self.take_text_label_requests() {
            let Ok(snapshot) = profile.text_label(id) else {
                continue;
            };
            let Ok(mut cache) = self.text_label_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = TextLabelCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_textdraw_exists(&self, profile: NativeProfile) {
        for pool_index in self.take_textdraw_exists_requests() {
            let Ok(exists) = profile.textdraw_exists(pool_index) else {
                continue;
            };
            let Ok(mut cache) = self.textdraw_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(pool_index)) {
                *entry = TextdrawExistsCacheEntry::Known(exists);
            }
        }
    }

    pub(super) fn refresh_textdraws(&self, profile: NativeProfile) {
        for pool_index in self.take_textdraw_requests() {
            let Ok(snapshot) = profile.textdraw(pool_index) else {
                continue;
            };
            let Ok(mut cache) = self.textdraw_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(pool_index)) {
                *entry = TextdrawCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_chat_entries(&self, profile: NativeProfile) {
        for id in self.take_chat_entry_requests() {
            let Ok(snapshot) = profile.chat_entry(id) else {
                continue;
            };
            let Ok(mut cache) = self.chat_entry_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = ChatEntryCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_object_exists(&self, profile: NativeProfile) {
        for id in self.take_object_exists_requests() {
            let Ok(exists) = profile.object_exists(id) else {
                continue;
            };
            let Ok(mut cache) = self.object_exists_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = ObjectExistsCacheEntry::Known(exists);
            }
        }
    }

    pub(super) fn refresh_gangzones(&self, profile: NativeProfile) {
        for id in self.take_gangzone_requests() {
            let Ok(snapshot) = profile.gangzone(id) else {
                continue;
            };
            let Ok(mut cache) = self.gangzone_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = GangzoneCacheEntry::Known(snapshot);
            }
        }
    }

    pub(super) fn refresh_object_handles(&self, profile: NativeProfile) {
        for id in self.take_object_handle_requests() {
            let Ok(handle) = profile.object_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.object_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    pub(super) fn refresh_pickup_handles(&self, profile: NativeProfile) {
        for id in self.take_pickup_handle_requests() {
            let Ok(handle) = profile.pickup_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.pickup_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    pub(super) fn refresh_vehicle_handles(&self, profile: NativeProfile) {
        for id in self.take_vehicle_handle_requests() {
            let Ok(handle) = profile.vehicle_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.vehicle_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    pub(super) fn refresh_player_handles(&self, profile: NativeProfile) {
        for id in self.take_player_handle_requests() {
            let Ok(handle) = profile.player_ped_handle(id) else {
                continue;
            };
            let Ok(mut cache) = self.player_handle_cache.try_lock() else {
                continue;
            };
            if let Some(entry) = cache.get_mut(usize::from(id)) {
                *entry = HandleCacheEntry::Known(handle);
            }
        }
    }

    pub(super) fn refresh_object_handle_ids(&self, profile: NativeProfile) {
        for handle in self.take_object_handle_id_requests() {
            let Ok(id) = profile.object_id_by_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.object_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    pub(super) fn refresh_pickup_handle_ids(&self, profile: NativeProfile) {
        for handle in self.take_pickup_handle_id_requests() {
            let Ok(id) = profile.pickup_id_by_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.pickup_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    pub(super) fn refresh_vehicle_handle_ids(&self, profile: NativeProfile) {
        for handle in self.take_vehicle_handle_id_requests() {
            let Ok(id) = profile.vehicle_id_by_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.vehicle_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    pub(super) fn refresh_player_handle_ids(&self, profile: NativeProfile) {
        for handle in self.take_player_handle_id_requests() {
            let Ok(id) = profile.player_id_by_ped_handle(handle) else {
                continue;
            };
            let Ok(mut cache) = self.player_handle_reverse_cache.try_lock() else {
                continue;
            };
            cache.insert(handle, id);
        }
    }

    pub(super) fn refresh_server_info_snapshot(&self, profile: NativeClientProfile) {
        let Ok(mut cached) = self.server_info_snapshot.try_lock() else {
            return;
        };
        *cached = profile.server_info().ok();
    }

    pub(super) fn refresh_samp_game_state(&self, profile: NativeClientProfile) {
        match profile.game_state() {
            Ok(game_state) => {
                let previous = self.samp_game_state.swap(game_state, Ordering::AcqRel);
                let was_ready = self.samp_game_state_ready.swap(true, Ordering::AcqRel);
                if crosses_connection_boundary(was_ready, previous, game_state) {
                    self.invalidate_connection_state();
                }
            }
            Err(DirectClientError::NotReady) => {
                self.samp_game_state_ready.store(false, Ordering::Release);
            }
            Err(
                DirectClientError::Busy
                | DirectClientError::UnsupportedVersion
                | DirectClientError::QueueFull,
            ) => {
                self.samp_game_state_ready.store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_chat_display_mode(&self, profile: NativeProfile) {
        match profile.chat_display_mode() {
            Ok(mode) => {
                self.local_chat_display_mode.store(mode, Ordering::Release);
                self.local_chat_display_mode_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_display_mode_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_cursor_mode(&self, profile: NativeProfile) {
        match profile.cursor_mode() {
            Ok(mode) => {
                self.local_cursor_mode.store(mode, Ordering::Release);
                self.local_cursor_mode_ready.store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_cursor_mode_ready.store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_scoreboard_open(&self, profile: NativeProfile) {
        match profile.scoreboard_is_open() {
            Ok(open) => {
                self.local_scoreboard_open.store(open, Ordering::Release);
                self.local_scoreboard_open_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_scoreboard_open_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_dialog_active(&self, profile: NativeProfile) {
        match profile.dialog_is_active() {
            Ok(active) => {
                self.local_dialog_active.store(active, Ordering::Release);
                self.local_dialog_active_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_dialog_active_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_dialog_state(&self, profile: NativeProfile) {
        match profile.dialog_state() {
            Ok(snapshot) => {
                let Ok(mut cached) = self.local_dialog_snapshot.try_lock() else {
                    return;
                };
                *cached = snapshot;
                self.local_dialog_snapshot_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => self
                .local_dialog_snapshot_ready
                .store(false, Ordering::Release),
        }
    }

    pub(super) fn refresh_local_chat_input_active(&self, profile: NativeProfile) {
        match profile.chat_input_is_active() {
            Ok(active) => {
                self.local_chat_input_active
                    .store(active, Ordering::Release);
                self.local_chat_input_active_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_input_active_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_chat_input_text(&self, profile: NativeProfile) {
        match profile.chat_input_text() {
            Ok(text) => {
                let Ok(mut snapshot) = self.local_chat_input_text.try_lock() else {
                    self.local_chat_input_text_ready
                        .store(false, Ordering::Release);
                    return;
                };
                *snapshot = Some(text);
                self.local_chat_input_text_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_input_text_ready
                    .store(false, Ordering::Release);
            }
        }
    }

    pub(super) fn refresh_local_chat_input_commands(&self, profile: NativeProfile) {
        match profile.chat_input_commands() {
            Ok(commands) => {
                let Ok(mut snapshot) = self.local_chat_input_commands.try_lock() else {
                    self.local_chat_input_commands_ready
                        .store(false, Ordering::Release);
                    return;
                };
                *snapshot = Some(commands);
                self.local_chat_input_commands_ready
                    .store(true, Ordering::Release);
            }
            Err(_) => {
                self.local_chat_input_commands_ready
                    .store(false, Ordering::Release);
                if let Ok(mut snapshot) = self.local_chat_input_commands.try_lock() {
                    *snapshot = None;
                }
            }
        }
    }

    pub(super) fn refresh_animation_catalog(&self, profile: NativeProfile) {
        let Ok(mut catalog) = self.animation_catalog.try_lock() else {
            return;
        };
        if catalog.is_none() {
            *catalog = profile.animation_catalog().ok();
        }
    }
}
