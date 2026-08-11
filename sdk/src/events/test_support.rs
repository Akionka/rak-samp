use super::{EncodedPayload, Event, Rpc, RpcAction, core::PayloadWriter};
use crate::{HostApi, SampClientSdkEventV1, SampClientSdkHookAction, SampClientSdkResult};
use ::core::{mem, ptr};
use std::sync::Mutex;

mod api;
mod callbacks;

pub(crate) use api::test_api;
pub(crate) use callbacks::TestEvent;

#[derive(Clone, Copy)]
struct RegisteredCallback {
    callback: crate::SampClientSdkEventCallbackV1,
    user_data: usize,
    subscription: crate::SampClientSdkSubscription,
}

struct RegistrationState {
    register_result: SampClientSdkResult,
    unregister_result: SampClientSdkResult,
    unregister_and_wait_result: SampClientSdkResult,
    next_id: u64,
    callbacks: Vec<RegisteredCallback>,
    unregister_calls: u32,
    unregister_and_wait_calls: u32,
}

impl RegistrationState {
    const fn new() -> Self {
        Self {
            register_result: SampClientSdkResult::Ok,
            unregister_result: SampClientSdkResult::Ok,
            unregister_and_wait_result: SampClientSdkResult::Ok,
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

pub(crate) fn set_register_result(result: SampClientSdkResult) {
    REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .register_result = result;
}

pub(crate) fn set_unregister_and_wait_result(result: SampClientSdkResult) {
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

pub(crate) fn invoke_registered_callback(id: u8) -> Option<SampClientSdkHookAction> {
    let payload = EncodedPayload::from_bits(Vec::new(), 0).expect("an empty payload is valid");
    invoke_registered_callback_with_payload(id, payload)
}

pub(crate) fn invoke_registered_callback_with_payload(
    id: u8,
    payload: EncodedPayload,
) -> Option<SampClientSdkHookAction> {
    let callback = *REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .callbacks
        .first()?;
    let mut event = TestEvent::new(id, payload);
    Some(unsafe {
        (callback.callback)(
            callback.user_data as *mut ::core::ffi::c_void,
            (&mut event as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    })
}

extern "system" fn test_status() -> crate::SampClientSdkHostStatus {
    crate::SampClientSdkHostStatus::Ready
}

unsafe extern "system" fn test_register(
    _direction: crate::SampClientSdkDirection,
    callback: Option<crate::SampClientSdkEventCallbackV1>,
    user_data: *mut ::core::ffi::c_void,
    subscription: *mut crate::SampClientSdkSubscription,
) -> SampClientSdkResult {
    let Some(callback) = callback else {
        return SampClientSdkResult::InvalidArgument;
    };
    if subscription.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let mut state = REGISTRATION
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    if state.register_result != SampClientSdkResult::Ok {
        return state.register_result;
    }
    let handle = crate::SampClientSdkSubscription { id: state.next_id };
    state.next_id += 1;
    state.callbacks.push(RegisteredCallback {
        callback,
        user_data: user_data as usize,
        subscription: handle,
    });
    unsafe { subscription.write(handle) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_unregister(
    subscription: crate::SampClientSdkSubscription,
) -> SampClientSdkResult {
    unregister(subscription, false)
}

unsafe extern "system" fn test_unregister_and_wait(
    subscription: crate::SampClientSdkSubscription,
) -> SampClientSdkResult {
    unregister(subscription, true)
}

fn unregister(subscription: crate::SampClientSdkSubscription, wait: bool) -> SampClientSdkResult {
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
        SampClientSdkResult::Ok | SampClientSdkResult::SubscriptionNotFound
    ) {
        state
            .callbacks
            .retain(|callback| callback.subscription != subscription);
    }
    result
}

unsafe extern "system" fn test_send(
    id: u8,
    bytes: *const u8,
    byte_len: usize,
    bit_len: usize,
    options: crate::SampClientSdkSendOptions,
) -> SampClientSdkResult {
    if options != crate::SampClientSdkSendOptions::default() {
        return SampClientSdkResult::NativeCallFailed;
    }
    let bytes = if byte_len == 0 {
        &[]
    } else if bytes.is_null() {
        return SampClientSdkResult::NativeCallFailed;
    } else {
        unsafe { std::slice::from_raw_parts(bytes, byte_len) }
    };
    if (id, bit_len, bytes) == (101, 24, &[2, b'h', b'i'])
        || (id, bit_len, bytes) == (50, 56, &[3, 0, 0, 0, b'/', b'h', b'i'])
        || (id, bit_len, bytes) == (129, 0, &[])
        || (id, bit_len, bytes) == (128, 32, &[9, 0, 0, 0])
        || (id, bit_len, bytes) == (118, 8, &[7])
        || (id, bit_len, bytes) == (52, 0, &[])
        || (id, bit_len, bytes) == (26, 24, &[0x34, 0x12, 1])
        || (id, bit_len, bytes) == (154, 16, &[0x34, 0x12])
        || (id, bit_len, bytes) == (62, 64, &[0x34, 0x12, 1, 0x56, 0x34, 2, b'o', b'k'])
        || (id, bit_len, bytes) == (23, 24, &[0x34, 0x12, 2])
        || (id, bit_len, bytes) == (83, 16, &[0x34, 0x12])
        || (id, bit_len, bytes) == (53, 24, &[9, 0x34, 0x12])
        || (id, bit_len, bytes) == (140, 0, &[])
        || (id, bit_len, bytes) == (132, 8, &[7])
        || (id, bit_len, bytes) == (131, 32, &[9, 0, 0, 0])
        || (id, bit_len, bytes) == (136, 16, &[0x34, 0x12])
        || (id, bit_len, bytes) == (106, 96, &[0x34, 0x12, 1, 0, 0, 0, 2, 0, 0, 0, 3, 4])
        || (id, bit_len, bytes) == (96, 128, &[1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0])
        || (id, bit_len, bytes)
            == (
                115,
                113,
                &[
                    0x1A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00,
                    0x00, 0x00,
                ],
            )
        || (id, bit_len, bytes)
            == (
                115,
                113,
                &[
                    0x9A, 0x09, 0x00, 0x00, 0x40, 0x1F, 0x8C, 0x00, 0x00, 0x00, 0x04, 0x80, 0x00,
                    0x00, 0x00,
                ],
            )
        || (id, bit_len, bytes) == (116, 480, &[0; 60])
        || (id, bit_len, bytes) == (117, 241, &[0; 31])
        || (id, bit_len, bytes) == (201, 64, &[4, 0, 0, 0, b'r', b'c', b'o', b'n'])
        || (id, bit_len, bytes) == (203, 248, &[0; 31])
        || (id, bit_len, bytes) == (206, 320, &[0; 40])
        || (id, bit_len, bytes) == (200, 504, &[0; 63])
        || (id, bit_len, bytes) == (207, 544, &[0; 68])
        || (id, bit_len, bytes) == (212, 144, &[0; 18])
        || (id, bit_len, bytes) == (210, 432, &[0; 54])
        || (id, bit_len, bytes) == (211, 192, &[0; 24])
        || (id, bit_len, bytes) == (209, 536, &[0; 67])
    {
        SampClientSdkResult::Ok
    } else {
        SampClientSdkResult::NativeCallFailed
    }
}

unsafe extern "system" fn test_emulate(
    _id: u8,
    _bytes: *const u8,
    _byte_len: usize,
    _bit_len: usize,
) -> SampClientSdkResult {
    SampClientSdkResult::NativeCallFailed
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
) -> SampClientSdkResult {
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_show_local_chat_message(
    style: u32,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    text_colour: u32,
    prefix_colour: u32,
) -> SampClientSdkResult {
    let text = if text_len == 0 {
        &[]
    } else if text.is_null() {
        return SampClientSdkResult::InvalidArgument;
    } else {
        unsafe { std::slice::from_raw_parts(text, text_len) }
    };
    let prefix = if prefix_len == 0 {
        &[]
    } else if prefix.is_null() {
        return SampClientSdkResult::InvalidArgument;
    } else {
        unsafe { std::slice::from_raw_parts(prefix, prefix_len) }
    };
    if style == 8
        && text == b"local message"
        && prefix == b"[samp-client-sdk]"
        && text_colour == 0xFF_A9_C4_E4
        && prefix_colour == u32::MAX
    {
        SampClientSdkResult::Ok
    } else {
        SampClientSdkResult::NativeCallFailed
    }
}

unsafe extern "system" fn test_show_local_death_message(
    killer: *const u8,
    killer_len: usize,
    victim: *const u8,
    victim_len: usize,
    killer_colour: u32,
    victim_colour: u32,
    weapon: u8,
) -> SampClientSdkResult {
    let killer = if killer_len == 0 {
        &[]
    } else if killer.is_null() {
        return SampClientSdkResult::InvalidArgument;
    } else {
        unsafe { std::slice::from_raw_parts(killer, killer_len) }
    };
    let victim = if victim_len == 0 {
        &[]
    } else if victim.is_null() {
        return SampClientSdkResult::InvalidArgument;
    } else {
        unsafe { std::slice::from_raw_parts(victim, victim_len) }
    };
    if killer == b"killer"
        && victim == b"victim"
        && killer_colour == 0xFFFF_0000
        && victim_colour == 0xFF00_FF00
        && weapon == 24
    {
        SampClientSdkResult::Ok
    } else {
        SampClientSdkResult::NativeCallFailed
    }
}

unsafe extern "system" fn test_local_player(
    output: *mut crate::SampClientSdkLocalPlayerV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = crate::SampClientSdkLocalPlayerV1 {
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
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_samp_game_state(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 14;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_chat_display_mode(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 2;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_cursor_mode(output: *mut i32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 3;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_scoreboard_open(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_dialog_active(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_chat_input_active(output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_chat_input_text(
    output: *mut crate::SampClientSdkChatInputTextV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = crate::SampClientSdkChatInputTextV1::default();
    output.len = 4;
    output.bytes[..4].copy_from_slice(b"/sdk");
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_animation(
    id: u16,
    output: *mut crate::SampClientSdkAnimationV1,
) -> SampClientSdkResult {
    if id != 0 {
        return SampClientSdkResult::NotReady;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    output.name[..7].copy_from_slice(b"AIRPORT");
    output.name_len = 7;
    output.file[..14].copy_from_slice(b"THRW_BARL_THRW");
    output.file_len = 14;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_animation_id(
    name: *const u8,
    name_len: usize,
    file: *const u8,
    file_len: usize,
    output: *mut i32,
) -> SampClientSdkResult {
    if (name.is_null() && name_len != 0) || (file.is_null() && file_len != 0) {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    let name = if name_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(name, name_len) }
    };
    let file = if file_len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(file, file_len) }
    };
    *output = if name == b"AIRPORT" && file == b"THRW_BARL_THRW" {
        0
    } else {
        -1
    };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_player_info(
    id: u16,
    output: *mut crate::SampClientSdkPlayerInfoV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match id {
        42 => {
            let mut nickname = [0; 256];
            nickname[..7].copy_from_slice(b"fixture");
            *output = crate::SampClientSdkPlayerInfoV1 {
                exists: 1,
                is_local: 1,
                is_npc: 0,
                _reserved: 0,
                id,
                nickname_len: 7,
                nickname,
                colour: 0xFF00_00FF,
                score: 123,
                ping: 45,
            };
        }
        7 => {
            let mut nickname = [0; 256];
            nickname[..6].copy_from_slice(b"remote");
            *output = crate::SampClientSdkPlayerInfoV1 {
                exists: 1,
                is_local: 0,
                is_npc: 1,
                _reserved: 0,
                id,
                nickname_len: 6,
                nickname,
                colour: 0xFF22_4466,
                score: -10,
                ping: 55,
            };
        }
        _ => *output = crate::SampClientSdkPlayerInfoV1::default(),
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_player_count(
    include_npcs: u8,
    output: *mut u16,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    match include_npcs {
        0 => *output = 2,
        1 => *output = 3,
        _ => return SampClientSdkResult::InvalidArgument,
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_remote_player_state(
    id: u16,
    output: *mut crate::SampClientSdkRemotePlayerStateV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = if id == 7 {
        crate::SampClientSdkRemotePlayerStateV1 {
            exists: 1,
            special_action: 3,
            _reserved: 0,
            id,
            animation_id: 123,
            health: 75.0,
            armour: 25.0,
        }
    } else {
        crate::SampClientSdkRemotePlayerStateV1::default()
    };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_player_defined(id: u16, output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = u8::from(id == 7 || id == 42);
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_player_paused(id: u16, output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = u8::from(id == 9);
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_player_max_id(output: *mut u16) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 42;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_vehicle_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = u8::from(id == 7);
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_active_local_dialog(
    output: *mut crate::SampClientSdkActiveDialogV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    output.active = 1;
    output.style = 1;
    output.server_side = 0;
    output.id = 7;
    output.title[..7].copy_from_slice(b"fixture");
    output.title_len = 7;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_text_label_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = u8::from(id == 7);
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_textdraw_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = u8::from(id == 7);
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_object_exists(id: u16, output: *mut u8) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = u8::from(id == 7);
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_gangzone_info(
    id: u16,
    output: *mut crate::SampClientSdkGangzoneV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if id == 7 {
        *output = crate::SampClientSdkGangzoneV1 {
            exists: 1,
            _reserved: [0; 3],
            id,
            _reserved2: 0,
            left: -1.0,
            bottom: -2.0,
            right: 3.0,
            top: 4.0,
            colour: 0xFF11_2233,
            alternate_colour: 0xFF44_5566,
        };
    } else {
        *output = crate::SampClientSdkGangzoneV1::default();
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_text_label_info(
    id: u16,
    output: *mut crate::SampClientSdkTextLabelV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if id == 7 {
        output.exists = 1;
        output.behind_walls = 1;
        output.id = id;
        output.attached_player_id = 8;
        output.attached_vehicle_id = u16::MAX;
        output.colour = 0xFF11_2233;
        output.position = crate::Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        output.draw_distance = 25.0;
        output.text[..7].copy_from_slice(b"fixture");
        output.text_len = 7;
    } else {
        *output = crate::SampClientSdkTextLabelV1::default();
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_textdraw_info(
    pool_index: u16,
    output: *mut crate::SampClientSdkTextDrawV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if pool_index == 7 {
        *output = crate::SampClientSdkTextDrawV1 {
            exists: 1,
            proportional: 1,
            align_left: 0,
            align_center: 1,
            align_right: 0,
            box_enabled: 1,
            _reserved: [0; 2],
            pool_index,
            shadow: 2,
            outline: 3,
            letter_width: 1.0,
            letter_height: 2.0,
            letter_colour: 0xFF11_2233,
            x: 3.0,
            y: 4.0,
            background_colour: 0xFF44_5566,
            style: 5,
            box_width: 6.0,
            box_height: 7.0,
            box_colour: 0xFF77_8899,
            model_id: 10,
            _reserved2: 0,
            rotation: crate::Vector3 {
                x: 8.0,
                y: 9.0,
                z: 10.0,
            },
            zoom: 11.0,
            model_colour1: 12,
            model_colour2: 13,
            text_len: 7,
            _reserved3: [0; 2],
            text: {
                let mut text = [0; crate::limits::MAX_SAMP_TEXTDRAW_STRING_BYTES];
                text[..7].copy_from_slice(b"fixture");
                text
            },
        };
    } else {
        *output = crate::SampClientSdkTextDrawV1::default();
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_chat_entry_info(
    id: u16,
    output: *mut crate::SampClientSdkChatEntryV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    if id >= crate::limits::MAX_SAMP_CHAT_ENTRIES {
        return SampClientSdkResult::InvalidArgument;
    }
    *output = crate::SampClientSdkChatEntryV1 {
        id,
        text_len: 7,
        prefix_len: 6,
        text_colour: 0xFF11_2233,
        prefix_colour: 0xFF44_5566,
        text: {
            let mut text = [0; crate::limits::MAX_SAMP_CHAT_ENTRY_TEXT_BYTES];
            text[..7].copy_from_slice(b"fixture");
            text
        },
        prefix: {
            let mut prefix = [0; crate::limits::MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES];
            prefix[..6].copy_from_slice(b"prefix");
            prefix
        },
    };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_samp_version(output: *mut u32) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = crate::SampClientSdkClientVersion::R1 as u32;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_server_info(
    output: *mut crate::SampClientSdkServerInfoV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    output.address[..9].copy_from_slice(b"127.0.0.1");
    output.address_len = 9;
    output.hostname[..7].copy_from_slice(b"fixture");
    output.hostname_len = 7;
    output.port = 7777;
    SampClientSdkResult::Ok
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
) -> SampClientSdkResult {
    if input.is_null()
        || input_len != 1
        || input_bit_len != 3
        || input_read_offset != 0
        || output.is_null()
        || output_capacity < b"fixture".len() + 1
        || output_len.is_null()
        || output_read_offset.is_null()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    let input = unsafe { std::slice::from_raw_parts(input, input_len) };
    if input != [0b1010_0000] {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(b"fixture".as_ptr(), output, b"fixture".len());
        output_len.write(b"fixture".len());
        output_read_offset.write(input_bit_len);
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_local_dialog(
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
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    let Some(receipt) = (unsafe { receipt.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *receipt = crate::SampClientSdkCommandReceipt { id: 1 };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_local_chat_message(
    _style: u32,
    _text: *const u8,
    _text_len: usize,
    _prefix: *const u8,
    _prefix_len: usize,
    _text_colour: u32,
    _prefix_colour: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    let Some(receipt) = (unsafe { receipt.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *receipt = crate::SampClientSdkCommandReceipt { id: 2 };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_local_death_message(
    _killer: *const u8,
    _killer_len: usize,
    _victim: *const u8,
    _victim_len: usize,
    _killer_colour: u32,
    _victim_colour: u32,
    _weapon: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    let Some(receipt) = (unsafe { receipt.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *receipt = crate::SampClientSdkCommandReceipt { id: 3 };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_command_try_take(
    receipt: crate::SampClientSdkCommandReceipt,
    output: *mut crate::SampClientSdkCommandResultV1,
) -> SampClientSdkResult {
    if receipt.id == 0 {
        return SampClientSdkResult::InvalidArgument;
    }
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    output.status = SampClientSdkResult::Ok;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_command_wait(
    receipt: crate::SampClientSdkCommandReceipt,
    _timeout_ms: u32,
    output: *mut crate::SampClientSdkCommandResultV1,
) -> SampClientSdkResult {
    unsafe { test_command_try_take(receipt, output) }
}

unsafe extern "system" fn test_command_release(
    receipt: crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.id == 0 {
        SampClientSdkResult::InvalidArgument
    } else {
        SampClientSdkResult::Ok
    }
}

unsafe fn test_submit_command(
    receipt: *mut crate::SampClientSdkCommandReceipt,
    id: u64,
) -> SampClientSdkResult {
    let Some(receipt) = (unsafe { receipt.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *receipt = crate::SampClientSdkCommandReceipt { id };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_send(
    _id: u8,
    _bytes: *const u8,
    _byte_len: usize,
    _bit_len: usize,
    _options: crate::SampClientSdkSendOptions,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    unsafe { test_submit_command(receipt, 4) }
}

unsafe extern "system" fn test_submit_emulate(
    _id: u8,
    _bytes: *const u8,
    _byte_len: usize,
    _bit_len: usize,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    unsafe { test_submit_command(receipt, 5) }
}

unsafe extern "system" fn test_raw_rakclient(
    output: *mut *mut std::ffi::c_void,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0x1000usize as *mut std::ffi::c_void;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_raw_rakpeer(
    output: *mut *mut std::ffi::c_void,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0x2222usize as *mut std::ffi::c_void;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_raw_player_pool(
    output: *mut *mut std::ffi::c_void,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0x1001usize as *mut std::ffi::c_void;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_raw_vehicle_pool(
    output: *mut *mut std::ffi::c_void,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0x1002usize as *mut std::ffi::c_void;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_raw_local_player(
    output: *mut *mut std::ffi::c_void,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = 0x1003usize as *mut std::ffi::c_void;
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_local_cursor_mode(
    mode: i32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(mode, 0..=4) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 6) }
}

unsafe extern "system" fn test_submit_local_scoreboard_open(
    open: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(open, 0 | 1) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 7) }
}

unsafe extern "system" fn test_submit_local_dialog_client_side(
    client_side: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(client_side, 0 | 1) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 8) }
}

unsafe extern "system" fn test_submit_samp_game_state(
    state: i32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(state, 0 | 9 | 13 | 14 | 15 | 18) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 10) }
}

unsafe extern "system" fn test_submit_local_player_spawn(
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    unsafe { test_submit_command(receipt, 11) }
}

unsafe extern "system" fn test_submit_local_player_special_action(
    action: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(action, 0..=12 | 20..=25 | 68) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 12) }
}

unsafe extern "system" fn test_submit_send_rate(
    kind: u8,
    milliseconds: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(kind, 0..=2) || i32::try_from(milliseconds).is_err() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 13) }
}

unsafe extern "system" fn test_submit_player_colour(
    id: u16,
    _colour: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 21) }
}

unsafe extern "system" fn test_submit_local_player_name(
    name: *const u8,
    name_len: usize,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if name_len > 255 || (name_len != 0 && name.is_null()) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 22) }
}

unsafe extern "system" fn test_submit_force_unoccupied_sync(
    vehicle: u16,
    _seat: i32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if vehicle >= crate::limits::MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 23) }
}

unsafe extern "system" fn test_submit_connect_to_server(
    address: *const u8,
    address_len: usize,
    port: u16,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if address_len == 0 || address_len > 256 || port == 0 || address.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 24) }
}

unsafe extern "system" fn test_submit_disconnect_with_reason(
    _block_duration: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    unsafe { test_submit_command(receipt, 25) }
}

unsafe extern "system" fn test_submit_delete_textdraw(
    id: u16,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 26) }
}

unsafe extern "system" fn test_submit_set_textdraw_position(
    id: u16,
    x: f32,
    y: f32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS || !x.is_finite() || !y.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 27) }
}

unsafe extern "system" fn test_submit_set_textdraw_letter_style(
    id: u16,
    width: f32,
    height: f32,
    _colour: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 28) }
}

unsafe extern "system" fn test_submit_set_textdraw_proportional(
    id: u16,
    proportional: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS || proportional > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 29) }
}

unsafe extern "system" fn test_submit_set_textdraw_shadow(
    id: u16,
    _shadow: u8,
    _colour: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 30) }
}

unsafe extern "system" fn test_submit_set_textdraw_outline(
    id: u16,
    _outline: u8,
    _colour: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 31) }
}

unsafe extern "system" fn test_submit_set_textdraw_box(
    id: u16,
    enabled: u8,
    _colour: u32,
    width: f32,
    height: f32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS
        || enabled > 1
        || !width.is_finite()
        || !height.is_finite()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 32) }
}

unsafe extern "system" fn test_submit_set_textdraw_alignment(
    id: u16,
    alignment: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS || !(1..=3).contains(&alignment) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 33) }
}

unsafe extern "system" fn test_submit_set_textdraw_string(
    id: u16,
    text: *const u8,
    text_len: usize,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS
        || text.is_null() && text_len != 0
        || text_len > 1_601
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 34) }
}

unsafe extern "system" fn test_local_dialog_selected_item(output: *mut i32) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(0) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_local_dialog_selected_item(
    _selected: i32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    unsafe { test_submit_command(receipt, 35) }
}

unsafe extern "system" fn test_submit_delete_text_label(
    id: u16,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXT_LABELS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 36) }
}

unsafe extern "system" fn test_submit_create_text_label(
    id: u16,
    text: *const u8,
    text_len: usize,
    _colour: u32,
    position: crate::Vector3,
    draw_distance: f32,
    behind_walls: u8,
    _attached_player_id: u16,
    _attached_vehicle_id: u16,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXT_LABELS
        || text.is_null()
        || text_len > crate::limits::MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || !position.x.is_finite()
        || !position.y.is_finite()
        || !position.z.is_finite()
        || !draw_distance.is_finite()
        || behind_walls > 1
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 39) }
}

unsafe extern "system" fn test_submit_create_text_label_auto(
    text: *const u8,
    text_len: usize,
    _colour: u32,
    position: crate::Vector3,
    draw_distance: f32,
    behind_walls: u8,
    attached_player_id: u16,
    attached_vehicle_id: u16,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if text.is_null()
        || text_len > crate::limits::MAX_SAMP_TEXT_LABEL_TEXT_BYTES
        || !position.x.is_finite()
        || !position.y.is_finite()
        || !position.z.is_finite()
        || !draw_distance.is_finite()
        || behind_walls > 1
        || (attached_player_id != u16::MAX && attached_player_id >= crate::limits::MAX_SAMP_PLAYERS)
        || (attached_vehicle_id != u16::MAX
            && attached_vehicle_id >= crate::limits::MAX_SAMP_VEHICLES)
    {
        return SampClientSdkResult::InvalidArgument;
    }
    if unsafe { std::slice::from_raw_parts(text, text_len) }.contains(&0) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 42) }
}

unsafe extern "system" fn test_text_label_create_try_take(
    receipt: crate::SampClientSdkCommandReceipt,
    output: *mut crate::SampClientSdkTextLabelCreateResultV1,
) -> SampClientSdkResult {
    if receipt.id != 42 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe {
        output.write(crate::SampClientSdkTextLabelCreateResultV1 {
            status: SampClientSdkResult::Ok,
            id: 7,
            reserved: 0,
        });
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_text_label_create_wait(
    receipt: crate::SampClientSdkCommandReceipt,
    _timeout_ms: u32,
    output: *mut crate::SampClientSdkTextLabelCreateResultV1,
) -> SampClientSdkResult {
    unsafe { test_text_label_create_try_take(receipt, output) }
}

unsafe extern "system" fn test_local_dialog_list_item_count(
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(3) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_dialog_snapshot(
    output: *mut crate::SampClientSdkDialogSnapshotV1,
) -> SampClientSdkResult {
    let Some(output) = (unsafe { output.as_mut() }) else {
        return SampClientSdkResult::InvalidArgument;
    };
    *output = crate::SampClientSdkDialogSnapshotV1::default();
    output.active = 1;
    output.style = 1;
    output.id = 7;
    output.title_len = 7;
    output.text_len = 7;
    output.has_editbox = 1;
    output.editbox_text_len = 7;
    output.listbox_item_count = 3;
    output.title[..7].copy_from_slice(b"fixture");
    output.text[..7].copy_from_slice(b"fixture");
    output.editbox_text[..7].copy_from_slice(b"fixture");
    for item in &mut output.listbox_items[..3] {
        item.len = 7;
        item.bytes[..7].copy_from_slice(b"fixture");
    }
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_local_dialog_editbox_text(
    text: *const u8,
    text_len: usize,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if receipt.is_null()
        || text_len > crate::limits::MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES
        || (text.is_null() && text_len != 0)
    {
        return SampClientSdkResult::InvalidArgument;
    }
    if text_len != 0 && unsafe { std::slice::from_raw_parts(text, text_len) }.contains(&0) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 40) }
}

unsafe extern "system" fn test_local_object_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() || id >= crate::limits::MAX_SAMP_OBJECTS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(0x1000 + i32::from(id)) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_object_id_by_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write((handle - 0x1000) as u16) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_pickup_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() || id >= crate::limits::MAX_SAMP_PICKUPS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(0x2000 + i32::from(id)) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_pickup_id_by_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write((handle - 0x2000) as u16) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_vehicle_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() || id >= crate::limits::MAX_SAMP_VEHICLES {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(0x3000 + i32::from(id)) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_vehicle_id_by_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write((handle - 0x3000) as u16) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_player_ped_handle(
    id: u16,
    output: *mut i32,
) -> SampClientSdkResult {
    if output.is_null() || id >= crate::limits::MAX_SAMP_PLAYERS {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(0x4000 + i32::from(id)) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_local_player_id_by_ped_handle(
    handle: i32,
    output: *mut u16,
) -> SampClientSdkResult {
    if output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write((handle - 0x4000) as u16) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_set_textdraw_model_style(
    id: u16,
    x: f32,
    y: f32,
    z: f32,
    zoom: f32,
    _colour1: u16,
    _colour2: u16,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= crate::limits::MAX_SAMP_TEXTDRAWS
        || !x.is_finite()
        || !y.is_finite()
        || !z.is_finite()
        || !zoom.is_finite()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 37) }
}

unsafe extern "system" fn test_submit_local_chat_entry(
    id: u16,
    text: *const u8,
    text_len: usize,
    prefix: *const u8,
    prefix_len: usize,
    _text_colour: u32,
    _prefix_colour: u32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if id >= 100 || text.is_null() || prefix.is_null() || text_len >= 144 || prefix_len >= 28 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 38) }
}

unsafe extern "system" fn test_submit_local_cursor_toggle(
    show: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if show > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 14) }
}

unsafe extern "system" fn test_submit_local_chat_display_mode(
    mode: i32,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if !matches!(mode, 0..=2) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 15) }
}

unsafe extern "system" fn test_submit_local_dialog_close(
    button: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if button > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 16) }
}

unsafe extern "system" fn test_submit_local_chat_input_text(
    text: *const u8,
    text_len: usize,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if text_len > 128 || (text_len != 0 && text.is_null()) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 17) }
}

unsafe extern "system" fn test_submit_local_chat_input_enabled(
    enabled: u8,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if enabled > 1 {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 18) }
}

unsafe extern "system" fn test_submit_local_chat_input_process(
    text: *const u8,
    text_len: usize,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if text_len > 128 || (text_len != 0 && text.is_null()) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { test_submit_command(receipt, 19) }
}

unsafe extern "system" fn test_local_chat_command_defined(
    name: *const u8,
    name_len: usize,
    output: *mut u8,
) -> SampClientSdkResult {
    if name.is_null() || name_len == 0 || name_len > 32 || output.is_null() {
        return SampClientSdkResult::InvalidArgument;
    }
    let name = unsafe { std::slice::from_raw_parts(name, name_len) };
    if name.contains(&0) {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { output.write(u8::from(name == b"sdk")) };
    SampClientSdkResult::Ok
}

unsafe extern "system" fn test_submit_register_chat_command(
    name: *const u8,
    name_len: usize,
    callback: Option<crate::SampClientSdkChatCommandCallbackV1>,
    user_data: *mut std::ffi::c_void,
    subscription: *mut crate::SampClientSdkSubscription,
    receipt: *mut crate::SampClientSdkCommandReceipt,
) -> SampClientSdkResult {
    if name_len == 0
        || name_len > 32
        || name.is_null()
        || callback.is_none()
        || user_data.is_null()
        || subscription.is_null()
    {
        return SampClientSdkResult::InvalidArgument;
    }
    unsafe { subscription.write(crate::SampClientSdkSubscription { id: 41 }) };
    unsafe { test_submit_command(receipt, 41) }
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
    let mut event = unsafe {
        Event::from_callback(
            api,
            (&mut raw as *mut TestEvent).cast::<SampClientSdkEventV1>(),
        )
    }
    .expect("test event is not null");
    assert_eq!(
        descriptor
            .handle(&mut event, |decoded| {
                assert_eq!(decoded, value);
                RpcAction::Replace(decoded)
            })
            .expect("typed replacement must succeed"),
        SampClientSdkHookAction::Continue
    );
    assert_eq!(raw.bit_len, encoded.bit_len);
    assert_eq!(raw.bytes, encoded.bytes);
}
