//! Minimal in-process host used only by the ASI ABI end-to-end fixture.

#[cfg(not(all(windows, target_arch = "x86")))]
compile_error!("rak_samp_e2e_host supports only 32-bit Windows x86 targets");

use rak_samp_plugin_api::{
    ABI_VERSION_V1, RakSampApiV1, RakSampDirection, RakSampEventCallbackV1, RakSampEventV1,
    RakSampHostStatus, RakSampLocalPlayerV1, RakSampResult, RakSampSendOptions,
    RakSampSubscription, Vector3,
};
use std::{
    ffi::c_void,
    ptr,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

#[derive(Clone, Copy)]
struct Listener {
    direction: RakSampDirection,
    callback: RakSampEventCallbackV1,
    user_data: usize,
    id: u64,
}

#[repr(C)]
struct FixtureEvent {
    id: u8,
}

static LISTENERS: Mutex<Vec<Listener>> = Mutex::new(Vec::new());
static NEXT_SUBSCRIPTION: AtomicU64 = AtomicU64::new(1);

#[unsafe(no_mangle)]
pub extern "system" fn RakSamp_GetApiV1(requested_version: u32) -> *const RakSampApiV1 {
    if requested_version == ABI_VERSION_V1 {
        &API
    } else {
        ptr::null()
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2eHost_DispatchIncomingRpc(id: u8) -> i32 {
    let listeners = LISTENERS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .iter()
        .copied()
        .filter(|listener| listener.direction == RakSampDirection::Incoming)
        .collect::<Vec<_>>();
    let mut event = FixtureEvent { id };
    for listener in listeners {
        let _action = unsafe {
            (listener.callback)(
                listener.user_data as *mut c_void,
                (&mut event as *mut FixtureEvent).cast::<RakSampEventV1>(),
            )
        };
    }
    i32::from(
        !LISTENERS
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .is_empty(),
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn RakSampE2eHost_ListenerCount() -> u32 {
    LISTENERS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .len()
        .try_into()
        .unwrap_or(u32::MAX)
}

extern "system" fn host_status() -> RakSampHostStatus {
    RakSampHostStatus::Ready
}

unsafe extern "system" fn register_packet(
    _direction: RakSampDirection,
    _callback: Option<RakSampEventCallbackV1>,
    _user_data: *mut c_void,
    _subscription: *mut RakSampSubscription,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn register_rpc(
    direction: RakSampDirection,
    callback: Option<RakSampEventCallbackV1>,
    user_data: *mut c_void,
    subscription: *mut RakSampSubscription,
) -> RakSampResult {
    let Some(callback) = callback else {
        return RakSampResult::InvalidArgument;
    };
    if subscription.is_null() {
        return RakSampResult::InvalidArgument;
    }
    let id = NEXT_SUBSCRIPTION.fetch_add(1, Ordering::AcqRel);
    LISTENERS
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .push(Listener {
            direction,
            callback,
            user_data: user_data as usize,
            id,
        });
    unsafe { subscription.write(RakSampSubscription { id }) };
    RakSampResult::Ok
}

unsafe extern "system" fn unregister(subscription: RakSampSubscription) -> RakSampResult {
    let mut listeners = LISTENERS.lock().unwrap_or_else(|error| error.into_inner());
    let Some(index) = listeners
        .iter()
        .position(|listener| listener.id == subscription.id)
    else {
        return RakSampResult::SubscriptionNotFound;
    };
    listeners.swap_remove(index);
    RakSampResult::Ok
}

unsafe extern "system" fn event_id(event: *const RakSampEventV1) -> u8 {
    unsafe { event.cast::<FixtureEvent>().as_ref() }.map_or(0, |event| event.id)
}

unsafe extern "system" fn event_result(_event: *mut RakSampEventV1) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn read_u8(_event: *mut RakSampEventV1, _output: *mut u8) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn read_u16(
    _event: *mut RakSampEventV1,
    _output: *mut u16,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn read_u32(
    _event: *mut RakSampEventV1,
    _output: *mut u32,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn read_f32(
    _event: *mut RakSampEventV1,
    _output: *mut f32,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn read_bytes(
    _event: *mut RakSampEventV1,
    _output: *mut u8,
    _len: usize,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn write_u8(_event: *mut RakSampEventV1, _value: u8) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn write_u16(_event: *mut RakSampEventV1, _value: u16) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn write_u32(_event: *mut RakSampEventV1, _value: u32) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn write_f32(_event: *mut RakSampEventV1, _value: f32) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn write_bytes(
    _event: *mut RakSampEventV1,
    _value: *const u8,
    _len: usize,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn send(
    _id: u8,
    _data: *const u8,
    _byte_len: usize,
    _bit_len: usize,
    _options: RakSampSendOptions,
) -> RakSampResult {
    RakSampResult::NotReady
}

unsafe extern "system" fn replace_bytes(
    _event: *mut RakSampEventV1,
    _value: *const u8,
    _len: usize,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn emulate(
    _id: u8,
    _data: *const u8,
    _byte_len: usize,
    _bit_len: usize,
) -> RakSampResult {
    RakSampResult::NotReady
}

unsafe extern "system" fn remaining_bits(_event: *mut RakSampEventV1) -> usize {
    0
}

unsafe extern "system" fn read_bits(
    _event: *mut RakSampEventV1,
    _output: *mut u8,
    _bit_len: usize,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn replace_bits(
    _event: *mut RakSampEventV1,
    _value: *const u8,
    _byte_len: usize,
    _bit_len: usize,
) -> RakSampResult {
    RakSampResult::InvalidArgument
}

unsafe extern "system" fn encode_string(
    _value: *const u8,
    _value_len: usize,
    _output: *mut u8,
    _output_capacity: usize,
    _output_len: *mut usize,
) -> RakSampResult {
    RakSampResult::NotReady
}

unsafe extern "system" fn read_encoded_string(
    _event: *mut RakSampEventV1,
    _output: *mut u8,
    _output_capacity: usize,
    _output_len: *mut usize,
) -> RakSampResult {
    RakSampResult::NotReady
}

unsafe extern "system" fn show_local_dialog(
    _id: u16,
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
    let valid_style = style <= 5;
    let values = [
        (title, title_len),
        (text, text_len),
        (button1, button1_len),
        (button2, button2_len),
    ];
    if !valid_style
        || values
            .into_iter()
            .any(|(value, len)| value.is_null() && len != 0)
    {
        RakSampResult::InvalidArgument
    } else {
        RakSampResult::Ok
    }
}

unsafe extern "system" fn local_player(output: *mut RakSampLocalPlayerV1) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    let mut nickname = [0; 256];
    nickname[..3].copy_from_slice(b"e2e");
    *output = RakSampLocalPlayerV1 {
        id: 77,
        nickname_len: 3,
        nickname,
        colour: 0xFF11_2233,
        spawned: 1,
        special_action: 2,
        animation_id: 5,
        health: 100.0,
        armour: 25.0,
        position: Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        velocity: Vector3::default(),
        has_vehicle: 0,
        _reserved: 0,
        vehicle_id: 0,
        score: 9,
        ping: 20,
    };
    RakSampResult::Ok
}

unsafe extern "system" fn samp_game_state(output: *mut i32) -> RakSampResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return RakSampResult::InvalidArgument;
    };
    *output = 14;
    RakSampResult::Ok
}

static API: RakSampApiV1 = RakSampApiV1 {
    abi_version: ABI_VERSION_V1,
    size: std::mem::size_of::<RakSampApiV1>() as u32,
    host_status,
    register_packet,
    register_rpc,
    unregister,
    event_id,
    event_reset_read: event_result,
    event_clear: event_result,
    event_read_u8: read_u8,
    event_read_u16: read_u16,
    event_read_u32: read_u32,
    event_read_f32: read_f32,
    event_read_bytes: read_bytes,
    event_write_u8: write_u8,
    event_write_u16: write_u16,
    event_write_u32: write_u32,
    event_write_f32: write_f32,
    event_write_bytes: write_bytes,
    send_packet: send,
    send_rpc: send,
    event_replace_bytes: replace_bytes,
    unregister_and_wait: unregister,
    emulate_incoming_packet: emulate,
    emulate_incoming_rpc: emulate,
    event_remaining_bits: remaining_bits,
    event_read_bits: read_bits,
    event_replace_bits: replace_bits,
    encode_string,
    event_read_encoded_string: read_encoded_string,
    show_local_dialog,
    local_player,
    samp_game_state,
};
