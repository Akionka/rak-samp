use crate::{
    AttachError, BitStream, BitStreamError, Direction, HookAction, ListenerHandle, PacketPriority,
    PacketReliability, Runtime, SampVersion, SendError, SendOptions, logging,
    runtime::{
        AnimationSnapshot, ClientHookStatus, CodecError, DirectClientError,
        LocalChatMessageRequest, LocalChatMessageStyle, LocalDeathMessageRequest,
        LocalDialogRequest, LocalDialogStyle, LocalPlayerSnapshot, PlayerInfoSnapshot,
        ServerInfoSnapshot,
    },
};
use log::{debug, error, info};
use rak_samp_plugin_api::{
    ABI_VERSION_V1, MAX_SAMP_PLAYERS, RakSampAnimationV1, RakSampApiV1, RakSampDirection,
    RakSampEventCallbackV1, RakSampEventV1, RakSampHookAction, RakSampHostStatus,
    RakSampLocalPlayerV1, RakSampPlayerInfoV1, RakSampResult, RakSampSendOptions,
    RakSampServerInfoV1, RakSampSubscription, Vector3,
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

const STATUS_WAITING: u32 = RakSampHostStatus::WaitingForSamp as u32;
const STATUS_READY: u32 = RakSampHostStatus::Ready as u32;
const STATUS_FAILED: u32 = RakSampHostStatus::Failed as u32;

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
pub extern "system" fn RakSamp_GetApiV1(requested_version: u32) -> *const RakSampApiV1 {
    if requested_version == ABI_VERSION_V1 {
        &RAK_SAMP_API_V1
    } else {
        debug!("rejected unsupported plugin ABI version {requested_version}");
        ptr::null()
    }
}

static RAK_SAMP_API_V1: RakSampApiV1 = RakSampApiV1 {
    abi_version: ABI_VERSION_V1,
    size: std::mem::size_of::<RakSampApiV1>() as u32,
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
};

extern "system" fn host_status() -> RakSampHostStatus {
    match host().status.load(Ordering::Acquire) {
        STATUS_READY => RakSampHostStatus::Ready,
        STATUS_FAILED => RakSampHostStatus::Failed,
        _ => RakSampHostStatus::WaitingForSamp,
    }
}

unsafe extern "system" fn register_packet(
    direction: RakSampDirection,
    callback: Option<RakSampEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakSampSubscription,
) -> RakSampResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Packet,
    )
}

unsafe extern "system" fn register_rpc(
    direction: RakSampDirection,
    callback: Option<RakSampEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakSampSubscription,
) -> RakSampResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Rpc,
    )
}

unsafe extern "system" fn unregister(subscription: RakSampSubscription) -> RakSampResult {
    if subscription.id == 0 {
        return RakSampResult::InvalidArgument;
    }
    let removed = host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&subscription.id)
        .is_some();
    if removed {
        debug!("unregistered plugin subscription {}", subscription.id);
        RakSampResult::Ok
    } else {
        RakSampResult::SubscriptionNotFound
    }
}

unsafe extern "system" fn unregister_and_wait(subscription: RakSampSubscription) -> RakSampResult {
    if subscription.id == 0 {
        return RakSampResult::InvalidArgument;
    }
    let listener = {
        let mut subscriptions = host()
            .subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(listener) = subscriptions.get(&subscription.id) else {
            return RakSampResult::SubscriptionNotFound;
        };
        if !listener.can_remove_and_wait() {
            return RakSampResult::CallbackInProgress;
        }
        let Some(listener) = subscriptions.remove(&subscription.id) else {
            return RakSampResult::SubscriptionNotFound;
        };
        listener
    };
    listener.remove_and_wait();
    debug!(
        "unregistered plugin subscription {} and synchronized callbacks",
        subscription.id
    );
    RakSampResult::Ok
}

unsafe extern "system" fn event_id(event: *const RakSampEventV1) -> u8 {
    if event.is_null() {
        return 0;
    }
    unsafe { event.cast::<AbiEvent>().as_ref() }.map_or(0, |event| event.id)
}

unsafe extern "system" fn event_reset_read(event: *mut RakSampEventV1) -> RakSampResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.reset_read();
    RakSampResult::Ok
}

unsafe extern "system" fn event_clear(event: *mut RakSampEventV1) -> RakSampResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.clear();
    RakSampResult::Ok
}

unsafe extern "system" fn event_read_u8(
    event: *mut RakSampEventV1,
    output: *mut u8,
) -> RakSampResult {
    if output.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u8() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakSampResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_u16(
    event: *mut RakSampEventV1,
    output: *mut u16,
) -> RakSampResult {
    if output.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u16() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakSampResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_u32(
    event: *mut RakSampEventV1,
    output: *mut u32,
) -> RakSampResult {
    if output.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u32() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakSampResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_f32(
    event: *mut RakSampEventV1,
    output: *mut f32,
) -> RakSampResult {
    if output.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_f32() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakSampResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_bytes(
    event: *mut RakSampEventV1,
    output: *mut u8,
    len: usize,
) -> RakSampResult {
    if output.is_null() && len != 0 {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bytes(len) {
        Ok(bytes) => {
            if len != 0 {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, len) };
            }
            RakSampResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_write_u8(event: *mut RakSampEventV1, value: u8) -> RakSampResult {
    write_event(event, |stream| stream.write_u8(value))
}

unsafe extern "system" fn event_write_u16(event: *mut RakSampEventV1, value: u16) -> RakSampResult {
    write_event(event, |stream| stream.write_u16(value))
}

unsafe extern "system" fn event_write_u32(event: *mut RakSampEventV1, value: u32) -> RakSampResult {
    write_event(event, |stream| stream.write_u32(value))
}

unsafe extern "system" fn event_write_f32(event: *mut RakSampEventV1, value: f32) -> RakSampResult {
    write_event(event, |stream| stream.write_f32(value))
}

unsafe extern "system" fn event_write_bytes(
    event: *mut RakSampEventV1,
    value: *const u8,
    len: usize,
) -> RakSampResult {
    if value.is_null() && len != 0 {
        return RakSampResult::InvalidArgument;
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
    options: RakSampSendOptions,
) -> RakSampResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Packet)
}

unsafe extern "system" fn send_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: RakSampSendOptions,
) -> RakSampResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Rpc)
}

unsafe extern "system" fn emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> RakSampResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Packet)
}

unsafe extern "system" fn emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> RakSampResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Rpc)
}

unsafe extern "system" fn event_replace_bytes(
    event: *mut RakSampEventV1,
    value: *const u8,
    len: usize,
) -> RakSampResult {
    if value.is_null() && len != 0 {
        return RakSampResult::InvalidArgument;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, len) }
    };
    write_event(event, |stream| stream.replace_bytes(bytes))
}

unsafe extern "system" fn event_remaining_bits(event: *mut RakSampEventV1) -> usize {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return 0;
    };
    unsafe { &*event.payload }.remaining_bits()
}

unsafe extern "system" fn event_read_bits(
    event: *mut RakSampEventV1,
    output: *mut u8,
    bit_len: usize,
) -> RakSampResult {
    if output.is_null() && bit_len != 0 {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bits(bit_len) {
        Ok(bytes) => {
            if !bytes.is_empty() {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len()) };
            }
            RakSampResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_replace_bits(
    event: *mut RakSampEventV1,
    value: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> RakSampResult {
    if value.is_null() && byte_len != 0 {
        return RakSampResult::InvalidArgument;
    }
    if bit_len > byte_len.saturating_mul(u8::BITS as usize) {
        return RakSampResult::InvalidArgument;
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
) -> RakSampResult {
    if (value.is_null() && value_len != 0)
        || (output.is_null() && output_capacity != 0)
        || output_bit_len.is_null()
    {
        return RakSampResult::InvalidArgument;
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len) }
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let encoded = match runtime.encode_string(value) {
        Ok(encoded) => encoded,
        Err(error) => return codec_result(error),
    };
    if encoded.len_bytes() > output_capacity {
        return RakSampResult::PayloadTooLarge;
    }
    if encoded.len_bytes() != 0 {
        unsafe {
            ptr::copy_nonoverlapping(encoded.as_bytes().as_ptr(), output, encoded.len_bytes())
        };
    }
    unsafe { output_bit_len.write(encoded.len_bits()) };
    RakSampResult::Ok
}

unsafe extern "system" fn event_read_encoded_string(
    event: *mut RakSampEventV1,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
) -> RakSampResult {
    if output.is_null() || output_capacity == 0 || output_len.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output, output_capacity) };
    match runtime.decode_string(unsafe { &mut *event.payload }, output) {
        Ok(length) => {
            unsafe { output_len.write(length) };
            RakSampResult::Ok
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
) -> RakSampResult {
    if (input.is_null() && input_byte_len != 0)
        || output.is_null()
        || output_capacity == 0
        || output_len.is_null()
        || output_read_offset.is_null()
        || input_bit_len > input_byte_len.saturating_mul(u8::BITS as usize)
        || input_read_offset > input_bit_len
    {
        return RakSampResult::InvalidArgument;
    }
    if input_bit_len > MAX_CODEC_INPUT_BITS || output_capacity > MAX_CODEC_OUTPUT_BYTES {
        return RakSampResult::PayloadTooLarge;
    }
    let input_len = input_bit_len.div_ceil(u8::BITS as usize);
    let input = if input_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input, input_len) }
    };
    let Ok(mut payload) = BitStream::from_bytes_with_bits(input.to_vec(), input_bit_len) else {
        return RakSampResult::InvalidArgument;
    };
    if payload.set_read_offset_bits(input_read_offset).is_err() {
        return RakSampResult::InvalidArgument;
    }
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let output = unsafe { std::slice::from_raw_parts_mut(output, output_capacity) };
    match runtime.decode_string(&mut payload, output) {
        Ok(length) => {
            let read_offset = payload.read_offset_bits();
            if length >= output_capacity || read_offset > input_bit_len {
                return RakSampResult::NativeCallFailed;
            }
            unsafe {
                output_len.write(length);
                output_read_offset.write(read_offset);
            }
            RakSampResult::Ok
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
) -> RakSampResult {
    let Some(style) = LocalDialogStyle::from_raw(style) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(title) = (unsafe { copied_nul_free_string(title, title_len, 255) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 4_095) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(button1) = (unsafe { copied_nul_free_string(button1, button1_len, 255) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(button2) = (unsafe { copied_nul_free_string(button2, button2_len, 255) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
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
        .map_or_else(direct_client_result, |_| RakSampResult::Ok)
}

unsafe extern "system" fn show_local_chat_message(
    style: u32,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    text_colour: u32,
    prefix_colour: u32,
) -> RakSampResult {
    let Some(style) = LocalChatMessageStyle::from_raw(style) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(text) = (unsafe { copied_nul_free_string(text, text_len, 143) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(prefix) = (unsafe { copied_nul_free_string(prefix, prefix_len, 27) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    runtime
        .show_local_chat_message(LocalChatMessageRequest {
            style,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
        .map_or_else(direct_client_result, |_| RakSampResult::Ok)
}

unsafe extern "system" fn show_local_death_message(
    killer: *const u8,
    killer_len: usize,
    victim: *const u8,
    victim_len: usize,
    killer_colour: u32,
    victim_colour: u32,
    weapon: u8,
) -> RakSampResult {
    let Ok(killer) = (unsafe { copied_nul_free_string(killer, killer_len, 24) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(victim) = (unsafe { copied_nul_free_string(victim, victim_len, 24) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    runtime
        .show_local_death_message(LocalDeathMessageRequest {
            killer,
            victim,
            killer_colour,
            victim_colour,
            weapon,
        })
        .map_or_else(direct_client_result, |_| RakSampResult::Ok)
}

unsafe extern "system" fn local_player(output: *mut RakSampLocalPlayerV1) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let snapshot = match runtime.local_player() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = local_player_to_abi(snapshot) else {
        return RakSampResult::NativeCallFailed;
    };
    *output = snapshot;
    RakSampResult::Ok
}

unsafe extern "system" fn samp_game_state(output: *mut i32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.samp_game_state() {
        Ok(game_state) => {
            *output = game_state;
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn server_info(output: *mut RakSampServerInfoV1) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let snapshot = match runtime.server_info() {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = server_info_to_abi(snapshot) else {
        return RakSampResult::NativeCallFailed;
    };
    *output = snapshot;
    RakSampResult::Ok
}

unsafe extern "system" fn local_chat_display_mode(output: *mut i32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.local_chat_display_mode() {
        Ok(mode) => {
            *output = mode;
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_cursor_mode(output: *mut i32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.local_cursor_mode() {
        Ok(mode) => {
            *output = mode;
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_scoreboard_open(output: *mut u8) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.local_scoreboard_open() {
        Ok(open) => {
            *output = u8::from(open);
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_dialog_active(output: *mut u8) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.local_dialog_active() {
        Ok(active) => {
            *output = u8::from(active);
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_chat_input_active(output: *mut u8) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.local_chat_input_active() {
        Ok(active) => {
            *output = u8::from(active);
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn local_animation(
    id: u16,
    output: *mut RakSampAnimationV1,
) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let snapshot = match runtime.local_animation(id) {
        Ok(snapshot) => snapshot,
        Err(error) => return direct_client_result(error),
    };
    let Ok(snapshot) = animation_to_abi(snapshot) else {
        return RakSampResult::NativeCallFailed;
    };
    *output = snapshot;
    RakSampResult::Ok
}

unsafe extern "system" fn local_animation_id(
    name: *const u8,
    name_len: usize,
    file: *const u8,
    file_len: usize,
    output: *mut i32,
) -> RakSampResult {
    let Ok(name) = (unsafe { copied_nul_free_string(name, name_len, 35) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(file) = (unsafe { copied_nul_free_string(file, file_len, 35) }) else {
        return RakSampResult::InvalidArgument;
    };
    if name.is_empty() || file.is_empty() {
        return RakSampResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.local_animation_id(&name, &file) {
        Ok(Some(id)) => {
            *output = i32::from(id);
            RakSampResult::Ok
        }
        Ok(None) => {
            *output = -1;
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_info(id: u16, output: *mut RakSampPlayerInfoV1) -> RakSampResult {
    if id >= MAX_SAMP_PLAYERS {
        return RakSampResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.player_info(id) {
        Ok(Some(snapshot)) => match player_info_to_abi(snapshot) {
            Ok(snapshot) => {
                *output = snapshot;
                RakSampResult::Ok
            }
            Err(()) => RakSampResult::NativeCallFailed,
        },
        Ok(None) => {
            *output = RakSampPlayerInfoV1::default();
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn player_count(include_npcs: u8, output: *mut u16) -> RakSampResult {
    let include_npcs = match include_npcs {
        0 => false,
        1 => true,
        _ => return RakSampResult::InvalidArgument,
    };
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    match runtime.player_count(include_npcs) {
        Ok(count) => {
            *output = count;
            RakSampResult::Ok
        }
        Err(error) => direct_client_result(error),
    }
}

unsafe extern "system" fn samp_version(output: *mut u32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    *output = samp_version_to_abi(runtime.samp_version());
    RakSampResult::Ok
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
    direction: RakSampDirection,
    callback: Option<RakSampEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakSampSubscription,
    kind: ListenerKind,
) -> RakSampResult {
    let Some(callback) = callback else {
        return RakSampResult::InvalidArgument;
    };
    if subscription.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let direction = match direction {
        RakSampDirection::Incoming => Direction::Incoming,
        RakSampDirection::Outgoing => Direction::Outgoing,
    };
    let user_data = user_data as usize;
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
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
    unsafe { subscription.write(RakSampSubscription { id }) };
    debug!("registered {kind:?} subscription {id}");
    RakSampResult::Ok
}

fn call_plugin_callback(
    callback: RakSampEventCallbackV1,
    user_data: usize,
    id: u8,
    payload: &mut BitStream,
) -> HookAction {
    let mut event = AbiEvent { id, payload };
    let action = unsafe {
        callback(
            user_data as *mut c_void,
            (&mut event as *mut AbiEvent).cast::<RakSampEventV1>(),
        )
    };
    match action {
        RakSampHookAction::Block => HookAction::Block,
        RakSampHookAction::Continue => HookAction::Continue,
    }
}

fn write_event(
    event: *mut RakSampEventV1,
    operation: impl FnOnce(&mut BitStream) -> Result<(), BitStreamError>,
) -> RakSampResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakSampResult::InvalidArgument;
    };
    operation(unsafe { &mut *event.payload }).map_or_else(bitstream_result, |_| RakSampResult::Ok)
}

fn send(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: RakSampSendOptions,
    kind: ListenerKind,
) -> RakSampResult {
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(options) = send_options(options) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.send_packet_with_options(id, &payload, options),
        ListenerKind::Rpc => runtime.send_rpc_with_options(id, &payload, options),
    };
    result.map_or_else(send_result, |sent| {
        if sent {
            RakSampResult::Ok
        } else {
            RakSampResult::NativeCallFailed
        }
    })
}

fn emulate_incoming(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    kind: ListenerKind,
) -> RakSampResult {
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Some(runtime) = clone_initialized(&host().runtime) else {
        return RakSampResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.emulate_incoming_packet(id, payload),
        ListenerKind::Rpc => runtime.emulate_incoming_rpc(id, payload),
    };
    result.map_or_else(send_result, |_| RakSampResult::Ok)
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

unsafe fn abi_event(event: *mut RakSampEventV1) -> Result<&'static mut AbiEvent, ()> {
    let event = unsafe { event.cast::<AbiEvent>().as_mut() }.ok_or(())?;
    if event.payload.is_null() {
        return Err(());
    }
    Ok(event)
}

fn bitstream_result(error: BitStreamError) -> RakSampResult {
    match error {
        BitStreamError::ReadOutOfBounds { .. } => RakSampResult::ReadOutOfBounds,
        BitStreamError::CapacityExceeded { .. } => RakSampResult::PayloadTooLarge,
        BitStreamError::InvalidOffset { .. } => RakSampResult::InvalidArgument,
    }
}

fn send_result(error: SendError) -> RakSampResult {
    match error {
        SendError::ClientNotReady => RakSampResult::NotReady,
        SendError::PayloadTooLarge => RakSampResult::PayloadTooLarge,
        SendError::NativeCallFailed => RakSampResult::NativeCallFailed,
        SendError::TimestampedPacketUnsupported => RakSampResult::InvalidArgument,
    }
}

fn codec_result(error: CodecError) -> RakSampResult {
    match error {
        CodecError::ClientNotReady => RakSampResult::NotReady,
        CodecError::InvalidArgument => RakSampResult::InvalidArgument,
        CodecError::PayloadTooLarge => RakSampResult::PayloadTooLarge,
        CodecError::NativeCallFailed => RakSampResult::NativeCallFailed,
    }
}

fn direct_client_result(error: DirectClientError) -> RakSampResult {
    match error {
        DirectClientError::NotReady => RakSampResult::NotReady,
        DirectClientError::UnsupportedVersion => RakSampResult::UnsupportedVersion,
        DirectClientError::QueueFull => RakSampResult::QueueFull,
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

fn local_player_to_abi(snapshot: LocalPlayerSnapshot) -> Result<RakSampLocalPlayerV1, ()> {
    let nickname_len = u16::try_from(snapshot.nickname.len()).map_err(|_| ())?;
    if snapshot.nickname.len() > 256 {
        return Err(());
    }
    let mut nickname = [0; 256];
    nickname[..snapshot.nickname.len()].copy_from_slice(&snapshot.nickname);
    Ok(RakSampLocalPlayerV1 {
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

fn player_info_to_abi(snapshot: PlayerInfoSnapshot) -> Result<RakSampPlayerInfoV1, ()> {
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
    Ok(RakSampPlayerInfoV1 {
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

fn server_info_to_abi(snapshot: ServerInfoSnapshot) -> Result<RakSampServerInfoV1, ()> {
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
    Ok(RakSampServerInfoV1 {
        address_len,
        hostname_len,
        address,
        hostname,
        port: snapshot.port,
    })
}

fn animation_to_abi(snapshot: AnimationSnapshot) -> Result<RakSampAnimationV1, ()> {
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
    Ok(RakSampAnimationV1 {
        name_len,
        file_len,
        name,
        file,
    })
}

fn send_options(options: RakSampSendOptions) -> Result<SendOptions, ()> {
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
    fn direct_client_abi_is_not_ready_without_a_runtime() {
        let mut output = RakSampLocalPlayerV1::default();
        assert_eq!(
            unsafe { local_player(&mut output) },
            RakSampResult::NotReady
        );
        let mut game_state = 0;
        assert_eq!(
            unsafe { samp_game_state(&mut game_state) },
            RakSampResult::NotReady
        );
        let mut chat_display_mode = 0;
        assert_eq!(
            unsafe { local_chat_display_mode(&mut chat_display_mode) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { local_chat_display_mode(std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut cursor_mode = 0;
        assert_eq!(
            unsafe { local_cursor_mode(&mut cursor_mode) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { local_cursor_mode(std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut scoreboard_open = 0;
        assert_eq!(
            unsafe { local_scoreboard_open(&mut scoreboard_open) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { local_scoreboard_open(std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut dialog_active = 0;
        assert_eq!(
            unsafe { local_dialog_active(&mut dialog_active) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { local_dialog_active(std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut chat_input_active = 0;
        assert_eq!(
            unsafe { local_chat_input_active(&mut chat_input_active) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { local_chat_input_active(std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut animation = RakSampAnimationV1::default();
        assert_eq!(
            unsafe { local_animation(0, &mut animation) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { local_animation(0, std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
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
            RakSampResult::NotReady
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
            RakSampResult::InvalidArgument
        );
        let mut player = RakSampPlayerInfoV1::default();
        assert_eq!(
            unsafe { player_info(7, &mut player) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { player_info(MAX_SAMP_PLAYERS, &mut player) },
            RakSampResult::InvalidArgument
        );
        assert_eq!(
            unsafe { player_info(7, std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut count = 0;
        assert_eq!(
            unsafe { player_count(1, &mut count) },
            RakSampResult::NotReady
        );
        assert_eq!(
            unsafe { player_count(2, &mut count) },
            RakSampResult::InvalidArgument
        );
        assert_eq!(
            unsafe { player_count(1, std::ptr::null_mut()) },
            RakSampResult::InvalidArgument
        );
        let mut server = RakSampServerInfoV1::default();
        assert_eq!(unsafe { server_info(&mut server) }, RakSampResult::NotReady);
        let mut version = 0;
        assert_eq!(
            unsafe { samp_version(&mut version) },
            RakSampResult::NotReady
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
            RakSampResult::NotReady
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
            RakSampResult::InvalidArgument
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
            RakSampResult::PayloadTooLarge
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
}
