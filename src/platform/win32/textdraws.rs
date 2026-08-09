//! Published global and local textdraw pool reads.

use super::{BackendState, MAX_SAMP_TEXTDRAWS, TextdrawCacheEntry, TextdrawExistsCacheEntry};
use crate::runtime::{DirectClientError, TextdrawSnapshot};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn textdraw_exists(&self, pool_index: u16) -> Result<bool, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(pool_index) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        match self
            .textdraw_exists_cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
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
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(pool_index) >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        match self
            .textdraw_cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
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
