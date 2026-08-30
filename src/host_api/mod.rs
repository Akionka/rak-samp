mod animations;
pub(crate) mod chat_commands;
mod chat_input;
mod commands;
mod connection;
mod conversions;
mod dialog;
mod environment;
mod events;
mod handles;
mod helpers;
mod listeners;
mod local_commands;
mod local_state;
mod messages;
pub(crate) mod modkit;
mod network;
mod player_commands;
mod players;
mod pools;
mod raw;
mod sampfuncs;
mod snapshots;
mod text_labels;
mod textdraws;

#[cfg(test)]
use helpers::finish_direct_command;
use helpers::{
    clone_initialized, copied_nul_free_string, direct_client_result, host, is_shutting_down,
    next_subscription_id, submit_direct_command,
};
pub(super) use listeners::ListenerKind;
use listeners::{register_packet, register_rpc, unregister, unregister_and_wait};

#[cfg(test)]
use crate::runtime::DirectClientError;
use crate::{
    AttachError, BitStream, Direction, HookAction, ListenerHandle, Runtime, logging,
    runtime::{
        ClientHookStatus, LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest,
        LocalDialogRequest, LocalDialogStyle,
    },
};
use log::{debug, error, info};
use sdk_abi::limits::MAX_SAMP_VEHICLES;
use sdk_abi::{
    ABI_VERSION_V1, SampClientSdkApiV1, SampClientSdkCommandReceipt, SampClientSdkDirection,
    SampClientSdkEventCallbackV1, SampClientSdkEventV1, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkResult, SampClientSdkSubscription,
};
#[cfg(test)]
use sdk_abi::{
    Vector3,
    limits::{MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS},
};
use std::{
    collections::HashMap,
    ffi::c_void,
    ptr,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::Duration,
};

const STATUS_WAITING: u32 = SampClientSdkHostStatus::WaitingForSamp as u32;
const STATUS_READY: u32 = SampClientSdkHostStatus::Ready as u32;
const STATUS_FAILED: u32 = SampClientSdkHostStatus::Failed as u32;

struct HostState {
    status: AtomicU32,
    bootstrap_started: AtomicBool,
    shutting_down: AtomicBool,
    runtime: OnceLock<Arc<Runtime>>,
    chat_commands: chat_commands::ChatCommandRegistry,
    subscriptions: Mutex<HashMap<u64, ListenerHandle>>,
    next_subscription: AtomicU64,
}

struct AbiEvent {
    id: u8,
    payload: *mut BitStream,
}

static HOST: OnceLock<HostState> = OnceLock::new();

pub(crate) fn begin_bootstrap() {
    let state = host();
    if state
        .bootstrap_started
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }
    std::thread::spawn(|| {
        logging::initialize();
        info!("waiting for samp.dll before attaching the host runtime");
        loop {
            match Runtime::attach() {
                Ok(runtime) => {
                    let runtime = Arc::new(runtime);
                    if !wait_for_client_hooks(&runtime) {
                        host().status.store(STATUS_FAILED, Ordering::Release);
                        error!("host runtime failed to install RakClient packet and RPC hooks");
                        return;
                    }
                    if host().runtime.set(Arc::clone(&runtime)).is_err() {
                        host().status.store(STATUS_FAILED, Ordering::Release);
                        error!("host runtime was initialized more than once");
                        return;
                    }
                    host().status.store(STATUS_READY, Ordering::Release);
                    info!("host runtime and RakClient packet and RPC hooks are ready");
                    return;
                }
                Err(AttachError::SampNotLoaded) => std::thread::sleep(Duration::from_millis(10)),
                Err(attach_error) => {
                    host().status.store(STATUS_FAILED, Ordering::Release);
                    error!("host runtime failed to attach: {attach_error}");
                    return;
                }
            }
        }
    });
}

fn wait_for_client_hooks(runtime: &Runtime) -> bool {
    loop {
        match runtime.client_hook_status() {
            ClientHookStatus::Pending => std::thread::sleep(Duration::from_millis(10)),
            ClientHookStatus::Ready => {
                return true;
            }
            ClientHookStatus::Failed => {
                return false;
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn SampClientSdk_GetApiV1(requested_version: u32) -> *const SampClientSdkApiV1 {
    if requested_version == ABI_VERSION_V1 {
        &SAMP_CLIENT_SDK_API_V1
    } else {
        debug!("rejected unsupported plugin ABI version {requested_version}");
        ptr::null()
    }
}

pub(crate) static SAMP_CLIENT_SDK_API_V1: SampClientSdkApiV1 = SampClientSdkApiV1 {
    abi_version: ABI_VERSION_V1,
    size: std::mem::size_of::<SampClientSdkApiV1>() as u32,
    host_status,
    register_packet,
    register_rpc,
    unregister,
    event_id: events::event_id,
    event_reset_read: events::event_reset_read,
    event_clear: events::event_clear,
    event_read_u8: events::event_read_u8,
    event_read_u16: events::event_read_u16,
    event_read_u32: events::event_read_u32,
    event_read_f32: events::event_read_f32,
    event_read_bytes: events::event_read_bytes,
    event_write_u8: events::event_write_u8,
    event_write_u16: events::event_write_u16,
    event_write_u32: events::event_write_u32,
    event_write_f32: events::event_write_f32,
    event_write_bytes: events::event_write_bytes,
    send_packet: network::send_packet,
    send_rpc: network::send_rpc,
    event_replace_bytes: events::event_replace_bytes,
    unregister_and_wait,
    emulate_incoming_packet: network::emulate_incoming_packet,
    emulate_incoming_rpc: network::emulate_incoming_rpc,
    event_remaining_bits: events::event_remaining_bits,
    event_read_bits: events::event_read_bits,
    event_replace_bits: events::event_replace_bits,
    encode_string: events::encode_string,
    event_read_encoded_string: events::event_read_encoded_string,
    show_local_dialog,
    local_player: players::local_player,
    samp_game_state: environment::samp_game_state,
    samp_version: environment::samp_version,
    decode_string: events::decode_string,
    server_info: environment::server_info,
    show_local_chat_message,
    show_local_death_message,
    local_chat_display_mode: local_state::local_chat_display_mode,
    local_cursor_mode: local_state::local_cursor_mode,
    local_scoreboard_open: local_state::local_scoreboard_open,
    local_dialog_active: local_state::local_dialog_active,
    local_chat_input_active: local_state::local_chat_input_active,
    local_animation: animations::local_animation,
    local_animation_id: animations::local_animation_id,
    player_info: players::player_info,
    player_count: players::player_count,
    player_max_id: players::player_max_id,
    vehicle_exists: pools::vehicle_exists,
    active_local_dialog: local_state::active_local_dialog,
    text_label_exists: pools::text_label_exists,
    textdraw_exists: pools::textdraw_exists,
    object_exists: pools::object_exists,
    gangzone_info: snapshots::gangzone_info,
    text_label_info: snapshots::text_label_info,
    textdraw_info: snapshots::textdraw_info,
    player_defined: players::player_defined,
    player_paused: players::player_paused,
    remote_player_state: players::remote_player_state,
    submit_local_dialog: dialog::submit_local_dialog,
    submit_local_chat_message: messages::submit_local_chat_message,
    submit_local_death_message: messages::submit_local_death_message,
    command_try_take: commands::command_try_take,
    command_wait: commands::command_wait,
    command_release: commands::command_release,
    submit_packet: network::submit_packet,
    submit_rpc: network::submit_rpc,
    submit_emulate_incoming_packet: network::submit_emulate_incoming_packet,
    submit_emulate_incoming_rpc: network::submit_emulate_incoming_rpc,
    raw_rakclient: raw::raw_rakclient,
    raw_player_pool: raw::raw_player_pool,
    raw_vehicle_pool: raw::raw_vehicle_pool,
    submit_local_cursor_mode: local_commands::submit_local_cursor_mode,
    submit_local_scoreboard_open: local_commands::submit_local_scoreboard_open,
    submit_local_dialog_client_side: dialog::submit_local_dialog_client_side,
    submit_samp_game_state,
    raw_local_player: raw::raw_local_player,
    submit_local_player_spawn: player_commands::submit_local_player_spawn,
    submit_local_player_special_action: player_commands::submit_local_player_special_action,
    submit_send_rate,
    submit_local_cursor_toggle: local_commands::submit_local_cursor_toggle,
    submit_local_chat_display_mode: local_commands::submit_local_chat_display_mode,
    raw_rakpeer: raw::raw_rakpeer,
    submit_local_dialog_close: dialog::submit_local_dialog_close,
    submit_local_chat_input_text: chat_input::submit_local_chat_input_text,
    submit_local_chat_input_enabled: chat_input::submit_local_chat_input_enabled,
    submit_local_chat_input_process: chat_input::submit_local_chat_input_process,
    local_chat_input_text: chat_input::local_chat_input_text,
    submit_player_colour: player_commands::submit_player_colour,
    submit_local_player_name: player_commands::submit_local_player_name,
    submit_force_unoccupied_sync,
    submit_force_aim_sync,
    submit_force_onfoot_sync,
    submit_force_stats_sync,
    submit_force_trailer_sync,
    submit_force_vehicle_sync,
    submit_connect_to_server: connection::submit_connect_to_server,
    submit_disconnect_with_reason: connection::submit_disconnect_with_reason,
    submit_delete_textdraw: textdraws::submit_delete_textdraw,
    submit_create_textdraw: textdraws::submit_create_textdraw,
    submit_set_textdraw_position: textdraws::submit_set_textdraw_position,
    submit_set_textdraw_style: textdraws::submit_set_textdraw_style,
    submit_set_textdraw_letter_style: textdraws::submit_set_textdraw_letter_style,
    submit_set_textdraw_proportional: textdraws::submit_set_textdraw_proportional,
    submit_set_textdraw_shadow: textdraws::submit_set_textdraw_shadow,
    submit_set_textdraw_outline: textdraws::submit_set_textdraw_outline,
    submit_set_textdraw_box: textdraws::submit_set_textdraw_box,
    submit_set_textdraw_alignment: textdraws::submit_set_textdraw_alignment,
    submit_set_textdraw_string: textdraws::submit_set_textdraw_string,
    local_dialog_selected_item: dialog::local_dialog_selected_item,
    submit_local_dialog_selected_item: dialog::submit_local_dialog_selected_item,
    submit_delete_text_label: text_labels::submit_delete_text_label,
    local_dialog_list_item_count: dialog::local_dialog_list_item_count,
    submit_set_textdraw_model_style: textdraws::submit_set_textdraw_model_style,
    submit_local_chat_entry,
    chat_entry_info: snapshots::chat_entry_info,
    submit_create_text_label: text_labels::submit_create_text_label,
    local_dialog_snapshot: dialog::local_dialog_snapshot,
    submit_local_dialog_editbox_text: dialog::submit_local_dialog_editbox_text,
    local_object_handle: handles::local_object_handle,
    local_object_id_by_handle: handles::local_object_id_by_handle,
    local_pickup_handle: handles::local_pickup_handle,
    local_pickup_id_by_handle: handles::local_pickup_id_by_handle,
    local_vehicle_handle: handles::local_vehicle_handle,
    local_vehicle_id_by_handle: handles::local_vehicle_id_by_handle,
    local_player_ped_handle: handles::local_player_ped_handle,
    local_player_id_by_ped_handle: handles::local_player_id_by_ped_handle,
    submit_register_chat_command: chat_commands::submit_register_chat_command,
    local_chat_command_defined: chat_input::local_chat_command_defined,
    submit_create_text_label_auto: text_labels::submit_create_text_label_auto,
    text_label_create_try_take: commands::text_label_create_try_take,
    text_label_create_wait: commands::text_label_create_wait,
    submit_set_text_label_text: text_labels::submit_set_text_label_text,
    onfoot_sync: players::onfoot_sync,
    vehicle_sync: players::vehicle_sync,
    passenger_sync: players::passenger_sync,
    trailer_sync: players::trailer_sync,
    aim_sync: players::aim_sync,
    take_local_dialog_response: dialog::take_local_dialog_response,
    submit_force_passenger_sync,
    submit_force_weapons_sync,
    streamed_out_player_position: players::streamed_out_player_position,
    sampfuncs_loaded: sampfuncs::sampfuncs_loaded,
    sampfuncs_log_console: sampfuncs::sampfuncs_log_console,
    incoming_emulation_ready: network::incoming_emulation_ready,
};

extern "system" fn host_status() -> SampClientSdkHostStatus {
    match host().status.load(Ordering::Acquire) {
        STATUS_READY => SampClientSdkHostStatus::Ready,
        STATUS_FAILED => SampClientSdkHostStatus::Failed,
        _ => SampClientSdkHostStatus::WaitingForSamp,
    }
}

unsafe extern "system" fn show_local_dialog(
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
) -> SampClientSdkResult {
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    runtime
        .show_local_dialog(LocalDialogRequest {
            id,
            style,
            title,
            text,
            button1,
            button2,
        })
        .map_or_else(direct_client_result, |_| SampClientSdkResult::Ok)
}

unsafe extern "system" fn show_local_chat_message(
    style: u32,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    text_colour: u32,
    prefix_colour: u32,
) -> SampClientSdkResult {
    let Some(style) = LocalChatMessageStyle::from_raw(style) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 143) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(prefix) = (unsafe { copied_nul_free_string(prefix, prefix_len, 27) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    runtime
        .show_local_chat_message(LocalChatMessageRequest {
            style,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
        .map_or_else(direct_client_result, |_| SampClientSdkResult::Ok)
}

unsafe extern "system" fn show_local_death_message(
    killer: *const u8,
    killer_len: usize,
    victim: *const u8,
    victim_len: usize,
    killer_colour: u32,
    victim_colour: u32,
    weapon: u8,
) -> SampClientSdkResult {
    let Ok(killer) = (unsafe { copied_nul_free_string(killer, killer_len, 24) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(victim) = (unsafe { copied_nul_free_string(victim, victim_len, 24) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    runtime
        .show_local_death_message(LocalDeathMessageRequest {
            killer,
            victim,
            killer_colour,
            victim_colour,
            weapon,
        })
        .map_or_else(direct_client_result, |_| SampClientSdkResult::Ok)
}

unsafe extern "system" fn submit_samp_game_state(
    state: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(state, 0 | 9 | 13 | 14 | 15 | 18) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_samp_game_state(state)) }
}

unsafe extern "system" fn submit_force_unoccupied_sync(
    vehicle: u16,
    seat: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || vehicle >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_force_unoccupied_sync(vehicle, seat)
        })
    }
}

unsafe extern "system" fn submit_force_aim_sync(
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_force_aim_sync()) }
}

unsafe extern "system" fn submit_force_onfoot_sync(
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_force_onfoot_sync()) }
}
unsafe extern "system" fn submit_force_stats_sync(
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_force_stats_sync()) }
}

unsafe extern "system" fn submit_force_trailer_sync(
    trailer: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || trailer >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_force_trailer_sync(trailer)
        })
    }
}

unsafe extern "system" fn submit_force_vehicle_sync(
    vehicle: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || vehicle >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_force_vehicle_sync(vehicle)
        })
    }
}

unsafe extern "system" fn submit_force_passenger_sync(
    vehicle: u16,
    seat: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || vehicle >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_force_passenger_sync(vehicle, seat)
        })
    }
}

unsafe extern "system" fn submit_force_weapons_sync(
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { submit_direct_command(receipt, |runtime| runtime.submit_force_weapons_sync()) }
}

unsafe extern "system" fn submit_send_rate(
    kind: u8,
    milliseconds: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(kind, 0..=2) || i32::try_from(milliseconds).is_err() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_send_rate(kind, milliseconds)
        })
    }
}

unsafe extern "system" fn submit_local_chat_entry(
    id: u16,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    text_colour: u32,
    prefix_colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null()
        || text.is_null()
        || prefix.is_null()
        || id >= 100
        || text_len >= 144
        || prefix_len >= 28
    {
        return SampClientSdkResult::InvalidArgument;
    }
    let text = unsafe { std::slice::from_raw_parts(text, text_len) };
    let prefix = unsafe { std::slice::from_raw_parts(prefix, prefix_len) };
    if text.contains(&0) || prefix.contains(&0) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        submit_direct_command(receipt, |runtime| {
            runtime.submit_local_chat_entry(
                id,
                text.to_vec(),
                prefix.to_vec(),
                text_colour,
                prefix_colour,
            )
        })
    }
}

#[cfg(test)]
mod tests;
