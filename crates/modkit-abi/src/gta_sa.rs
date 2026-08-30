//! GTA San Andreas service v1 ABI.

use core::ffi::c_void;

use crate::{CommandReceiptId, GameContextTokenV1, ModResult, ServiceHeader, SubscriptionId};

pub const GTA_SA_SERVICE_VERSION_V1: u32 = 1;

/// Plugin callback invoked during a validated post-`CGame::Process` scope.
pub type GtaTickCallbackV1 =
    unsafe extern "system" fn(user_data: *mut c_void, context: GameContextTokenV1);

/// Plugin callback used to reclaim one opaque tick-registration context.
pub type GtaReleaseCallbackV1 = unsafe extern "system" fn(user_data: *mut c_void);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GtaVector3V1 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GtaEntitySnapshotV1 {
    pub position: GtaVector3V1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GtaPedSnapshotV1 {
    pub handle: i32,
    pub reserved: u32,
    pub entity: GtaEntitySnapshotV1,
    pub health: f32,
    pub armour: f32,
}

/// Immutable exact-version GTA SA service table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GtaSaServiceV1 {
    pub header: ServiceHeader,
    /// `ANY_THREAD + CALLBACK_SAFE`; successful registration transfers context ownership.
    pub register_tick: unsafe extern "system" fn(
        callback: Option<GtaTickCallbackV1>,
        user_data: *mut c_void,
        release: Option<GtaReleaseCallbackV1>,
        out_subscription: *mut SubscriptionId,
    ) -> ModResult,
    /// `GAME_THREAD_ONLY + CALLBACK_SAFE`; `POST_GAME_PROCESS_ONLY`.
    pub local_ped_snapshot: unsafe extern "system" fn(
        context: GameContextTokenV1,
        out: *mut GtaPedSnapshotV1,
    ) -> ModResult,
    /// `GAME_THREAD_ONLY + CALLBACK_SAFE`; `POST_GAME_PROCESS_ONLY`.
    pub teleport_local_ped: unsafe extern "system" fn(
        context: GameContextTokenV1,
        destination: GtaVector3V1,
    ) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; queues one owned compound read.
    pub submit_local_ped_snapshot:
        unsafe extern "system" fn(out_receipt: *mut CommandReceiptId) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; call after successful Core receipt completion.
    pub take_local_ped_snapshot: unsafe extern "system" fn(
        receipt: CommandReceiptId,
        out: *mut GtaPedSnapshotV1,
    ) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; queues one copied mutation.
    pub submit_teleport_local_ped: unsafe extern "system" fn(
        destination: GtaVector3V1,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gta_values_have_fixed_layouts() {
        assert_eq!(core::mem::size_of::<GtaVector3V1>(), 12);
        assert_eq!(core::mem::size_of::<GtaEntitySnapshotV1>(), 12);
        assert_eq!(core::mem::size_of::<GtaPedSnapshotV1>(), 28);
        assert_eq!(core::mem::offset_of!(GtaPedSnapshotV1, entity), 8);
    }

    #[test]
    fn gta_service_layout_is_fixed() {
        let pointer_size = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(GtaSaServiceV1, header), 0);
        assert_eq!(core::mem::offset_of!(GtaSaServiceV1, register_tick), 16);
        assert_eq!(
            core::mem::size_of::<GtaSaServiceV1>(),
            16 + 6 * pointer_size
        );
    }
}
