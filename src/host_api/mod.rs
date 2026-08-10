mod animations;
mod chat_input;
mod commands;
mod connection;
mod conversions;
mod dialog;
mod environment;
mod events;
mod handles;
mod local_commands;
mod local_state;
mod network;
mod player_commands;
mod players;
mod pools;
mod raw;
mod snapshots;
mod text_labels;

use crate::{
    AttachError, BitStream, Direction, HookAction, ListenerHandle, Runtime, logging,
    runtime::{
        ClientHookStatus, DirectClientError, LocalChatMessageRequest, LocalChatMessageStyle,
        LocalDeathMessageRequest, LocalDialogRequest, LocalDialogStyle,
    },
};
use log::{debug, error, info};
use sdk_abi::limits::{MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES};
use sdk_abi::{
    ABI_VERSION_V1, SampClientSdkApiV1, SampClientSdkCommandReceipt, SampClientSdkDirection,
    SampClientSdkEventCallbackV1, SampClientSdkEventV1, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkResult, SampClientSdkSubscription,
};
#[cfg(test)]
use sdk_abi::{
    Vector3,
    limits::{MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABELS},
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
    runtime: OnceLock<Arc<Runtime>>,
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
                    if host().runtime.set(Arc::clone(&runtime)).is_err() {
                        host().status.store(STATUS_FAILED, Ordering::Release);
                        error!("host runtime was initialized more than once");
                        return;
                    }
                    host().status.store(STATUS_READY, Ordering::Release);
                    info!("host runtime is ready");
                    monitor_client_hooks(runtime);
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

fn monitor_client_hooks(runtime: Arc<Runtime>) {
    loop {
        match runtime.client_hook_status() {
            ClientHookStatus::Pending => std::thread::sleep(Duration::from_millis(10)),
            ClientHookStatus::Ready => {
                info!("RakClient packet and RPC hooks are ready");
                return;
            }
            ClientHookStatus::Failed => {
                host().status.store(STATUS_FAILED, Ordering::Release);
                error!("host runtime failed to install RakClient packet and RPC hooks");
                return;
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

static SAMP_CLIENT_SDK_API_V1: SampClientSdkApiV1 = SampClientSdkApiV1 {
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
    submit_local_dialog,
    submit_local_chat_message,
    submit_local_death_message,
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
    submit_local_dialog_client_side,
    submit_samp_game_state,
    raw_local_player: raw::raw_local_player,
    submit_local_player_spawn: player_commands::submit_local_player_spawn,
    submit_local_player_special_action: player_commands::submit_local_player_special_action,
    submit_send_rate,
    submit_local_cursor_toggle,
    submit_local_chat_display_mode,
    raw_rakpeer: raw::raw_rakpeer,
    submit_local_dialog_close,
    submit_local_chat_input_text: chat_input::submit_local_chat_input_text,
    submit_local_chat_input_enabled: chat_input::submit_local_chat_input_enabled,
    submit_local_chat_input_process: chat_input::submit_local_chat_input_process,
    local_chat_input_text: chat_input::local_chat_input_text,
    submit_player_colour: player_commands::submit_player_colour,
    submit_local_player_name: player_commands::submit_local_player_name,
    submit_force_unoccupied_sync,
    submit_connect_to_server: connection::submit_connect_to_server,
    submit_disconnect_with_reason: connection::submit_disconnect_with_reason,
    submit_delete_textdraw,
    submit_set_textdraw_position,
    submit_set_textdraw_letter_style,
    submit_set_textdraw_proportional,
    submit_set_textdraw_shadow,
    submit_set_textdraw_outline,
    submit_set_textdraw_box,
    submit_set_textdraw_alignment,
    submit_set_textdraw_string,
    local_dialog_selected_item: dialog::local_dialog_selected_item,
    submit_local_dialog_selected_item,
    submit_delete_text_label: text_labels::submit_delete_text_label,
    local_dialog_list_item_count: dialog::local_dialog_list_item_count,
    submit_set_textdraw_model_style,
    submit_local_chat_entry,
    chat_entry_info: snapshots::chat_entry_info,
    submit_create_text_label: text_labels::submit_create_text_label,
    local_dialog_snapshot: dialog::local_dialog_snapshot,
    submit_local_dialog_editbox_text,
    local_object_handle: handles::local_object_handle,
    local_object_id_by_handle: handles::local_object_id_by_handle,
    local_pickup_handle: handles::local_pickup_handle,
    local_pickup_id_by_handle: handles::local_pickup_id_by_handle,
    local_vehicle_handle: handles::local_vehicle_handle,
    local_vehicle_id_by_handle: handles::local_vehicle_id_by_handle,
    local_player_ped_handle: handles::local_player_ped_handle,
    local_player_id_by_ped_handle: handles::local_player_id_by_ped_handle,
};

extern "system" fn host_status() -> SampClientSdkHostStatus {
    match host().status.load(Ordering::Acquire) {
        STATUS_READY => SampClientSdkHostStatus::Ready,
        STATUS_FAILED => SampClientSdkHostStatus::Failed,
        _ => SampClientSdkHostStatus::WaitingForSamp,
    }
}

unsafe extern "system" fn register_packet(
    direction: SampClientSdkDirection,
    callback: Option<SampClientSdkEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
) -> SampClientSdkResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Packet,
    )
}

unsafe extern "system" fn register_rpc(
    direction: SampClientSdkDirection,
    callback: Option<SampClientSdkEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
) -> SampClientSdkResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Rpc,
    )
}

unsafe extern "system" fn unregister(
    subscription: SampClientSdkSubscription,
) -> SampClientSdkResult {
    if subscription.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let removed = host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&subscription.id)
        .is_some();
    if removed {
        debug!("unregistered plugin subscription {}", subscription.id);
        SampClientSdkResult::Ok
    } else {
        SampClientSdkResult::SubscriptionNotFound
    }
}

unsafe extern "system" fn unregister_and_wait(
    subscription: SampClientSdkSubscription,
) -> SampClientSdkResult {
    if subscription.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let listener = {
        let mut subscriptions = host()
            .subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(listener) = subscriptions.get(&subscription.id) else {
            return SampClientSdkResult::SubscriptionNotFound;
        };
        if !listener.can_remove_and_wait() {
            return SampClientSdkResult::CallbackInProgress;
        }
        let Some(listener) = subscriptions.remove(&subscription.id) else {
            return SampClientSdkResult::SubscriptionNotFound;
        };
        listener
    };
    listener.remove_and_wait();
    debug!(
        "unregistered plugin subscription {} and synchronized callbacks",
        subscription.id
    );
    SampClientSdkResult::Ok
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

unsafe extern "system" fn submit_local_dialog(
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_dialog(LocalDialogRequest {
        id,
        style,
        title,
        text,
        button1,
        button2,
    }) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_chat_message(
    style: u32,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    text_colour: u32,
    prefix_colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
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
    match runtime.submit_local_chat_message(LocalChatMessageRequest {
        style,
        text,
        prefix,
        text_colour,
        prefix_colour,
    }) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_death_message(
    killer: *const u8,
    killer_len: usize,
    victim: *const u8,
    victim_len: usize,
    killer_colour: u32,
    victim_colour: u32,
    weapon: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(killer) = (unsafe { copied_nul_free_string(killer, killer_len, 24) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(victim) = (unsafe { copied_nul_free_string(victim, victim_len, 24) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_death_message(LocalDeathMessageRequest {
        killer,
        victim,
        killer_colour,
        victim_colour,
        weapon,
    }) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_dialog_client_side(
    client_side: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(client_side, 0 | 1) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_dialog_client_side(client_side != 0) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_samp_game_state(
    state: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(state, 0 | 9 | 13 | 14 | 15 | 18) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_samp_game_state(state) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_force_unoccupied_sync(
    vehicle: u16,
    seat: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || vehicle >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_force_unoccupied_sync(vehicle, seat) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_delete_textdraw(
    id: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_delete_textdraw(id) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_position(
    id: u16,
    x: f32,
    y: f32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !x.is_finite() || !y.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_position(id, x, y) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_letter_style(
    id: u16,
    width: f32,
    height: f32,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_letter_style(id, width, height, colour) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_proportional(
    id: u16,
    proportional: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || proportional > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_proportional(id, proportional != 0) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_shadow(
    id: u16,
    shadow: u8,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_shadow(id, shadow, colour) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_outline(
    id: u16,
    outline: u8,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_outline(id, outline, colour) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_box(
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_box(id, enabled != 0, colour, width, height) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_alignment(
    id: u16,
    alignment: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXTDRAWS || !(1..=3).contains(&alignment) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_alignment(id, alignment) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_string(
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_string(id, text) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_dialog_selected_item(
    selected: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_dialog_selected_item(selected) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_dialog_editbox_text(
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_dialog_editbox_text(text) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_set_textdraw_model_style(
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_set_textdraw_model_style(
        id,
        crate::runtime::Vector3 { x, y, z },
        zoom,
        colour1,
        colour2,
    ) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_send_rate(
    kind: u8,
    milliseconds: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(kind, 0..=2) || i32::try_from(milliseconds).is_err() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_send_rate(kind, milliseconds) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_cursor_toggle(
    show: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || show > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_cursor_toggle(show != 0) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_chat_display_mode(
    mode: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(mode, 0..=2) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_chat_display_mode(mode) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
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
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_chat_entry(
        id,
        text.to_vec(),
        prefix.to_vec(),
        text_colour,
        prefix_colour,
    ) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_dialog_close(
    button: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || button > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_dialog_close(button) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

fn register_listener(
    direction: SampClientSdkDirection,
    callback: Option<SampClientSdkEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut SampClientSdkSubscription,
    kind: ListenerKind,
) -> SampClientSdkResult {
    let Some(callback) = callback else {
        return SampClientSdkResult::InvalidArgument;
    };
    if subscription.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let direction = match direction {
        SampClientSdkDirection::Incoming => Direction::Incoming,
        SampClientSdkDirection::Outgoing => Direction::Outgoing,
    };
    let user_data = user_data as usize;
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let listener = match kind {
        ListenerKind::Packet => runtime.on_packet(direction, move |event| {
            call_plugin_callback(callback, user_data, event.id(), event.payload_mut())
        }),
        ListenerKind::Rpc => runtime.on_rpc(direction, move |event| {
            call_plugin_callback(callback, user_data, event.id(), event.payload_mut())
        }),
    };

    let id = host().next_subscription.fetch_add(1, Ordering::AcqRel);
    host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(id, listener);
    unsafe { subscription.write(SampClientSdkSubscription { id }) };
    debug!("registered {kind:?} subscription {id}");
    SampClientSdkResult::Ok
}

fn call_plugin_callback(
    callback: SampClientSdkEventCallbackV1,
    user_data: usize,
    id: u8,
    payload: &mut BitStream,
) -> HookAction {
    let mut event = AbiEvent { id, payload };
    let action = unsafe {
        callback(
            user_data as *mut c_void,
            (&mut event as *mut AbiEvent).cast::<SampClientSdkEventV1>(),
        )
    };
    match action {
        SampClientSdkHookAction::Block => HookAction::Block,
        SampClientSdkHookAction::Continue => HookAction::Continue,
    }
}

fn direct_client_result(error: DirectClientError) -> SampClientSdkResult {
    match error {
        DirectClientError::NotReady => SampClientSdkResult::NotReady,
        DirectClientError::UnsupportedVersion => SampClientSdkResult::UnsupportedVersion,
        DirectClientError::QueueFull => SampClientSdkResult::QueueFull,
    }
}

unsafe fn copied_nul_free_string(
    value: *const u8,
    value_len: usize,
    maximum: usize,
) -> Result<Vec<u8>, ()> {
    if value_len > maximum || (value.is_null() && value_len != 0) {
        return Err(());
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len) }
    };
    if value.contains(&0) {
        return Err(());
    }
    Ok(value.to_vec())
}

fn host() -> &'static HostState {
    HOST.get_or_init(|| HostState {
        status: AtomicU32::new(STATUS_WAITING),
        bootstrap_started: AtomicBool::new(false),
        runtime: OnceLock::new(),
        subscriptions: Mutex::new(HashMap::new()),
        next_subscription: AtomicU64::new(1),
    })
}

fn clone_initialized<T>(slot: &OnceLock<Arc<T>>) -> Option<Arc<T>> {
    slot.get().cloned()
}

#[derive(Clone, Copy, Debug)]
enum ListenerKind {
    Packet,
    Rpc,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SampVersion;
    use crate::runtime::{
        ChatEntrySnapshot, LocalDialogSnapshot, LocalPlayerSnapshot, ServerInfoSnapshot,
        TextLabelSnapshot, TextdrawSnapshot,
    };
    use sdk_abi::{
        SampClientSdkActiveDialogV1, SampClientSdkAnimationV1, SampClientSdkCommandResultV1,
        SampClientSdkDialogSnapshotV1, SampClientSdkGangzoneV1, SampClientSdkLocalPlayerV1,
        SampClientSdkPlayerInfoV1, SampClientSdkServerInfoV1, SampClientSdkTextDrawV1,
        SampClientSdkTextLabelV1,
        limits::{MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS},
    };
    use std::sync::{Arc, OnceLock};

    #[test]
    fn initialized_runtime_slot_can_be_reentered_while_a_handle_is_alive() {
        let slot = OnceLock::new();
        slot.set(Arc::new(7_u8)).unwrap();

        let outer = clone_initialized(&slot).unwrap();
        let nested = clone_initialized(&slot).unwrap();

        assert_eq!((*outer, *nested), (7, 7));
        assert_eq!(Arc::strong_count(&outer), 3);
    }

    #[test]
    fn dialog_snapshot_conversion_is_coherent_and_preserves_absence() {
        let raw = conversions::local_dialog_snapshot_to_abi(LocalDialogSnapshot {
            id: 7,
            style: LocalDialogStyle::MessageBox,
            title: b"fixture".to_vec(),
            server_side: false,
            selected_item: None,
            list_item_count: None,
            text: b"body".to_vec(),
            editbox_text: None,
            listbox_items: vec![vec![b'x'; u8::MAX as usize]],
        })
        .expect("bounded snapshot converts");

        assert_eq!(raw.active, 1);
        assert_eq!(raw.has_editbox, 0);
        assert_eq!(raw.editbox_text_len, 0);
        assert_eq!(raw.listbox_item_count, 1);
        assert_eq!(raw.listbox_items[0].len, u8::MAX);
        assert_eq!(raw.listbox_items[0].bytes, [b'x'; u8::MAX as usize]);
    }

    #[test]
    fn dialog_snapshot_conversion_rejects_a_256_byte_list_item() {
        assert!(
            conversions::local_dialog_snapshot_to_abi(LocalDialogSnapshot {
                id: 7,
                style: LocalDialogStyle::List,
                title: b"fixture".to_vec(),
                server_side: false,
                selected_item: None,
                list_item_count: Some(1),
                text: Vec::new(),
                editbox_text: None,
                listbox_items: vec![vec![b'x'; usize::from(u8::MAX) + 1]],
            })
            .is_err()
        );
    }

    #[test]
    fn direct_client_abi_is_not_ready_without_a_runtime() {
        let mut output = SampClientSdkLocalPlayerV1::default();
        assert_eq!(
            unsafe { players::local_player(&mut output) },
            SampClientSdkResult::NotReady
        );
        let mut game_state = 0;
        assert_eq!(
            unsafe { environment::samp_game_state(&mut game_state) },
            SampClientSdkResult::NotReady
        );
        let mut chat_display_mode = 0;
        assert_eq!(
            unsafe { local_state::local_chat_display_mode(&mut chat_display_mode) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_state::local_chat_display_mode(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut cursor_mode = 0;
        assert_eq!(
            unsafe { local_state::local_cursor_mode(&mut cursor_mode) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_state::local_cursor_mode(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut scoreboard_open = 0;
        assert_eq!(
            unsafe { local_state::local_scoreboard_open(&mut scoreboard_open) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_state::local_scoreboard_open(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut dialog_active = 0;
        assert_eq!(
            unsafe { local_state::local_dialog_active(&mut dialog_active) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_state::local_dialog_active(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut active_dialog = SampClientSdkActiveDialogV1::default();
        assert_eq!(
            unsafe { local_state::active_local_dialog(&mut active_dialog) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_state::active_local_dialog(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut dialog_snapshot = SampClientSdkDialogSnapshotV1::default();
        assert_eq!(
            unsafe { dialog::local_dialog_snapshot(&mut dialog_snapshot) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { dialog::local_dialog_snapshot(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut chat_input_active = 0;
        assert_eq!(
            unsafe { local_state::local_chat_input_active(&mut chat_input_active) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_state::local_chat_input_active(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut animation = SampClientSdkAnimationV1::default();
        assert_eq!(
            unsafe { animations::local_animation(0, &mut animation) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { animations::local_animation(0, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut animation_id = 0;
        assert_eq!(
            unsafe {
                animations::local_animation_id(
                    b"AIRPORT".as_ptr(),
                    b"AIRPORT".len(),
                    b"THRW_BARL_THRW".as_ptr(),
                    b"THRW_BARL_THRW".len(),
                    &mut animation_id,
                )
            },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe {
                animations::local_animation_id(
                    std::ptr::null(),
                    1,
                    b"THRW_BARL_THRW".as_ptr(),
                    b"THRW_BARL_THRW".len(),
                    &mut animation_id,
                )
            },
            SampClientSdkResult::InvalidArgument
        );
        let mut player = SampClientSdkPlayerInfoV1::default();
        assert_eq!(
            unsafe { players::player_info(7, &mut player) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { players::player_info(MAX_SAMP_PLAYERS, &mut player) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { players::player_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut player_defined_output = 0;
        assert_eq!(
            unsafe { players::player_defined(7, &mut player_defined_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { players::player_defined(MAX_SAMP_PLAYERS, &mut player_defined_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { players::player_defined(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut player_paused_output = 0;
        assert_eq!(
            unsafe { players::player_paused(7, &mut player_paused_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { players::player_paused(MAX_SAMP_PLAYERS, &mut player_paused_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { players::player_paused(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut count = 0;
        assert_eq!(
            unsafe { players::player_count(1, &mut count) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { players::player_count(2, &mut count) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { players::player_count(1, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut max_id = 0;
        assert_eq!(
            unsafe { players::player_max_id(&mut max_id) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { players::player_max_id(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut vehicle_exists_output = 0;
        assert_eq!(
            unsafe { pools::vehicle_exists(7, &mut vehicle_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { pools::vehicle_exists(MAX_SAMP_VEHICLES, &mut vehicle_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { pools::vehicle_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut text_label_exists_output = 0;
        assert_eq!(
            unsafe { pools::text_label_exists(7, &mut text_label_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe {
                pools::text_label_exists(MAX_SAMP_TEXT_LABELS, &mut text_label_exists_output)
            },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { pools::text_label_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut text_label = SampClientSdkTextLabelV1::default();
        assert_eq!(
            unsafe { snapshots::text_label_info(7, &mut text_label) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { snapshots::text_label_info(MAX_SAMP_TEXT_LABELS, &mut text_label) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { snapshots::text_label_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut textdraw = SampClientSdkTextDrawV1::default();
        assert_eq!(
            unsafe { snapshots::textdraw_info(7, &mut textdraw) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { snapshots::textdraw_info(MAX_SAMP_TEXTDRAWS, &mut textdraw) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { snapshots::textdraw_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut textdraw_exists_output = 0;
        assert_eq!(
            unsafe { pools::textdraw_exists(7, &mut textdraw_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { pools::textdraw_exists(MAX_SAMP_TEXTDRAWS, &mut textdraw_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { pools::textdraw_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut object_exists_output = 0;
        assert_eq!(
            unsafe { pools::object_exists(7, &mut object_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { pools::object_exists(MAX_SAMP_OBJECTS, &mut object_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { pools::object_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut gangzone = SampClientSdkGangzoneV1::default();
        assert_eq!(
            unsafe { snapshots::gangzone_info(7, &mut gangzone) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { snapshots::gangzone_info(MAX_SAMP_GANGZONES, &mut gangzone) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { snapshots::gangzone_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut server = SampClientSdkServerInfoV1::default();
        assert_eq!(
            unsafe { environment::server_info(&mut server) },
            SampClientSdkResult::NotReady
        );
        let mut version = 0;
        assert_eq!(
            unsafe { environment::samp_version(&mut version) },
            SampClientSdkResult::NotReady
        );
        let mut decoded = [0; 1];
        let mut decoded_len = 0;
        let mut read_offset = 0;
        assert_eq!(
            unsafe {
                events::decode_string(
                    std::ptr::null(),
                    0,
                    0,
                    0,
                    decoded.as_mut_ptr(),
                    decoded.len(),
                    &raw mut decoded_len,
                    &raw mut read_offset,
                )
            },
            SampClientSdkResult::NotReady
        );
        let mut receipt = SampClientSdkCommandReceipt::default();
        assert_eq!(
            unsafe {
                submit_local_dialog(
                    7,
                    0,
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    0,
                    &mut receipt,
                )
            },
            SampClientSdkResult::NotReady
        );
        let mut command_result = SampClientSdkCommandResultV1::default();
        let receipt = SampClientSdkCommandReceipt { id: 1 };
        assert_eq!(
            unsafe { commands::command_try_take(receipt, &mut command_result) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { commands::command_wait(receipt, 0, &mut command_result) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { commands::command_release(receipt) },
            SampClientSdkResult::NotReady
        );
    }

    #[test]
    fn owned_string_decode_rejects_invalid_abi_metadata_before_runtime_access() {
        let mut decoded = [0; 1];
        let mut decoded_len = 0;
        let mut read_offset = 0;
        assert_eq!(
            unsafe {
                events::decode_string(
                    std::ptr::null(),
                    0,
                    1,
                    0,
                    decoded.as_mut_ptr(),
                    decoded.len(),
                    &raw mut decoded_len,
                    &raw mut read_offset,
                )
            },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe {
                events::decode_string(
                    std::ptr::null(),
                    0,
                    0,
                    0,
                    decoded.as_mut_ptr(),
                    events::MAX_CODEC_OUTPUT_BYTES + 1,
                    &raw mut decoded_len,
                    &raw mut read_offset,
                )
            },
            SampClientSdkResult::PayloadTooLarge
        );
    }

    #[test]
    fn client_version_uses_stable_abi_values() {
        assert_eq!(environment::samp_version_to_abi(SampVersion::R1), 1);
        assert_eq!(environment::samp_version_to_abi(SampVersion::R5_1), 5);
        assert_eq!(environment::samp_version_to_abi(SampVersion::Dl), 6);
    }

    #[test]
    fn local_snapshot_conversion_uses_only_fixed_abi_storage() {
        let snapshot = LocalPlayerSnapshot {
            id: 5,
            nickname: b"player".to_vec(),
            colour: 0xAABB_CCDD,
            spawned: true,
            health: 75.0,
            armour: 25.0,
            position: crate::runtime::Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            velocity: crate::runtime::Vector3 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            special_action: 7,
            animation_id: 8,
            vehicle_id: Some(9),
            score: 10,
            ping: 11,
        };

        let raw =
            conversions::local_player_to_abi(snapshot).expect("fixture snapshot fits the ABI");
        assert_eq!(raw.nickname_len, 6);
        assert_eq!(&raw.nickname[..6], b"player");
        assert_eq!(raw.has_vehicle, 1);
        assert_eq!(raw.vehicle_id, 9);
        assert_eq!(
            raw.position,
            Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
    }

    #[test]
    fn server_snapshot_conversion_uses_only_fixed_abi_storage() {
        let raw = conversions::server_info_to_abi(ServerInfoSnapshot {
            address: b"127.0.0.1".to_vec(),
            hostname: b"fixture".to_vec(),
            port: 7777,
        })
        .expect("fixture server snapshot fits the ABI");
        assert_eq!(raw.address_len, 9);
        assert_eq!(&raw.address[..9], b"127.0.0.1");
        assert_eq!(raw.hostname_len, 7);
        assert_eq!(&raw.hostname[..7], b"fixture");
        assert_eq!(raw.port, 7777);
    }

    #[test]
    fn text_label_snapshot_conversion_uses_only_fixed_abi_storage() {
        let raw = conversions::text_label_to_abi(TextLabelSnapshot {
            id: 7,
            text: b"fixture label".to_vec(),
            colour: 0xFF11_2233,
            position: crate::runtime::Vector3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            draw_distance: 25.0,
            behind_walls: true,
            attached_player_id: Some(8),
            attached_vehicle_id: None,
        })
        .expect("fixture text label fits the ABI");
        assert_eq!(raw.exists, 1);
        assert_eq!(raw.text_len, 13);
        assert_eq!(&raw.text[..13], b"fixture label");
        assert_eq!(raw.attached_player_id, 8);
        assert_eq!(raw.attached_vehicle_id, u16::MAX);
    }

    #[test]
    fn textdraw_snapshot_conversion_uses_only_fixed_abi_storage() {
        let raw = conversions::textdraw_to_abi(TextdrawSnapshot {
            pool_index: 7,
            text: Vec::new(),
            letter_width: 1.0,
            letter_height: 2.0,
            letter_colour: 0xFF11_2233,
            x: 3.0,
            y: 4.0,
            shadow: 2,
            outline: 3,
            background_colour: 0xFF44_5566,
            style: 5,
            proportional: true,
            align_left: false,
            align_center: true,
            align_right: false,
            box_enabled: true,
            box_width: 6.0,
            box_height: 7.0,
            box_colour: 0xFF77_8899,
            model_id: 10,
            rotation: crate::runtime::Vector3 {
                x: 8.0,
                y: 9.0,
                z: 10.0,
            },
            zoom: 11.0,
            model_colour1: 12,
            model_colour2: 13,
        })
        .expect("fixture textdraw fits the ABI");
        assert_eq!(raw.exists, 1);
        assert_eq!(raw.pool_index, 7);
        assert_eq!(raw.align_center, 1);
        assert_eq!(raw.model_colour2, 13);
    }

    #[test]
    fn chat_entry_snapshot_conversion_uses_only_fixed_abi_storage() {
        let raw = conversions::chat_entry_to_abi(ChatEntrySnapshot {
            id: 7,
            text: b"fixture".to_vec(),
            prefix: b"prefix".to_vec(),
            text_colour: 0xFF11_2233,
            prefix_colour: 0xFF44_5566,
        })
        .expect("fixture chat entry fits the ABI");
        assert_eq!(raw.id, 7);
        assert_eq!(raw.text_len, 7);
        assert_eq!(&raw.text[..7], b"fixture");
        assert_eq!(raw.prefix_len, 6);
        assert_eq!(&raw.prefix[..6], b"prefix");
    }
}
