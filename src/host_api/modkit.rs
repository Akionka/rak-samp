//! Modkit host bootstrap and exact-version service discovery.
//!
//! This module introduces the new `GtaModHost_GetApiV1` export and the
//! `query_service` registry beside the legacy `SampClientSdk_GetApiV1` export.
//! It implements the Core service v1 and the migration-only Legacy SA-MP
//! service wrapper. The legacy export is left unchanged.

use super::{clone_initialized, host, unregister};
use crate::command::CommandError;
use log::{debug, error, info, warn};
use modkit_abi::{
    CommandCompletionV1, CommandReceiptId, CoreServiceV1, GTA_SA_SERVICE_VERSION_V1,
    GTA_SA_SERVICE_VERSION_V2, GtaSaServiceV1, GtaSaServiceV2, HostStatusV1, LegacySampServiceV1,
    MOD_BUFFER_TOO_SMALL, MOD_HOST_ABI_VERSION_V1, MOD_INVALID_ARGUMENT, MOD_NOT_FOUND,
    MOD_NOT_READY, MOD_OK, MOD_SHUTTING_DOWN, MOD_UNSUPPORTED_VERSION, ModHostApiV1, ModResult,
    SAMP_CODEC_SERVICE_VERSION_V1, SAMP_CONTROL_SERVICE_VERSION_V1, SAMP_NET_SERVICE_VERSION_V1,
    SAMP_PLAYER_SERVICE_VERSION_V1, SAMP_POOL_SERVICE_VERSION_V1, SAMP_SERVICE_VERSION_V1,
    SAMP_TEXT_LABEL_SERVICE_VERSION_V1, SAMP_TEXTDRAW_SERVICE_VERSION_V1,
    SAMP_UI_SERVICE_VERSION_V1, SERVICE_ID_CORE, SERVICE_ID_GTA_SA, SERVICE_ID_LEGACY_SAMP_ABI,
    SERVICE_ID_SAMP, SERVICE_ID_SAMP_CODEC, SERVICE_ID_SAMP_CONTROL, SERVICE_ID_SAMP_NETWORK,
    SERVICE_ID_SAMP_PLAYER, SERVICE_ID_SAMP_POOL, SERVICE_ID_SAMP_TEXT_LABEL,
    SERVICE_ID_SAMP_TEXTDRAW, SERVICE_ID_SAMP_UI, SampCodecServiceV1, SampControlServiceV1,
    SampLocalPlayerV1, SampNetEventV1, SampNetSendOptionsV1, SampNetServiceV1, SampPlayerInfoV1,
    SampPlayerServiceV1, SampPoolServiceV1, SampServerInfoV1, SampServiceV1,
    SampTextLabelServiceV1, SampTextdrawServiceV1, SampUiServiceV1, SampVector3V1, ServiceHeader,
    SubscriptionId,
};
use sdk_abi::{
    SampClientSdkCommandReceipt, SampClientSdkEventV1, SampClientSdkLocalPlayerV1,
    SampClientSdkPlayerInfoV1, SampClientSdkResult, SampClientSdkSendOptions,
    SampClientSdkServerInfoV1, SampClientSdkSubscription,
};
use std::{ffi::c_void, ptr, sync::atomic::Ordering, time::Duration};

/// The published Core service version.
const CORE_SERVICE_VERSION: u32 = 1;
/// The published Legacy SA-MP service version.
const LEGACY_SERVICE_VERSION: u32 = 1;

/// The host-owned immutable bootstrap table.
static MOD_HOST_API_V1: ModHostApiV1 = ModHostApiV1 {
    abi_version: MOD_HOST_ABI_VERSION_V1,
    size: std::mem::size_of::<ModHostApiV1>() as u32,
    query_service,
};

/// The host-owned immutable Core service table.
static CORE_SERVICE_V1: CoreServiceV1 = CoreServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_CORE,
        version: CORE_SERVICE_VERSION,
        size: std::mem::size_of::<CoreServiceV1>() as u32,
        reserved: 0,
    },
    host_status: core_host_status,
    unregister: core_unregister,
    unregister_and_wait: core_unregister_and_wait,
    receipt_poll: core_receipt_poll,
    receipt_wait: core_receipt_wait,
    receipt_release: core_receipt_release,
    log_utf8: core_log_utf8,
};

static GTA_SA_SERVICE_V1: GtaSaServiceV1 = GtaSaServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_GTA_SA,
        version: GTA_SA_SERVICE_VERSION_V1,
        size: std::mem::size_of::<GtaSaServiceV1>() as u32,
        reserved: 0,
    },
    register_tick: super::gta::register_tick,
    local_ped_snapshot: super::gta::local_ped_snapshot,
    teleport_local_ped: super::gta::teleport_local_ped,
    submit_local_ped_snapshot: super::gta::submit_local_ped_snapshot,
    take_local_ped_snapshot: super::gta::take_local_ped_snapshot,
    submit_teleport_local_ped: super::gta::submit_teleport_local_ped,
};

static GTA_SA_SERVICE_V2: GtaSaServiceV2 = GtaSaServiceV2 {
    header: ServiceHeader {
        service_id: SERVICE_ID_GTA_SA,
        version: GTA_SA_SERVICE_VERSION_V2,
        size: std::mem::size_of::<GtaSaServiceV2>() as u32,
        reserved: 0,
    },
    register_tick: super::gta::register_tick,
    local_ped_snapshot: super::gta::local_ped_snapshot,
    teleport_local_ped: super::gta::teleport_local_ped,
    submit_local_ped_snapshot: super::gta::submit_local_ped_snapshot,
    take_local_ped_snapshot: super::gta::take_local_ped_snapshot,
    submit_teleport_local_ped: super::gta::submit_teleport_local_ped,
    entity_exists: super::gta::entity_exists,
    submit_entity_exists: super::gta::submit_entity_exists,
    take_entity_exists: super::gta::take_entity_exists,
    vehicle_snapshot: super::gta::vehicle_snapshot,
    submit_vehicle_snapshot: super::gta::submit_vehicle_snapshot,
    take_vehicle_snapshot: super::gta::take_vehicle_snapshot,
    find_ground_z: super::gta::find_ground_z,
    submit_find_ground_z: super::gta::submit_find_ground_z,
    take_find_ground_z: super::gta::take_find_ground_z,
    timer_snapshot: super::gta::timer_snapshot,
    submit_timer_snapshot: super::gta::submit_timer_snapshot,
    take_timer_snapshot: super::gta::take_timer_snapshot,
    camera_snapshot: super::gta::camera_snapshot,
    submit_camera_snapshot: super::gta::submit_camera_snapshot,
    take_camera_snapshot: super::gta::take_camera_snapshot,
};

static SAMP_NET_SERVICE_V1: SampNetServiceV1 = SampNetServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_NETWORK,
        version: SAMP_NET_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampNetServiceV1>() as u32,
        reserved: 0,
    },
    register_packet: super::listeners::register_modkit_packet,
    register_rpc: super::listeners::register_modkit_rpc,
    event_id: samp_net_event_id,
    event_reset: samp_net_event_reset,
    event_remaining_bits: samp_net_event_remaining_bits,
    event_read_bits: samp_net_event_read_bits,
    event_replace_bits: samp_net_event_replace_bits,
    encode_string: samp_net_encode_string,
    event_read_encoded_string: samp_net_event_read_encoded_string,
    submit_packet: samp_net_submit_packet,
    submit_rpc: samp_net_submit_rpc,
    submit_emulate_incoming_packet: samp_net_submit_emulate_incoming_packet,
    submit_emulate_incoming_rpc: samp_net_submit_emulate_incoming_rpc,
    incoming_emulation_ready: samp_net_incoming_emulation_ready,
};

static SAMP_SERVICE_V1: SampServiceV1 = SampServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP,
        version: SAMP_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampServiceV1>() as u32,
        reserved: 0,
    },
    version: samp_version,
    game_state: samp_game_state,
    server_info: samp_server_info,
    local_player: samp_local_player,
    player_info: samp_player_info,
    submit_chat_add: samp_submit_chat_add,
    submit_register_chat_command: super::chat_commands::submit_register_chat_command_modkit,
};

static SAMP_TEXT_LABEL_SERVICE_V1: SampTextLabelServiceV1 = SampTextLabelServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_TEXT_LABEL,
        version: SAMP_TEXT_LABEL_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampTextLabelServiceV1>() as u32,
        reserved: 0,
    },
    snapshot: super::text_labels::modkit_snapshot,
    submit_delete: super::text_labels::modkit_submit_delete,
    submit_set_text: super::text_labels::modkit_submit_set_text,
    submit_create_at: super::text_labels::modkit_submit_create_at,
    submit_create: super::text_labels::modkit_submit_create,
};

static SAMP_CONTROL_SERVICE_V1: SampControlServiceV1 = SampControlServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_CONTROL,
        version: SAMP_CONTROL_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampControlServiceV1>() as u32,
        reserved: 0,
    },
    submit_game_state: samp_control_submit_game_state,
    submit_send_rate: samp_control_submit_send_rate,
    submit_connect: samp_control_submit_connect,
    submit_disconnect: samp_control_submit_disconnect,
};

static SAMP_UI_SERVICE_V1: SampUiServiceV1 = SampUiServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_UI,
        version: SAMP_UI_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampUiServiceV1>() as u32,
        reserved: 0,
    },
    chat_display_mode: super::ui::chat_display_mode,
    chat_entry: super::ui::chat_entry,
    chat_input_active: super::ui::chat_input_active,
    chat_input_text: super::ui::chat_input_text,
    chat_command_defined: super::ui::chat_command_defined,
    cursor_mode: super::ui::cursor_mode,
    scoreboard_open: super::ui::scoreboard_open,
    dialog_active: super::ui::dialog_active,
    dialog_snapshot: super::ui::dialog_snapshot,
    take_dialog_response: super::ui::take_dialog_response,
    dialog_selected_item: super::ui::dialog_selected_item,
    dialog_list_item_count: super::ui::dialog_list_item_count,
    submit_chat_message: super::ui::submit_chat_message,
    submit_death_message: super::ui::submit_death_message,
    submit_chat_display_mode: super::ui::submit_chat_display_mode,
    submit_chat_entry: super::ui::submit_chat_entry,
    submit_chat_input_text: super::ui::submit_chat_input_text,
    submit_chat_input_enabled: super::ui::submit_chat_input_enabled,
    submit_chat_input_process: super::ui::submit_chat_input_process,
    submit_cursor_mode: super::ui::submit_cursor_mode,
    submit_cursor_toggle: super::ui::submit_cursor_toggle,
    submit_scoreboard_open: super::ui::submit_scoreboard_open,
    submit_dialog: super::ui::submit_dialog,
    submit_dialog_client_side: super::ui::submit_dialog_client_side,
    submit_dialog_selected_item: super::ui::submit_dialog_selected_item,
    submit_dialog_editbox_text: super::ui::submit_dialog_editbox_text,
    submit_dialog_close: super::ui::submit_dialog_close,
};

static SAMP_PLAYER_SERVICE_V1: SampPlayerServiceV1 = SampPlayerServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_PLAYER,
        version: SAMP_PLAYER_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampPlayerServiceV1>() as u32,
        reserved: 0,
    },
    remote_state: super::player_service::remote_state,
    streamed_out_position: super::player_service::streamed_out_position,
    onfoot_sync: super::player_service::onfoot_sync,
    vehicle_sync: super::player_service::vehicle_sync,
    passenger_sync: super::player_service::passenger_sync,
    trailer_sync: super::player_service::trailer_sync,
    aim_sync: super::player_service::aim_sync,
    player_defined: super::player_service::player_defined,
    player_paused: super::player_service::player_paused,
    player_count: super::player_service::player_count,
    player_max_id: super::player_service::player_max_id,
    animation: super::player_service::animation,
    animation_id: super::player_service::animation_id,
    submit_spawn: super::player_service::submit_spawn,
    submit_special_action: super::player_service::submit_special_action,
    submit_name: super::player_service::submit_name,
    submit_colour: super::player_service::submit_colour,
    submit_force_unoccupied_sync: super::player_service::submit_force_unoccupied_sync,
    submit_force_aim_sync: super::player_service::submit_force_aim_sync,
    submit_force_onfoot_sync: super::player_service::submit_force_onfoot_sync,
    submit_force_stats_sync: super::player_service::submit_force_stats_sync,
    submit_force_trailer_sync: super::player_service::submit_force_trailer_sync,
    submit_force_vehicle_sync: super::player_service::submit_force_vehicle_sync,
    submit_force_passenger_sync: super::player_service::submit_force_passenger_sync,
    submit_force_weapons_sync: super::player_service::submit_force_weapons_sync,
};

static SAMP_POOL_SERVICE_V1: SampPoolServiceV1 = SampPoolServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_POOL,
        version: SAMP_POOL_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampPoolServiceV1>() as u32,
        reserved: 0,
    },
    object_exists: super::pool_service::object_exists,
    vehicle_exists: super::pool_service::vehicle_exists,
    object_handle: super::pool_service::object_handle,
    object_id_by_handle: super::pool_service::object_id_by_handle,
    pickup_handle: super::pool_service::pickup_handle,
    pickup_id_by_handle: super::pool_service::pickup_id_by_handle,
    vehicle_handle: super::pool_service::vehicle_handle,
    vehicle_id_by_handle: super::pool_service::vehicle_id_by_handle,
    player_ped_handle: super::pool_service::player_ped_handle,
    player_id_by_ped_handle: super::pool_service::player_id_by_ped_handle,
    gangzone: super::pool_service::gangzone,
};

static SAMP_TEXTDRAW_SERVICE_V1: SampTextdrawServiceV1 = SampTextdrawServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_TEXTDRAW,
        version: SAMP_TEXTDRAW_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampTextdrawServiceV1>() as u32,
        reserved: 0,
    },
    exists: super::textdraw_service::exists,
    snapshot: super::textdraw_service::snapshot,
    submit_create: super::textdraw_service::submit_create,
    submit_delete: super::textdraw_service::submit_delete,
    submit_set_position: super::textdraw_service::submit_set_position,
    submit_set_style: super::textdraw_service::submit_set_style,
    submit_set_letter_style: super::textdraw_service::submit_set_letter_style,
    submit_set_proportional: super::textdraw_service::submit_set_proportional,
    submit_set_shadow: super::textdraw_service::submit_set_shadow,
    submit_set_outline: super::textdraw_service::submit_set_outline,
    submit_set_box: super::textdraw_service::submit_set_box,
    submit_set_alignment: super::textdraw_service::submit_set_alignment,
    submit_set_text: super::textdraw_service::submit_set_text,
    submit_set_model_style: super::textdraw_service::submit_set_model_style,
};

static SAMP_CODEC_SERVICE_V1: SampCodecServiceV1 = SampCodecServiceV1 {
    header: ServiceHeader {
        service_id: SERVICE_ID_SAMP_CODEC,
        version: SAMP_CODEC_SERVICE_VERSION_V1,
        size: std::mem::size_of::<SampCodecServiceV1>() as u32,
        reserved: 0,
    },
    decode_string: samp_codec_decode_string,
};

/// The host-owned immutable Legacy SA-MP service table.
static LEGACY_SAMP_SERVICE_V1: std::sync::OnceLock<LegacySampServiceV1> =
    std::sync::OnceLock::new();

fn legacy_samp_service() -> &'static LegacySampServiceV1 {
    LEGACY_SAMP_SERVICE_V1.get_or_init(|| LegacySampServiceV1 {
        header: ServiceHeader {
            service_id: SERVICE_ID_LEGACY_SAMP_ABI,
            version: LEGACY_SERVICE_VERSION,
            size: std::mem::size_of::<LegacySampServiceV1>() as u32,
            reserved: 0,
        },
        api: (&super::SAMP_CLIENT_SDK_API_V1 as *const sdk_abi::SampClientSdkApiV1)
            .cast::<c_void>(),
    })
}

/// The new host bootstrap export.
///
/// `ANY_THREAD + CALLBACK_SAFE`; returns without blocking.
///
/// # Safety
///
/// `out_api` must be null or point to writable storage for one pointer.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn GtaModHost_GetApiV1(out_api: *mut *const ModHostApiV1) -> ModResult {
    if out_api.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    unsafe { out_api.write(&MOD_HOST_API_V1) };
    MOD_OK
}

/// Exact-version service discovery.
///
/// # Safety
///
/// `out_service` must be null or point to writable storage for one pointer.
unsafe extern "system" fn query_service(
    service: modkit_abi::ServiceId,
    requested_version: u32,
    out_service: *mut *const ServiceHeader,
) -> ModResult {
    if out_service.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    unsafe { out_service.write(ptr::null()) };

    if host().shutting_down.load(Ordering::Acquire) {
        return MOD_SHUTTING_DOWN;
    }

    match service {
        SERVICE_ID_CORE => {
            if requested_version != CORE_SERVICE_VERSION {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe { out_service.write((&CORE_SERVICE_V1 as *const CoreServiceV1).cast()) };
            MOD_OK
        }
        SERVICE_ID_GTA_SA => match requested_version {
            GTA_SA_SERVICE_VERSION_V1 => {
                unsafe { out_service.write((&GTA_SA_SERVICE_V1 as *const GtaSaServiceV1).cast()) };
                MOD_OK
            }
            GTA_SA_SERVICE_VERSION_V2 => {
                unsafe { out_service.write((&GTA_SA_SERVICE_V2 as *const GtaSaServiceV2).cast()) };
                MOD_OK
            }
            _ => MOD_UNSUPPORTED_VERSION,
        },
        SERVICE_ID_LEGACY_SAMP_ABI => {
            if requested_version != LEGACY_SERVICE_VERSION {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service.write((legacy_samp_service() as *const LegacySampServiceV1).cast())
            };
            MOD_OK
        }
        SERVICE_ID_SAMP_NETWORK => {
            if requested_version != SAMP_NET_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe { out_service.write((&SAMP_NET_SERVICE_V1 as *const SampNetServiceV1).cast()) };
            MOD_OK
        }
        SERVICE_ID_SAMP => {
            if requested_version != SAMP_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe { out_service.write((&SAMP_SERVICE_V1 as *const SampServiceV1).cast()) };
            MOD_OK
        }
        SERVICE_ID_SAMP_TEXT_LABEL => {
            if requested_version != SAMP_TEXT_LABEL_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service
                    .write((&SAMP_TEXT_LABEL_SERVICE_V1 as *const SampTextLabelServiceV1).cast())
            };
            MOD_OK
        }
        SERVICE_ID_SAMP_CONTROL => {
            if requested_version != SAMP_CONTROL_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service.write((&SAMP_CONTROL_SERVICE_V1 as *const SampControlServiceV1).cast())
            };
            MOD_OK
        }
        SERVICE_ID_SAMP_UI => {
            if requested_version != SAMP_UI_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe { out_service.write((&SAMP_UI_SERVICE_V1 as *const SampUiServiceV1).cast()) };
            MOD_OK
        }
        SERVICE_ID_SAMP_PLAYER => {
            if requested_version != SAMP_PLAYER_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service.write((&SAMP_PLAYER_SERVICE_V1 as *const SampPlayerServiceV1).cast())
            };
            MOD_OK
        }
        SERVICE_ID_SAMP_POOL => {
            if requested_version != SAMP_POOL_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service.write((&SAMP_POOL_SERVICE_V1 as *const SampPoolServiceV1).cast())
            };
            MOD_OK
        }
        SERVICE_ID_SAMP_TEXTDRAW => {
            if requested_version != SAMP_TEXTDRAW_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service
                    .write((&SAMP_TEXTDRAW_SERVICE_V1 as *const SampTextdrawServiceV1).cast())
            };
            MOD_OK
        }
        SERVICE_ID_SAMP_CODEC => {
            if requested_version != SAMP_CODEC_SERVICE_VERSION_V1 {
                return MOD_UNSUPPORTED_VERSION;
            }
            unsafe {
                out_service.write((&SAMP_CODEC_SERVICE_V1 as *const SampCodecServiceV1).cast())
            };
            MOD_OK
        }
        _ => MOD_NOT_FOUND,
    }
}

unsafe extern "system" fn samp_codec_decode_string(
    input: *const u8,
    input_byte_len: u32,
    input_bit_len: u32,
    input_read_offset: u32,
    output: *mut u8,
    output_capacity: u32,
    output_len: *mut u32,
    output_read_offset: *mut u32,
) -> ModResult {
    if output_len.is_null() || output_read_offset.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut legacy_len = 0usize;
    let mut legacy_offset = 0usize;
    let result = unsafe {
        super::events::decode_string(
            input,
            input_byte_len as usize,
            input_bit_len as usize,
            input_read_offset as usize,
            output,
            output_capacity as usize,
            &mut legacy_len,
            &mut legacy_offset,
        )
    };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    let (Ok(length), Ok(read_offset)) = (u32::try_from(legacy_len), u32::try_from(legacy_offset))
    else {
        return modkit_abi::MOD_NATIVE_CALL_FAILED;
    };
    unsafe {
        output_len.write(length);
        output_read_offset.write(read_offset);
    }
    MOD_OK
}

unsafe extern "system" fn samp_control_submit_game_state(
    state: i32,
    out: *mut CommandReceiptId,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe { super::submit_samp_game_state(state, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = CommandReceiptId(legacy.id);
    }
    subscription_result(result)
}

unsafe extern "system" fn samp_control_submit_send_rate(
    kind: u32,
    milliseconds: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    let Ok(kind) = u8::try_from(kind) else {
        return MOD_INVALID_ARGUMENT;
    };
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe { super::submit_send_rate(kind, milliseconds, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = CommandReceiptId(legacy.id);
    }
    subscription_result(result)
}

unsafe extern "system" fn samp_control_submit_connect(
    address: *const u8,
    address_len: u32,
    port: u16,
    out: *mut CommandReceiptId,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result = unsafe {
        super::connection::submit_connect_to_server(
            address,
            address_len as usize,
            port,
            &mut legacy,
        )
    };
    if result == SampClientSdkResult::Ok {
        *out = CommandReceiptId(legacy.id);
    }
    subscription_result(result)
}

unsafe extern "system" fn samp_control_submit_disconnect(
    block_duration: u32,
    out: *mut CommandReceiptId,
) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = CommandReceiptId(0);
    let mut legacy = SampClientSdkCommandReceipt::default();
    let result =
        unsafe { super::connection::submit_disconnect_with_reason(block_duration, &mut legacy) };
    if result == SampClientSdkResult::Ok {
        *out = CommandReceiptId(legacy.id);
    }
    subscription_result(result)
}

unsafe extern "system" fn samp_version(out: *mut u32) -> ModResult {
    subscription_result(unsafe { super::environment::samp_version(out) })
}

unsafe extern "system" fn samp_game_state(out: *mut i32) -> ModResult {
    subscription_result(unsafe { super::environment::samp_game_state(out) })
}

unsafe extern "system" fn samp_server_info(out: *mut SampServerInfoV1) -> ModResult {
    if out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut legacy = SampClientSdkServerInfoV1::default();
    let result = unsafe { super::environment::server_info(&mut legacy) };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    unsafe {
        out.write(SampServerInfoV1 {
            address_len: legacy.address_len,
            hostname_len: legacy.hostname_len,
            address: legacy.address,
            hostname: legacy.hostname,
            port: legacy.port,
        })
    };
    MOD_OK
}

unsafe extern "system" fn samp_local_player(out: *mut SampLocalPlayerV1) -> ModResult {
    if out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut legacy = SampClientSdkLocalPlayerV1::default();
    let result = unsafe { super::players::local_player(&mut legacy) };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    unsafe { out.write(local_player_from_legacy(legacy)) };
    MOD_OK
}

unsafe extern "system" fn samp_player_info(id: u16, out: *mut SampPlayerInfoV1) -> ModResult {
    if out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut legacy = SampClientSdkPlayerInfoV1::default();
    let result = unsafe { super::players::player_info(id, &mut legacy) };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    unsafe {
        out.write(SampPlayerInfoV1 {
            exists: legacy.exists,
            is_local: legacy.is_local,
            is_npc: legacy.is_npc,
            reserved: 0,
            id: legacy.id,
            nickname_len: legacy.nickname_len,
            nickname: legacy.nickname,
            colour: legacy.colour,
            score: legacy.score,
            ping: legacy.ping,
        })
    };
    MOD_OK
}

unsafe extern "system" fn samp_submit_chat_add(
    style: u32,
    text: *const u8,
    text_len: u32,
    prefix: *const u8,
    prefix_len: u32,
    text_colour: u32,
    prefix_colour: u32,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    if out_receipt.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut receipt = SampClientSdkCommandReceipt::default();
    let result = unsafe {
        super::messages::submit_local_chat_message(
            style,
            text,
            text_len as usize,
            prefix,
            prefix_len as usize,
            text_colour,
            prefix_colour,
            &mut receipt,
        )
    };
    if result == SampClientSdkResult::Ok {
        unsafe { out_receipt.write(CommandReceiptId(receipt.id)) };
    }
    subscription_result(result)
}

fn local_player_from_legacy(legacy: SampClientSdkLocalPlayerV1) -> SampLocalPlayerV1 {
    SampLocalPlayerV1 {
        id: legacy.id,
        nickname_len: legacy.nickname_len,
        nickname: legacy.nickname,
        colour: legacy.colour,
        spawned: legacy.spawned,
        special_action: legacy.special_action,
        animation_id: legacy.animation_id,
        health: legacy.health,
        armour: legacy.armour,
        position: SampVector3V1 {
            x: legacy.position.x,
            y: legacy.position.y,
            z: legacy.position.z,
        },
        velocity: SampVector3V1 {
            x: legacy.velocity.x,
            y: legacy.velocity.y,
            z: legacy.velocity.z,
        },
        has_vehicle: legacy.has_vehicle,
        reserved: 0,
        vehicle_id: legacy.vehicle_id,
        score: legacy.score,
        ping: legacy.ping,
    }
}

unsafe extern "system" fn samp_net_event_id(
    event: *const SampNetEventV1,
    out: *mut u8,
) -> ModResult {
    if event.is_null() || out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    unsafe {
        out.write(super::events::event_id(
            event.cast::<SampClientSdkEventV1>(),
        ))
    };
    MOD_OK
}

unsafe extern "system" fn samp_net_event_reset(event: *mut SampNetEventV1) -> ModResult {
    subscription_result(unsafe {
        super::events::event_reset_read(event.cast::<SampClientSdkEventV1>())
    })
}

unsafe extern "system" fn samp_net_event_remaining_bits(
    event: *const SampNetEventV1,
    out: *mut u32,
) -> ModResult {
    if event.is_null() || out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let remaining = unsafe {
        super::events::event_remaining_bits(event.cast_mut().cast::<SampClientSdkEventV1>())
    };
    let Ok(remaining) = u32::try_from(remaining) else {
        return modkit_abi::MOD_OUT_OF_BOUNDS;
    };
    unsafe { out.write(remaining) };
    MOD_OK
}

unsafe extern "system" fn samp_net_event_read_bits(
    event: *mut SampNetEventV1,
    out: *mut u8,
    out_capacity: u32,
    bit_len: u32,
) -> ModResult {
    let required = bit_len.div_ceil(u8::BITS);
    if required > out_capacity || (out.is_null() && required != 0) {
        return MOD_INVALID_ARGUMENT;
    }
    let output = if required == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(out, required as usize) }
    };
    subscription_result(super::events::event_read_bits_into(
        event.cast::<SampClientSdkEventV1>(),
        output,
        bit_len as usize,
    ))
}

unsafe extern "system" fn samp_net_event_replace_bits(
    event: *mut SampNetEventV1,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
) -> ModResult {
    subscription_result(unsafe {
        super::events::event_replace_bits(
            event.cast::<SampClientSdkEventV1>(),
            data,
            byte_len as usize,
            bit_len as usize,
        )
    })
}

unsafe extern "system" fn samp_net_encode_string(
    value: *const u8,
    value_len: u32,
    out: *mut u8,
    out_capacity: u32,
    out_byte_len: *mut u32,
    out_bit_len: *mut u32,
) -> ModResult {
    if (value.is_null() && value_len != 0)
        || (out.is_null() && out_capacity != 0)
        || out_byte_len.is_null()
        || out_bit_len.is_null()
    {
        return MOD_INVALID_ARGUMENT;
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len as usize) }
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    let encoded = match runtime.encode_string(value) {
        Ok(encoded) => encoded,
        Err(error) => return subscription_result(super::events::codec_result(error)),
    };
    let Ok(byte_len) = u32::try_from(encoded.len_bytes()) else {
        return modkit_abi::MOD_OUT_OF_BOUNDS;
    };
    let Ok(bit_len) = u32::try_from(encoded.len_bits()) else {
        return modkit_abi::MOD_OUT_OF_BOUNDS;
    };
    unsafe {
        out_byte_len.write(byte_len);
        out_bit_len.write(bit_len);
    }
    if byte_len > out_capacity {
        return MOD_BUFFER_TOO_SMALL;
    }
    if byte_len != 0 {
        unsafe { ptr::copy_nonoverlapping(encoded.as_bytes().as_ptr(), out, byte_len as usize) };
    }
    MOD_OK
}

unsafe extern "system" fn samp_net_event_read_encoded_string(
    event: *mut SampNetEventV1,
    out: *mut u8,
    out_capacity: u32,
    out_len: *mut u32,
) -> ModResult {
    if out_len.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut len = 0usize;
    let result = unsafe {
        super::events::event_read_encoded_string(
            event.cast::<SampClientSdkEventV1>(),
            out,
            out_capacity as usize,
            &mut len,
        )
    };
    if result != SampClientSdkResult::Ok {
        return subscription_result(result);
    }
    let Ok(len) = u32::try_from(len) else {
        return modkit_abi::MOD_OUT_OF_BOUNDS;
    };
    unsafe { out_len.write(len) };
    MOD_OK
}

unsafe extern "system" fn samp_net_submit_packet(
    id: u8,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
    options: SampNetSendOptionsV1,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    unsafe { samp_net_submit(id, data, byte_len, bit_len, options, out_receipt, true) }
}

unsafe extern "system" fn samp_net_submit_rpc(
    id: u8,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
    options: SampNetSendOptionsV1,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    unsafe { samp_net_submit(id, data, byte_len, bit_len, options, out_receipt, false) }
}

unsafe fn samp_net_submit(
    id: u8,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
    options: SampNetSendOptionsV1,
    out_receipt: *mut CommandReceiptId,
    packet: bool,
) -> ModResult {
    if out_receipt.is_null() || options.reserved != [0; 2] || options.timestamp > 1 {
        return MOD_INVALID_ARGUMENT;
    }
    let mut receipt = SampClientSdkCommandReceipt::default();
    let options = SampClientSdkSendOptions {
        priority: options.priority,
        reliability: options.reliability,
        ordering_channel: options.ordering_channel,
        timestamp: options.timestamp != 0,
    };
    let result = unsafe {
        if packet {
            super::network::submit_packet(
                id,
                data,
                byte_len as usize,
                bit_len as usize,
                options,
                &mut receipt,
            )
        } else {
            super::network::submit_rpc(
                id,
                data,
                byte_len as usize,
                bit_len as usize,
                options,
                &mut receipt,
            )
        }
    };
    if result == SampClientSdkResult::Ok {
        unsafe { out_receipt.write(CommandReceiptId(receipt.id)) };
    }
    subscription_result(result)
}

unsafe extern "system" fn samp_net_submit_emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    unsafe { samp_net_submit_emulate(id, data, byte_len, bit_len, out_receipt, true) }
}

unsafe extern "system" fn samp_net_submit_emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
    out_receipt: *mut CommandReceiptId,
) -> ModResult {
    unsafe { samp_net_submit_emulate(id, data, byte_len, bit_len, out_receipt, false) }
}

unsafe fn samp_net_submit_emulate(
    id: u8,
    data: *const u8,
    byte_len: u32,
    bit_len: u32,
    out_receipt: *mut CommandReceiptId,
    packet: bool,
) -> ModResult {
    if out_receipt.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let mut receipt = SampClientSdkCommandReceipt::default();
    let result = unsafe {
        if packet {
            super::network::submit_emulate_incoming_packet(
                id,
                data,
                byte_len as usize,
                bit_len as usize,
                &mut receipt,
            )
        } else {
            super::network::submit_emulate_incoming_rpc(
                id,
                data,
                byte_len as usize,
                bit_len as usize,
                &mut receipt,
            )
        }
    };
    if result == SampClientSdkResult::Ok {
        unsafe { out_receipt.write(CommandReceiptId(receipt.id)) };
    }
    subscription_result(result)
}

unsafe extern "system" fn samp_net_incoming_emulation_ready(out: *mut u8) -> ModResult {
    let Some(out) = (unsafe { out.as_mut() }) else {
        return MOD_INVALID_ARGUMENT;
    };
    *out = super::network::incoming_emulation_ready();
    MOD_OK
}

unsafe extern "system" fn core_host_status(out: *mut HostStatusV1) -> ModResult {
    if out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let state = if host().shutting_down.load(Ordering::Acquire) {
        HostStatusV1::STATE_SHUTTING_DOWN
    } else {
        match host().status.load(Ordering::Acquire) {
            super::STATUS_READY => HostStatusV1::STATE_READY,
            super::STATUS_FAILED => HostStatusV1::STATE_FAILED,
            _ => HostStatusV1::STATE_WAITING,
        }
    };
    unsafe {
        out.write(HostStatusV1 {
            state,
            reserved: [0; 3],
        })
    };
    MOD_OK
}

unsafe extern "system" fn core_unregister(id: SubscriptionId) -> ModResult {
    if id.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let result = unsafe { unregister(SampClientSdkSubscription { id: id.0 }) };
    subscription_result(result)
}

unsafe extern "system" fn core_unregister_and_wait(
    id: SubscriptionId,
    timeout_ms: u32,
) -> ModResult {
    if id.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    if !runtime.command_wait_allowed() {
        return modkit_abi::MOD_WAIT_REJECTED;
    }
    let result = super::listeners::unregister_and_wait_with_timeout(
        SampClientSdkSubscription { id: id.0 },
        Some(timeout_duration(timeout_ms)),
    );
    subscription_result(result)
}

pub(super) fn subscription_result(result: SampClientSdkResult) -> ModResult {
    match result {
        SampClientSdkResult::Ok => MOD_OK,
        SampClientSdkResult::NotReady => MOD_NOT_READY,
        SampClientSdkResult::InvalidArgument => MOD_INVALID_ARGUMENT,
        SampClientSdkResult::UnsupportedVersion => MOD_UNSUPPORTED_VERSION,
        SampClientSdkResult::SubscriptionNotFound => MOD_NOT_FOUND,
        SampClientSdkResult::ReadOutOfBounds => modkit_abi::MOD_OUT_OF_BOUNDS,
        SampClientSdkResult::PayloadTooLarge => modkit_abi::MOD_PAYLOAD_TOO_LARGE,
        SampClientSdkResult::NativeCallFailed => modkit_abi::MOD_NATIVE_CALL_FAILED,
        SampClientSdkResult::CallbackInProgress => modkit_abi::MOD_CALLBACK_IN_PROGRESS,
        SampClientSdkResult::QueueFull => modkit_abi::MOD_QUEUE_FULL,
        SampClientSdkResult::CommandPending => modkit_abi::MOD_PENDING,
        SampClientSdkResult::TimedOut => modkit_abi::MOD_TIMED_OUT,
        SampClientSdkResult::WaitRejected => modkit_abi::MOD_WAIT_REJECTED,
        SampClientSdkResult::ShuttingDown => MOD_SHUTTING_DOWN,
        SampClientSdkResult::Busy => modkit_abi::MOD_BUSY,
    }
}

unsafe extern "system" fn core_receipt_poll(
    id: CommandReceiptId,
    out: *mut CommandCompletionV1,
) -> ModResult {
    if id.is_zero() || out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    if runtime.is_created_text_label_receipt(id.0) {
        return match runtime.try_take_created_text_label(id.0) {
            Ok(Some(result)) => {
                unsafe { out.write(text_label_completion(result)) };
                MOD_OK
            }
            Ok(None) => modkit_abi::MOD_PENDING,
            Err(error) => command_error_result(error),
        };
    }
    match runtime.try_take_command(id.0) {
        Ok(Some(result)) => {
            unsafe { out.write(completion(result)) };
            MOD_OK
        }
        Ok(None) => modkit_abi::MOD_PENDING,
        Err(error) => command_error_result(error),
    }
}

unsafe extern "system" fn core_receipt_wait(
    id: CommandReceiptId,
    timeout_ms: u32,
    out: *mut CommandCompletionV1,
) -> ModResult {
    if id.is_zero() || out.is_null() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    if !runtime.command_wait_allowed() {
        return modkit_abi::MOD_WAIT_REJECTED;
    }
    if runtime.is_created_text_label_receipt(id.0) {
        return match runtime.wait_for_created_text_label(id.0, timeout_duration(timeout_ms)) {
            Ok(result) => {
                unsafe { out.write(text_label_completion(result)) };
                MOD_OK
            }
            Err(error) => command_error_result(error),
        };
    }
    match runtime.wait_for_command(id.0, timeout_duration(timeout_ms)) {
        Ok(result) => {
            unsafe { out.write(completion(result)) };
            MOD_OK
        }
        Err(error) => command_error_result(error),
    }
}

unsafe extern "system" fn core_receipt_release(id: CommandReceiptId) -> ModResult {
    if id.is_zero() {
        return MOD_INVALID_ARGUMENT;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return MOD_NOT_READY;
    };
    runtime
        .release_command(id.0)
        .map_or_else(command_error_result, |_| MOD_OK)
}

unsafe extern "system" fn core_log_utf8(level: u32, ptr: *const u8, len: u32) -> ModResult {
    if (ptr.is_null() && len != 0) || len > modkit_abi::MAX_LOG_MESSAGE_BYTES {
        return MOD_INVALID_ARGUMENT;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, len as usize) }
    };
    let message = String::from_utf8_lossy(bytes);
    match level {
        modkit_abi::LOG_LEVEL_ERROR => error!("{message}"),
        modkit_abi::LOG_LEVEL_WARN => warn!("{message}"),
        modkit_abi::LOG_LEVEL_INFO => info!("{message}"),
        modkit_abi::LOG_LEVEL_DEBUG => debug!("{message}"),
        _ => return MOD_INVALID_ARGUMENT,
    }
    MOD_OK
}

fn timeout_duration(timeout_ms: u32) -> Duration {
    if timeout_ms == modkit_abi::TIMEOUT_INFINITE {
        Duration::MAX
    } else {
        Duration::from_millis(u64::from(timeout_ms))
    }
}

fn completion(result: Result<(), CommandError>) -> CommandCompletionV1 {
    match result {
        Ok(()) => CommandCompletionV1::default(),
        Err(error) => CommandCompletionV1 {
            status: command_error_result(error),
            reserved: 0,
            value0: 0,
            value1: 0,
        },
    }
}

fn text_label_completion(result: Result<u16, CommandError>) -> CommandCompletionV1 {
    match result {
        Ok(id) => CommandCompletionV1 {
            status: MOD_OK,
            reserved: 0,
            value0: u64::from(id),
            value1: 0,
        },
        Err(error) => CommandCompletionV1 {
            status: command_error_result(error),
            reserved: 0,
            value0: 0,
            value1: 0,
        },
    }
}

fn command_error_result(error: CommandError) -> ModResult {
    match error {
        CommandError::QueueFull => modkit_abi::MOD_QUEUE_FULL,
        CommandError::IdExhausted => modkit_abi::MOD_BUSY,
        CommandError::ShuttingDown => MOD_SHUTTING_DOWN,
        CommandError::NativeFailure => modkit_abi::MOD_NATIVE_CALL_FAILED,
        CommandError::UnknownReceipt => MOD_INVALID_ARGUMENT,
        CommandError::TimedOut => modkit_abi::MOD_TIMED_OUT,
        CommandError::WaitRejected => modkit_abi::MOD_WAIT_REJECTED,
    }
}

/// Marks the host as shutting down so discovery and new operations fail closed.
pub(crate) fn begin_shutdown() {
    host().shutting_down.store(true, Ordering::Release);
    super::gta::shutdown();
}

#[cfg(test)]
mod tests;
