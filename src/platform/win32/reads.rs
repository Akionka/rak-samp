//! Reads from game-thread-published scalar and snapshot caches.

use super::{BackendState, cached_direct_client_value};
use crate::runtime::{DirectClientError, LocalDialogSnapshot, ServerInfoSnapshot};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        cached_direct_client_value(
            self.r1_client.is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.samp_game_state_ready
                .load(Ordering::Acquire)
                .then(|| self.samp_game_state.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        cached_direct_client_value(
            self.r1_client.is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_chat_display_mode_ready
                .load(Ordering::Acquire)
                .then(|| self.local_chat_display_mode.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        cached_direct_client_value(
            self.r1_client.is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_cursor_mode_ready
                .load(Ordering::Acquire)
                .then(|| self.local_cursor_mode.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        cached_direct_client_value(
            self.r1_client.is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_scoreboard_open_ready
                .load(Ordering::Acquire)
                .then(|| self.local_scoreboard_open.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        cached_direct_client_value(
            self.r1_client.is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_dialog_active_ready
                .load(Ordering::Acquire)
                .then(|| self.local_dialog_active.load(Ordering::Acquire)),
        )
    }

    pub(super) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        self.server_info_snapshot
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .clone()
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.local_dialog_snapshot_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(self
            .local_dialog_snapshot
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .clone())
    }

    pub(super) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        cached_direct_client_value(
            self.r1_client.is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_chat_input_active_ready
                .load(Ordering::Acquire)
                .then(|| self.local_chat_input_active.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_chat_input_text(&self) -> Result<Vec<u8>, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.local_chat_input_text_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        self.local_chat_input_text
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .clone()
            .ok_or(DirectClientError::NotReady)
    }
}
