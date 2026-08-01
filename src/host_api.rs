use crate::{
    AttachError, BitStream, BitStreamError, Direction, HookAction, ListenerHandle, PacketPriority,
    PacketReliability, Runtime, SendError, SendOptions, logging,
};
use log::{debug, error, info};
use rak_rs_plugin_api::{
    ABI_VERSION_V1, RakRsApiV1, RakRsDirection, RakRsEventCallbackV1, RakRsEventV1,
    RakRsHookAction, RakRsHostStatus, RakRsResult, RakRsSendOptions, RakRsSubscription,
};
use std::{
    collections::HashMap,
    ffi::c_void,
    ptr,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::Duration,
};

const STATUS_WAITING: u32 = RakRsHostStatus::WaitingForSamp as u32;
const STATUS_READY: u32 = RakRsHostStatus::Ready as u32;
const STATUS_FAILED: u32 = RakRsHostStatus::Failed as u32;

struct HostState {
    status: AtomicU32,
    bootstrap_started: AtomicBool,
    runtime: Mutex<Option<Runtime>>,
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
                    *host()
                        .runtime
                        .lock()
                        .unwrap_or_else(|error| error.into_inner()) = Some(runtime);
                    host().status.store(STATUS_READY, Ordering::Release);
                    info!("host runtime is ready");
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

#[unsafe(no_mangle)]
pub extern "system" fn RakRs_GetApiV1(requested_version: u32) -> *const RakRsApiV1 {
    if requested_version == ABI_VERSION_V1 {
        &RAK_RS_API_V1
    } else {
        debug!("rejected unsupported plugin ABI version {requested_version}");
        ptr::null()
    }
}

static RAK_RS_API_V1: RakRsApiV1 = RakRsApiV1 {
    abi_version: ABI_VERSION_V1,
    size: std::mem::size_of::<RakRsApiV1>() as u32,
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
};

extern "system" fn host_status() -> RakRsHostStatus {
    match host().status.load(Ordering::Acquire) {
        STATUS_READY => RakRsHostStatus::Ready,
        STATUS_FAILED => RakRsHostStatus::Failed,
        _ => RakRsHostStatus::WaitingForSamp,
    }
}

unsafe extern "system" fn register_packet(
    direction: RakRsDirection,
    callback: Option<RakRsEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakRsSubscription,
) -> RakRsResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Packet,
    )
}

unsafe extern "system" fn register_rpc(
    direction: RakRsDirection,
    callback: Option<RakRsEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakRsSubscription,
) -> RakRsResult {
    register_listener(
        direction,
        callback,
        user_data,
        subscription,
        ListenerKind::Rpc,
    )
}

unsafe extern "system" fn unregister(subscription: RakRsSubscription) -> RakRsResult {
    if subscription.id == 0 {
        return RakRsResult::InvalidArgument;
    }
    let removed = host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&subscription.id)
        .is_some();
    if removed {
        debug!("unregistered plugin subscription {}", subscription.id);
        RakRsResult::Ok
    } else {
        RakRsResult::SubscriptionNotFound
    }
}

unsafe extern "system" fn unregister_and_wait(subscription: RakRsSubscription) -> RakRsResult {
    if subscription.id == 0 {
        return RakRsResult::InvalidArgument;
    }
    let listener = {
        let mut subscriptions = host()
            .subscriptions
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(listener) = subscriptions.get(&subscription.id) else {
            return RakRsResult::SubscriptionNotFound;
        };
        if !listener.can_remove_and_wait() {
            return RakRsResult::CallbackInProgress;
        }
        let Some(listener) = subscriptions.remove(&subscription.id) else {
            return RakRsResult::SubscriptionNotFound;
        };
        listener
    };
    listener.remove_and_wait();
    debug!(
        "unregistered plugin subscription {} and synchronized callbacks",
        subscription.id
    );
    RakRsResult::Ok
}

unsafe extern "system" fn event_id(event: *const RakRsEventV1) -> u8 {
    if event.is_null() {
        return 0;
    }
    unsafe { event.cast::<AbiEvent>().as_ref() }.map_or(0, |event| event.id)
}

unsafe extern "system" fn event_reset_read(event: *mut RakRsEventV1) -> RakRsResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.reset_read();
    RakRsResult::Ok
}

unsafe extern "system" fn event_clear(event: *mut RakRsEventV1) -> RakRsResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    unsafe { &mut *event.payload }.clear();
    RakRsResult::Ok
}

unsafe extern "system" fn event_read_u8(event: *mut RakRsEventV1, output: *mut u8) -> RakRsResult {
    if output.is_null() {
        return RakRsResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u8() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakRsResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_u16(
    event: *mut RakRsEventV1,
    output: *mut u16,
) -> RakRsResult {
    if output.is_null() {
        return RakRsResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u16() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakRsResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_u32(
    event: *mut RakRsEventV1,
    output: *mut u32,
) -> RakRsResult {
    if output.is_null() {
        return RakRsResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_u32() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakRsResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_f32(
    event: *mut RakRsEventV1,
    output: *mut f32,
) -> RakRsResult {
    if output.is_null() {
        return RakRsResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_f32() {
        Ok(value) => {
            unsafe { output.write(value) };
            RakRsResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_read_bytes(
    event: *mut RakRsEventV1,
    output: *mut u8,
    len: usize,
) -> RakRsResult {
    if output.is_null() && len != 0 {
        return RakRsResult::InvalidArgument;
    }
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    match unsafe { &mut *event.payload }.read_bytes(len) {
        Ok(bytes) => {
            if len != 0 {
                unsafe { ptr::copy_nonoverlapping(bytes.as_ptr(), output, len) };
            }
            RakRsResult::Ok
        }
        Err(error) => bitstream_result(error),
    }
}

unsafe extern "system" fn event_write_u8(event: *mut RakRsEventV1, value: u8) -> RakRsResult {
    write_event(event, |stream| stream.write_u8(value))
}

unsafe extern "system" fn event_write_u16(event: *mut RakRsEventV1, value: u16) -> RakRsResult {
    write_event(event, |stream| stream.write_u16(value))
}

unsafe extern "system" fn event_write_u32(event: *mut RakRsEventV1, value: u32) -> RakRsResult {
    write_event(event, |stream| stream.write_u32(value))
}

unsafe extern "system" fn event_write_f32(event: *mut RakRsEventV1, value: f32) -> RakRsResult {
    write_event(event, |stream| stream.write_f32(value))
}

unsafe extern "system" fn event_write_bytes(
    event: *mut RakRsEventV1,
    value: *const u8,
    len: usize,
) -> RakRsResult {
    if value.is_null() && len != 0 {
        return RakRsResult::InvalidArgument;
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
    options: RakRsSendOptions,
) -> RakRsResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Packet)
}

unsafe extern "system" fn send_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: RakRsSendOptions,
) -> RakRsResult {
    send(id, data, byte_len, bit_len, options, ListenerKind::Rpc)
}

unsafe extern "system" fn emulate_incoming_packet(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> RakRsResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Packet)
}

unsafe extern "system" fn emulate_incoming_rpc(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> RakRsResult {
    emulate_incoming(id, data, byte_len, bit_len, ListenerKind::Rpc)
}

unsafe extern "system" fn event_replace_bytes(
    event: *mut RakRsEventV1,
    value: *const u8,
    len: usize,
) -> RakRsResult {
    if value.is_null() && len != 0 {
        return RakRsResult::InvalidArgument;
    }
    let bytes = if len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, len) }
    };
    write_event(event, |stream| stream.replace_bytes(bytes))
}

fn register_listener(
    direction: RakRsDirection,
    callback: Option<RakRsEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakRsSubscription,
    kind: ListenerKind,
) -> RakRsResult {
    let Some(callback) = callback else {
        return RakRsResult::InvalidArgument;
    };
    if subscription.is_null() {
        return RakRsResult::InvalidArgument;
    }
    let direction = match direction {
        RakRsDirection::Incoming => Direction::Incoming,
        RakRsDirection::Outgoing => Direction::Outgoing,
    };
    let user_data = user_data as usize;
    let listener = {
        let runtime = host()
            .runtime
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let Some(runtime) = runtime.as_ref() else {
            return RakRsResult::NotReady;
        };
        match kind {
            ListenerKind::Packet => runtime.on_packet(direction, move |event| {
                call_plugin_callback(callback, user_data, event.id(), event.payload_mut())
            }),
            ListenerKind::Rpc => runtime.on_rpc(direction, move |event| {
                call_plugin_callback(callback, user_data, event.id(), event.payload_mut())
            }),
        }
    };

    let id = host().next_subscription.fetch_add(1, Ordering::AcqRel);
    host()
        .subscriptions
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .insert(id, listener);
    unsafe { subscription.write(RakRsSubscription { id }) };
    debug!("registered {kind:?} subscription {id}");
    RakRsResult::Ok
}

fn call_plugin_callback(
    callback: RakRsEventCallbackV1,
    user_data: usize,
    id: u8,
    payload: &mut BitStream,
) -> HookAction {
    let mut event = AbiEvent { id, payload };
    let action = unsafe {
        callback(
            user_data as *mut c_void,
            (&mut event as *mut AbiEvent).cast::<RakRsEventV1>(),
        )
    };
    match action {
        RakRsHookAction::Block => HookAction::Block,
        RakRsHookAction::Continue => HookAction::Continue,
    }
}

fn write_event(
    event: *mut RakRsEventV1,
    operation: impl FnOnce(&mut BitStream) -> Result<(), BitStreamError>,
) -> RakRsResult {
    let Ok(event) = (unsafe { abi_event(event) }) else {
        return RakRsResult::InvalidArgument;
    };
    operation(unsafe { &mut *event.payload }).map_or_else(bitstream_result, |_| RakRsResult::Ok)
}

fn send(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: RakRsSendOptions,
    kind: ListenerKind,
) -> RakRsResult {
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return RakRsResult::InvalidArgument;
    };
    let Ok(options) = send_options(options) else {
        return RakRsResult::InvalidArgument;
    };
    let runtime = host()
        .runtime
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let Some(runtime) = runtime.as_ref() else {
        return RakRsResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.send_packet_with_options(id, &payload, options),
        ListenerKind::Rpc => runtime.send_rpc_with_options(id, &payload, options),
    };
    result.map_or_else(send_result, |sent| {
        if sent {
            RakRsResult::Ok
        } else {
            RakRsResult::NativeCallFailed
        }
    })
}

fn emulate_incoming(
    id: u8,
    data: *const u8,
    byte_len: usize,
    bit_len: usize,
    kind: ListenerKind,
) -> RakRsResult {
    let Ok(payload) = (unsafe { stream_from_abi(data, byte_len, bit_len) }) else {
        return RakRsResult::InvalidArgument;
    };
    let runtime = host()
        .runtime
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let Some(runtime) = runtime.as_ref() else {
        return RakRsResult::NotReady;
    };
    let result = match kind {
        ListenerKind::Packet => runtime.emulate_incoming_packet(id, payload),
        ListenerKind::Rpc => runtime.emulate_incoming_rpc(id, payload),
    };
    result.map_or_else(send_result, |_| RakRsResult::Ok)
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

unsafe fn abi_event(event: *mut RakRsEventV1) -> Result<&'static mut AbiEvent, ()> {
    let event = unsafe { event.cast::<AbiEvent>().as_mut() }.ok_or(())?;
    if event.payload.is_null() {
        return Err(());
    }
    Ok(event)
}

fn bitstream_result(error: BitStreamError) -> RakRsResult {
    match error {
        BitStreamError::ReadOutOfBounds { .. } => RakRsResult::ReadOutOfBounds,
        BitStreamError::CapacityExceeded { .. } => RakRsResult::PayloadTooLarge,
        BitStreamError::InvalidOffset { .. } => RakRsResult::InvalidArgument,
    }
}

fn send_result(error: SendError) -> RakRsResult {
    match error {
        SendError::ClientNotReady => RakRsResult::NotReady,
        SendError::PayloadTooLarge => RakRsResult::PayloadTooLarge,
        SendError::NativeCallFailed => RakRsResult::NativeCallFailed,
    }
}

fn send_options(options: RakRsSendOptions) -> Result<SendOptions, ()> {
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
        runtime: Mutex::new(None),
        subscriptions: Mutex::new(HashMap::new()),
        next_subscription: AtomicU64::new(1),
    })
}

#[derive(Clone, Copy, Debug)]
enum ListenerKind {
    Packet,
    Rpc,
}
