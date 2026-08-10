//! Cached object, pickup, vehicle, and player-handle `HostApi` wrappers.

use crate::{HostApi, SampClientSdkResult};

impl HostApi {
    /// Returns the cached R1 object GTAREF for an object-pool ID.
    pub fn object_handle(self, id: u16) -> Result<Option<i32>, SampClientSdkResult> {
        let mut output = 0;
        match unsafe { (self.raw.local_object_handle)(id, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != 0).then_some(output)),
            error => Err(error),
        }
    }

    /// Resolves the cached R1 object-pool ID for a GTAREF.
    pub fn object_id_by_handle(self, handle: i32) -> Result<Option<u16>, SampClientSdkResult> {
        let mut output = u16::MAX;
        match unsafe { (self.raw.local_object_id_by_handle)(handle, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != u16::MAX).then_some(output)),
            error => Err(error),
        }
    }

    /// Returns the cached R1 pickup GTAREF for a pickup-pool ID.
    pub fn pickup_handle(self, id: u16) -> Result<Option<i32>, SampClientSdkResult> {
        let mut output = 0;
        match unsafe { (self.raw.local_pickup_handle)(id, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != 0).then_some(output)),
            error => Err(error),
        }
    }

    /// Resolves the cached R1 pickup-pool ID for a GTAREF.
    pub fn pickup_id_by_handle(self, handle: i32) -> Result<Option<u16>, SampClientSdkResult> {
        let mut output = u16::MAX;
        match unsafe { (self.raw.local_pickup_id_by_handle)(handle, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != u16::MAX).then_some(output)),
            error => Err(error),
        }
    }

    /// Returns the cached R1 vehicle GTA handle for a vehicle-pool ID.
    pub fn vehicle_handle(self, id: u16) -> Result<Option<i32>, SampClientSdkResult> {
        let mut output = 0;
        match unsafe { (self.raw.local_vehicle_handle)(id, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != 0).then_some(output)),
            error => Err(error),
        }
    }

    /// Resolves the cached R1 vehicle-pool ID for a GTA handle.
    pub fn vehicle_id_by_handle(self, handle: i32) -> Result<Option<u16>, SampClientSdkResult> {
        let mut output = u16::MAX;
        match unsafe { (self.raw.local_vehicle_id_by_handle)(handle, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != u16::MAX).then_some(output)),
            error => Err(error),
        }
    }

    /// Returns the cached R1 player GTA ped handle for a player-pool ID.
    pub fn player_ped_handle(self, id: u16) -> Result<Option<i32>, SampClientSdkResult> {
        let mut output = 0;
        match unsafe { (self.raw.local_player_ped_handle)(id, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != 0).then_some(output)),
            error => Err(error),
        }
    }

    /// Resolves the cached R1 player-pool ID for a GTA ped handle.
    pub fn player_id_by_ped_handle(self, handle: i32) -> Result<Option<u16>, SampClientSdkResult> {
        let mut output = u16::MAX;
        match unsafe { (self.raw.local_player_id_by_ped_handle)(handle, &mut output) } {
            SampClientSdkResult::Ok => Ok((output != u16::MAX).then_some(output)),
            error => Err(error),
        }
    }
}
