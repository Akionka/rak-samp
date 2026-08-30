//! General SA-MP service v1 ABI.

use core::ffi::c_void;

use crate::{CommandReceiptId, ModResult, ServiceHeader, SubscriptionId};

pub const SAMP_SERVICE_VERSION_V1: u32 = 1;
pub const SAMP_MAX_PLAYERS: u16 = 1_004;
pub const SAMP_MAX_NICKNAME_BYTES: usize = 256;
pub const SAMP_MAX_SERVER_ADDRESS_BYTES: usize = 257;
pub const SAMP_MAX_SERVER_HOSTNAME_BYTES: usize = 257;
pub const SAMP_MAX_CHAT_TEXT_BYTES: u32 = 143;
pub const SAMP_MAX_CHAT_PREFIX_BYTES: u32 = 27;
pub const SAMP_MAX_CHAT_COMMAND_NAME_BYTES: u32 = 32;
pub const SAMP_MAX_CHAT_COMMAND_ARGUMENT_BYTES: u32 = 128;

pub const SAMP_VERSION_R1: u32 = 1;
pub const SAMP_VERSION_R2: u32 = 2;
pub const SAMP_VERSION_R3_1: u32 = 3;
pub const SAMP_VERSION_R4_2: u32 = 4;
pub const SAMP_VERSION_R5_1: u32 = 5;
pub const SAMP_VERSION_DL: u32 = 6;

pub const SAMP_CHAT_STYLE_CHAT: u32 = 2;
pub const SAMP_CHAT_STYLE_INFO: u32 = 4;
pub const SAMP_CHAT_STYLE_DEBUG: u32 = 8;

/// Plugin callback used to reclaim one opaque registration context.
pub type SampReleaseCallbackV1 = unsafe extern "system" fn(user_data: *mut c_void);

/// One copied local chat-command invocation.
pub type SampChatCommandCallbackV1 =
    unsafe extern "system" fn(user_data: *mut c_void, arguments: *const u8, arguments_len: u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampVector3V1 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Caller-owned output storage for the current server identity.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampServerInfoV1 {
    pub address_len: u16,
    pub hostname_len: u16,
    pub address: [u8; SAMP_MAX_SERVER_ADDRESS_BYTES],
    pub hostname: [u8; SAMP_MAX_SERVER_HOSTNAME_BYTES],
    pub port: u16,
}

/// Caller-owned output storage for the local player snapshot.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampLocalPlayerV1 {
    pub id: u16,
    pub nickname_len: u16,
    pub nickname: [u8; SAMP_MAX_NICKNAME_BYTES],
    pub colour: u32,
    pub spawned: u8,
    pub special_action: u8,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
    pub position: SampVector3V1,
    pub velocity: SampVector3V1,
    pub has_vehicle: u8,
    pub reserved: u8,
    pub vehicle_id: u16,
    pub score: i32,
    pub ping: u32,
}

/// Caller-owned output storage for one player lookup.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampPlayerInfoV1 {
    pub exists: u8,
    pub is_local: u8,
    pub is_npc: u8,
    pub reserved: u8,
    pub id: u16,
    pub nickname_len: u16,
    pub nickname: [u8; SAMP_MAX_NICKNAME_BYTES],
    pub colour: u32,
    pub score: i32,
    pub ping: u32,
}

macro_rules! impl_zeroed_default {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Default for $type {
                fn default() -> Self {
                    // Every field in these output-only ABI values accepts zero.
                    unsafe { core::mem::zeroed() }
                }
            }
        )+
    };
}

impl_zeroed_default!(SampServerInfoV1, SampLocalPlayerV1, SampPlayerInfoV1);

/// Small immutable general SA-MP service table.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SampServiceV1 {
    pub header: ServiceHeader,
    /// `ANY_THREAD + CALLBACK_SAFE`.
    pub version: unsafe extern "system" fn(out: *mut u32) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`.
    pub game_state: unsafe extern "system" fn(out: *mut i32) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`.
    pub server_info: unsafe extern "system" fn(out: *mut SampServerInfoV1) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`.
    pub local_player: unsafe extern "system" fn(out: *mut SampLocalPlayerV1) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`.
    pub player_info: unsafe extern "system" fn(id: u16, out: *mut SampPlayerInfoV1) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; copies inputs and queues one game command.
    pub submit_chat_add: unsafe extern "system" fn(
        style: u32,
        text: *const u8,
        text_len: u32,
        prefix: *const u8,
        prefix_len: u32,
        text_colour: u32,
        prefix_colour: u32,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
    /// `ANY_THREAD + CALLBACK_SAFE`; successful submission transfers context ownership.
    pub submit_register_chat_command: unsafe extern "system" fn(
        name: *const u8,
        name_len: u32,
        callback: Option<SampChatCommandCallbackV1>,
        user_data: *mut c_void,
        release: Option<SampReleaseCallbackV1>,
        out_subscription: *mut SubscriptionId,
        out_receipt: *mut CommandReceiptId,
    ) -> ModResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samp_values_have_fixed_layouts() {
        assert_eq!(core::mem::size_of::<SampVector3V1>(), 12);
        assert_eq!(core::mem::size_of::<SampServerInfoV1>(), 520);
        assert_eq!(core::mem::size_of::<SampLocalPlayerV1>(), 312);
        assert_eq!(core::mem::size_of::<SampPlayerInfoV1>(), 276);
        assert_eq!(core::mem::offset_of!(SampLocalPlayerV1, nickname), 4);
        assert_eq!(core::mem::offset_of!(SampLocalPlayerV1, position), 276);
        assert_eq!(core::mem::offset_of!(SampPlayerInfoV1, nickname), 8);
    }

    #[test]
    fn samp_service_layout_is_fixed() {
        let pointer_size = core::mem::size_of::<usize>();
        assert_eq!(core::mem::offset_of!(SampServiceV1, header), 0);
        assert_eq!(core::mem::offset_of!(SampServiceV1, version), 16);
        assert_eq!(
            core::mem::offset_of!(SampServiceV1, submit_register_chat_command),
            16 + 6 * pointer_size
        );
        assert_eq!(core::mem::size_of::<SampServiceV1>(), 16 + 7 * pointer_size);
    }
}
