//! Published object pool reads.

use super::{BackendState, MAX_SAMP_OBJECTS, ObjectExistsCacheEntry};
use crate::runtime::DirectClientError;
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn object_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(id) >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        match self
            .object_exists_cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?
        {
            ObjectExistsCacheEntry::Known(exists) => {
                let _ = self.queue_object_exists_request(id);
                Ok(exists)
            }
            ObjectExistsCacheEntry::Unknown => {
                self.queue_object_exists_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
}
