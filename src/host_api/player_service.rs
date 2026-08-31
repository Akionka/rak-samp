//! Adapters from the exact-version player service to the frozen legacy host implementation.

use modkit_abi::{
    CommandReceiptId, MOD_INVALID_ARGUMENT, ModResult, SampAimSyncV1, SampAnimationV1,
    SampInCarSyncV1, SampOnFootSyncV1, SampPassengerSyncV1, SampRemotePlayerStateV1,
    SampStreamedOutPlayerPositionV1, SampTrailerSyncV1,
};
use sdk_abi::{
    SampClientSdkAimSyncV1, SampClientSdkAnimationV1, SampClientSdkCommandReceipt,
    SampClientSdkInCarSyncV1, SampClientSdkOnFootSyncV1, SampClientSdkPassengerSyncV1,
    SampClientSdkRemotePlayerStateV1, SampClientSdkResult,
    SampClientSdkStreamedOutPlayerPositionV1, SampClientSdkTrailerSyncV1,
};

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

macro_rules! copied_output {
    ($name:ident, $legacy:ty, $new:ty, $target:path) => {
        pub(super) unsafe extern "system" fn $name(id: u16, out: *mut $new) -> ModResult {
            let Some(out) = (unsafe { out.as_mut() }) else {
                return MOD_INVALID_ARGUMENT;
            };
            let mut legacy = <$legacy>::default();
            let result = unsafe { $target(id, &mut legacy) };
            if result == SampClientSdkResult::Ok {
                // Both exact-version values deliberately copy the frozen C layout field-for-field.
                *out = unsafe { core::mem::transmute::<$legacy, $new>(legacy) };
            }
            subscription_result(result)
        }
    };
}

copied_output!(
    remote_state,
    SampClientSdkRemotePlayerStateV1,
    SampRemotePlayerStateV1,
    super::players::remote_player_state
);
copied_output!(
    streamed_out_position,
    SampClientSdkStreamedOutPlayerPositionV1,
    SampStreamedOutPlayerPositionV1,
    super::players::streamed_out_player_position
);
copied_output!(
    onfoot_sync,
    SampClientSdkOnFootSyncV1,
    SampOnFootSyncV1,
    super::players::onfoot_sync
);
copied_output!(
    vehicle_sync,
    SampClientSdkInCarSyncV1,
    SampInCarSyncV1,
    super::players::vehicle_sync
);
copied_output!(
    passenger_sync,
    SampClientSdkPassengerSyncV1,
    SampPassengerSyncV1,
    super::players::passenger_sync
);
copied_output!(
    trailer_sync,
    SampClientSdkTrailerSyncV1,
    SampTrailerSyncV1,
    super::players::trailer_sync
);
copied_output!(
    aim_sync,
    SampClientSdkAimSyncV1,
    SampAimSyncV1,
    super::players::aim_sync
);

pub(super) unsafe extern "system" fn player_defined(id: u16, out: *mut u8) -> ModResult {
    subscription_result(unsafe { super::players::player_defined(id, out) })
}

pub(super) unsafe extern "system" fn player_paused(id: u16, out: *mut u8) -> ModResult {
    subscription_result(unsafe { super::players::player_paused(id, out) })
}

pub(super) unsafe extern "system" fn player_count(include_npcs: u8, out: *mut u16) -> ModResult {
    subscription_result(unsafe { super::players::player_count(include_npcs, out) })
}

pub(super) unsafe extern "system" fn player_max_id(out: *mut u16) -> ModResult {
    subscription_result(unsafe { super::players::player_max_id(out) })
}

pub(super) unsafe extern "system" fn animation(id: u16, out: *mut SampAnimationV1) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    let mut legacy = SampClientSdkAnimationV1::default();
    let result = unsafe { super::animations::local_animation(id, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = unsafe { core::mem::transmute::<SampClientSdkAnimationV1, SampAnimationV1>(legacy) };
    }
    subscription_result(result)
}

pub(super) unsafe extern "system" fn animation_id(
    name: *const u8,
    name_len: u32,
    file: *const u8,
    file_len: u32,
    out: *mut i32,
) -> ModResult {
    subscription_result(unsafe {
        super::animations::local_animation_id(name, name_len as usize, file, file_len as usize, out)
    })
}

pub(super) unsafe extern "system" fn submit_spawn(out: *mut CommandReceiptId) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::player_commands::submit_local_player_spawn(receipt)
    })
}

pub(super) unsafe extern "system" fn submit_special_action(
    action: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::player_commands::submit_local_player_special_action(action, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_name(
    name: *const u8,
    name_len: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::player_commands::submit_local_player_name(name, name_len as usize, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_colour(
    id: u16,
    colour: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::player_commands::submit_player_colour(id, colour, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_force_unoccupied_sync(
    vehicle: u16,
    seat: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::submit_force_unoccupied_sync(vehicle, seat, receipt)
    })
}

macro_rules! receipt_only {
    ($name:ident, $target:path) => {
        pub(super) unsafe extern "system" fn $name(out: *mut CommandReceiptId) -> ModResult {
            submit_with_receipt(out, |receipt| unsafe { $target(receipt) })
        }
    };
}

receipt_only!(submit_force_aim_sync, super::submit_force_aim_sync);
receipt_only!(submit_force_onfoot_sync, super::submit_force_onfoot_sync);
receipt_only!(submit_force_stats_sync, super::submit_force_stats_sync);
receipt_only!(submit_force_weapons_sync, super::submit_force_weapons_sync);

pub(super) unsafe extern "system" fn submit_force_trailer_sync(
    trailer: u16,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::submit_force_trailer_sync(trailer, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_force_vehicle_sync(
    vehicle: u16,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::submit_force_vehicle_sync(vehicle, receipt)
    })
}

pub(super) unsafe extern "system" fn submit_force_passenger_sync(
    vehicle: u16,
    seat: u8,
    out: *mut CommandReceiptId,
) -> ModResult {
    submit_with_receipt(out, |receipt| unsafe {
        super::submit_force_passenger_sync(vehicle, seat, receipt)
    })
}
