//! Textdraw command ABI entry points.

use super::{copied_nul_free_string, submit_direct_command};
use sdk_abi::limits::MAX_SAMP_TEXTDRAWS;
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_delete_textdraw(
    id: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_delete_textdraw(id)) }
}

pub(super) unsafe extern "system" fn submit_create_textdraw(
    id: u16,
    text: *const u8,
    text_len: usize,
    x: f32,
    y: f32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !x.is_finite() || !y.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 800) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_create_textdraw(id, text, x, y)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_position(
    id: u16,
    x: f32,
    y: f32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !x.is_finite() || !y.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_position(id, x, y)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_letter_style(
    id: u16,
    width: f32,
    height: f32,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_letter_style(id, width, height, colour)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_proportional(
    id: u16,
    proportional: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || proportional > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_proportional(id, proportional != 0)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_shadow(
    id: u16,
    shadow: u8,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_shadow(id, shadow, colour)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_outline(
    id: u16,
    outline: u8,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_outline(id, outline, colour)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_box(
    id: u16,
    enabled: u8,
    colour: u32,
    width: f32,
    height: f32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null()
        || id >= MAX_SAMP_TEXTDRAWS
        || enabled > 1
        || !width.is_finite()
        || !height.is_finite()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_box(id, enabled != 0, colour, width, height)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_alignment(
    id: u16,
    alignment: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !(1..=3).contains(&alignment) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_alignment(id, alignment)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_string(
    id: u16,
    text: *const u8,
    text_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 1_601) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_string(id, text)
        })
    }
}

pub(super) unsafe extern "system" fn submit_set_textdraw_model_style(
    id: u16,
    x: f32,
    y: f32,
    z: f32,
    zoom: f32,
    colour1: u16,
    colour2: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null()
        || id >= MAX_SAMP_TEXTDRAWS
        || !x.is_finite()
        || !y.is_finite()
        || !z.is_finite()
        || !zoom.is_finite()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_textdraw_model_style(
                id,
                crate::runtime::Vector3 { x, y, z },
                zoom,
                colour1,
                colour2,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_textdraw_rejects_invalid_abi_inputs() {
        let mut receipt = SampClientSdkCommandReceipt::default();
        assert_eq!(
            unsafe {
                submit_create_textdraw(7, std::ptr::null(), 0, 1.0, 2.0, std::ptr::null_mut())
            },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe {
                submit_create_textdraw(
                    MAX_SAMP_TEXTDRAWS,
                    std::ptr::null(),
                    0,
                    1.0,
                    2.0,
                    &mut receipt,
                )
            },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { submit_create_textdraw(7, std::ptr::null(), 0, f32::NAN, 2.0, &mut receipt) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { submit_create_textdraw(7, b"bad\0text".as_ptr(), 8, 1.0, 2.0, &mut receipt) },
            SampClientSdkResult::InvalidArgument
        );
        let too_long = [b'x'; 801];
        assert_eq!(
            unsafe {
                submit_create_textdraw(7, too_long.as_ptr(), too_long.len(), 1.0, 2.0, &mut receipt)
            },
            SampClientSdkResult::InvalidArgument
        );
    }
}
