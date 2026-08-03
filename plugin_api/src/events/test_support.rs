use super::{EncodedPayload, Event, Rpc, RpcAction, core::PayloadWriter};
use crate::{HostApi, RakSampEventV1, RakSampHookAction, RakSampResult};
use ::core::{mem, ptr};
use std::sync::Mutex;

#[derive(Clone, Copy)]
struct RegisteredCallback {
    callback: crate::RakSampEventCallbackV1,
    user_data: usize,
    subscription: crate::RakSampSubscription,
}

struct RegistrationState {
    register_result: RakSampResult,
    unregister_result: RakSampResult,
    unregister_and_wait_result: RakSampResult,
    next_id: u64,
    callbacks: Vec<RegisteredCallback>,
    unregister_calls: u32,
    unregister_and_wait_calls: u32,
}

impl RegistrationState {
    const fn new() -> Self {
        Self {
            register_result: RakSampResult::Ok,
            unregister_result: RakSampResult::Ok,
            unregister_and_wait_result: RakSampResult::Ok,
            next_id: 1,
            callbacks: Vec::new(),
            unregister_calls: 0,
            unregister_and_wait_calls: 0,
        }
    }
}

static REGISTRATION: Mutex<RegistrationState> = Mutex::new(RegistrationState::new());

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegistrationStats {
    pub(crate) unregister_calls: u32,
    pub(crate) unregister_and_wait_calls: u32,
    pub(crate) registered_callbacks: usize,
}

pub(crate) fn reset_registration() {
    *REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = RegistrationState::new();
}

pub(crate) fn set_register_result(result: RakSampResult) {
    REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .register_result = result;
}

pub(crate) fn set_unregister_and_wait_result(result: RakSampResult) {
    REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .unregister_and_wait_result = result;
}

pub(crate) fn registration_stats() -> RegistrationStats {
    let state = REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    RegistrationStats {
        unregister_calls: state.unregister_calls,
        unregister_and_wait_calls: state.unregister_and_wait_calls,
        registered_callbacks: state.callbacks.len(),
    }
}

pub(crate) fn invoke_registered_callback(id: u8) -> Option<RakSampHookAction> {
    let payload = EncodedPayload::from_bits(Vec::new(), 0).expect("an empty payload is valid");
    invoke_registered_callback_with_payload(id, payload)
}

pub(crate) fn invoke_registered_callback_with_payload(
    id: u8,
    payload: EncodedPayload,
) -> Option<RakSampHookAction> {
    let callback = *REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .callbacks
        .first()?;
    let mut event = TestEvent::new(id, payload);
    Some(unsafe {
        (callback.callback)(
            callback.user_data as *mut ::core::ffi::c_void,
            (&mut event as *mut TestEvent).cast::<RakSampEventV1>(),
        )
    })
}

#[repr(C)]
pub(crate) struct TestEvent {
    id: u8,
    pub(crate) bytes: Vec<u8>,
    pub(crate) bit_len: usize,
    read_offset: usize,
}

impl TestEvent {
    pub(crate) fn new(id: u8, payload: EncodedPayload) -> Self {
        Self {
            id,
            bytes: payload.bytes,
            bit_len: payload.bit_len,
            read_offset: 0,
        }
    }
}

unsafe fn test_event<'a>(event: *mut RakSampEventV1) -> &'a mut TestEvent {
    unsafe { &mut *event.cast::<TestEvent>() }
}

unsafe extern "system" fn test_event_id(event: *const RakSampEventV1) -> u8 {
    unsafe { (&*event.cast::<TestEvent>()).id }
}

unsafe extern "system" fn test_event_reset_read(event: *mut RakSampEventV1) -> RakSampResult {
    unsafe { test_event(event) }.read_offset = 0;
    RakSampResult::Ok
}

unsafe extern "system" fn test_event_clear(event: *mut RakSampEventV1) -> RakSampResult {
    let event = unsafe { test_event(event) };
    event.bytes.clear();
    event.bit_len = 0;
    event.read_offset = 0;
    RakSampResult::Ok
}

unsafe extern "system" fn test_event_read_bits(
    event: *mut RakSampEventV1,
    output: *mut u8,
    bit_len: usize,
) -> RakSampResult {
    let event = unsafe { test_event(event) };
    if event.read_offset.saturating_add(bit_len) > event.bit_len {
        return RakSampResult::ReadOutOfBounds;
    }
    let byte_len = bit_len.div_ceil(u8::BITS as usize);
    if byte_len != 0 {
        unsafe { ptr::write_bytes(output, 0, byte_len) };
    }
    for bit in 0..bit_len {
        let source =
            event.bytes[(event.read_offset + bit) / 8] & (0x80 >> ((event.read_offset + bit) % 8));
        if source != 0 {
            unsafe { *output.add(bit / 8) |= 0x80 >> (bit % 8) };
        }
    }
    event.read_offset += bit_len;
    RakSampResult::Ok
}

unsafe extern "system" fn test_event_read_u8(
    event: *mut RakSampEventV1,
    output: *mut u8,
) -> RakSampResult {
    unsafe { test_event_read_bits(event, output, 8) }
}

unsafe extern "system" fn test_event_read_u16(
    event: *mut RakSampEventV1,
    output: *mut u16,
) -> RakSampResult {
    let mut bytes = [0; 2];
    let result = unsafe { test_event_read_bits(event, bytes.as_mut_ptr(), 16) };
    if result == RakSampResult::Ok {
        unsafe { output.write(u16::from_le_bytes(bytes)) };
    }
    result
}

unsafe extern "system" fn test_event_read_u32(
    event: *mut RakSampEventV1,
    output: *mut u32,
) -> RakSampResult {
    let mut bytes = [0; 4];
    let result = unsafe { test_event_read_bits(event, bytes.as_mut_ptr(), 32) };
    if result == RakSampResult::Ok {
        unsafe { output.write(u32::from_le_bytes(bytes)) };
    }
    result
}

unsafe extern "system" fn test_event_read_f32(
    event: *mut RakSampEventV1,
    output: *mut f32,
) -> RakSampResult {
    let mut bits = 0;
    let result = unsafe { test_event_read_u32(event, &raw mut bits) };
    if result == RakSampResult::Ok {
        unsafe { output.write(f32::from_bits(bits)) };
    }
    result
}

unsafe extern "system" fn test_event_read_bytes(
    event: *mut RakSampEventV1,
    output: *mut u8,
    byte_len: usize,
) -> RakSampResult {
    unsafe { test_event_read_bits(event, output, byte_len * 8) }
}

unsafe extern "system" fn test_event_write_u8(
    _event: *mut RakSampEventV1,
    _value: u8,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_event_write_u16(
    _event: *mut RakSampEventV1,
    _value: u16,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_event_write_u32(
    _event: *mut RakSampEventV1,
    _value: u32,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_event_write_f32(
    _event: *mut RakSampEventV1,
    _value: f32,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_event_write_bytes(
    _event: *mut RakSampEventV1,
    _value: *const u8,
    _byte_len: usize,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_event_replace_bytes(
    event: *mut RakSampEventV1,
    bytes: *const u8,
    byte_len: usize,
) -> RakSampResult {
    unsafe { test_event_replace_bits(event, bytes, byte_len, byte_len * 8) }
}

unsafe extern "system" fn test_event_replace_bits(
    event: *mut RakSampEventV1,
    bytes: *const u8,
    byte_len: usize,
    bit_len: usize,
) -> RakSampResult {
    if bit_len > byte_len.saturating_mul(8) {
        return RakSampResult::InvalidArgument;
    }
    let event = unsafe { test_event(event) };
    event.bytes = if byte_len == 0 {
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(bytes, byte_len) }.to_vec()
    };
    event.bit_len = bit_len;
    event.read_offset = 0;
    RakSampResult::Ok
}

unsafe extern "system" fn test_event_remaining_bits(event: *mut RakSampEventV1) -> usize {
    let event = unsafe { test_event(event) };
    event.bit_len - event.read_offset
}

unsafe extern "system" fn test_encoded_string(
    value: *const u8,
    value_len: usize,
    output: *mut u8,
    output_capacity: usize,
    bit_len: *mut usize,
) -> RakSampResult {
    if (value.is_null() && value_len != 0) || output.is_null() || bit_len.is_null() {
        return RakSampResult::InvalidArgument;
    }
    if value_len > u16::MAX as usize {
        return RakSampResult::PayloadTooLarge;
    }
    let value = if value_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value, value_len) }
    };
    let mut writer = PayloadWriter::new();
    writer.u16(value_len as u16);
    writer.bytes(value);
    let encoded = writer.finish_bits();
    if encoded.bytes.len() > output_capacity {
        return RakSampResult::PayloadTooLarge;
    }
    unsafe {
        ptr::copy_nonoverlapping(encoded.bytes.as_ptr(), output, encoded.bytes.len());
        bit_len.write(encoded.bit_len);
    }
    RakSampResult::Ok
}

unsafe extern "system" fn test_read_encoded_string(
    event: *mut RakSampEventV1,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
) -> RakSampResult {
    if output.is_null() || output_len.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let mut length = 0;
    let result = unsafe { test_event_read_u16(event, &raw mut length) };
    if result != RakSampResult::Ok {
        return result;
    }
    let length = usize::from(length);
    if length > output_capacity {
        return RakSampResult::PayloadTooLarge;
    }
    let result = unsafe { test_event_read_bytes(event, output, length) };
    if result == RakSampResult::Ok {
        unsafe { output_len.write(length) };
    }
    result
}

extern "system" fn test_status() -> crate::RakSampHostStatus {
    crate::RakSampHostStatus::Ready
}

unsafe extern "system" fn test_register(
    _direction: crate::RakSampDirection,
    callback: Option<crate::RakSampEventCallbackV1>,
    user_data: *mut ::core::ffi::c_void,
    subscription: *mut crate::RakSampSubscription,
) -> RakSampResult {
    let Some(callback) = callback else {
        return RakSampResult::InvalidArgument;
    };
    if subscription.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let mut state = REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if state.register_result != RakSampResult::Ok {
        return state.register_result;
    }
    let handle = crate::RakSampSubscription { id: state.next_id };
    state.next_id += 1;
    state.callbacks.push(RegisteredCallback {
        callback,
        user_data: user_data as usize,
        subscription: handle,
    });
    unsafe { subscription.write(handle) };
    RakSampResult::Ok
}

unsafe extern "system" fn test_unregister(
    subscription: crate::RakSampSubscription,
) -> RakSampResult {
    unregister(subscription, false)
}

unsafe extern "system" fn test_unregister_and_wait(
    subscription: crate::RakSampSubscription,
) -> RakSampResult {
    unregister(subscription, true)
}

fn unregister(subscription: crate::RakSampSubscription, wait: bool) -> RakSampResult {
    let mut state = REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let result = if wait {
        state.unregister_and_wait_calls += 1;
        state.unregister_and_wait_result
    } else {
        state.unregister_calls += 1;
        state.unregister_result
    };
    if matches!(
        result,
        RakSampResult::Ok | RakSampResult::SubscriptionNotFound
    ) {
        state
            .callbacks
            .retain(|callback| callback.subscription != subscription);
    }
    result
}

unsafe extern "system" fn test_send(
    _id: u8,
    _bytes: *const u8,
    _byte_len: usize,
    _bit_len: usize,
    _options: crate::RakSampSendOptions,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_emulate(
    _id: u8,
    _bytes: *const u8,
    _byte_len: usize,
    _bit_len: usize,
) -> RakSampResult {
    RakSampResult::NativeCallFailed
}

unsafe extern "system" fn test_show_local_dialog(
    _id: u16,
    _style: u32,
    _title: *const u8,
    _title_len: usize,
    _text: *const u8,
    _text_len: usize,
    _button1: *const u8,
    _button1_len: usize,
    _button2: *const u8,
    _button2_len: usize,
) -> RakSampResult {
    RakSampResult::Ok
}

unsafe extern "system" fn test_local_player(
    output: *mut crate::RakSampLocalPlayerV1,
) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    *output = crate::RakSampLocalPlayerV1 {
        id: 42,
        nickname_len: 7,
        nickname: {
            let mut value = [0; 256];
            value[..7].copy_from_slice(b"fixture");
            value
        },
        colour: 0xFF00_00FF,
        spawned: 1,
        special_action: 3,
        animation_id: 12,
        health: 99.0,
        armour: 50.0,
        position: crate::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        velocity: crate::Vector3 {
            x: 4.0,
            y: 5.0,
            z: 6.0,
        },
        has_vehicle: 1,
        _reserved: 0,
        vehicle_id: 19,
        score: 123,
        ping: 45,
    };
    RakSampResult::Ok
}

unsafe extern "system" fn test_samp_game_state(output: *mut i32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    *output = 14;
    RakSampResult::Ok
}

unsafe extern "system" fn test_samp_version(output: *mut u32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    *output = crate::RakSampClientVersion::R1 as u32;
    RakSampResult::Ok
}

unsafe extern "system" fn test_decode_string(
    input: *const u8,
    input_len: usize,
    input_bit_len: usize,
    input_read_offset: usize,
    output: *mut u8,
    output_capacity: usize,
    output_len: *mut usize,
    output_read_offset: *mut usize,
) -> RakSampResult {
    if input.is_null()
        || input_len != 1
        || input_bit_len != 3
        || input_read_offset != 0
        || output.is_null()
        || output_capacity < b"fixture".len() + 1
        || output_len.is_null()
        || output_read_offset.is_null()
    {
        return RakSampResult::InvalidArgument;
    }
    let input = unsafe { std::slice::from_raw_parts(input, input_len) };
    if input != [0b1010_0000] {
        return RakSampResult::InvalidArgument;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(b"fixture".as_ptr(), output, b"fixture".len());
        output_len.write(b"fixture".len());
        output_read_offset.write(input_bit_len);
    }
    RakSampResult::Ok
}

static TEST_API: crate::RakSampApiV1 = crate::RakSampApiV1 {
    abi_version: crate::ABI_VERSION_V1,
    size: mem::size_of::<crate::RakSampApiV1>() as u32,
    host_status: test_status,
    register_packet: test_register,
    register_rpc: test_register,
    unregister: test_unregister,
    event_id: test_event_id,
    event_reset_read: test_event_reset_read,
    event_clear: test_event_clear,
    event_read_u8: test_event_read_u8,
    event_read_u16: test_event_read_u16,
    event_read_u32: test_event_read_u32,
    event_read_f32: test_event_read_f32,
    event_read_bytes: test_event_read_bytes,
    event_write_u8: test_event_write_u8,
    event_write_u16: test_event_write_u16,
    event_write_u32: test_event_write_u32,
    event_write_f32: test_event_write_f32,
    event_write_bytes: test_event_write_bytes,
    send_packet: test_send,
    send_rpc: test_send,
    event_replace_bytes: test_event_replace_bytes,
    unregister_and_wait: test_unregister_and_wait,
    emulate_incoming_packet: test_emulate,
    emulate_incoming_rpc: test_emulate,
    event_remaining_bits: test_event_remaining_bits,
    event_read_bits: test_event_read_bits,
    event_replace_bits: test_event_replace_bits,
    encode_string: test_encoded_string,
    event_read_encoded_string: test_read_encoded_string,
    show_local_dialog: test_show_local_dialog,
    local_player: test_local_player,
    samp_game_state: test_samp_game_state,
    samp_version: test_samp_version,
    decode_string: test_decode_string,
};

pub(crate) fn test_api() -> HostApi {
    unsafe { HostApi::from_raw(&TEST_API) }.expect("test API is complete")
}

pub(crate) fn assert_replacement_round_trip<T>(descriptor: Rpc<T>, value: T)
where
    T: Clone + ::core::fmt::Debug + PartialEq,
{
    let api = test_api();
    let id = descriptor.id();
    let encoded = descriptor
        .encode(api, value.clone())
        .expect("test payload must encode");
    let mut raw = TestEvent::new(id, encoded.clone());
    let mut event =
        unsafe { Event::from_callback(api, (&mut raw as *mut TestEvent).cast::<RakSampEventV1>()) }
            .expect("test event is not null");
    assert_eq!(
        descriptor
            .handle(&mut event, |decoded| {
                assert_eq!(decoded, value);
                RpcAction::Replace(decoded)
            })
            .expect("typed replacement must succeed"),
        RakSampHookAction::Continue
    );
    assert_eq!(raw.bit_len, encoded.bit_len);
    assert_eq!(raw.bytes, encoded.bytes);
}
