//! Cached local-dialog detail ABI reads.

use super::{
    clone_initialized, conversions, copied_nul_free_string, direct_client_result, host,
    submit_direct_command,
};
use crate::runtime::{LocalDialogRequest, LocalDialogStyle};
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkDialogSnapshotV1, SampClientSdkResult};

pub(super) unsafe extern "system" fn submit_local_dialog(
    id: u16,
    style: u32,
    title: *const u8,
    title_len: usize,
    text: *const u8,
    text_len: usize,
    button1: *const u8,
    button1_len: usize,
    button2: *const u8,
    button2_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(style) = LocalDialogStyle::from_raw(style) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(title) = (unsafe { copied_nul_free_string(title, title_len, 255) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 4_095) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(button1) = (unsafe { copied_nul_free_string(button1, button1_len, 255) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(button2) = (unsafe { copied_nul_free_string(button2, button2_len, 255) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_dialog(LocalDialogRequest {
                id,
                style,
                title,
                text,
                button1,
                button2,
            })
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_dialog_client_side(
    client_side: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(client_side, 0 | 1) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_dialog_client_side(client_side != 0)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_dialog_selected_item(
    selected: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_dialog_selected_item(selected)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_dialog_editbox_text(
    text: *const u8,
    text_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 128) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_dialog_editbox_text(text)
        })
    }
}

pub(super) unsafe extern "system" fn submit_local_dialog_close(
    button: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || button > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_local_dialog_close(button)) }
}

pub(super) unsafe extern "system" fn local_dialog_selected_item(
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_dialog_selected_item() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_dialog_list_item_count(
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_dialog_list_item_count() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn local_dialog_snapshot(
    output: *mut SampClientSdkDialogSnapshotV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_dialog_state() {
        Ok(Some(snapshot)) => match conversions::local_dialog_snapshot_to_abi(snapshot) {
            Ok(snapshot) => snapshot,
            Err(()) => return SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => SampClientSdkDialogSnapshotV1::default(),
        Err(error) => return direct_client_result(error),
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}
