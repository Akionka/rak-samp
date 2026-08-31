//! Text-label command ABI entry points.

use super::{
    clone_initialized, copied_nul_free_string, direct_client_result, host, is_shutting_down,
    submit_direct_command,
};
use sdk_abi::limits::{
    MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABEL_TEXT_BYTES, MAX_SAMP_TEXT_LABELS, MAX_SAMP_VEHICLES,
};
use sdk_abi::{SampClientSdkCommandReceipt, SampClientSdkResult, Vector3};
use std::slice;

pub(super) unsafe extern "system" fn submit_delete_text_label(
    id: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_delete_text_label(id)) }
}

pub(super) unsafe extern "system" fn submit_set_text_label_text(
    id: u16,
    text: *const u8,
    text_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(text) =
        (unsafe { copied_nul_free_string(text, text_len, MAX_SAMP_TEXT_LABEL_TEXT_BYTES) })
    else {
        return SampClientSdkResult::InvalidArgument;
    };
    if text.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_set_text_label_text(id, text)
        })
    }
}

pub(super) unsafe extern "system" fn submit_create_text_label(
    id: u16,
    text: *const u8,
    text_len: usize,
    colour: u32,
    position: Vector3,
    draw_distance: f32,
    behind_walls: u8,
    attached_player_id: u16,
    attached_vehicle_id: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null()
        || id >= MAX_SAMP_TEXT_LABELS
        || text_len > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || text.is_null()
        || !position.x.is_finite()
        || !position.y.is_finite()
        || !position.z.is_finite()
        || !draw_distance.is_finite()
        || behind_walls > 1
        || (attached_player_id != u16::MAX && attached_player_id >= MAX_SAMP_PLAYERS)
        || (attached_vehicle_id != u16::MAX && attached_vehicle_id >= MAX_SAMP_VEHICLES)
    {
        return SampClientSdkResult::InvalidArgument;
    }
    let text = unsafe { slice::from_raw_parts(text, text_len) };
    if text.contains(&0) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_create_text_label(
                id,
                text.to_vec(),
                colour,
                crate::runtime::Vector3 {
                    x: position.x,
                    y: position.y,
                    z: position.z,
                },
                draw_distance,
                behind_walls != 0,
                attached_player_id,
                attached_vehicle_id,
            )
        })
    }
}

pub(super) unsafe extern "system" fn submit_create_text_label_auto(
    text: *const u8,
    text_len: usize,
    colour: u32,
    position: Vector3,
    draw_distance: f32,
    behind_walls: u8,
    attached_player_id: u16,
    attached_vehicle_id: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if is_shutting_down() {
        return SampClientSdkResult::ShuttingDown;
    }
    if receipt.is_null()
        || text.is_null()
        || text_len > MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || !position.x.is_finite()
        || !position.y.is_finite()
        || !position.z.is_finite()
        || !draw_distance.is_finite()
        || behind_walls > 1
        || (attached_player_id != u16::MAX && attached_player_id >= MAX_SAMP_PLAYERS)
        || (attached_vehicle_id != u16::MAX && attached_vehicle_id >= MAX_SAMP_VEHICLES)
    {
        return SampClientSdkResult::InvalidArgument;
    }
    let text = unsafe { slice::from_raw_parts(text, text_len) };
    if text.contains(&0) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_create_text_label_auto(
        text.to_vec(),
        colour,
        crate::runtime::Vector3 {
            x: position.x,
            y: position.y,
            z: position.z,
        },
        draw_distance,
        behind_walls != 0,
        attached_player_id,
        attached_vehicle_id,
    ) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

pub(super) unsafe extern "system" fn modkit_snapshot(
    id: u16,
    output: *mut modkit_abi::SampTextLabelV1,
) -> modkit_abi::ModResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    *output = modkit_abi::SampTextLabelV1::default();
    let mut legacy = sdk_abi::SampClientSdkTextLabelV1::default();
    let result = unsafe { super::snapshots::text_label_info(id, &mut legacy) };
    if result != SampClientSdkResult::Ok {
        return super::modkit::subscription_result(result);
    }
    if legacy.exists == 0 {
        return modkit_abi::MOD_NOT_FOUND;
    }
    let text_len = usize::from(legacy.text_len);
    if text_len > modkit_abi::SAMP_MAX_TEXT_LABEL_TEXT_BYTES {
        return modkit_abi::MOD_NATIVE_CALL_FAILED;
    }
    output.id = legacy.id;
    output.attached_player_id = legacy.attached_player_id;
    output.attached_vehicle_id = legacy.attached_vehicle_id;
    output.text_len = legacy.text_len;
    output.colour = legacy.colour;
    output.position = modkit_abi::SampVector3V1 {
        x: legacy.position.x,
        y: legacy.position.y,
        z: legacy.position.z,
    };
    output.draw_distance = legacy.draw_distance;
    output.behind_walls = legacy.behind_walls;
    output.text[..text_len].copy_from_slice(&legacy.text[..text_len]);
    modkit_abi::MOD_OK
}

pub(super) unsafe extern "system" fn modkit_submit_delete(
    id: u16,
    output: *mut modkit_abi::CommandReceiptId,
) -> modkit_abi::ModResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    *output = modkit_abi::CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe { submit_delete_text_label(id, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *output = modkit_abi::CommandReceiptId(legacy.id);
    }
    super::modkit::subscription_result(result)
}

pub(super) unsafe extern "system" fn modkit_submit_set_text(
    id: u16,
    text: *const u8,
    text_len: u32,
    output: *mut modkit_abi::CommandReceiptId,
) -> modkit_abi::ModResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    *output = modkit_abi::CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe { submit_set_text_label_text(id, text, text_len as usize, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *output = modkit_abi::CommandReceiptId(legacy.id);
    }
    super::modkit::subscription_result(result)
}

pub(super) unsafe extern "system" fn modkit_submit_create_at(
    id: u16,
    request: *const modkit_abi::SampTextLabelCreateV1,
    output: *mut modkit_abi::CommandReceiptId,
) -> modkit_abi::ModResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    *output = modkit_abi::CommandReceiptId(0);
    let Some(request) = (unsafe { request.as_ref() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe {
        submit_create_text_label(
            id,
            request.text,
            request.text_len as usize,
            request.colour,
            Vector3 {
                x: request.position.x,
                y: request.position.y,
                z: request.position.z,
            },
            request.draw_distance,
            request.behind_walls,
            request.attached_player_id,
            request.attached_vehicle_id,
            &mut legacy,
        )
    };
    if result == SampClientSdkResult::Ok {
        *output = modkit_abi::CommandReceiptId(legacy.id);
    }
    super::modkit::subscription_result(result)
}

pub(super) unsafe extern "system" fn modkit_submit_create(
    request: *const modkit_abi::SampTextLabelCreateV1,
    output: *mut modkit_abi::CommandReceiptId,
) -> modkit_abi::ModResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    *output = modkit_abi::CommandReceiptId(0);
    let Some(request) = (unsafe { request.as_ref() }) else {
        return modkit_abi::MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe {
        submit_create_text_label_auto(
            request.text,
            request.text_len as usize,
            request.colour,
            Vector3 {
                x: request.position.x,
                y: request.position.y,
                z: request.position.z,
            },
            request.draw_distance,
            request.behind_walls,
            request.attached_player_id,
            request.attached_vehicle_id,
            &mut legacy,
        )
    };
    if result == SampClientSdkResult::Ok {
        *output = modkit_abi::CommandReceiptId(legacy.id);
    }
    super::modkit::subscription_result(result)
}
