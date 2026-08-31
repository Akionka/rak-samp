//! Exact-version SA-MP textdraw service ABI.

use crate::{CommandReceiptId, ModResult, SampVector3V1, ServiceHeader};

pub const SAMP_TEXTDRAW_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_MAX_TEXTDRAWS: u16 = 2_304;
pub const SAMP_MAX_TEXTDRAW_TEXT_BYTES: usize = 1_601;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampTextdrawV1 {
    pub exists: u8,
    pub proportional: u8,
    pub align_left: u8,
    pub align_center: u8,
    pub align_right: u8,
    pub box_enabled: u8,
    pub reserved: [u8; 2],
    pub pool_index: u16,
    pub shadow: u8,
    pub outline: u8,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_colour: u32,
    pub x: f32,
    pub y: f32,
    pub background_colour: u32,
    pub style: i32,
    pub box_width: f32,
    pub box_height: f32,
    pub box_colour: u32,
    pub model_id: u16,
    pub reserved2: u16,
    pub rotation: SampVector3V1,
    pub zoom: f32,
    pub model_colour1: u16,
    pub model_colour2: u16,
    pub text_len: u16,
    pub reserved3: [u8; 2],
    pub text: [u8; SAMP_MAX_TEXTDRAW_TEXT_BYTES],
}

impl Default for SampTextdrawV1 {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}

/// `ANY_THREAD + CALLBACK_SAFE`; submissions are non-blocking and return Core receipts.
#[repr(C)]
pub struct SampTextdrawServiceV1 {
    pub header: ServiceHeader,
    pub exists: unsafe extern "system" fn(id: u16, out: *mut u8) -> ModResult,
    pub snapshot: unsafe extern "system" fn(id: u16, out: *mut SampTextdrawV1) -> ModResult,
    pub submit_create: unsafe extern "system" fn(
        id: u16,
        text: *const u8,
        text_len: u32,
        x: f32,
        y: f32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_delete: unsafe extern "system" fn(id: u16, out: *mut CommandReceiptId) -> ModResult,
    pub submit_set_position:
        unsafe extern "system" fn(id: u16, x: f32, y: f32, out: *mut CommandReceiptId) -> ModResult,
    pub submit_set_style:
        unsafe extern "system" fn(id: u16, style: i32, out: *mut CommandReceiptId) -> ModResult,
    pub submit_set_letter_style: unsafe extern "system" fn(
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_set_proportional: unsafe extern "system" fn(
        id: u16,
        proportional: u8,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_set_shadow: unsafe extern "system" fn(
        id: u16,
        shadow: u8,
        colour: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_set_outline: unsafe extern "system" fn(
        id: u16,
        outline: u8,
        colour: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_set_box: unsafe extern "system" fn(
        id: u16,
        enabled: u8,
        colour: u32,
        width: f32,
        height: f32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_set_alignment:
        unsafe extern "system" fn(id: u16, alignment: u8, out: *mut CommandReceiptId) -> ModResult,
    pub submit_set_text: unsafe extern "system" fn(
        id: u16,
        text: *const u8,
        text_len: u32,
        out: *mut CommandReceiptId,
    ) -> ModResult,
    pub submit_set_model_style: unsafe extern "system" fn(
        id: u16,
        x: f32,
        y: f32,
        z: f32,
        zoom: f32,
        colour1: u16,
        colour2: u16,
        out: *mut CommandReceiptId,
    ) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_layout_is_header_plus_fourteen_functions() {
        let pointer = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(SampTextdrawServiceV1, exists), 16);
        assert_eq!(
            core::mem::size_of::<SampTextdrawServiceV1>(),
            16 + 14 * pointer
        );
    }
}
