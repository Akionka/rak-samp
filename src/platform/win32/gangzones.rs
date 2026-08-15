//! Published gangzone pool reads.

use super::{BackendState, GangzoneCacheEntry, MAX_SAMP_GANGZONES, try_lock_direct};
use crate::runtime::{DirectClientError, GangzoneSnapshot};
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn gangzone(&self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        if self.scalar_profile().is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(id) >= MAX_SAMP_GANGZONES {
            return Err(DirectClientError::NotReady);
        }
        match try_lock_direct(&self.gangzone_cache)?
            .get(usize::from(id))
            .cloned()
            .ok_or(DirectClientError::NotReady)?
        {
            GangzoneCacheEntry::Known(snapshot) => {
                let _ = self.queue_gangzone_request(id);
                Ok(snapshot)
            }
            GangzoneCacheEntry::Unknown => {
                self.queue_gangzone_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
}
