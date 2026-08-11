//! Published 3D text-label pool reads.

use super::{
    BackendState, MAX_SAMP_TEXT_LABELS, TextLabelCacheEntry, TextLabelExistsCacheEntry,
    try_lock_direct,
};
use crate::runtime::{DirectClientError, TextLabelSnapshot};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn publish_created_text_label(&self, id: u16, snapshot: TextLabelSnapshot) {
        let mut exists = self
            .text_label_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = exists.get_mut(usize::from(id)) {
            *entry = TextLabelExistsCacheEntry::Known(true);
        }
        drop(exists);
        let mut cache = self
            .text_label_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = cache.get_mut(usize::from(id)) {
            *entry = TextLabelCacheEntry::Known(Some(snapshot));
        }
    }

    pub(super) fn text_label_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(id) >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        match try_lock_direct(&self.text_label_exists_cache)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?
        {
            TextLabelExistsCacheEntry::Known(exists) => {
                // Refresh opportunistically without making the cached read
                // fail if a busy plugin filled the bounded request queue.
                let _ = self.queue_text_label_exists_request(id);
                Ok(exists)
            }
            TextLabelExistsCacheEntry::Unknown => {
                self.queue_text_label_exists_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn text_label(
        &self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(id) >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        match try_lock_direct(&self.text_label_cache)?
            .get(usize::from(id))
            .cloned()
            .ok_or(DirectClientError::NotReady)?
        {
            TextLabelCacheEntry::Known(snapshot) => {
                let _ = self.queue_text_label_request(id);
                Ok(snapshot)
            }
            TextLabelCacheEntry::Unknown => {
                self.queue_text_label_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
}
