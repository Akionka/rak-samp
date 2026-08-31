//! Adapters from the exact-version textdraw service to the frozen legacy host implementation.

use modkit_abi::{CommandReceiptId, MOD_INVALID_ARGUMENT, ModResult, SampTextdrawV1};
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult, SampClientSdkTextDrawV1};

use super::modkit::subscription_result;

fn submit_with_receipt(
    out: *mut CommandReceiptId,
    submit: impl FnOnce(*mut SampClientSdkCommandReceipt) -> SampClientSdkResult,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = submit(&mut legacy);
    if result == SampClientSdkResult::Ok {
        *out = CommandReceiptId(legacy.id);
    }
    subscription_result(result)
}

pub(super) unsafe extern "system" fn exists(id: u16, out: *mut u8) -> ModResult {
    subscription_result(unsafe { super::pools::textdraw_exists(id, out) })
}

pub(super) unsafe extern "system" fn snapshot(id: u16, out: *mut SampTextdrawV1) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkTextDrawV1::default();
    let result = unsafe { super::snapshots::textdraw_info(id, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = unsafe { core::mem::transmute::<SampClientSdkTextDrawV1, SampTextdrawV1>(legacy) };
    }
    subscription_result(result)
}

pub(super) unsafe extern "system" fn submit_create(
    id: u16,
    text: *const u8,
    text_len: u32,
    x: f32,
    y: f32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_create_textdraw(id, text, text_len as usize, x, y, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_delete(
    id: u16,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_delete_textdraw(id, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_position(
    id: u16,
    x: f32,
    y: f32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_position(id, x, y, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_style(
    id: u16,
    style: i32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_style(id, style, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_letter_style(
    id: u16,
    width: f32,
    height: f32,
    colour: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_letter_style(id, width, height, colour, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_proportional(
    id: u16,
    proportional: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_proportional(id, proportional, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_shadow(
    id: u16,
    shadow: u8,
    colour: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_shadow(id, shadow, colour, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_outline(
    id: u16,
    outline: u8,
    colour: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_outline(id, outline, colour, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_box(
    id: u16,
    enabled: u8,
    colour: u32,
    width: f32,
    height: f32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_box(id, enabled, colour, width, height, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_alignment(
    id: u16,
    alignment: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_alignment(id, alignment, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_text(
    id: u16,
    text: *const u8,
    text_len: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_string(id, text, text_len as usize, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_set_model_style(
    id: u16,
    x: f32,
    y: f32,
    z: f32,
    zoom: f32,
    colour1: u16,
    colour2: u16,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::textdraws::submit_set_textdraw_model_style(
            id, x, y, z, zoom, colour1, colour2, receipt,
        )
    })
}
