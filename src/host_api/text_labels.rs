//! Text-label command ABI entry points.

use super::{clone_initialized, direct_client_result, host, submit_direct_command};
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
