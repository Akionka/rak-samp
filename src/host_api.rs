use crate::{
    AttachError, BitStream, BitStreamError, Direction, HookAction, ListenerHandle, PacketPriority,
    PacketReliability, Runtime, SendError, SendOptions, logging,
    runtime::{
        ClientHookStatus, CodecError, DirectClientError, LocalDialogRequest, LocalDialogStyle,
        LocalPlayerSnapshot,
    },
};
use log::{debug, error, info};
use rak_samp_plugin_api::{
    ABI_VERSION_V1, RakSampApiV1, RakSampDirection, RakSampEventCallbackV1, RakSampEventV1,
    RakSampHookAction, RakSampHostStatus, RakSampLocalPlayerV1, RakSampResult, RakSampSendOptions,
    RakSampSubscription, Vector3,
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
    let Ok(title) = (unsafe { copied_dialog_string(title, title_len, 255) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(text) = (unsafe { copied_dialog_string(text, text_len, 4_095) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(button1) = (unsafe { copied_dialog_string(button1, button1_len, 255) }) else {
        return RakSampResult::InvalidArgument;
    };
    let Ok(button2) = (unsafe { copied_dialog_string(button2, button2_len, 255) }) else {
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

unsafe fn copied_dialog_string(
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
}
