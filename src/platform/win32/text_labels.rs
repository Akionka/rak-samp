//! Published 3D text-label pool reads.

use super::{BackendState, MAX_SAMP_TEXT_LABELS, TextLabelCacheEntry, TextLabelExistsCacheEntry};
use crate::runtime::{DirectClientError, TextLabelSnapshot};
use std::sync::atomic::Ordering;

impl BackendState {
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
        match self
            .text_label_exists_cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
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
        match self
            .text_label_cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
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
