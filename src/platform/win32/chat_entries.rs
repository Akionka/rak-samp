//! Published fixed chat-history entry reads.

use super::{BackendState, ChatEntryCacheEntry, MAX_CHAT_ENTRIES, try_lock_direct};
use crate::runtime::{ChatEntrySnapshot, DirectClientError};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn chat_entry(&self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        let index = usize::from(id);
        if index >= MAX_CHAT_ENTRIES {
            return Err(DirectClientError::NotReady);
        }
        match try_lock_direct(&self.chat_entry_cache)?
            .get(index)
            .cloned()
            .ok_or(DirectClientError::NotReady)?
        {
            ChatEntryCacheEntry::Known(snapshot) => {
                let _ = self.queue_chat_entry_request(id);
                Ok(snapshot)
            }
            ChatEntryCacheEntry::Unknown => {
                self.queue_chat_entry_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
}
