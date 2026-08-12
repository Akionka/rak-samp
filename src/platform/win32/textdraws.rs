//! Published global and local textdraw pool reads.

use super::{
    BackendState, MAX_SAMP_TEXTDRAWS, TextdrawCacheEntry, TextdrawExistsCacheEntry, try_lock_direct,
};
use crate::runtime::{DirectClientError, TextdrawSnapshot};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn invalidate_textdraw_snapshot(&self, id: u16) {
        let mut cache = self
            .textdraw_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = cache.get_mut(usize::from(id)) {
            *entry = TextdrawCacheEntry::Unknown;
        }
    }

    pub(super) fn publish_created_textdraw(&self, id: u16) {
        let mut exists = self
            .textdraw_exists_cache
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(entry) = exists.get_mut(usize::from(id)) {
            *entry = TextdrawExistsCacheEntry::Known(true);
        }
        drop(exists);
        self.invalidate_textdraw_snapshot(id);
    }

    pub(super) fn textdraw_exists(&self, pool_index: u16) -> Result<bool, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(pool_index) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        match try_lock_direct(&self.textdraw_exists_cache)?
            .get(usize::from(pool_index))
            .cloned()
            .ok_or(DirectClientError::NotReady)?
        {
            TextdrawExistsCacheEntry::Known(exists) => {
                // Refresh opportunistically without making the cached read
                // fail if a busy plugin filled the bounded request queue.
                let _ = self.queue_textdraw_exists_request(pool_index);
                Ok(exists)
            }
            TextdrawExistsCacheEntry::Unknown => {
                self.queue_textdraw_exists_request(pool_index)?;
                Err(DirectClientError::NotReady)
            }
        }
    }

    pub(super) fn textdraw(
        &self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        if self.r1_client().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(pool_index) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        match try_lock_direct(&self.textdraw_cache)?
            .get(usize::from(pool_index))
            .cloned()
            .ok_or(DirectClientError::NotReady)?
        {
            TextdrawCacheEntry::Known(snapshot) => {
                let _ = self.queue_textdraw_request(pool_index);
                Ok(snapshot)
            }
            TextdrawCacheEntry::Unknown => {
                self.queue_textdraw_request(pool_index)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
}
