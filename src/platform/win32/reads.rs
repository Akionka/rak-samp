//! Reads from game-thread-published scalar and snapshot caches.

use super::{BackendState, cached_direct_client_value, try_lock_direct};
use crate::runtime::{
    AnimationSnapshot, DirectClientError, LocalDialogResponseSnapshot, LocalDialogSnapshot,
    ServerInfoSnapshot,
};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn samp_game_state(&self) -> Result<i32, DirectClientError> {
        cached_direct_client_value(
            self.r1_client().is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.samp_game_state_ready
                .load(Ordering::Acquire)
                .then(|| self.samp_game_state.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_chat_display_mode(&self) -> Result<i32, DirectClientError> {
        cached_direct_client_value(
            self.r1_client().is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_chat_display_mode_ready
                .load(Ordering::Acquire)
                .then(|| self.local_chat_display_mode.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_cursor_mode(&self) -> Result<i32, DirectClientError> {
        cached_direct_client_value(
            self.r1_client().is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_cursor_mode_ready
                .load(Ordering::Acquire)
                .then(|| self.local_cursor_mode.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_scoreboard_open(&self) -> Result<bool, DirectClientError> {
        cached_direct_client_value(
            self.r1_client().is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_scoreboard_open_ready
                .load(Ordering::Acquire)
                .then(|| self.local_scoreboard_open.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_dialog_active(&self) -> Result<bool, DirectClientError> {
        cached_direct_client_value(
            self.r1_client().is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_dialog_active_ready
                .load(Ordering::Acquire)
                .then(|| self.local_dialog_active.load(Ordering::Acquire)),
        )
    }

    pub(super) fn server_info(&self) -> Result<ServerInfoSnapshot, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        try_lock_direct(&self.server_info_snapshot)?
            .clone()
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn local_dialog_state(
        &self,
    ) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.local_dialog_snapshot_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(try_lock_direct(&self.local_dialog_snapshot)?.clone())
    }

    pub(super) fn take_local_dialog_response(
        &self,
    ) -> Result<Option<LocalDialogResponseSnapshot>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 {
            return Err(DirectClientError::NotReady);
        }
        Ok(try_lock_direct(&self.local_dialog_response)?.take())
    }

    pub(super) fn local_chat_input_active(&self) -> Result<bool, DirectClientError> {
        cached_direct_client_value(
            self.r1_client().is_some(),
            self.rak_client.load(Ordering::Acquire) != 0,
            self.cache_is_published(),
            self.local_chat_input_active_ready
                .load(Ordering::Acquire)
                .then(|| self.local_chat_input_active.load(Ordering::Acquire)),
        )
    }

    pub(super) fn local_chat_input_text(&self) -> Result<Vec<u8>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.local_chat_input_text_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        try_lock_direct(&self.local_chat_input_text)?
            .clone()
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn local_chat_command_defined(
        &self,
        name: &[u8],
    ) -> Result<bool, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0
            || !self.cache_is_published()
            || !self.local_chat_input_commands_ready.load(Ordering::Acquire)
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(try_lock_direct(&self.local_chat_input_commands)?
            .as_ref()
            .ok_or(DirectClientError::NotReady)?
            .iter()
            .any(|candidate| candidate == name))
    }

    pub(super) fn local_animation(&self, id: u16) -> Result<AnimationSnapshot, DirectClientError> {
        self.animation_catalog().and_then(|catalog| {
            catalog
                .get(usize::from(id))
                .cloned()
                .ok_or(DirectClientError::NotReady)
        })
    }

    pub(super) fn local_animation_id(
        &self,
        name: &[u8],
        file: &[u8],
    ) -> Result<Option<u16>, DirectClientError> {
        let catalog = self.animation_catalog()?;
        Ok(catalog
            .iter()
            .position(|entry| entry.name == name && entry.file == file)
            .and_then(|index| u16::try_from(index).ok()))
    }

    pub(super) fn animation_catalog(&self) -> Result<Vec<AnimationSnapshot>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        try_lock_direct(&self.animation_catalog)?
            .clone()
            .ok_or(DirectClientError::NotReady)
    }
}
