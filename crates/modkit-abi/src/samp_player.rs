//! Exact-version SA-MP player, synchronization, and animation service ABI.

use crate::{CommandReceiptId, ModResult, SampVector3V1, ServiceHeader};

pub const SAMP_PLAYER_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_MAX_ANIMATION_NAME_BYTES: usize = 36;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampRemotePlayerStateV1 {
    pub exists: u8,
    pub special_action: u8,
    pub reserved: u16,
    pub id: u16,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampStreamedOutPlayerPositionV1 {
    pub exists: u8,
    pub reserved: [u8; 3],
    pub position: SampVector3V1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampOnFootSyncV1 {
    pub exists: u8,
    pub health: u8,
    pub armour: u8,
    pub weapon: u8,
    pub special_action: u8,
    pub reserved: [u8; 3],
    pub id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub reserved2: u16,
    pub position: SampVector3V1,
    pub quaternion: [f32; 4],
    pub speed: SampVector3V1,
    pub surfing_offset: SampVector3V1,
    pub surfing_vehicle_id: u16,
    pub reserved3: u16,
    pub animation: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampInCarSyncV1 {
    pub exists: u8,
    pub driver_health: u8,
    pub driver_armour: u8,
    pub weapon: u8,
    pub siren: u8,
    pub landing_gear: u8,
    pub reserved: [u8; 2],
    pub id: u16,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub reserved2: u16,
    pub quaternion: [f32; 4],
    pub position: SampVector3V1,
    pub speed: SampVector3V1,
    pub vehicle_health: f32,
    pub trailer_id: u16,
    pub vehicle_specific: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampPassengerSyncV1 {
    pub exists: u8,
    pub seat_id: u8,
    pub weapon: u8,
    pub health: u8,
    pub armour: u8,
    pub reserved: [u8; 3],
    pub id: u16,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub reserved2: u16,
    pub position: SampVector3V1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampTrailerSyncV1 {
    pub exists: u8,
    pub reserved: [u8; 3],
    pub id: u16,
    pub trailer_id: u16,
    pub position: SampVector3V1,
    pub quaternion: [f32; 4],
    pub speed: SampVector3V1,
    pub turn_speed: SampVector3V1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampAimSyncV1 {
    pub exists: u8,
    pub camera_mode: u8,
    pub zoom_and_weapon_state: u8,
    pub aspect_ratio: u8,
    pub id: u16,
    pub reserved: u16,
    pub aim_first: SampVector3V1,
    pub aim_position: SampVector3V1,
    pub aim_z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampAnimationV1 {
    pub name_len: u8,
    pub file_len: u8,
    pub name: [u8; SAMP_MAX_ANIMATION_NAME_BYTES],
    pub file: [u8; SAMP_MAX_ANIMATION_NAME_BYTES],
}

impl Default for SampAnimationV1 {
    fn default() -> Self {
        Self {
            name_len: 0,
            file_len: 0,
            name: [0; SAMP_MAX_ANIMATION_NAME_BYTES],
            file: [0; SAMP_MAX_ANIMATION_NAME_BYTES],
        }
    }
}

/// `ANY_THREAD + CALLBACK_SAFE`; submissions are non-blocking and return Core receipts.
#[repr(C)]
pub struct SampPlayerServiceV1 {
    pub header: ServiceHeader,
    pub remote_state:
        unsafe extern "system" fn(id: u16, out: *mut SampRemotePlayerStateV1) -> ModResult,
    pub streamed_out_position:
        unsafe extern "system" fn(id: u16, out: *mut SampStreamedOutPlayerPositionV1) -> ModResult,
    pub onfoot_sync: unsafe extern "system" fn(id: u16, out: *mut SampOnFootSyncV1) -> ModResult,
    pub vehicle_sync: unsafe extern "system" fn(id: u16, out: *mut SampInCarSyncV1) -> ModResult,
    pub passenger_sync:
        unsafe extern "system" fn(id: u16, out: *mut SampPassengerSyncV1) -> ModResult,
    pub trailer_sync: unsafe extern "system" fn(id: u16, out: *mut SampTrailerSyncV1) -> ModResult,
    pub aim_sync: unsafe extern "system" fn(id: u16, out: *mut SampAimSyncV1) -> ModResult,
    pub player_defined: unsafe extern "system" fn(id: u16, out: *mut u8) -> ModResult,
    pub player_paused: unsafe extern "system" fn(id: u16, out: *mut u8) -> ModResult,
    pub player_count: unsafe extern "system" fn(include_npcs: u8, out: *mut u16) -> ModResult,
    pub player_max_id: unsafe extern "system" fn(out: *mut u16) -> ModResult,
    pub animation: unsafe extern "system" fn(id: u16, out: *mut SampAnimationV1) -> ModResult,
    pub animation_id: unsafe extern "system" fn(
        name: *const u8,
        name_len: u32,
        file: *const u8,
        file_len: u32,
        out: *mut i32,
    ) -> ModResult,
    pub submit_spawn: unsafe extern "system" fn(out: *mut CommandReceiptId) -> ModResult,
    pub submit_special_action:
        unsafe extern "system" fn(action: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_name: unsafe extern "system" fn(
        name: *const u8,
        name_len: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_colour:
        unsafe extern "system" fn(id: u16, colour: u32, out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_unoccupied_sync:
        unsafe extern "system" fn(vehicle: u16, seat: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_aim_sync: unsafe extern "system" fn(out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_onfoot_sync:
        unsafe extern "system" fn(out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_stats_sync: unsafe extern "system" fn(out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_trailer_sync:
        unsafe extern "system" fn(trailer: u16, out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_vehicle_sync:
        unsafe extern "system" fn(vehicle: u16, out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_passenger_sync:
        unsafe extern "system" fn(vehicle: u16, seat: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_force_weapons_sync:
        unsafe extern "system" fn(out: *mut CommandReceiptId) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_layout_is_header_plus_twenty_five_functions() {
        let pointer = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(SampPlayerServiceV1, remote_state), 16);
        assert_eq!(
            core::mem::size_of::<SampPlayerServiceV1>(),
            16 + 25 * pointer
        );
    }
}
