//! Exact-version SA-MP pool mappings and gangzone service ABI.

use crate::{ModResult, ServiceHeader};

pub const SAMP_POOL_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_MAX_OBJECTS: u16 = 2_100;
pub const SAMP_MAX_PICKUPS: u16 = 4_096;
pub const SAMP_MAX_VEHICLES: u16 = 2_000;
pub const SAMP_MAX_GANGZONES: u16 = 1_024;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampGangzoneV1 {
    pub exists: u8,
    pub reserved: [u8; 3],
    pub id: u16,
    pub reserved2: u16,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub colour: u32,
    pub alt_colour: u32,
}

/// `ANY_THREAD + CALLBACK_SAFE`; all reads use game-thread-refreshed owned state.
#[repr(C)]
pub struct SampPoolServiceV1 {
    pub header: ServiceHeader,
    pub object_exists: unsafe extern "system" fn(id: u16, out: *mut u8) -> ModResult,
    pub vehicle_exists: unsafe extern "system" fn(id: u16, out: *mut u8) -> ModResult,
    pub object_handle: unsafe extern "system" fn(id: u16, out: *mut i32) -> ModResult,
    pub object_id_by_handle: unsafe extern "system" fn(handle: i32, out: *mut u16) -> ModResult,
    pub pickup_handle: unsafe extern "system" fn(id: u16, out: *mut i32) -> ModResult,
    pub pickup_id_by_handle: unsafe extern "system" fn(handle: i32, out: *mut u16) -> ModResult,
    pub vehicle_handle: unsafe extern "system" fn(id: u16, out: *mut i32) -> ModResult,
    pub vehicle_id_by_handle: unsafe extern "system" fn(handle: i32, out: *mut u16) -> ModResult,
    pub player_ped_handle: unsafe extern "system" fn(id: u16, out: *mut i32) -> ModResult,
    pub player_id_by_ped_handle: unsafe extern "system" fn(handle: i32, out: *mut u16) -> ModResult,
    pub gangzone: unsafe extern "system" fn(id: u16, out: *mut SampGangzoneV1) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_layout_is_header_plus_eleven_functions() {
        let pointer = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(SampPoolServiceV1, object_exists), 16);
        assert_eq!(core::mem::size_of::<SampPoolServiceV1>(), 16 + 11 * pointer);
    }
}
