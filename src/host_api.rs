use crate::{
    AttachError, BitStream, BitStreamError, Direction, HookAction, ListenerHandle, PacketPriority,
    PacketReliability, Runtime, SampVersion, SendError, SendOptions,
    command::CommandError,
    logging,
    runtime::{
        AnimationSnapshot, ChatEntrySnapshot, ClientHookStatus, CodecError, DirectClientError,
        GangzoneSnapshot, LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest,
        LocalDialogRequest, LocalDialogSnapshot, LocalDialogStyle, LocalPlayerSnapshot,
        PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot, TextLabelSnapshot,
        TextdrawSnapshot,
    },
};
use log::{debug, error, info};
use sdk_abi::{
    ABI_VERSION_V1, MAX_SAMP_CHAT_ENTRIES, MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS,
    MAX_SAMP_TEXT_LABELS, MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES, SampClientSdkActiveDialogV1,
    SampClientSdkAnimationV1, SampClientSdkApiV1, SampClientSdkChatEntryV1,
    SampClientSdkChatInputTextV1, SampClientSdkCommandReceipt, SampClientSdkCommandResultV1,
    SampClientSdkDialogSnapshotV1, SampClientSdkDirection, SampClientSdkEventCallbackV1,
    SampClientSdkEventV1, SampClientSdkGangzoneV1, SampClientSdkHookAction,
    SampClientSdkHostStatus, SampClientSdkLocalPlayerV1, SampClientSdkPlayerInfoV1,
    SampClientSdkRemotePlayerStateV1, SampClientSdkResult, SampClientSdkSendOptions,
    SampClientSdkServerInfoV1, SampClientSdkSubscription, SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1, Vector3,
};
use std::{
    collections::HashMap,
    ffi::c_void,
    ptr, slice,
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
    event_id,
    event_reset_read,
    event_clear,
    event_read_u8,
    event_read_u16,
    event_read_u32,
    event_read_f32,
    event_read_bytes,
    event_write_u8,
    event_write_u16,
    event_write_u32,
    event_write_f32,
    event_write_bytes,
    send_packet,
    send_rpc,
    event_replace_bytes,
    unregister_and_wait,
    emulate_incoming_packet,
    emulate_incoming_rpc,
    event_remaining_bits,
    event_read_bits,
    event_replace_bits,
    encode_string,
    event_read_encoded_string,
    show_local_dialog,
    local_player,
    samp_game_state,
    samp_version,
    decode_string,
    server_info,
    show_local_chat_message,
    show_local_death_message,
    local_chat_display_mode,
    local_cursor_mode,
    local_scoreboard_open,
    local_dialog_active,
    local_chat_input_active,
    local_animation,
    local_animation_id,
    player_info,
    player_count,
    player_max_id,
    vehicle_exists,
    active_local_dialog,
    text_label_exists,
    textdraw_exists,
    object_exists,
    gangzone_info,
    text_label_info,
    textdraw_info,
    player_defined,
    player_paused,
    remote_player_state,
    submit_local_dialog,
    submit_local_chat_message,
    submit_local_death_message,
    command_try_take,
    command_wait,
    command_release,
    submit_packet,
    submit_rpc,
    submit_emulate_incoming_packet,
    submit_emulate_incoming_rpc,
    raw_rakclient,
    raw_player_pool,
    raw_vehicle_pool,
    submit_local_cursor_mode,
    submit_local_scoreboard_open,
    submit_local_dialog_client_side,
    submit_samp_game_state,
    raw_local_player,
    submit_local_player_spawn,
    submit_local_player_special_action,
    submit_send_rate,
    submit_local_cursor_toggle,
    submit_local_chat_display_mode,
    raw_rakpeer,
    submit_local_dialog_close,
    submit_local_chat_input_text,
    submit_local_chat_input_enabled,
    submit_local_chat_input_process,
    local_chat_input_text,
    submit_player_colour,
    submit_local_player_name,
    submit_force_unoccupied_sync,
    submit_connect_to_server,
    submit_disconnect_with_reason,
    submit_delete_textdraw,
    submit_set_textdraw_position,
    submit_set_textdraw_letter_style,
    submit_set_textdraw_proportional,
    submit_set_textdraw_shadow,
    submit_set_textdraw_outline,
    submit_set_textdraw_box,
    submit_set_textdraw_alignment,
    submit_set_textdraw_string,
    local_dialog_selected_item,
    submit_local_dialog_selected_item,
    submit_delete_text_label,
    local_dialog_list_item_count,
    submit_set_textdraw_model_style,
    submit_local_chat_entry,
    chat_entry_info,
    submit_create_text_label,
    local_dialog_snapshot,
    submit_local_dialog_editbox_text,
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

unsafe extern "system" fn event_id(event: *const SampClientSdkEventV1) -> u8 {
    if event.is_null() {
        return 0;
    }
    unsafe { event.cast::<AbiEvent>().as_ref() }.map_or(0, |event| event.id)
}

unsafe extern "system" fn event_reset_read(
    event: *mut SampClientSdkEventV1,
) -> SampClientSdkResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.reset_read();
    SampClientSdkResult::Ok
}

unsafe extern "system" fn event_clear(event: *mut SampClientSdkEventV1) -> SampClientSdkResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.clear();
    SampClientSdkResult::Ok
}

unsafe extern "system" fn event_read_u8(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u8() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_u16(
    event: *mut SampClientSdkEventV1,
    output: *mut u16,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u16() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_u32(
    event: *mut SampClientSdkEventV1,
    output: *mut u32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u32() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_f32(
    event: *mut SampClientSdkEventV1,
    output: *mut f32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_f32() {
        Ok(value) => {
            unsafe { output.write(value) };
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_bytes(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    len: usize,
) -> SampClientSdkResult {
    if output.is_null() && len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bytes(len) {
        Ok(bytes) => {
            if len != 0 {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, len) };
            }
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_write_u8(
    event: *mut SampClientSdkEventV1,
    value: u8,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_u8(value))
}

unsafe extern "system" fn event_write_u16(
    event: *mut SampClientSdkEventV1,
    value: u16,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_u16(value))
}

unsafe extern "system" fn event_write_u32(
    event: *mut SampClientSdkEventV1,
    value: u32,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_u32(value))
}

unsafe extern "system" fn event_write_f32(
    event: *mut SampClientSdkEventV1,
    value: f32,
) -> SampClientSdkResult {
    write_event(event, |stream| stream.write_f32(value))
}

unsafe extern "system" fn event_write_bytes(
    event: *mut SampClientSdkEventV1,
    value: *const u8,
    len: usize,
) -> SampClientSdkResult {
    if value.is_null() && len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, len) }
    };
    write_event(event, |stream| stream.write_bytes(bytes))
}

unsafe extern "system" fn send_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
) -> SampClientSdkResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Packet)
}

unsafe extern "system" fn send_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
) -> SampClientSdkResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Rpc)
}

unsafe extern "system" fn emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Packet)
}

unsafe extern "system" fn emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Rpc)
}

unsafe extern "system" fn submit_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_send(
        id,
        data,
        byte_len,
        bit_len,
        options,
        ListenerKind::Packet,
        receipt,
    )
}

unsafe extern "system" fn submit_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_send(
        id,
        data,
        byte_len,
        bit_len,
        options,
        ListenerKind::Rpc,
        receipt,
    )
}

unsafe extern "system" fn submit_emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Packet, receipt)
}

unsafe extern "system" fn submit_emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    submit_emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Rpc, receipt)
}

unsafe extern "system" fn raw_rakclient(output: *mut *mut c_void) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_rakclient)
}

unsafe extern "system" fn raw_rakpeer(output: *mut *mut c_void) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_rakpeer)
}

unsafe extern "system" fn raw_player_pool(output: *mut *mut c_void) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_player_pool)
}

unsafe extern "system" fn raw_vehicle_pool(output: *mut *mut c_void) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_vehicle_pool)
}

unsafe extern "system" fn raw_local_player(output: *mut *mut c_void) -> SampClientSdkResult {
    raw_native_address(output, Runtime::raw_local_player)
}

fn raw_native_address(
    output: *mut *mut c_void,
    lookup: fn(&Runtime) -> Option<*mut c_void>,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = ptr::null_mut();
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let Some(address) = lookup(&runtime) else {
        return SampClientSdkResult::NotReady;
    };
    *output = address;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn event_replace_bytes(
    event: *mut SampClientSdkEventV1,
    value: *const u8,
    len: usize,
) -> SampClientSdkResult {
    if value.is_null() && len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, len) }
    };
    write_event(event, |stream| stream.replace_bytes(bytes))
}

unsafe extern "system" fn event_remaining_bits(event: *mut SampClientSdkEventV1) -> usize {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return 0;
    };
    unsafe { &*event.payload }.remaining_bits()
}

unsafe extern "system" fn event_read_bits(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    bit_len: usize,
) -> SampClientSdkResult {
    if output.is_null() && bit_len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bits(bit_len) {
        Ok(bytes) => {
            if !bytes.is_empty() {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
            }
            SampClientSdkResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_replace_bits(
    event: *mut SampClientSdkEventV1,
    value: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> SampClientSdkResult {
    if value.is_null() && byte_len != 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    if bit_len > byte_len.saturating_mul(u8::BITS as usize) {
        return SampClientSdkResult::InvalidArgument;
    }
    let bytes = if byte_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, byte_len) }
    };
    write_event(event, |stream| stream.replace_bits(bytes, bit_len))
}

unsafe extern "system" fn encode_string(
    value: *const u8,
    value_len: usize,
    output: *mut u8,
    output_capacity: usize,
    output_bit_len: *mut usize,
) -> SampClientSdkResult {
    if (value.is_null() && value_len != 0)
        || (output.is_null() && output_capacity != 0)
        || output_bit_len.is_null()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len) }
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let encoded = match runtime.encode_string(value) {
        Ok(encoded) => encoded,
        Err(error) => return codec_result(error),
    };
    if encoded.len_bytes() > output_capacity {
        return SampClientSdkResult::PayloadTooLarge;
    }
    if encoded.len_bytes() != 0 {
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_bytes().as_ptr(), output, encoded.len_bytes())
        };
    }
    unsafe { output_bit_len.write(encoded.len_bits()) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn event_read_encoded_string(
    event: *mut SampClientSdkEventV1,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
) -> SampClientSdkResult {
    if output.is_null() || output_capacity == 0 || output_len.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output, output_capacity) };
    match runtime.decode_string(unsafe { &mut *event.payload }, output) {
        Ok(length) => {
            unsafe { output_len.write(length) };
            SampClientSdkResult::Ok
        }
        Err(error) => codec_result(error),
    }
}

const MAX_CODEC_INPUT_BITS: usize = 16 * 1024 * 1024 * u8::BITS as usize;
const MAX_CODEC_OUTPUT_BYTES: usize = 4_096;

unsafe extern "system" fn decode_string(
    input: *const u8,
    input_byte_len: usize,
    input_bit_len: usize,
    input_read_offset: usize,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
    output_read_offset: *mut usize,
) -> SampClientSdkResult {
    if (input.is_null() && input_byte_len != 0)
        || output.is_null()
        || output_capacity == 0
        || output_len.is_null()
        || output_read_offset.is_null()
        || input_bit_len > input_byte_len.saturating_mul(u8::BITS as usize)
        || input_read_offset > input_bit_len
    {
        return SampClientSdkResult::InvalidArgument;
    }
    if input_bit_len > MAX_CODEC_INPUT_BITS || output_capacity > MAX_CODEC_OUTPUT_BYTES {
        return SampClientSdkResult::PayloadTooLarge;
    }
    let input_len = input_bit_len.div_ceil(u8::BITS as usize);
    let input = if input_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input, input_len) }
    };
    let Ok(mut payload) = BitStream::from_bytes_with_bits(input.to_vec(), input_bit_len) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if payload.set_read_offset_bits(input_read_offset).is_err() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output, output_capacity) };
    match runtime.decode_string(&mut payload, output) {
        Ok(length) => {
            let read_offset = payload.read_offset_bits();
            if length >= output_capacity || read_offset > input_bit_len {
                return SampClientSdkResult::NativeCallFailed;
            }
            unsafe {
                output_len.write(length);
                output_read_offset.write(read_offset);
            }
            SampClientSdkResult::Ok
        }
        Err(error) => codec_result(error),
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

unsafe extern "system" fn submit_local_cursor_mode(
    mode: i32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(mode, 0..=4) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_cursor_mode(mode) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_scoreboard_open(
    open: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(open, 0 | 1) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_scoreboard_open(open != 0) {
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

unsafe extern "system" fn submit_local_player_spawn(
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_player_spawn() {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_player_special_action(
    action: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || !matches!(action, 0..=12 | 20..=25 | 68) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_player_special_action(action) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_player_colour(
    id: u16,
    colour: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_player_colour(id, colour) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_player_name(
    name: *const u8,
    name_len: usize,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, 255) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_player_name(name) {
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

unsafe extern "system" fn submit_connect_to_server(
    address: *const u8,
    address_len: usize,
    port: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || port == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(address) = (unsafe { copied_nul_free_string(address, address_len, 256) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if address.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_connect_to_server(address, port) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_disconnect_with_reason(
    block_duration: u32,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_disconnect_with_reason(block_duration) {
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

unsafe extern "system" fn local_dialog_selected_item(output: *mut i32) -> SampClientSdkResult {
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

unsafe extern "system" fn submit_delete_text_label(
    id: u16,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || id >= MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_delete_text_label(id) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_create_text_label(
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
        || text_len > sdk_abi::MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || text.is_null()
        || !position.x.is_finite()
        || !position.y.is_finite()
        || !position.z.is_finite()
        || !draw_distance.is_finite()
        || behind_walls > 1
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
    match runtime.submit_create_text_label(
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
    ) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_dialog_list_item_count(output: *mut i32) -> SampClientSdkResult {
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

unsafe extern "system" fn local_dialog_snapshot(
    output: *mut SampClientSdkDialogSnapshotV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_dialog_state() {
        Ok(Some(snapshot)) => match local_dialog_snapshot_to_abi(snapshot) {
            Ok(snapshot) => snapshot,
            Err(()) => return SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => SampClientSdkDialogSnapshotV1::default(),
        Err(error) => return direct_client_result(error),
    };
    *output = snapshot;
    SampClientSdkResult::Ok
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

unsafe extern "system" fn submit_local_chat_input_text(
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
    match runtime.submit_local_chat_input_text(text) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_chat_input_enabled(
    enabled: u8,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() || enabled > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.submit_local_chat_input_enabled(enabled != 0) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn submit_local_chat_input_process(
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
    match runtime.submit_local_chat_input_process(text) {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_chat_input_text(
    output: *mut SampClientSdkChatInputTextV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_input_text() {
        Ok(text) => {
            if text.len() > output.bytes.len() {
                return SampClientSdkResult::NativeCallFailed;
            }
            *output = SampClientSdkChatInputTextV1::default();
            output.len = text.len() as u8;
            output.bytes[..text.len()].copy_from_slice(&text);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn command_try_take(
    receipt: SampClientSdkCommandReceipt,
    output: *mut SampClientSdkCommandResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.try_take_command(receipt.id) {
        Ok(Some(result)) => {
            unsafe {
                output.write(SampClientSdkCommandResultV1 {
                    status: command_completion_result(result),
                });
            }
            SampClientSdkResult::Ok
        }
        Ok(None) => SampClientSdkResult::CommandPending,
        Err(error) => command_error_result(error),
    }
}

unsafe extern "system" fn command_wait(
    receipt: SampClientSdkCommandReceipt,
    timeout_ms: u32,
    output: *mut SampClientSdkCommandResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.wait_for_command(receipt.id, Duration::from_millis(u64::from(timeout_ms))) {
        Ok(result) => {
            unsafe {
                output.write(SampClientSdkCommandResultV1 {
                    status: command_completion_result(result),
                });
            }
            SampClientSdkResult::Ok
        }
        Err(error) => command_error_result(error),
    }
}

unsafe extern "system" fn command_release(
    receipt: SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    runtime
        .release_command(receipt.id)
        .map_or_else(command_error_result, |_| SampClientSdkResult::Ok)
}

unsafe extern "system" fn local_player(
    output: *mut SampClientSdkLocalPlayerV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_player() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = local_player_to_abi(snapshot) else {
        return SampClientSdkResult::NativeCallFailed;
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn samp_game_state(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.samp_game_state() {
        Ok(game_state) => {
            *output = game_state;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn server_info(
    output: *mut SampClientSdkServerInfoV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.server_info() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = server_info_to_abi(snapshot) else {
        return SampClientSdkResult::NativeCallFailed;
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn local_chat_display_mode(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_display_mode() {
        Ok(mode) => {
            *output = mode;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_cursor_mode(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_cursor_mode() {
        Ok(mode) => {
            *output = mode;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_scoreboard_open(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_scoreboard_open() {
        Ok(open) => {
            *output = u8::from(open);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_dialog_active(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_dialog_active() {
        Ok(active) => {
            *output = u8::from(active);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn active_local_dialog(
    output: *mut SampClientSdkActiveDialogV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_dialog_state() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let snapshot = match snapshot {
        Some(snapshot) => match local_dialog_state_to_abi(&snapshot) {
            Ok(snapshot) => snapshot,
            Err(()) => return SampClientSdkResult::NativeCallFailed,
        },
        None => SampClientSdkActiveDialogV1::default(),
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn local_chat_input_active(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_chat_input_active() {
        Ok(active) => {
            *output = u8::from(active);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_animation(
    id: u16,
    output: *mut SampClientSdkAnimationV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let snapshot = match runtime.local_animation(id) {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = animation_to_abi(snapshot) else {
        return SampClientSdkResult::NativeCallFailed;
    };
    *output = snapshot;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn local_animation_id(
    name: *const u8,
    name_len: usize,
    file: *const u8,
    file_len: usize,
    output: *mut i32,
) -> SampClientSdkResult {
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, 35) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(file) = (unsafe { copied_nul_free_string(file, file_len, 35) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if name.is_empty() || file.is_empty() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.local_animation_id(&name, &file) {
        Ok(Some(id)) => {
            *output = i32::from(id);
            SampClientSdkResult::Ok
        }
        Ok(None) => {
            *output = -1;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_info(
    id: u16,
    output: *mut SampClientSdkPlayerInfoV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_info(id) {
        Ok(Some(snapshot)) => match player_info_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkPlayerInfoV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn remote_player_state(
    id: u16,
    output: *mut SampClientSdkRemotePlayerStateV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.remote_player_state(id) {
        Ok(Some(snapshot)) => match remote_player_state_to_abi(snapshot) {
            Ok(snapshot) => {
                unsafe { *output = snapshot };
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            unsafe { *output = SampClientSdkRemotePlayerStateV1::default() };
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_defined(id: u16, output: *mut u8) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_defined(id) {
        Ok(defined) => {
            *output = u8::from(defined);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_paused(id: u16, output: *mut u8) -> SampClientSdkResult {
    if id >= MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_paused(id) {
        Ok(paused) => {
            *output = u8::from(paused);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_count(include_npcs: u8, output: *mut u16) -> SampClientSdkResult {
    let include_npcs = match include_npcs {
        0 => false,
        1 => true,
        _ => return SampClientSdkResult::InvalidArgument,
    };
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_count(include_npcs) {
        Ok(count) => {
            *output = count;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_max_id(output: *mut u16) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.player_max_id() {
        Ok(id) => {
            *output = id;
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn vehicle_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    if id >= MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.vehicle_exists(id) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn text_label_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    if id >= MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.text_label_exists(id) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn textdraw_exists(pool_index: u16, output: *mut u8) -> SampClientSdkResult {
    if pool_index >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.textdraw_exists(pool_index) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn object_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    if id >= MAX_SAMP_OBJECTS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.object_exists(id) {
        Ok(exists) => {
            *output = u8::from(exists);
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn gangzone_info(
    id: u16,
    output: *mut SampClientSdkGangzoneV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_GANGZONES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.gangzone(id) {
        Ok(Some(snapshot)) => match gangzone_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkGangzoneV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn text_label_info(
    id: u16,
    output: *mut SampClientSdkTextLabelV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.text_label(id) {
        Ok(Some(snapshot)) => match text_label_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkTextLabelV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn textdraw_info(
    pool_index: u16,
    output: *mut SampClientSdkTextDrawV1,
) -> SampClientSdkResult {
    if pool_index >= MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.textdraw(pool_index) {
        Ok(Some(snapshot)) => match textdraw_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = SampClientSdkTextDrawV1::default();
            SampClientSdkResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn chat_entry_info(
    id: u16,
    output: *mut SampClientSdkChatEntryV1,
) -> SampClientSdkResult {
    if id >= MAX_SAMP_CHAT_ENTRIES {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    match runtime.chat_entry(id) {
        Ok(snapshot) => match chat_entry_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                SampClientSdkResult::Ok
            }
            Err(()) => SampClientSdkResult::NativeCallFailed,
        },
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn samp_version(output: *mut u32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    *output = samp_version_to_abi(runtime.samp_version());
    SampClientSdkResult::Ok
}

const fn samp_version_to_abi(version: SampVersion) -> u32 {
    match version {
        SampVersion::R1 => 1,
        SampVersion::R2 => 2,
        SampVersion::R3_1 => 3,
        SampVersion::R4_2 => 4,
        SampVersion::R5_1 => 5,
        SampVersion::Dl => 6,
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

fn write_event(
    event: *mut SampClientSdkEventV1,
    operation: impl FnOnce(&mut BitStream) -> Result<(), BitStreamError>,
) -> SampClientSdkResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    operation(unsafe { &mut *event.payload })
        .map_or_else(bitstream_result, |_| SampClientSdkResult::Ok)
}

fn send(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    kind: ListenerKind,
) -> SampClientSdkResult {
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(options) = send_options(options) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.send_packet_with_options(id, &payload, options),
        ListenerKind::Rpc => runtime.send_rpc_with_options(id, &payload, options),
    };
    result.map_or_else(send_result, |sent| {
        if sent {
            SampClientSdkResult::Ok
        } else {
            SampClientSdkResult::NativeCallFailed
        }
    })
}

fn submit_send(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: SampClientSdkSendOptions,
    kind: ListenerKind,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Ok(options) = send_options(options) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.submit_packet_with_options(id, &payload, options),
        ListenerKind::Rpc => runtime.submit_rpc_with_options(id, &payload, options),
    };
    match result {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => send_result(error),
    }
}

fn emulate_incoming(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    kind: ListenerKind,
) -> SampClientSdkResult {
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.emulate_incoming_packet(id, payload),
        ListenerKind::Rpc => runtime.emulate_incoming_rpc(id, payload),
    };
    result.map_or_else(send_result, |_| SampClientSdkResult::Ok)
}

fn submit_emulate_incoming(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    kind: ListenerKind,
    receipt: *mut SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return SampClientSdkResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.submit_emulate_incoming_packet(id, payload),
        ListenerKind::Rpc => runtime.submit_emulate_incoming_rpc(id, payload),
    };
    match result {
        Ok(id) => {
            unsafe { receipt.write(SampClientSdkCommandReceipt { id }) };
            SampClientSdkResult::Ok
        }
        Err(error) => send_result(error),
    }
}

unsafe fn stream_from_abi(
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> Result<BitStream, BitStreamError> {
    if data.is_null() && byte_len != 0 {
        return Err(BitStreamError::InvalidOffset {
            offset_bits: bit_len,
            length_bits: 0,
        });
    }
    let bytes = if byte_len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data, byte_len) }.to_vec()
    };
    BitStream::from_bytes_with_bits(bytes, bit_len)
}

unsafe fn abi_event(event: *mut SampClientSdkEventV1) -> Result<&'static mut AbiEvent, ()> {
    let event = unsafe { event.cast::<AbiEvent>().as_mut() }.ok_or(())?;
    if event.payload.is_null() {
        return Err(());
    }
    Ok(event)
}

fn bitstream_result(error: BitStreamError) -> SampClientSdkResult {
    match error {
        BitStreamError::ReadOutOfBounds { .. } => SampClientSdkResult::ReadOutOfBounds,
        BitStreamError::CapacityExceeded { .. } => SampClientSdkResult::PayloadTooLarge,
        BitStreamError::InvalidOffset { .. } => SampClientSdkResult::InvalidArgument,
    }
}

fn send_result(error: SendError) -> SampClientSdkResult {
    match error {
        SendError::ClientNotReady => SampClientSdkResult::NotReady,
        SendError::QueueFull => SampClientSdkResult::QueueFull,
        SendError::PayloadTooLarge => SampClientSdkResult::PayloadTooLarge,
        SendError::NativeCallFailed => SampClientSdkResult::NativeCallFailed,
        SendError::TimestampedPacketUnsupported => SampClientSdkResult::InvalidArgument,
    }
}

fn codec_result(error: CodecError) -> SampClientSdkResult {
    match error {
        CodecError::ClientNotReady => SampClientSdkResult::NotReady,
        CodecError::InvalidArgument => SampClientSdkResult::InvalidArgument,
        CodecError::PayloadTooLarge => SampClientSdkResult::PayloadTooLarge,
        CodecError::NativeCallFailed => SampClientSdkResult::NativeCallFailed,
    }
}

fn direct_client_result(error: DirectClientError) -> SampClientSdkResult {
    match error {
        DirectClientError::NotReady => SampClientSdkResult::NotReady,
        DirectClientError::UnsupportedVersion => SampClientSdkResult::UnsupportedVersion,
        DirectClientError::QueueFull => SampClientSdkResult::QueueFull,
    }
}

fn command_completion_result(result: Result<(), CommandError>) -> SampClientSdkResult {
    result.map_or_else(command_error_result, |_| SampClientSdkResult::Ok)
}

fn command_error_result(error: CommandError) -> SampClientSdkResult {
    match error {
        CommandError::QueueFull => SampClientSdkResult::QueueFull,
        CommandError::ShuttingDown => SampClientSdkResult::ShuttingDown,
        CommandError::NativeFailure => SampClientSdkResult::NativeCallFailed,
        CommandError::UnknownReceipt => SampClientSdkResult::InvalidArgument,
        CommandError::TimedOut => SampClientSdkResult::TimedOut,
        CommandError::WaitRejected => SampClientSdkResult::WaitRejected,
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

fn local_player_to_abi(snapshot: LocalPlayerSnapshot) -> Result<SampClientSdkLocalPlayerV1, ()> {
    let nickname_len = u16::try_from(snapshot.nickname.len()).map_err(|_| ())?;
    if snapshot.nickname.len() > 256 {
        return Err(());
    }
    let mut nickname = [0; 256];
    nickname[..snapshot.nickname.len()].copy_from_slice(&snapshot.nickname);
    Ok(SampClientSdkLocalPlayerV1 {
        id: snapshot.id,
        nickname_len,
        nickname,
        colour: snapshot.colour,
        spawned: u8::from(snapshot.spawned),
        special_action: snapshot.special_action,
        animation_id: snapshot.animation_id,
        health: snapshot.health,
        armour: snapshot.armour,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        velocity: Vector3 {
            x: snapshot.velocity.x,
            y: snapshot.velocity.y,
            z: snapshot.velocity.z,
        },
        has_vehicle: u8::from(snapshot.vehicle_id.is_some()),
        _reserved: 0,
        vehicle_id: snapshot.vehicle_id.unwrap_or_default(),
        score: snapshot.score,
        ping: snapshot.ping,
    })
}

fn local_dialog_state_to_abi(
    snapshot: &LocalDialogSnapshot,
) -> Result<SampClientSdkActiveDialogV1, ()> {
    let title_len = u8::try_from(snapshot.title.len()).map_err(|_| ())?;
    if snapshot.title.len() > 65 || snapshot.title.contains(&0) {
        return Err(());
    }
    let mut title = [0; 65];
    title[..snapshot.title.len()].copy_from_slice(&snapshot.title);
    Ok(SampClientSdkActiveDialogV1 {
        active: 1,
        style: snapshot.style.as_raw() as u8,
        server_side: u8::from(snapshot.server_side),
        _reserved: 0,
        id: snapshot.id,
        title_len,
        title,
    })
}

fn local_dialog_snapshot_to_abi(
    snapshot: LocalDialogSnapshot,
) -> Result<SampClientSdkDialogSnapshotV1, ()> {
    let core = local_dialog_state_to_abi(&snapshot)?;
    let text_len = u16::try_from(snapshot.text.len()).map_err(|_| ())?;
    let listbox_item_count = u8::try_from(snapshot.listbox_items.len()).map_err(|_| ())?;
    let mut output = SampClientSdkDialogSnapshotV1::default();
    if snapshot.text.len() > output.text.len()
        || snapshot.text.contains(&0)
        || snapshot.listbox_items.len() > output.listbox_items.len()
    {
        return Err(());
    }

    output.active = core.active;
    output.style = core.style;
    output.server_side = core.server_side;
    output.id = core.id;
    output.title_len = core.title_len;
    output.title = core.title;
    output.text_len = text_len;
    output.text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    output.listbox_item_count = listbox_item_count;

    if let Some(editbox_text) = snapshot.editbox_text {
        let editbox_text_len = u8::try_from(editbox_text.len()).map_err(|_| ())?;
        if editbox_text.len() > output.editbox_text.len() || editbox_text.contains(&0) {
            return Err(());
        }
        output.has_editbox = 1;
        output.editbox_text_len = editbox_text_len;
        output.editbox_text[..editbox_text.len()].copy_from_slice(&editbox_text);
    }

    for (raw, item) in output.listbox_items.iter_mut().zip(snapshot.listbox_items) {
        let len = u8::try_from(item.len()).map_err(|_| ())?;
        if item.len() > raw.bytes.len() || item.contains(&0) {
            return Err(());
        }
        raw.len = len;
        raw.bytes[..item.len()].copy_from_slice(&item);
    }

    Ok(output)
}

fn player_info_to_abi(snapshot: PlayerInfoSnapshot) -> Result<SampClientSdkPlayerInfoV1, ()> {
    let nickname_len = u16::try_from(snapshot.nickname.len()).map_err(|_| ())?;
    if snapshot.nickname.is_empty()
        || snapshot.nickname.len() > 256
        || snapshot.nickname.contains(&0)
        || (snapshot.is_local && snapshot.is_npc)
    {
        return Err(());
    }
    let mut nickname = [0; 256];
    nickname[..snapshot.nickname.len()].copy_from_slice(&snapshot.nickname);
    Ok(SampClientSdkPlayerInfoV1 {
        exists: 1,
        is_local: u8::from(snapshot.is_local),
        is_npc: u8::from(snapshot.is_npc),
        _reserved: 0,
        id: snapshot.id,
        nickname_len,
        nickname,
        colour: snapshot.colour,
        score: snapshot.score,
        ping: snapshot.ping,
    })
}

fn remote_player_state_to_abi(
    snapshot: RemotePlayerStateSnapshot,
) -> Result<SampClientSdkRemotePlayerStateV1, ()> {
    if !snapshot.health.is_finite() || !snapshot.armour.is_finite() {
        return Err(());
    }
    Ok(SampClientSdkRemotePlayerStateV1 {
        exists: 1,
        special_action: snapshot.special_action,
        _reserved: 0,
        id: snapshot.id,
        animation_id: snapshot.animation_id,
        health: snapshot.health,
        armour: snapshot.armour,
    })
}

fn gangzone_to_abi(snapshot: GangzoneSnapshot) -> Result<SampClientSdkGangzoneV1, ()> {
    if !snapshot.left.is_finite()
        || !snapshot.bottom.is_finite()
        || !snapshot.right.is_finite()
        || !snapshot.top.is_finite()
    {
        return Err(());
    }
    Ok(SampClientSdkGangzoneV1 {
        exists: 1,
        _reserved: [0; 3],
        id: snapshot.id,
        _reserved2: 0,
        left: snapshot.left,
        bottom: snapshot.bottom,
        right: snapshot.right,
        top: snapshot.top,
        colour: snapshot.colour,
        alternate_colour: snapshot.alternate_colour,
    })
}

fn text_label_to_abi(snapshot: TextLabelSnapshot) -> Result<SampClientSdkTextLabelV1, ()> {
    let text_len = u16::try_from(snapshot.text.len()).map_err(|_| ())?;
    if snapshot.text.len() > sdk_abi::MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || snapshot.text.contains(&0)
        || !snapshot.position.x.is_finite()
        || !snapshot.position.y.is_finite()
        || !snapshot.position.z.is_finite()
        || !snapshot.draw_distance.is_finite()
    {
        return Err(());
    }
    let mut text = [0; sdk_abi::MAX_SAMP_TEXT_LABEL_TEXT_BYTES];
    text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    Ok(SampClientSdkTextLabelV1 {
        exists: 1,
        behind_walls: u8::from(snapshot.behind_walls),
        _reserved: [0; 2],
        id: snapshot.id,
        attached_player_id: snapshot.attached_player_id.unwrap_or(u16::MAX),
        attached_vehicle_id: snapshot.attached_vehicle_id.unwrap_or(u16::MAX),
        _reserved2: 0,
        colour: snapshot.colour,
        position: Vector3 {
            x: snapshot.position.x,
            y: snapshot.position.y,
            z: snapshot.position.z,
        },
        draw_distance: snapshot.draw_distance,
        text_len,
        _reserved3: [0; 2],
        text,
    })
}

fn textdraw_to_abi(snapshot: TextdrawSnapshot) -> Result<SampClientSdkTextDrawV1, ()> {
    if !snapshot.letter_width.is_finite()
        || !snapshot.letter_height.is_finite()
        || !snapshot.x.is_finite()
        || !snapshot.y.is_finite()
        || !snapshot.box_width.is_finite()
        || !snapshot.box_height.is_finite()
        || !snapshot.rotation.x.is_finite()
        || !snapshot.rotation.y.is_finite()
        || !snapshot.rotation.z.is_finite()
        || !snapshot.zoom.is_finite()
    {
        return Err(());
    }
    if snapshot.text.len() > sdk_abi::MAX_SAMP_TEXTDRAW_STRING_BYTES || snapshot.text.contains(&0) {
        return Err(());
    }
    let mut text = [0; sdk_abi::MAX_SAMP_TEXTDRAW_STRING_BYTES];
    text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    Ok(SampClientSdkTextDrawV1 {
        exists: 1,
        proportional: u8::from(snapshot.proportional),
        align_left: u8::from(snapshot.align_left),
        align_center: u8::from(snapshot.align_center),
        align_right: u8::from(snapshot.align_right),
        box_enabled: u8::from(snapshot.box_enabled),
        _reserved: [0; 2],
        pool_index: snapshot.pool_index,
        shadow: snapshot.shadow,
        outline: snapshot.outline,
        letter_width: snapshot.letter_width,
        letter_height: snapshot.letter_height,
        letter_colour: snapshot.letter_colour,
        x: snapshot.x,
        y: snapshot.y,
        background_colour: snapshot.background_colour,
        style: snapshot.style,
        box_width: snapshot.box_width,
        box_height: snapshot.box_height,
        box_colour: snapshot.box_colour,
        model_id: snapshot.model_id,
        _reserved2: 0,
        rotation: Vector3 {
            x: snapshot.rotation.x,
            y: snapshot.rotation.y,
            z: snapshot.rotation.z,
        },
        zoom: snapshot.zoom,
        model_colour1: snapshot.model_colour1,
        model_colour2: snapshot.model_colour2,
        text_len: snapshot.text.len() as u16,
        _reserved3: [0; 2],
        text,
    })
}

fn chat_entry_to_abi(snapshot: ChatEntrySnapshot) -> Result<SampClientSdkChatEntryV1, ()> {
    if snapshot.id >= MAX_SAMP_CHAT_ENTRIES
        || snapshot.text.len() > sdk_abi::MAX_SAMP_CHAT_ENTRY_TEXT_BYTES
        || snapshot.prefix.len() > sdk_abi::MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES
        || snapshot.text.contains(&0)
        || snapshot.prefix.contains(&0)
    {
        return Err(());
    }
    let mut text = [0; sdk_abi::MAX_SAMP_CHAT_ENTRY_TEXT_BYTES];
    text[..snapshot.text.len()].copy_from_slice(&snapshot.text);
    let mut prefix = [0; sdk_abi::MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES];
    prefix[..snapshot.prefix.len()].copy_from_slice(&snapshot.prefix);
    Ok(SampClientSdkChatEntryV1 {
        id: snapshot.id,
        text_len: snapshot.text.len() as u8,
        prefix_len: snapshot.prefix.len() as u8,
        text_colour: snapshot.text_colour,
        prefix_colour: snapshot.prefix_colour,
        text,
        prefix,
    })
}

fn server_info_to_abi(snapshot: ServerInfoSnapshot) -> Result<SampClientSdkServerInfoV1, ()> {
    let address_len = u16::try_from(snapshot.address.len()).map_err(|_| ())?;
    let hostname_len = u16::try_from(snapshot.hostname.len()).map_err(|_| ())?;
    if snapshot.address.is_empty()
        || snapshot.port == 0
        || snapshot.address.len() > 257
        || snapshot.hostname.len() > 257
    {
        return Err(());
    }
    let mut address = [0; 257];
    address[..snapshot.address.len()].copy_from_slice(&snapshot.address);
    let mut hostname = [0; 257];
    hostname[..snapshot.hostname.len()].copy_from_slice(&snapshot.hostname);
    Ok(SampClientSdkServerInfoV1 {
        address_len,
        hostname_len,
        address,
        hostname,
        port: snapshot.port,
    })
}

fn animation_to_abi(snapshot: AnimationSnapshot) -> Result<SampClientSdkAnimationV1, ()> {
    let name_len = u8::try_from(snapshot.name.len()).map_err(|_| ())?;
    let file_len = u8::try_from(snapshot.file.len()).map_err(|_| ())?;
    if snapshot.name.is_empty()
        || snapshot.file.is_empty()
        || snapshot.name.len() > 35
        || snapshot.file.len() > 35
        || snapshot.name.contains(&0)
        || snapshot.file.contains(&0)
    {
        return Err(());
    }
    let mut name = [0; 36];
    name[..snapshot.name.len()].copy_from_slice(&snapshot.name);
    let mut file = [0; 36];
    file[..snapshot.file.len()].copy_from_slice(&snapshot.file);
    Ok(SampClientSdkAnimationV1 {
        name_len,
        file_len,
        name,
        file,
    })
}

fn send_options(options: SampClientSdkSendOptions) -> Result<SendOptions, ()> {
    let priority = match options.priority {
        0 => PacketPriority::System,
        1 => PacketPriority::High,
        2 => PacketPriority::Medium,
        3 => PacketPriority::Low,
        _ => return Err(()),
    };
    let reliability = match options.reliability {
        6 => PacketReliability::Unreliable,
        7 => PacketReliability::UnreliableSequenced,
        8 => PacketReliability::Reliable,
        9 => PacketReliability::ReliableOrdered,
        10 => PacketReliability::ReliableSequenced,
        _ => return Err(()),
    };
    Ok(SendOptions {
        priority,
        reliability,
        ordering_channel: options.ordering_channel,
        timestamp: options.timestamp,
    })
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
        let raw = local_dialog_snapshot_to_abi(LocalDialogSnapshot {
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
            local_dialog_snapshot_to_abi(LocalDialogSnapshot {
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
            unsafe { local_player(&mut output) },
            SampClientSdkResult::NotReady
        );
        let mut game_state = 0;
        assert_eq!(
            unsafe { samp_game_state(&mut game_state) },
            SampClientSdkResult::NotReady
        );
        let mut chat_display_mode = 0;
        assert_eq!(
            unsafe { local_chat_display_mode(&mut chat_display_mode) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_chat_display_mode(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut cursor_mode = 0;
        assert_eq!(
            unsafe { local_cursor_mode(&mut cursor_mode) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_cursor_mode(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut scoreboard_open = 0;
        assert_eq!(
            unsafe { local_scoreboard_open(&mut scoreboard_open) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_scoreboard_open(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut dialog_active = 0;
        assert_eq!(
            unsafe { local_dialog_active(&mut dialog_active) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_dialog_active(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut active_dialog = SampClientSdkActiveDialogV1::default();
        assert_eq!(
            unsafe { active_local_dialog(&mut active_dialog) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { active_local_dialog(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut dialog_snapshot = SampClientSdkDialogSnapshotV1::default();
        assert_eq!(
            unsafe { local_dialog_snapshot(&mut dialog_snapshot) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_dialog_snapshot(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut chat_input_active = 0;
        assert_eq!(
            unsafe { local_chat_input_active(&mut chat_input_active) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_chat_input_active(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut animation = SampClientSdkAnimationV1::default();
        assert_eq!(
            unsafe { local_animation(0, &mut animation) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { local_animation(0, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut animation_id = 0;
        assert_eq!(
            unsafe {
                local_animation_id(
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
                local_animation_id(
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
            unsafe { player_info(7, &mut player) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { player_info(MAX_SAMP_PLAYERS, &mut player) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { player_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut player_defined_output = 0;
        assert_eq!(
            unsafe { player_defined(7, &mut player_defined_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { player_defined(MAX_SAMP_PLAYERS, &mut player_defined_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { player_defined(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut player_paused_output = 0;
        assert_eq!(
            unsafe { player_paused(7, &mut player_paused_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { player_paused(MAX_SAMP_PLAYERS, &mut player_paused_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { player_paused(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut count = 0;
        assert_eq!(
            unsafe { player_count(1, &mut count) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { player_count(2, &mut count) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { player_count(1, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut max_id = 0;
        assert_eq!(
            unsafe { player_max_id(&mut max_id) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { player_max_id(std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut vehicle_exists_output = 0;
        assert_eq!(
            unsafe { vehicle_exists(7, &mut vehicle_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { vehicle_exists(MAX_SAMP_VEHICLES, &mut vehicle_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { vehicle_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut text_label_exists_output = 0;
        assert_eq!(
            unsafe { text_label_exists(7, &mut text_label_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { text_label_exists(MAX_SAMP_TEXT_LABELS, &mut text_label_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { text_label_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut text_label = SampClientSdkTextLabelV1::default();
        assert_eq!(
            unsafe { text_label_info(7, &mut text_label) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { text_label_info(MAX_SAMP_TEXT_LABELS, &mut text_label) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { text_label_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut textdraw = SampClientSdkTextDrawV1::default();
        assert_eq!(
            unsafe { textdraw_info(7, &mut textdraw) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { textdraw_info(MAX_SAMP_TEXTDRAWS, &mut textdraw) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { textdraw_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut textdraw_exists_output = 0;
        assert_eq!(
            unsafe { textdraw_exists(7, &mut textdraw_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { textdraw_exists(MAX_SAMP_TEXTDRAWS, &mut textdraw_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { textdraw_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut object_exists_output = 0;
        assert_eq!(
            unsafe { object_exists(7, &mut object_exists_output) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { object_exists(MAX_SAMP_OBJECTS, &mut object_exists_output) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { object_exists(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut gangzone = SampClientSdkGangzoneV1::default();
        assert_eq!(
            unsafe { gangzone_info(7, &mut gangzone) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { gangzone_info(MAX_SAMP_GANGZONES, &mut gangzone) },
            SampClientSdkResult::InvalidArgument
        );
        assert_eq!(
            unsafe { gangzone_info(7, std::ptr::null_mut()) },
            SampClientSdkResult::InvalidArgument
        );
        let mut server = SampClientSdkServerInfoV1::default();
        assert_eq!(
            unsafe { server_info(&mut server) },
            SampClientSdkResult::NotReady
        );
        let mut version = 0;
        assert_eq!(
            unsafe { samp_version(&mut version) },
            SampClientSdkResult::NotReady
        );
        let mut decoded = [0; 1];
        let mut decoded_len = 0;
        let mut read_offset = 0;
        assert_eq!(
            unsafe {
                decode_string(
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
            unsafe { command_try_take(receipt, &mut command_result) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { command_wait(receipt, 0, &mut command_result) },
            SampClientSdkResult::NotReady
        );
        assert_eq!(
            unsafe { command_release(receipt) },
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
                decode_string(
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
                decode_string(
                    std::ptr::null(),
                    0,
                    0,
                    0,
                    decoded.as_mut_ptr(),
                    MAX_CODEC_OUTPUT_BYTES + 1,
                    &raw mut decoded_len,
                    &raw mut read_offset,
                )
            },
            SampClientSdkResult::PayloadTooLarge
        );
    }

    #[test]
    fn client_version_uses_stable_abi_values() {
        assert_eq!(samp_version_to_abi(SampVersion::R1), 1);
        assert_eq!(samp_version_to_abi(SampVersion::R5_1), 5);
        assert_eq!(samp_version_to_abi(SampVersion::Dl), 6);
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

        let raw = local_player_to_abi(snapshot).expect("fixture snapshot fits the ABI");
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
        let raw = server_info_to_abi(ServerInfoSnapshot {
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
        let raw = text_label_to_abi(TextLabelSnapshot {
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
        let raw = textdraw_to_abi(TextdrawSnapshot {
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
        let raw = chat_entry_to_abi(ChatEntrySnapshot {
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
