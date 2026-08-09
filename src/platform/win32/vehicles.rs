//! Published vehicle pool reads.

use super::{BackendState, MAX_SAMP_VEHICLES, VehicleExistsCacheEntry};
use crate::runtime::DirectClientError;
use std::sync::atomic::Ordering;

impl BackendState {
    pub(super) fn vehicle_exists(&self, id: u16) -> Result<bool, DirectClientError> {
        if self.r1_client.is_none() {
            return Err(DirectClientError::UnsupportedVersion);
        }
        if self.rak_client.load(Ordering::Acquire) == 0 || !self.cache_is_published() {
            return Err(DirectClientError::NotReady);
        }
        if usize::from(id) >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        match self
            .vehicle_exists_cache
            .try_lock()
            .map_err(|_| DirectClientError::NotReady)?
            .get(usize::from(id))
            .copied()
            .ok_or(DirectClientError::NotReady)?
        {
            VehicleExistsCacheEntry::Known(exists) => {
                // Refresh opportunistically without making the cached read
                // fail if a busy plugin filled the bounded request queue.
                let _ = self.queue_vehicle_exists_request(id);
                Ok(exists)
            }
            VehicleExistsCacheEntry::Unknown => {
                self.queue_vehicle_exists_request(id)?;
                Err(DirectClientError::NotReady)
            }
        }
    }
}
