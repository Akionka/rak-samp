//! Exact-version SA-MP text-label service ABI.

use crate::{CommandReceiptId, ModResult, SampVector3V1, ServiceHeader};

pub const SAMP_TEXT_LABEL_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_MAX_TEXT_LABELS: u16 = 2_048;
pub const SAMP_MAX_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
pub const SAMP_TEXT_LABEL_NO_ATTACHMENT: u16 = u16::MAX;

/// Owned fixed-capacity text-label snapshot.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampTextLabelV1 {
    pub id: u16,
    pub attached_player_id: u16,
    pub attached_vehicle_id: u16,
    pub text_len: u16,
    pub colour: u32,
    pub position: SampVector3V1,
    pub draw_distance: f32,
    pub behind_walls: u8,
    pub reserved: [u8; 3],
    pub text: [u8; SAMP_MAX_TEXT_LABEL_TEXT_BYTES],
}

impl Default for SampTextLabelV1 {
    fn default() -> Self {
        Self {
            id: 0,
            attached_player_id: SAMP_TEXT_LABEL_NO_ATTACHMENT,
            attached_vehicle_id: SAMP_TEXT_LABEL_NO_ATTACHMENT,
            text_len: 0,
            colour: 0,
            position: SampVector3V1::default(),
            draw_distance: 0.0,
            behind_walls: 0,
            reserved: [0; 3],
            text: [0; SAMP_MAX_TEXT_LABEL_TEXT_BYTES],
        }
    }
}

/// Borrowed creation arguments. The Host copies `text` before returning.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SampTextLabelCreateV1 {
    pub text: *const u8,
    pub text_len: u32,
    pub colour: u32,
    pub position: SampVector3V1,
    pub draw_distance: f32,
    pub attached_player_id: u16,
    pub attached_vehicle_id: u16,
    pub behind_walls: u8,
    pub reserved: [u8; 3],
}

/// `ANY_THREAD + CALLBACK_SAFE`; submissions are non-blocking and return Core receipts.
#[repr(C)]
pub struct SampTextLabelServiceV1 {
    pub header: ServiceHeader,
    pub snapshot: unsafe extern "system" fn(id: u16, out: *mut SampTextLabelV1) -> ModResult,
    pub submit_delete:
        unsafe extern "system" fn(id: u16, out_receipt: *mut CommandReceiptId) -> ModResult,
    pub submit_set_text: unsafe extern "system" fn(
        id: u16,
        text: *const u8,
        text_len: u32,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_create_at: unsafe extern "system" fn(
        id: u16,
        request: *const SampTextLabelCreateV1,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    /// On successful completion, Core `CommandCompletionV1.value0` contains the created ID.
    pub submit_create: unsafe extern "system" fn(
        request: *const SampTextLabelCreateV1,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_uses_fixed_owned_storage() {
        assert_eq!(core::mem::align_of::<SampTextLabelV1>(), 4);
        assert_eq!(SampTextLabelV1::default().text.len(), 4_095);
        assert_eq!(SampTextLabelV1::default().attached_player_id, u16::MAX);
    }

    #[test]
    fn service_layout_is_header_plus_five_functions() {
        let pointer = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(SampTextLabelServiceV1, snapshot), 16);
        assert_eq!(
            core::mem::size_of::<SampTextLabelServiceV1>(),
            16 + 5 * pointer
        );
    }
}
