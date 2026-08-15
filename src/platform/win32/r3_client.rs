//! Private SA-MP 0.3.7 R3-1 profile for fixture-backed direct helpers.
//!
//! The profile exposes only copied snapshots and native calls whose object
//! prefixes, method ABI, and fixed RVAs have all been pinned for R3-1. Raw
//! singleton access and every unverified pool family remain unavailable.

use super::r1_client::memory::{
    bounded_c_string, read_pointer, read_unaligned, read_vector3, readable_range, writable_range,
};
use crate::runtime::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, DirectClientError, GangzoneSnapshot,
    InCarSyncSnapshot, LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest,
    LocalDialogResponseSnapshot, LocalDialogSnapshot, LocalDialogStyle, LocalPlayerSnapshot,
    OnFootSyncSnapshot, PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot,
    ServerInfoSnapshot, TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
};
use std::{ffi::c_void, mem, ptr};

const SAMP_R3_1_ENTRY_POINT: u32 = 0x0C_C4_D0;
const NET_GAME_SINGLETON_RVA: usize = 0x26_E8_DC;
const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x30;
const NET_GAME_HOSTNAME_OFFSET: usize = 0x131;
const NET_GAME_PORT_OFFSET: usize = 0x235;
const NET_GAME_GAME_STATE_OFFSET: usize = 0x3CD;
const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
const NET_GAME_SCALAR_READABLE_SIZE: usize = NET_GAME_GAME_STATE_OFFSET + mem::size_of::<i32>();
const NET_GAME_GET_PLAYER_POOL_RVA: usize = 0x1160;
const NET_GAME_GET_VEHICLE_POOL_RVA: usize = 0x1170;
const NET_GAME_SHUTDOWN_FOR_RESTART_RVA: usize = 0xA1E0;
const NET_GAME_POOLS_OFFSET: usize = 0x3DE;
const NET_GAME_POOLS_LABEL_POOL_OFFSET: usize = 0x1C;
const NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET: usize = 0x20;
const NET_GAME_POOLS_OBJECT_POOL_OFFSET: usize = 0x14;
const NET_GAME_POOLS_GANGZONE_POOL_OFFSET: usize = 0x18;
const NET_GAME_POOLS_PICKUP_POOL_OFFSET: usize = 0x10;
const ONFOOT_SEND_RATE_RVA: usize = 0xFE0A8;
const INCAR_SEND_RATE_RVA: usize = 0xFE0AC;
const AIM_SEND_RATE_RVA: usize = 0xFE0B0;
const ANIMATION_TABLE_RVA: usize = 0x1039D0;
const ANIMATION_TABLE_ENTRY_COUNT: usize = 1812;
const ANIMATION_TABLE_ENTRY_SIZE: usize = 36;
const PLAYER_POOL_GET_LOCAL_PLAYER_RVA: usize = 0x1A30;
const PLAYER_POOL_GET_COUNT_RVA: usize = 0x13670;
const PLAYER_POOL_IS_CONNECTED_RVA: usize = 0x10B0;
const PLAYER_POOL_GET_REMOTE_PLAYER_RVA: usize = 0x10F0;
const PLAYER_POOL_GET_SCORE_RVA: usize = 0x6E0E0;
const PLAYER_POOL_GET_PING_RVA: usize = 0x6E110;
const PLAYER_POOL_GET_LOCAL_SCORE_RVA: usize = 0x6E140;
const PLAYER_POOL_GET_LOCAL_PING_RVA: usize = 0x6E150;
const PLAYER_POOL_GET_NAME_RVA: usize = 0x16F00;
const LOCAL_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x3DA0;
const PED_GET_HEALTH_RVA: usize = 0xAB4C0;
const PED_GET_ARMOUR_RVA: usize = 0xAB500;
const REMOTE_PLAYER_DOES_EXIST_RVA: usize = 0x1080;
const REMOTE_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x15C10;
const REMOTE_PLAYER_GET_STATUS_RVA: usize = 0x15DB0;
const REMOTE_PLAYER_SPECIAL_ACTION_OFFSET: usize = 0x18;
const REMOTE_PLAYER_PED_OFFSET: usize = 0x00;
const REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET: usize = 0x1AC;
const REMOTE_PLAYER_REPORTED_HEALTH_OFFSET: usize = 0x1B0;
const REMOTE_PLAYER_ANIMATION_OFFSET: usize = 0x1C0;
const REMOTE_PLAYER_STATE_READABLE_SIZE: usize = REMOTE_PLAYER_ANIMATION_OFFSET + 4;
const REMOTE_PLAYER_ONFOOT_OFFSET: usize = 0xC5;
const REMOTE_PLAYER_INCAR_OFFSET: usize = 0x19;
const REMOTE_PLAYER_PASSENGER_OFFSET: usize = 0xAD;
const REMOTE_PLAYER_TRAILER_OFFSET: usize = 0x58;
const REMOTE_PLAYER_AIM_OFFSET: usize = 0x8E;
const PLAYER_POOL_LARGEST_ID_OFFSET: usize = 0x00;
const PLAYER_POOL_OBJECTS_OFFSET: usize = 0x04;
const PLAYER_INFO_IS_NPC_OFFSET: usize = 0x28;
const PLAYER_INFO_READABLE_SIZE: usize = 0x2C;
const PLAYER_POOL_LOCAL_ID_OFFSET: usize = 0x2F1C;
const LOCAL_PLAYER_INCAR_OFFSET: usize = 0x04;
const LOCAL_PLAYER_ONFOOT_OFFSET: usize = 0x98;
const LOCAL_PLAYER_ACTIVE_OFFSET: usize = 0xF4;
const LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET: usize = 0xFC;
const LOCAL_PLAYER_SNAPSHOT_READABLE_SIZE: usize = LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET + 2;
const SAMP_PED_GAME_PED_OFFSET: usize = 0x2A4;
const INVALID_ID: u16 = u16::MAX;
const MAX_SAMP_PLAYERS: u16 = 1004;
const MAX_SAMP_VEHICLES: u16 = 2000;
const MAX_SAMP_OBJECTS: u16 = 1000;
const OBJECT_POOL_NOT_EMPTY_OFFSET: usize = 0x04;
const OBJECT_POOL_OBJECTS_OFFSET: usize = 0xFA4;
const PICKUP_POOL_HANDLES_OFFSET: usize = 0x04;
const ENTITY_HANDLE_OFFSET: usize = 0x44;
const MAX_SAMP_GANGZONES: u16 = 1024;
const GANGZONE_POOL_NOT_EMPTY_OFFSET: usize = 0x1000;
const GANGZONE_LEFT_OFFSET: usize = 0x00;
const GANGZONE_BOTTOM_OFFSET: usize = 0x04;
const GANGZONE_RIGHT_OFFSET: usize = 0x08;
const GANGZONE_TOP_OFFSET: usize = 0x0C;
const GANGZONE_COLOUR_OFFSET: usize = 0x10;
const GANGZONE_ALTERNATE_COLOUR_OFFSET: usize = 0x14;
const VEHICLE_POOL_NOT_EMPTY_OFFSET: usize = 0x3074;
const VEHICLE_POOL_GAME_OBJECTS_OFFSET: usize = 0x4FB4;
const VEHICLE_POOL_DOES_EXIST_RVA: usize = 0x1140;
const CPOOLS_GET_PED_REF: usize = 0x54_FF60;
const CPOOLS_GET_VEHICLE_REF: usize = 0x54_FFC0;
const RAKPEER_SIZE: usize = 0xDDE;
const RAK_CLIENT_DISCONNECT_VTABLE_SLOT: usize = 2;
const MAX_SAMP_TEXT_LABELS: u16 = 2048;
const MAX_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
const LABEL_POOL_NOT_EMPTY_OFFSET: usize = 0xE800;
const LABEL_SIZE: usize = 0x1D;
const LABEL_TEXT_OFFSET: usize = 0x00;
const LABEL_COLOUR_OFFSET: usize = 0x04;
const LABEL_POSITION_OFFSET: usize = 0x08;
const LABEL_DRAW_DISTANCE_OFFSET: usize = 0x14;
const LABEL_BEHIND_WALLS_OFFSET: usize = 0x18;
const LABEL_ATTACHED_PLAYER_OFFSET: usize = 0x19;
const LABEL_ATTACHED_VEHICLE_OFFSET: usize = 0x1B;
const LABEL_POOL_CREATE_RVA: usize = 0x11C0;
const LABEL_POOL_DELETE_RVA: usize = 0x12D0;
const MAX_SAMP_TEXTDRAWS: u16 = 2304;
const TEXTDRAW_POOL_OBJECTS_OFFSET: usize = 0x2400;
const TEXTDRAW_CREATE_RVA: usize = 0x1E1C0;
const TEXTDRAW_DELETE_RVA: usize = 0x1E0A0;
const TEXTDRAW_SET_TEXT_RVA: usize = 0xB26D0;
const TEXTDRAW_TRANSMIT_SIZE: usize = 0x3F;
const TEXTDRAW_TRANSMIT_X_OFFSET: usize = 0x21;
const TEXTDRAW_TRANSMIT_Y_OFFSET: usize = 0x25;
const TEXTDRAW_STRING_OFFSET: usize = 801;
const MAX_TEXTDRAW_CREATE_TEXT_BYTES: usize = 800;
const MAX_TEXTDRAW_STRING_BYTES: usize = 1601;
const TEXTDRAW_DATA_OFFSET: usize = 0x963;
const TEXTDRAW_X_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x28;
const TEXTDRAW_Y_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x2C;
const TEXTDRAW_STYLE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x24;
const TEXTDRAW_LETTER_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET;
const TEXTDRAW_PROPORTIONAL_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1B;
const TEXTDRAW_BACKGROUND_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1C;
const TEXTDRAW_SHADOW_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x20;
const TEXTDRAW_OUTLINE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x21;
const TEXTDRAW_ALIGN_CENTER_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0D;
const TEXTDRAW_ALIGN_LEFT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x22;
const TEXTDRAW_ALIGN_RIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x23;
const TEXTDRAW_BOX_ENABLED_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0E;
const TEXTDRAW_BOX_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0F;
const TEXTDRAW_BOX_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x13;
const TEXTDRAW_BOX_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x17;
const TEXTDRAW_MODEL_ID_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x45;
const TEXTDRAW_ROTATION_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x47;
const TEXTDRAW_ZOOM_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x53;
const TEXTDRAW_MODEL_COLOUR1_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x57;
const TEXTDRAW_MODEL_COLOUR2_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x59;
const ONFOOT_POSITION_OFFSET: usize = 0x06;
const ONFOOT_SPECIAL_ACTION_OFFSET: usize = 0x25;
const ONFOOT_SPEED_OFFSET: usize = 0x26;
const ONFOOT_ANIMATION_OFFSET: usize = 0x40;
const INCAR_POSITION_OFFSET: usize = 0x18;
const INCAR_SPEED_OFFSET: usize = 0x24;
const INCAR_SYNC_SIZE: usize = 0x3F;
const INCAR_VEHICLE_ID_OFFSET: usize = 0x00;
const INCAR_CONTROLLER_LEFT_STICK_X_OFFSET: usize = 0x02;
const INCAR_CONTROLLER_LEFT_STICK_Y_OFFSET: usize = 0x04;
const INCAR_CONTROLLER_BUTTONS_OFFSET: usize = 0x06;
const INCAR_QUATERNION_OFFSET: usize = 0x08;
const INCAR_VEHICLE_HEALTH_OFFSET: usize = 0x30;
const INCAR_DRIVER_HEALTH_OFFSET: usize = 0x34;
const INCAR_DRIVER_ARMOUR_OFFSET: usize = 0x35;
const INCAR_WEAPON_OFFSET: usize = 0x36;
const INCAR_SIREN_OFFSET: usize = 0x37;
const INCAR_LANDING_GEAR_OFFSET: usize = 0x38;
const INCAR_TRAILER_ID_OFFSET: usize = 0x39;
const INCAR_VEHICLE_SPECIFIC_OFFSET: usize = 0x3B;
const LOCAL_PLAYER_PASSENGER_OFFSET: usize = 0xDC;
const LOCAL_PLAYER_TRAILER_OFFSET: usize = 0x62;
const LOCAL_PLAYER_AIM_OFFSET: usize = 0x43;
const LOCAL_PLAYER_LAST_ANY_UPDATE_OFFSET: usize = 0x13F;
const PASSENGER_SYNC_SIZE: usize = 0x18;
const PASSENGER_VEHICLE_ID_OFFSET: usize = 0x00;
const PASSENGER_SEAT_ID_OFFSET: usize = 0x02;
const PASSENGER_WEAPON_OFFSET: usize = 0x03;
const PASSENGER_HEALTH_OFFSET: usize = 0x04;
const PASSENGER_ARMOUR_OFFSET: usize = 0x05;
const PASSENGER_CONTROLLER_LEFT_STICK_X_OFFSET: usize = 0x06;
const PASSENGER_CONTROLLER_LEFT_STICK_Y_OFFSET: usize = 0x08;
const PASSENGER_CONTROLLER_BUTTONS_OFFSET: usize = 0x0A;
const PASSENGER_POSITION_OFFSET: usize = 0x0C;
const TRAILER_SYNC_SIZE: usize = 0x36;
const TRAILER_ID_OFFSET: usize = 0x00;
const TRAILER_POSITION_OFFSET: usize = 0x02;
const TRAILER_QUATERNION_OFFSET: usize = 0x0E;
const TRAILER_SPEED_OFFSET: usize = 0x1E;
const TRAILER_TURN_SPEED_OFFSET: usize = 0x2A;
const AIM_SYNC_SIZE: usize = 0x1F;
const AIM_CAMERA_MODE_OFFSET: usize = 0x00;
const AIM_FIRST_OFFSET: usize = 0x01;
const AIM_POSITION_OFFSET: usize = 0x0D;
const AIM_Z_OFFSET: usize = 0x19;
const AIM_ZOOM_WEAPON_STATE_OFFSET: usize = 0x1D;
const AIM_ASPECT_RATIO_OFFSET: usize = 0x1E;
const ONFOOT_SYNC_SIZE: usize = 0x44;
const ONFOOT_CONTROLLER_LEFT_STICK_X_OFFSET: usize = 0x00;
const ONFOOT_CONTROLLER_LEFT_STICK_Y_OFFSET: usize = 0x02;
const ONFOOT_CONTROLLER_BUTTONS_OFFSET: usize = 0x04;
const ONFOOT_QUATERNION_OFFSET: usize = 0x12;
const ONFOOT_HEALTH_OFFSET: usize = 0x22;
const ONFOOT_ARMOUR_OFFSET: usize = 0x23;
const ONFOOT_WEAPON_OFFSET: usize = 0x24;
const ONFOOT_SURFING_OFFSET_OFFSET: usize = 0x32;
const ONFOOT_SURFING_VEHICLE_ID_OFFSET: usize = 0x3E;
const INPUT_SINGLETON_RVA: usize = 0x26_E8_CC;
const INPUT_OPEN_RVA: usize = 0x68D10;
const INPUT_CLOSE_RVA: usize = 0x68E10;
const INPUT_GET_COMMAND_HANDLER_RVA: usize = 0x68FA0;
const INPUT_ADD_COMMAND_RVA: usize = 0x69000;
const INPUT_PROCESS_RVA: usize = 0x69260;
const INPUT_EDIT_BOX_OFFSET: usize = 0x08;
const INPUT_COMMAND_PROC_OFFSET: usize = 0x0C;
const INPUT_COMMAND_NAME_OFFSET: usize = 0x24C;
const INPUT_COMMAND_NAME_CAPACITY: usize = 33;
const INPUT_COMMAND_COUNT_OFFSET: usize = 0x14DC;
const INPUT_ENABLED_OFFSET: usize = 0x14E0;
const INPUT_CACHE_READABLE_SIZE: usize = INPUT_ENABLED_OFFSET + mem::size_of::<i32>();
const MAX_CHAT_COMMANDS: usize = 144;
const CHAT_INPUT_TEXT_CAPACITY: usize = 129;
const DXUT_EDIT_BOX_SET_TEXT_RVA: usize = 0x84E70;
const DXUT_EDIT_BOX_GET_TEXT_RVA: usize = 0x84F40;
const CHAT_SINGLETON_RVA: usize = 0x26_E8_C8;
const CHAT_GET_MODE_RVA: usize = 0x60B40;
const CHAT_DISPLAY_MODE_OFFSET: usize = 0x08;
const CHAT_ENTRIES_OFFSET: usize = 0x132;
const CHAT_ENTRY_SIZE: usize = 0xFC;
const CHAT_ENTRY_PREFIX_OFFSET: usize = 0x04;
const CHAT_ENTRY_PREFIX_CAPACITY: usize = 28;
const CHAT_ENTRY_TEXT_OFFSET: usize = 0x20;
const CHAT_ENTRY_TEXT_CAPACITY: usize = 144;
const CHAT_ENTRY_TEXT_COLOUR_OFFSET: usize = 0xF4;
const CHAT_ENTRY_PREFIX_COLOUR_OFFSET: usize = 0xF8;
const MAX_CHAT_ENTRIES: u16 = 100;
const DEATH_WINDOW_SINGLETON_RVA: usize = 0x26_E8_D0;
const DIALOG_SINGLETON_RVA: usize = 0x26_E8_98;
const DIALOG_SHOW_RVA: usize = 0x6F8C0;
const DIALOG_CLOSE_RVA: usize = 0x6FF40;
const DIALOG_ACTIVE_OFFSET: usize = 0x28;
const DIALOG_ACTIVE_READABLE_SIZE: usize = DIALOG_ACTIVE_OFFSET + mem::size_of::<i32>();
const DIALOG_TYPE_OFFSET: usize = 0x2C;
const DIALOG_ID_OFFSET: usize = 0x30;
const DIALOG_TEXT_OFFSET: usize = 0x34;
const DIALOG_CAPTION_OFFSET: usize = 0x40;
const DIALOG_CAPTION_CAPACITY: usize = 65;
const DIALOG_EDITBOX_OFFSET: usize = 0x24;
const DIALOG_LISTBOX_OFFSET: usize = 0x20;
const DIALOG_SERVER_SIDE_OFFSET: usize = 0x81;
const DIALOG_SNAPSHOT_READABLE_SIZE: usize = DIALOG_SERVER_SIDE_OFFSET + mem::size_of::<i32>();
const DXUT_LISTBOX_SELECTED_OFFSET: usize = 0x143;
const DXUT_LISTBOX_ITEMS_OFFSET: usize = 0x14C;
const DXUT_LISTBOX_ITEM_COUNT_OFFSET: usize = 0x150;
const DXUT_LISTBOX_ITEM_TEXT_CAPACITY: usize = 256;
const MAX_DIALOG_TEXT_BYTES: usize = 4_096;
const MAX_DIALOG_EDITBOX_TEXT_BYTES: usize = 128;
const MAX_DIALOG_LISTBOX_ITEMS: usize = 100;
const SCOREBOARD_SINGLETON_RVA: usize = 0x26_E8_94;
const SCOREBOARD_ENABLED_OFFSET: usize = 0x00;
const SCOREBOARD_READABLE_SIZE: usize = 0x44;
const GAME_SINGLETON_RVA: usize = 0x26_E8_F4;
const GAME_CURSOR_MODE_OFFSET: usize = 0x61;
const GAME_CURSOR_MODE_READABLE_SIZE: usize = GAME_CURSOR_MODE_OFFSET + mem::size_of::<i32>();
const GAME_SET_CURSOR_MODE_RVA: usize = 0x9FFE0;
const GAME_PROCESS_INPUT_ENABLING_RVA: usize = 0x9FEC0;

type NetGameGetPlayerPoolFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type PlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type PlayerPoolGetRemotePlayerFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
type PlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type PlayerPoolGetLocalStatFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolGetNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type LocalPlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
type DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type ChatGetModeFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type RemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type RemotePlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type RemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type LocalPlayerNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type LocalPlayerTrailerFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type LocalPlayerUnoccupiedFn = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type LocalPlayerSpawnFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type LocalPlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type RemotePlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type PlayerPoolSetLocalPlayerNameFn = unsafe extern "thiscall" fn(*mut c_void, *const i8);
type LocalPlayerSetSpecialActionFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type ChatAddEntryFn = unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type DialogShowFn = unsafe extern "thiscall" fn(
    *mut c_void,
    i32,
    i32,
    *const i8,
    *const i8,
    *const i8,
    *const i8,
    i32,
);
type DialogCloseFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type GameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type GameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);
type InputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type InputGetCommandHandlerFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8) -> *const c_void;
type InputAddCommandFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, unsafe extern "cdecl" fn(*const i8));
type DxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);
type LabelPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, u16, *const u8, u32, NativeVector3, f32, u8, u16, u16);
type LabelPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type TextdrawPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *mut c_void, *const u8) -> *mut c_void;
type TextdrawPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type TextdrawSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const u8);
type CpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;
type NetGameNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type RakClientDisconnectFn = unsafe extern "thiscall" fn(*mut c_void, u32, u8);

#[repr(C)]
#[derive(Clone, Copy)]
struct NativeVector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vector3> for NativeVector3 {
    fn from(value: Vector3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

/// The narrowly verified R3-1 read-only cache profile.
#[derive(Clone, Copy, Debug)]
pub(super) struct R3ClientProfile {
    module_base: usize,
}

impl R3ClientProfile {
    /// Selects this partial profile only for the pinned R3-1 executable.
    pub(super) fn verify(module_base: usize, entry_point: u32) -> Option<Self> {
        (module_base != 0 && entry_point == SAMP_R3_1_ENTRY_POINT).then_some(Self { module_base })
    }

    pub(super) const fn dialog_close_target(self) -> usize {
        self.module_base + DIALOG_CLOSE_RVA
    }

    /// Returns the R3 RakPeer base proved by the R3 RakClient constructor.
    pub(super) fn rakpeer_address(
        self,
        rakclient: *mut c_void,
    ) -> Result<*mut c_void, DirectClientError> {
        let peer = (rakclient as usize)
            .checked_sub(RAKPEER_SIZE)
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if peer.is_null() || !readable_range(peer.cast(), RAKPEER_SIZE + 1) {
            return Err(DirectClientError::NotReady);
        }
        Ok(peer)
    }

    /// Captures the R3-1 CNetGame state with a guarded scalar read.
    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let field = (net_game as usize)
            .checked_add(NET_GAME_GAME_STATE_OFFSET)
            .ok_or(DirectClientError::NotReady)?;
        unsafe { read_unaligned::<i32>(field) }.ok_or(DirectClientError::NotReady)
    }

    pub(super) fn set_game_state(self, state: i32) -> Result<(), DirectClientError> {
        let native_state = match state {
            0 => 0,
            9 => 1,
            13 => 2,
            14 => 5,
            15 => 6,
            18 => 11,
            _ => return Err(DirectClientError::NotReady),
        };
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let field = (net_game as *mut u8)
            .wrapping_add(NET_GAME_GAME_STATE_OFFSET)
            .cast::<i32>();
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, native_state) };
        Ok(())
    }

    pub(super) fn connect_to_server(
        self,
        address: &[u8],
        port: u16,
    ) -> Result<(), DirectClientError> {
        if address.is_empty()
            || address.len() >= NET_GAME_HOST_STRING_CAPACITY
            || address.contains(&0)
            || port == 0
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let host_address = (net_game as *mut u8).wrapping_add(NET_GAME_HOST_ADDRESS_OFFSET);
        let port_field = (net_game as *mut u8)
            .wrapping_add(NET_GAME_PORT_OFFSET)
            .cast::<i32>();
        if !writable_range(host_address, NET_GAME_HOST_STRING_CAPACITY)
            || !writable_range(port_field.cast(), mem::size_of::<i32>())
        {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_bytes(host_address, 0, NET_GAME_HOST_STRING_CAPACITY);
            ptr::copy_nonoverlapping(address.as_ptr(), host_address, address.len());
            ptr::write_unaligned(port_field, i32::from(port));
        }
        self.set_game_state(9)
    }

    /// Executes the R3 RakClient disconnect virtual followed by CNetGame's
    /// R3 restart shutdown routine on the game thread.
    pub(super) fn disconnect_with_reason(
        self,
        rak_client: *mut c_void,
        block_duration: u32,
    ) -> Result<(), DirectClientError> {
        if rak_client.is_null() {
            return Err(DirectClientError::NotReady);
        }
        let vtable = unsafe { read_pointer(rak_client as usize) }
            .filter(|vtable| !vtable.is_null())
            .ok_or(DirectClientError::NotReady)?;
        let offset = RAK_CLIENT_DISCONNECT_VTABLE_SLOT * mem::size_of::<usize>();
        if !readable_range(vtable, offset + mem::size_of::<usize>()) {
            return Err(DirectClientError::NotReady);
        }
        let address = unsafe { read_pointer(vtable as usize + offset) }
            .filter(|address| !address.is_null())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(address, 1) {
            return Err(DirectClientError::NotReady);
        }
        let disconnect: RakClientDisconnectFn = unsafe { mem::transmute(address) };
        unsafe { disconnect(rak_client, block_duration, 0) };
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let shutdown: NetGameNoArgFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_SHUTDOWN_FOR_RESTART_RVA) };
        unsafe { shutdown(net_game) };
        Ok(())
    }

    pub(super) fn animation_catalog(self) -> Result<Vec<AnimationSnapshot>, DirectClientError> {
        let table = self.module_base + ANIMATION_TABLE_RVA;
        (0..ANIMATION_TABLE_ENTRY_COUNT)
            .map(|index| {
                let entry = table
                    .checked_add(index * ANIMATION_TABLE_ENTRY_SIZE)
                    .ok_or(DirectClientError::NotReady)?;
                if !readable_range(entry as *const u8, ANIMATION_TABLE_ENTRY_SIZE) {
                    return Err(DirectClientError::NotReady);
                }
                let bytes = unsafe {
                    std::slice::from_raw_parts(entry as *const u8, ANIMATION_TABLE_ENTRY_SIZE)
                };
                parse_animation_entry(bytes)
            })
            .collect()
    }

    /// Captures copied R3-1 server metadata from the guarded CNetGame fields.
    pub(super) fn server_info(self) -> Result<ServerInfoSnapshot, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let address = unsafe {
            bounded_c_string(
                net_game
                    .cast::<u8>()
                    .wrapping_add(NET_GAME_HOST_ADDRESS_OFFSET),
                NET_GAME_HOST_STRING_CAPACITY,
            )
        }
        .filter(|address| !address.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let hostname = unsafe {
            bounded_c_string(
                net_game.cast::<u8>().wrapping_add(NET_GAME_HOSTNAME_OFFSET),
                NET_GAME_HOST_STRING_CAPACITY,
            )
        }
        .filter(|hostname| !hostname.is_empty())
        .ok_or(DirectClientError::NotReady)?;
        let port_field = (net_game as usize)
            .checked_add(NET_GAME_PORT_OFFSET)
            .ok_or(DirectClientError::NotReady)?;
        let port = unsafe { read_unaligned::<i32>(port_field) }
            .and_then(|port| u16::try_from(port).ok())
            .filter(|port| *port != 0)
            .ok_or(DirectClientError::NotReady)?;
        Ok(ServerInfoSnapshot {
            address,
            hostname,
            port,
        })
    }

    /// Copies the verified R3-1 local-player cache surface on the game thread.
    pub(super) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        let pool = self.player_pool()?;
        let id = unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
            .filter(|id| *id < MAX_SAMP_PLAYERS)
            .ok_or(DirectClientError::NotReady)?;
        let get_local: PlayerPoolGetLocalPlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
        let local = unsafe { get_local(pool) };
        if local.is_null() || !readable_range(local.cast(), LOCAL_PLAYER_SNAPSHOT_READABLE_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let ped = unsafe { read_pointer(local as usize) }
            .filter(|ped| !ped.is_null())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            ped.cast(),
            SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let game_ped = unsafe { read_pointer(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
            .filter(|ped| !ped.is_null())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(game_ped.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let get_name: PlayerPoolGetNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
        let get_score: PlayerPoolGetLocalStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_SCORE_RVA) };
        let get_ping: PlayerPoolGetLocalStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PING_RVA) };
        let get_colour: LocalPlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_COLOUR_ARGB_RVA) };
        let get_health: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_HEALTH_RVA) };
        let get_armour: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_ARMOUR_RVA) };

        let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
            .filter(|name| !name.is_empty())
            .ok_or(DirectClientError::NotReady)?;
        let current_vehicle =
            unsafe { read_unaligned::<u16>(local as usize + LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let vehicle_id = (current_vehicle != INVALID_ID).then_some(current_vehicle);
        let (position, velocity) = if vehicle_id.is_some() {
            (
                unsafe {
                    read_vector3(local as usize + LOCAL_PLAYER_INCAR_OFFSET + INCAR_POSITION_OFFSET)
                },
                unsafe {
                    read_vector3(local as usize + LOCAL_PLAYER_INCAR_OFFSET + INCAR_SPEED_OFFSET)
                },
            )
        } else {
            (
                unsafe {
                    read_vector3(
                        local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + ONFOOT_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + ONFOOT_SPEED_OFFSET)
                },
            )
        };

        Ok(LocalPlayerSnapshot {
            id,
            nickname,
            colour: unsafe { get_colour(local) },
            spawned: unsafe { read_unaligned::<u32>(local as usize + LOCAL_PLAYER_ACTIVE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?
                != 0,
            health: unsafe { get_health(ped.cast()) },
            armour: unsafe { get_armour(ped.cast()) },
            position: position.ok_or(DirectClientError::NotReady)?,
            velocity: velocity.ok_or(DirectClientError::NotReady)?,
            special_action: unsafe {
                read_unaligned::<u8>(
                    local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + ONFOOT_SPECIAL_ACTION_OFFSET,
                )
            }
            .ok_or(DirectClientError::NotReady)?,
            animation_id: unsafe {
                read_unaligned::<u32>(
                    local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + ONFOOT_ANIMATION_OFFSET,
                )
            }
            .ok_or(DirectClientError::NotReady)? as u16,
            vehicle_id,
            score: unsafe { get_score(pool) },
            ping: (unsafe { get_ping(pool) }).max(0) as u32,
        })
    }

    /// Copies both R3-1 `CPlayerPool::GetCount` modes on the game thread.
    pub(super) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        let pool = self.player_pool()?;
        let get_count: PlayerPoolGetCountFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_COUNT_RVA) };
        copy_player_counts(pool, get_count)
    }

    /// Copies the R3-1 player-pool largest ID from its verified prefix.
    pub(super) fn player_max_id(self) -> Result<u16, DirectClientError> {
        let pool = self.player_pool()?;
        copy_player_max_id(pool)
    }

    /// Copies one R3-1 remote-player record on the game thread.
    pub(super) fn player_info(
        self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        let pool = self.player_pool()?;
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        let get_name: PlayerPoolGetNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
        let get_score: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_SCORE_RVA) };
        let get_ping: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_PING_RVA) };
        let get_colour: RemotePlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_COLOUR_ARGB_RVA) };
        let get_status: RemotePlayerGetStatusFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_STATUS_RVA) };
        copy_player_info(
            pool,
            id,
            is_connected,
            get_player,
            does_exist,
            get_name,
            get_score,
            get_ping,
            get_colour,
            get_status,
        )
    }

    /// Copies R3-1 remote state fields maintained by the network update path.
    pub(super) fn remote_player_state(
        self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        let pool = self.player_pool()?;
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        copy_remote_player_state(pool, id, is_connected, get_player, does_exist)
    }

    /// Determines whether the R3-1 remote player currently lacks a GTA ped.
    pub(super) fn remote_player_is_streamed_out(
        self,
        id: u16,
    ) -> Result<Option<bool>, DirectClientError> {
        let pool = self.player_pool()?;
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        copy_remote_player_is_streamed_out(pool, id, is_connected, get_player, does_exist)
    }

    /// Copies one R3-1 on-foot synchronization record on the game thread.
    pub(super) fn onfoot_sync(
        self,
        id: u16,
    ) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|local_id| *local_id < MAX_SAMP_PLAYERS);
        if local_id == Some(id) {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            return (!local.is_null())
                .then(|| copy_onfoot_sync(id, local as usize + LOCAL_PLAYER_ONFOOT_OFFSET))
                .transpose();
        }
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        copy_remote_onfoot_sync(pool, id, is_connected, get_player, does_exist)
    }

    /// Copies one R3-1 in-car synchronization record on the game thread.
    pub(super) fn incar_sync(
        self,
        id: u16,
    ) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|local_id| *local_id < MAX_SAMP_PLAYERS);
        if local_id == Some(id) {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            return (!local.is_null())
                .then(|| copy_incar_sync(id, local as usize + LOCAL_PLAYER_INCAR_OFFSET))
                .transpose();
        }
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        copy_remote_incar_sync(pool, id, is_connected, get_player, does_exist)
    }

    /// Copies one R3-1 passenger synchronization record on the game thread.
    pub(super) fn passenger_sync(
        self,
        id: u16,
    ) -> Result<Option<PassengerSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|local_id| *local_id < MAX_SAMP_PLAYERS);
        if local_id == Some(id) {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            return (!local.is_null())
                .then(|| copy_passenger_sync(id, local as usize + LOCAL_PLAYER_PASSENGER_OFFSET))
                .transpose();
        }
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { is_connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null()
            || !readable_range(
                remote.cast(),
                REMOTE_PLAYER_PASSENGER_OFFSET + PASSENGER_SYNC_SIZE,
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => {
                copy_passenger_sync(id, remote as usize + REMOTE_PLAYER_PASSENGER_OFFSET).map(Some)
            }
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Copies one R3-1 trailer synchronization record on the game thread.
    pub(super) fn trailer_sync(
        self,
        id: u16,
    ) -> Result<Option<TrailerSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|local_id| *local_id < MAX_SAMP_PLAYERS);
        if local_id == Some(id) {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            return (!local.is_null())
                .then(|| copy_trailer_sync(id, local as usize + LOCAL_PLAYER_TRAILER_OFFSET))
                .transpose();
        }
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { is_connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null()
            || !readable_range(
                remote.cast(),
                REMOTE_PLAYER_TRAILER_OFFSET + TRAILER_SYNC_SIZE,
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => copy_trailer_sync(id, remote as usize + REMOTE_PLAYER_TRAILER_OFFSET).map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Copies one R3-1 aim synchronization record on the game thread.
    pub(super) fn aim_sync(self, id: u16) -> Result<Option<AimSyncSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|local_id| *local_id < MAX_SAMP_PLAYERS);
        if local_id == Some(id) {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            return (!local.is_null())
                .then(|| copy_aim_sync(id, local as usize + LOCAL_PLAYER_AIM_OFFSET))
                .transpose();
        }
        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        match unsafe { is_connected(pool, id) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null()
            || !readable_range(remote.cast(), REMOTE_PLAYER_AIM_OFFSET + AIM_SYNC_SIZE)
        {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => Ok(None),
            1 => copy_aim_sync(id, remote as usize + REMOTE_PLAYER_AIM_OFFSET).map(Some),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Invokes R3-1 `SCLocalPlayer::SendAimData` on the game thread.
    pub(super) fn force_aim_sync(self) -> Result<(), DirectClientError> {
        self.force_no_arg_sync(0x5040, true)
    }

    /// Invokes R3-1 `SCLocalPlayer::SendOnfootData` on the game thread.
    pub(super) fn force_onfoot_sync(self) -> Result<(), DirectClientError> {
        self.force_no_arg_sync(0x4D40, true)
    }

    /// Invokes R3-1 `SCLocalPlayer::SendStats` on the game thread.
    pub(super) fn force_stats_sync(self) -> Result<(), DirectClientError> {
        self.force_no_arg_sync(0x5B10, true)
    }

    /// Invokes R3-1 `SCLocalPlayer::UpdateWeapons` on the game thread.
    pub(super) fn force_weapons_sync(self) -> Result<(), DirectClientError> {
        self.force_no_arg_sync(0x6090, false)
    }

    /// Invokes R3-1 `SCLocalPlayer::SendTrailerData` on the game thread.
    pub(super) fn force_trailer_sync(self, trailer: u16) -> Result<(), DirectClientError> {
        if trailer >= 2000 {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        self.reset_last_any_update(local)?;
        let send: LocalPlayerTrailerFn = unsafe { mem::transmute(self.module_base + 0x51F0) };
        unsafe { send(local, trailer) };
        Ok(())
    }

    /// Updates R3-1 local in-car data and invokes `SendIncarData`.
    pub(super) fn force_vehicle_sync(self, vehicle: u16) -> Result<(), DirectClientError> {
        if vehicle >= 2000 {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let target = (local as usize + LOCAL_PLAYER_INCAR_OFFSET) as *mut u16;
        if !writable_range(target.cast(), mem::size_of::<u16>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { std::ptr::write_unaligned(target, vehicle) };
        self.reset_last_any_update(local)?;
        let send: LocalPlayerNoArgFn = unsafe { mem::transmute(self.module_base + 0x6E40) };
        unsafe { send(local) };
        Ok(())
    }

    /// Updates R3-1 passenger data and invokes `SendPassengerData`.
    pub(super) fn force_passenger_sync(
        self,
        vehicle: u16,
        seat: u8,
    ) -> Result<(), DirectClientError> {
        if vehicle >= 2000 {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let vehicle_field = (local as usize + LOCAL_PLAYER_PASSENGER_OFFSET) as *mut u16;
        let seat_field =
            (local as usize + LOCAL_PLAYER_PASSENGER_OFFSET + PASSENGER_SEAT_ID_OFFSET) as *mut u8;
        if !writable_range(vehicle_field.cast(), mem::size_of::<u16>())
            || !writable_range(seat_field.cast(), mem::size_of::<u8>())
        {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_unaligned(vehicle_field, vehicle);
            std::ptr::write_unaligned(seat_field, seat);
        }
        self.reset_last_any_update(local)?;
        let send: LocalPlayerNoArgFn = unsafe { mem::transmute(self.module_base + 0x53B0) };
        unsafe { send(local) };
        Ok(())
    }

    /// Invokes R3-1 `SCLocalPlayer::SendUnoccupiedData`.
    pub(super) fn force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: i32,
    ) -> Result<(), DirectClientError> {
        if vehicle >= 2000 || !(i32::from(i8::MIN)..=i32::from(i8::MAX)).contains(&seat) {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let send: LocalPlayerUnoccupiedFn = unsafe { mem::transmute(self.module_base + 0x4B60) };
        unsafe { send(local, vehicle, seat) };
        Ok(())
    }

    pub(super) fn set_send_rate(
        self,
        kind: u8,
        milliseconds: u32,
    ) -> Result<(), DirectClientError> {
        let rate = i32::try_from(milliseconds).map_err(|_| DirectClientError::NotReady)?;
        let rva = match kind {
            0 => ONFOOT_SEND_RATE_RVA,
            1 => INCAR_SEND_RATE_RVA,
            2 => AIM_SEND_RATE_RVA,
            _ => return Err(DirectClientError::NotReady),
        };
        let field = (self.module_base + rva) as *mut i32;
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, rate) };
        Ok(())
    }

    pub(super) fn spawn_local_player(self) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        let spawn: LocalPlayerSpawnFn = unsafe { mem::transmute(self.module_base + 0x3AD0) };
        (unsafe { spawn(local) } != 0)
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn set_local_player_name(self, name: &[u8]) -> Result<(), DirectClientError> {
        if name.len() > 255 || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let mut name = name.to_vec();
        name.push(0);
        let set_name: PlayerPoolSetLocalPlayerNameFn =
            unsafe { mem::transmute(self.module_base + 0xB5C0) };
        unsafe { set_name(pool, name.as_ptr().cast()) };
        Ok(())
    }

    pub(super) fn set_player_colour(self, id: u16, colour: u32) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) };
        if local_id == Some(id) {
            let local = self.local_player_address()?;
            let set: LocalPlayerSetColourFn = unsafe { mem::transmute(self.module_base + 0x3D50) };
            unsafe { set(local, colour) };
            return Ok(());
        }
        let connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        if unsafe { connected(pool, id) } != 1 {
            return Err(DirectClientError::NotReady);
        }
        let get: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get(pool, id) };
        if remote.is_null() || !readable_range(remote.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let set: RemotePlayerSetColourFn = unsafe { mem::transmute(self.module_base + 0x15BE0) };
        unsafe { set(remote, colour) };
        Ok(())
    }

    pub(super) fn set_local_player_special_action(
        self,
        action: u8,
    ) -> Result<(), DirectClientError> {
        if !matches!(action, 0..=12 | 20..=25 | 68) {
            return Err(DirectClientError::NotReady);
        }
        let local = self.local_player_address()?;
        let set: LocalPlayerSetSpecialActionFn =
            unsafe { mem::transmute(self.module_base + 0x30C0) };
        unsafe { set(local, action) };
        Ok(())
    }

    pub(super) fn show_chat_message(
        self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        if request.text.contains(&0) || request.prefix.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let mut text = request.text;
        let mut prefix = request.prefix;
        text.push(0);
        prefix.push(0);
        let add: ChatAddEntryFn = unsafe { mem::transmute(self.module_base + 0x67460) };
        unsafe {
            add(
                chat,
                request.style.as_raw(),
                text.as_ptr().cast(),
                prefix.as_ptr().cast(),
                request.text_colour,
                request.prefix_colour,
            )
        };
        Ok(())
    }

    pub(super) fn chat_is_ready(self) -> bool {
        self.chat().is_some()
    }

    pub(super) fn show_death_message(
        self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let window = self.death_window().ok_or(DirectClientError::NotReady)?;
        if request.killer.contains(&0) || request.victim.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let mut killer = request.killer;
        let mut victim = request.victim;
        killer.push(0);
        victim.push(0);
        let add: DeathWindowAddMessageFn = unsafe { mem::transmute(self.module_base + 0x69F40) };
        unsafe {
            add(
                window,
                killer.as_ptr().cast(),
                victim.as_ptr().cast(),
                request.killer_colour,
                request.victim_colour,
                request.weapon,
            )
        };
        Ok(())
    }

    pub(super) fn death_window_is_ready(self) -> bool {
        self.death_window().is_some()
    }

    /// Copies the R3-1 chat-input enabled flag without invoking its UI methods.
    pub(super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(input as usize + INPUT_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Opens or closes the R3 chat input through its native transition method.
    pub(super) fn set_chat_input_enabled(self, enabled: bool) -> Result<(), DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let operation: InputNoArgFn = unsafe {
            mem::transmute(
                self.module_base
                    + if enabled {
                        INPUT_OPEN_RVA
                    } else {
                        INPUT_CLOSE_RVA
                    },
            )
        };
        unsafe { operation(input) };
        Ok(())
    }

    pub(super) fn register_chat_command(
        self,
        name: &[u8],
        callback: unsafe extern "cdecl" fn(*const i8),
    ) -> Result<(), DirectClientError> {
        if name.is_empty() || name.len() > 32 || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let count = unsafe { read_unaligned::<i32>(input as usize + INPUT_COMMAND_COUNT_OFFSET) }
            .filter(|count| (0..MAX_CHAT_COMMANDS as i32).contains(count))
            .ok_or(DirectClientError::NotReady)?;
        let _ = count;
        let mut name = name.to_vec();
        name.push(0);
        let get_handler: InputGetCommandHandlerFn =
            unsafe { mem::transmute(self.module_base + INPUT_GET_COMMAND_HANDLER_RVA) };
        if !unsafe { get_handler(input, name.as_ptr().cast()) }.is_null() {
            return Err(DirectClientError::NotReady);
        }
        let add_command: InputAddCommandFn =
            unsafe { mem::transmute(self.module_base + INPUT_ADD_COMMAND_RVA) };
        unsafe { add_command(input, name.as_ptr().cast(), callback) };
        Ok(())
    }

    pub(super) fn unregister_chat_command(self, name: &[u8]) -> Result<(), DirectClientError> {
        if name.is_empty() || name.len() > 32 || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)? as *mut u8;
        let count = unsafe { read_unaligned::<i32>(input as usize + INPUT_COMMAND_COUNT_OFFSET) }
            .filter(|count| (1..=MAX_CHAT_COMMANDS as i32).contains(count))
            .ok_or(DirectClientError::NotReady)? as usize;
        let proc_base = unsafe { input.add(INPUT_COMMAND_PROC_OFFSET) };
        let name_base = unsafe { input.add(INPUT_COMMAND_NAME_OFFSET) };
        let count_field = unsafe { input.add(INPUT_COMMAND_COUNT_OFFSET) };
        if !writable_range(proc_base, count * mem::size_of::<usize>())
            || !writable_range(name_base, count * INPUT_COMMAND_NAME_CAPACITY)
            || !writable_range(count_field, mem::size_of::<i32>())
        {
            return Err(DirectClientError::NotReady);
        }
        let Some(index) = (0..count).find(|index| {
            unsafe {
                bounded_c_string(
                    name_base.add(index * INPUT_COMMAND_NAME_CAPACITY),
                    INPUT_COMMAND_NAME_CAPACITY,
                )
            }
            .as_deref()
                == Some(name)
        }) else {
            return Err(DirectClientError::NotReady);
        };
        let remaining = count - index - 1;
        if remaining != 0 {
            unsafe {
                std::ptr::copy(
                    name_base.add((index + 1) * INPUT_COMMAND_NAME_CAPACITY),
                    name_base.add(index * INPUT_COMMAND_NAME_CAPACITY),
                    remaining * INPUT_COMMAND_NAME_CAPACITY,
                );
                std::ptr::copy(
                    proc_base.add((index + 1) * mem::size_of::<usize>()),
                    proc_base.add(index * mem::size_of::<usize>()),
                    remaining * mem::size_of::<usize>(),
                );
            }
        }
        let last = count - 1;
        unsafe {
            std::ptr::write_bytes(
                name_base.add(last * INPUT_COMMAND_NAME_CAPACITY),
                0,
                INPUT_COMMAND_NAME_CAPACITY,
            );
            std::ptr::write_bytes(
                proc_base.add(last * mem::size_of::<usize>()),
                0,
                mem::size_of::<usize>(),
            );
            std::ptr::write_unaligned(count_field.cast::<i32>(), (count - 1) as i32);
        }
        Ok(())
    }

    /// Updates the R3 chat edit box through its pinned DXUT method.
    pub(super) fn set_chat_input_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        if text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let editbox: *mut c_void = unsafe { read_pointer(input as usize + INPUT_EDIT_BOX_OFFSET) }
            .filter(|editbox| !editbox.is_null())
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if !readable_range(editbox.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let mut text = text.to_vec();
        text.push(0);
        let set_text: DxutEditBoxSetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_SET_TEXT_RVA) };
        unsafe { set_text(editbox, text.as_ptr().cast(), false) };
        Ok(())
    }

    /// Replaces the R3 chat-input text and dispatches its native command path.
    pub(super) fn process_chat_input(self, text: &[u8]) -> Result<(), DirectClientError> {
        self.set_chat_input_text(text)?;
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let process: InputNoArgFn = unsafe { mem::transmute(self.module_base + INPUT_PROCESS_RVA) };
        unsafe { process(input) };
        Ok(())
    }

    /// Copies the bounded R3-1 native chat-command names on the game thread.
    pub(super) fn chat_input_commands(self) -> Result<Vec<Vec<u8>>, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)? as *const u8;
        let count = unsafe { read_unaligned::<i32>(input as usize + INPUT_COMMAND_COUNT_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if !(0..=MAX_CHAT_COMMANDS as i32).contains(&count) {
            return Err(DirectClientError::NotReady);
        }
        let count = count as usize;
        let names = (input as usize)
            .checked_add(INPUT_COMMAND_NAME_OFFSET)
            .ok_or(DirectClientError::NotReady)?;
        let names_length = count
            .checked_mul(INPUT_COMMAND_NAME_CAPACITY)
            .ok_or(DirectClientError::NotReady)?;
        if names_length != 0 && !readable_range(names as *const u8, names_length) {
            return Err(DirectClientError::NotReady);
        }

        let mut commands = Vec::with_capacity(count);
        for index in 0..count {
            let address = names
                .checked_add(index * INPUT_COMMAND_NAME_CAPACITY)
                .ok_or(DirectClientError::NotReady)?;
            let name =
                unsafe { bounded_c_string(address as *const u8, INPUT_COMMAND_NAME_CAPACITY) }
                    .ok_or(DirectClientError::NotReady)?;
            commands.push(name);
        }
        Ok(commands)
    }

    /// Copies the R3-1 chat-input editbox text on the game thread.
    pub(super) fn chat_input_text(self) -> Result<Vec<u8>, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe { read_pointer(input as usize + INPUT_EDIT_BOX_OFFSET) }
            .filter(|editbox| !editbox.is_null())
            .ok_or(DirectClientError::NotReady)?;
        let get_text: DxutEditBoxGetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_GET_TEXT_RVA) };
        copy_chat_input_text(editbox.cast(), get_text)
    }

    /// Copies the R3-1 chat display mode through the native accessor.
    pub(super) fn chat_display_mode(self) -> Result<i32, DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let get_mode: ChatGetModeFn =
            unsafe { mem::transmute(self.module_base + CHAT_GET_MODE_RVA) };
        copy_chat_display_mode(chat, get_mode)
    }

    /// Writes one of R3's established `SCChat::m_nMode` values on the game thread.
    pub(super) fn set_chat_display_mode(self, mode: i32) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=2) {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (chat as *mut u8)
                .add(CHAT_DISPLAY_MODE_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { std::ptr::write_unaligned(field, mode) };
        Ok(())
    }

    /// Replaces one bounded R3 chat-history entry on the game thread.
    pub(super) fn set_chat_entry(
        self,
        id: u16,
        text: &[u8],
        prefix: &[u8],
        text_colour: u32,
        prefix_colour: u32,
    ) -> Result<(), DirectClientError> {
        if id >= MAX_CHAT_ENTRIES
            || text.len() >= CHAT_ENTRY_TEXT_CAPACITY
            || prefix.len() >= CHAT_ENTRY_PREFIX_CAPACITY
            || text.contains(&0)
            || prefix.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)? as *mut u8;
        let entry = unsafe { chat.add(CHAT_ENTRIES_OFFSET + usize::from(id) * CHAT_ENTRY_SIZE) };
        if !writable_range(entry, CHAT_ENTRY_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_bytes(
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                0,
                CHAT_ENTRY_PREFIX_CAPACITY,
            );
            std::ptr::write_bytes(
                entry.add(CHAT_ENTRY_TEXT_OFFSET),
                0,
                CHAT_ENTRY_TEXT_CAPACITY,
            );
            std::ptr::copy_nonoverlapping(
                prefix.as_ptr(),
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                prefix.len(),
            );
            std::ptr::copy_nonoverlapping(
                text.as_ptr(),
                entry.add(CHAT_ENTRY_TEXT_OFFSET),
                text.len(),
            );
            std::ptr::write_unaligned(
                entry.add(CHAT_ENTRY_TEXT_COLOUR_OFFSET).cast::<u32>(),
                text_colour,
            );
            std::ptr::write_unaligned(
                entry.add(CHAT_ENTRY_PREFIX_COLOUR_OFFSET).cast::<u32>(),
                prefix_colour,
            );
        }
        Ok(())
    }

    /// Copies one bounded R3 chat-history entry on the game thread.
    pub(super) fn chat_entry(self, id: u16) -> Result<ChatEntrySnapshot, DirectClientError> {
        if id >= MAX_CHAT_ENTRIES {
            return Err(DirectClientError::NotReady);
        }
        let chat = self.chat().ok_or(DirectClientError::NotReady)? as *const u8;
        let entry = unsafe { chat.add(CHAT_ENTRIES_OFFSET + usize::from(id) * CHAT_ENTRY_SIZE) };
        if !readable_range(entry, CHAT_ENTRY_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let prefix = unsafe {
            bounded_c_string(
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                CHAT_ENTRY_PREFIX_CAPACITY,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            bounded_c_string(entry.add(CHAT_ENTRY_TEXT_OFFSET), CHAT_ENTRY_TEXT_CAPACITY)
        }
        .ok_or(DirectClientError::NotReady)?;
        let text_colour =
            unsafe { read_unaligned::<u32>(entry.add(CHAT_ENTRY_TEXT_COLOUR_OFFSET) as usize) }
                .ok_or(DirectClientError::NotReady)?;
        let prefix_colour =
            unsafe { read_unaligned::<u32>(entry.add(CHAT_ENTRY_PREFIX_COLOUR_OFFSET) as usize) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(ChatEntrySnapshot {
            id,
            text,
            prefix,
            text_colour,
            prefix_colour,
        })
    }

    /// Copies the R3-1 dialog active flag without reading dialog controls.
    pub(super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_ACTIVE_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Copies the R3-1 scoreboard enabled flag without invoking UI methods.
    pub(super) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(scoreboard as usize + SCOREBOARD_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Writes the fixture-verified R3 scoreboard enabled flag on the game thread.
    pub(super) fn set_scoreboard_open(self, open: bool) -> Result<(), DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (scoreboard as *mut u8)
                .add(SCOREBOARD_ENABLED_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { std::ptr::write_unaligned(field, i32::from(open)) };
        Ok(())
    }

    /// Copies the R3-1 cursor mode from the guarded `CGame` scalar.
    pub(super) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe { read_unaligned::<i32>(game as usize + GAME_CURSOR_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    /// Applies the R3 `CGame::SetCursorMode` transition on the game thread.
    pub(super) fn set_cursor_mode(self, mode: i32) -> Result<(), DirectClientError> {
        if !matches!(mode, 0..=4) {
            return Err(DirectClientError::NotReady);
        }
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let set_cursor_mode: GameSetCursorModeFn =
            unsafe { mem::transmute(self.module_base + GAME_SET_CURSOR_MODE_RVA) };
        unsafe { set_cursor_mode(game, mode, i32::from(mode != 0)) };
        Ok(())
    }

    /// Mirrors the established cursor-toggle behaviour and re-enables input
    /// after hiding the R3 cursor.
    pub(super) fn toggle_cursor(self, show: bool) -> Result<(), DirectClientError> {
        self.set_cursor_mode(if show { 3 } else { 0 })?;
        if !show {
            let game = self.game().ok_or(DirectClientError::NotReady)?;
            let process_input_enabling: GameProcessInputEnablingFn =
                unsafe { mem::transmute(self.module_base + GAME_PROCESS_INPUT_ENABLING_RVA) };
            unsafe { process_input_enabling(game) };
        }
        Ok(())
    }

    pub(super) fn player_pool(self) -> Result<*mut c_void, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_pool(net_game) };
        if pool.is_null()
            || !readable_range(
                pool.cast(),
                PLAYER_POOL_LOCAL_ID_OFFSET + mem::size_of::<u16>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        Ok(pool)
    }

    pub(super) fn vehicle_pool(self) -> Result<*mut c_void, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_VEHICLE_POOL_RVA) };
        let pool = unsafe { get_pool(net_game) };
        (!pool.is_null() && readable_range(pool.cast(), 1))
            .then_some(pool)
            .ok_or(DirectClientError::NotReady)
    }

    fn local_player_address(self) -> Result<*mut c_void, DirectClientError> {
        let pool = self.player_pool()?;
        let get_local: PlayerPoolGetLocalPlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
        let local = unsafe { get_local(pool) };
        (!local.is_null() && readable_range(local.cast(), LOCAL_PLAYER_LAST_ANY_UPDATE_OFFSET + 4))
            .then_some(local)
            .ok_or(DirectClientError::NotReady)
    }

    fn reset_last_any_update(self, local: *mut c_void) -> Result<(), DirectClientError> {
        let field = (local as usize + LOCAL_PLAYER_LAST_ANY_UPDATE_OFFSET) as *mut u32;
        if !writable_range(field.cast(), mem::size_of::<u32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { std::ptr::write_unaligned(field, 0) };
        Ok(())
    }

    fn force_no_arg_sync(
        self,
        rva: usize,
        reset_last_update: bool,
    ) -> Result<(), DirectClientError> {
        let local = self.local_player_address()?;
        if reset_last_update {
            self.reset_last_any_update(local)?;
        }
        let send: LocalPlayerNoArgFn = unsafe { mem::transmute(self.module_base + rva) };
        unsafe { send(local) };
        Ok(())
    }

    pub(super) fn text_label_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.label_pool()?;
        let flag = LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>();
        if !readable_range(
            (pool as *const u8).wrapping_add(flag),
            mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        match unsafe { read_unaligned::<i32>(pool + flag) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    pub(super) fn vehicle_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_VEHICLE_POOL_RVA) };
        let pool = unsafe { get_pool(net_game) };
        let required = VEHICLE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * 4;
        if pool.is_null() || !readable_range(pool.cast(), required) {
            return Err(DirectClientError::NotReady);
        }
        let exists: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + VEHICLE_POOL_DOES_EXIST_RVA) };
        match unsafe { exists(pool, id) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    pub(super) fn object_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|p| *p != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(pools as *const u8, NET_GAME_POOLS_OBJECT_POOL_OFFSET + 4) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|p| *p != 0)
            .ok_or(DirectClientError::NotReady)?;
        let offset = OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * 4;
        if !readable_range(pool as *const u8, offset + 4) {
            return Err(DirectClientError::NotReady);
        }
        match unsafe { read_unaligned::<i32>(pool + offset) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Converts one R3 object-pool entry's pinned GTA entity handle to GTAREF.
    pub(super) fn object_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        let required_pool_size = NET_GAME_POOLS_OBJECT_POOL_OFFSET + mem::size_of::<usize>();
        if !readable_range(pools as *const u8, required_pool_size) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flag = OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>();
        let object = OBJECT_POOL_OBJECTS_OFFSET + usize::from(id) * mem::size_of::<usize>();
        if !readable_range(
            pool as *const u8,
            (flag + 4).max(object + mem::size_of::<usize>()),
        ) {
            return Err(DirectClientError::NotReady);
        }
        if unsafe { read_unaligned::<i32>(pool + flag) } != Some(1) {
            return Ok(None);
        }
        let object = unsafe { read_unaligned::<usize>(pool + object) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            object as *const u8,
            ENTITY_HANDLE_OFFSET + mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        Ok(
            unsafe { read_unaligned::<i32>(object + ENTITY_HANDLE_OFFSET) }
                .filter(|handle| *handle != 0),
        )
    }

    pub(super) fn object_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_OBJECTS {
            if self.object_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies the GTAREF maintained by the R3 pickup pool.
    pub(super) fn pickup_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        const MAX_SAMP_PICKUPS: u16 = 4096;
        if id >= MAX_SAMP_PICKUPS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_PICKUP_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_PICKUP_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = PICKUP_POOL_HANDLES_OFFSET + usize::from(id) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, field + mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        Ok(unsafe { read_unaligned::<i32>(pool + field) }.filter(|handle| *handle != 0))
    }

    pub(super) fn pickup_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        for id in 0..4096 {
            if self.pickup_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Converts the R3 vehicle's verified GTA object pointer to GTAREF.
    pub(super) fn vehicle_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if !self.vehicle_exists(id)? {
            return Ok(None);
        }
        let pool = self.vehicle_pool()? as usize;
        let field = VEHICLE_POOL_GAME_OBJECTS_OFFSET + usize::from(id) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, field + mem::size_of::<usize>()) {
            return Err(DirectClientError::NotReady);
        }
        let game_object = unsafe { read_unaligned::<usize>(pool + field) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(game_object as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_ref: CpoolRefFn = unsafe { mem::transmute(CPOOLS_GET_VEHICLE_REF) };
        let handle = unsafe { get_ref(game_object as *mut c_void) };
        Ok((handle != 0).then_some(handle))
    }

    pub(super) fn vehicle_id_by_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_VEHICLES {
            if self.vehicle_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Converts the local or remote R3 player's GTA ped pointer to GTAREF.
    pub(super) fn player_ped_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|id| *id < MAX_SAMP_PLAYERS)
                .ok_or(DirectClientError::NotReady)?;
        let game_ped = if id == local_id {
            let get_local: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local(pool) };
            if local.is_null() || !readable_range(local.cast(), mem::size_of::<usize>()) {
                return Err(DirectClientError::NotReady);
            }
            let ped = unsafe { read_unaligned::<usize>(local as usize) }
                .filter(|ped| *ped != 0)
                .ok_or(DirectClientError::NotReady)?;
            if !readable_range(
                ped as *const u8,
                SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
            ) {
                return Err(DirectClientError::NotReady);
            }
            unsafe { read_unaligned::<usize>(ped + SAMP_PED_GAME_PED_OFFSET) }
        } else {
            let is_connected: PlayerPoolPlayerBooleanFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
            if unsafe { is_connected(pool, id) } != 1 {
                return Ok(None);
            }
            let get_remote: PlayerPoolGetRemotePlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
            let remote = unsafe { get_remote(pool, id) };
            if remote.is_null() || !readable_range(remote.cast(), mem::size_of::<usize>()) {
                return Err(DirectClientError::NotReady);
            }
            let ped = unsafe { read_unaligned::<usize>(remote as usize) }
                .filter(|ped| *ped != 0)
                .ok_or(DirectClientError::NotReady)?;
            if !readable_range(
                ped as *const u8,
                SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
            ) {
                return Err(DirectClientError::NotReady);
            }
            unsafe { read_unaligned::<usize>(ped + SAMP_PED_GAME_PED_OFFSET) }
        }
        .filter(|ped| *ped != 0)
        .ok_or(DirectClientError::NotReady)?;
        if !readable_range(game_ped as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_ref: CpoolRefFn = unsafe { mem::transmute(CPOOLS_GET_PED_REF) };
        let handle = unsafe { get_ref(game_ped as *mut c_void) };
        Ok((handle != 0).then_some(handle))
    }

    pub(super) fn player_id_by_ped_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        let pool = self.player_pool()?;
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .filter(|id| *id < MAX_SAMP_PLAYERS)
                .ok_or(DirectClientError::NotReady)?;
        if self.player_ped_handle(local_id)? == Some(handle) {
            return Ok(Some(local_id));
        }
        for id in 0..MAX_SAMP_PLAYERS {
            if id != local_id && self.player_ped_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    pub(super) fn gangzone(self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_GANGZONES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|p| *p != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_GANGZONE_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_GANGZONE_POOL_OFFSET) }
            .filter(|p| *p != 0)
            .ok_or(DirectClientError::NotReady)?;
        let occupied_offset = GANGZONE_POOL_NOT_EMPTY_OFFSET + usize::from(id) * 4;
        if !readable_range(pool as *const u8, occupied_offset + 4) {
            return Err(DirectClientError::NotReady);
        }
        match unsafe { read_unaligned::<i32>(pool + occupied_offset) } {
            Some(0) => return Ok(None),
            Some(1) => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let gangzone =
            unsafe { read_unaligned::<usize>(pool + usize::from(id) * mem::size_of::<usize>()) }
                .filter(|p| *p != 0)
                .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            gangzone as *const u8,
            GANGZONE_ALTERNATE_COLOUR_OFFSET + mem::size_of::<u32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let left = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_LEFT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let bottom = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_BOTTOM_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let right = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_RIGHT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let top = unsafe { read_unaligned::<f32>(gangzone + GANGZONE_TOP_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe { read_unaligned::<u32>(gangzone + GANGZONE_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let alternate_colour =
            unsafe { read_unaligned::<u32>(gangzone + GANGZONE_ALTERNATE_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(GangzoneSnapshot {
            id,
            left,
            bottom,
            right,
            top,
            colour,
            alternate_colour,
        }))
    }

    pub(super) fn first_free_text_label_id(self) -> Result<u16, DirectClientError> {
        let pool = self.label_pool()?;
        let end =
            LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(MAX_SAMP_TEXT_LABELS) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, end) {
            return Err(DirectClientError::NotReady);
        }
        for id in 0..usize::from(MAX_SAMP_TEXT_LABELS) {
            match unsafe { read_unaligned::<i32>(pool + LABEL_POOL_NOT_EMPTY_OFFSET + id * 4) } {
                Some(0) => return Ok(id as u16),
                Some(1) => {}
                _ => return Err(DirectClientError::NotReady),
            }
        }
        Err(DirectClientError::NotReady)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn create_text_label(
        self,
        id: u16,
        text: &[u8],
        colour: u32,
        position: Vector3,
        draw_distance: f32,
        behind_walls: bool,
        attached_player_id: u16,
        attached_vehicle_id: u16,
    ) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS
            || text.len() > MAX_TEXT_LABEL_TEXT_BYTES
            || text.contains(&0)
            || !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !draw_distance.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.label_pool()? as *mut c_void;
        let mut text = text.to_vec();
        text.push(0);
        let create: LabelPoolCreateFn =
            unsafe { mem::transmute(self.module_base + LABEL_POOL_CREATE_RVA) };
        unsafe {
            create(
                pool,
                id,
                text.as_ptr(),
                argb_to_native_rgba(colour),
                position.into(),
                draw_distance,
                u8::from(behind_walls),
                attached_player_id,
                attached_vehicle_id,
            )
        };
        Ok(())
    }

    pub(super) fn delete_text_label(self, id: u16) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.label_pool()? as *mut c_void;
        let delete: LabelPoolDeleteFn =
            unsafe { mem::transmute(self.module_base + LABEL_POOL_DELETE_RVA) };
        (unsafe { delete(pool, id) } != 0)
            .then_some(())
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn text_label(
        self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        if !self.text_label_exists(id)? {
            return Ok(None);
        }
        let pool = self.label_pool()?;
        let label = pool + usize::from(id) * LABEL_SIZE;
        if !readable_range(label as *const u8, LABEL_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let text = unsafe { read_unaligned::<usize>(label + LABEL_TEXT_OFFSET) }
            .filter(|text| *text != 0)
            .and_then(|text| unsafe {
                bounded_c_string(text as *const u8, MAX_TEXT_LABEL_TEXT_BYTES + 1)
            })
            .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(label + LABEL_POSITION_OFFSET) }
            .filter(|p| p.x.is_finite() && p.y.is_finite() && p.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let draw_distance = unsafe { read_unaligned::<f32>(label + LABEL_DRAW_DISTANCE_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let behind_walls = match unsafe { read_unaligned::<u8>(label + LABEL_BEHIND_WALLS_OFFSET) }
        {
            Some(0) => false,
            Some(1) => true,
            _ => return Err(DirectClientError::NotReady),
        };
        let attached_player =
            unsafe { read_unaligned::<u16>(label + LABEL_ATTACHED_PLAYER_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let attached_vehicle =
            unsafe { read_unaligned::<u16>(label + LABEL_ATTACHED_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextLabelSnapshot {
            id,
            text,
            colour: unsafe { read_unaligned::<u32>(label + LABEL_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            position,
            draw_distance,
            behind_walls,
            attached_player_id: (attached_player != u16::MAX).then_some(attached_player),
            attached_vehicle_id: (attached_vehicle != u16::MAX).then_some(attached_vehicle),
        }))
    }

    fn label_pool(self) -> Result<usize, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        readable_range(pool as *const u8, 1)
            .then_some(pool)
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn textdraw_exists(self, id: u16) -> Result<bool, DirectClientError> {
        let (pool, _) = self.textdraw_pool_slot(id)?;
        match unsafe { read_unaligned::<i32>(pool + usize::from(id) * mem::size_of::<i32>()) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    pub(super) fn delete_textdraw(self, id: u16) -> Result<(), DirectClientError> {
        let (pool, _) = self.textdraw_pool_slot(id)?;
        let delete: TextdrawPoolDeleteFn =
            unsafe { mem::transmute(self.module_base + TEXTDRAW_DELETE_RVA) };
        unsafe { delete((pool as *mut u8).cast(), id) };
        Ok(())
    }

    pub(super) fn create_textdraw(
        self,
        id: u16,
        text: &[u8],
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if text.len() > MAX_TEXTDRAW_CREATE_TEXT_BYTES
            || text.contains(&0)
            || !x.is_finite()
            || !y.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let (pool, slot) = self.textdraw_pool_slot(id)?;
        if matches!(
            unsafe { read_unaligned::<i32>(pool + usize::from(id) * 4) },
            Some(1)
        ) || unsafe { read_unaligned::<usize>(slot) }.unwrap_or(usize::MAX) != 0
        {
            return Err(DirectClientError::NotReady);
        }
        let mut transmit = [0_u8; TEXTDRAW_TRANSMIT_SIZE];
        transmit[TEXTDRAW_TRANSMIT_X_OFFSET..TEXTDRAW_TRANSMIT_X_OFFSET + 4]
            .copy_from_slice(&x.to_le_bytes());
        transmit[TEXTDRAW_TRANSMIT_Y_OFFSET..TEXTDRAW_TRANSMIT_Y_OFFSET + 4]
            .copy_from_slice(&y.to_le_bytes());
        let mut text = text.to_vec();
        text.push(0);
        let create: TextdrawPoolCreateFn =
            unsafe { mem::transmute(self.module_base + TEXTDRAW_CREATE_RVA) };
        (!unsafe {
            create(
                (pool as *mut u8).cast(),
                i32::from(id),
                transmit.as_mut_ptr().cast(),
                text.as_ptr(),
            )
        }
        .is_null())
        .then_some(())
        .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn set_textdraw_position(
        self,
        id: u16,
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let object = self.textdraw_object(id)?;
        let field = (object + TEXTDRAW_X_OFFSET) as *mut f32;
        if !writable_range(field.cast(), 8) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_unaligned(field, x);
            std::ptr::write_unaligned(field.add(1), y);
        }
        Ok(())
    }

    pub(super) fn set_textdraw_style(self, id: u16, style: i32) -> Result<(), DirectClientError> {
        if !(0..=5).contains(&style) {
            return Err(DirectClientError::NotReady);
        }
        self.write_textdraw(id, TEXTDRAW_STYLE_OFFSET, &style.to_le_bytes())
    }

    pub(super) fn set_textdraw_letter_style(
        self,
        id: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let mut bytes = Vec::with_capacity(12);
        bytes.extend_from_slice(&width.to_le_bytes());
        bytes.extend_from_slice(&height.to_le_bytes());
        bytes.extend_from_slice(&colour.to_le_bytes());
        self.write_textdraw(id, TEXTDRAW_LETTER_WIDTH_OFFSET, &bytes)
    }

    pub(super) fn set_textdraw_proportional(
        self,
        id: u16,
        proportional: bool,
    ) -> Result<(), DirectClientError> {
        self.write_textdraw(id, TEXTDRAW_PROPORTIONAL_OFFSET, &[u8::from(proportional)])
    }
    pub(super) fn set_textdraw_shadow(
        self,
        id: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        let mut bytes = colour.to_le_bytes().to_vec();
        bytes.resize(
            TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_BACKGROUND_COLOUR_OFFSET,
            0,
        );
        bytes.push(shadow);
        self.write_textdraw(id, TEXTDRAW_BACKGROUND_COLOUR_OFFSET, &bytes)
    }
    pub(super) fn set_textdraw_outline(
        self,
        id: u16,
        outline: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        let object = self.textdraw_object(id)?;
        let background = (object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) as *mut u32;
        let outline_field = (object + TEXTDRAW_OUTLINE_OFFSET) as *mut u8;
        if !writable_range(background.cast(), 4) || !writable_range(outline_field, 1) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_unaligned(background, colour);
            std::ptr::write_unaligned(outline_field, outline);
        }
        Ok(())
    }

    pub(super) fn set_textdraw_box(
        self,
        id: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<(), DirectClientError> {
        if !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let object = self.textdraw_object(id)?;
        let field = (object + TEXTDRAW_BOX_ENABLED_OFFSET) as *mut u8;
        let len = TEXTDRAW_BOX_COLOUR_OFFSET + 4 - TEXTDRAW_BOX_ENABLED_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_unaligned(field, u8::from(enabled));
            std::ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<f32>(),
                width,
            );
            std::ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<f32>(),
                height,
            );
            std::ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<u32>(),
                colour,
            );
        }
        Ok(())
    }

    pub(super) fn set_textdraw_alignment(
        self,
        id: u16,
        alignment: u8,
    ) -> Result<(), DirectClientError> {
        if !(1..=3).contains(&alignment) {
            return Err(DirectClientError::NotReady);
        }
        let object = self.textdraw_object(id)?;
        let field = (object + TEXTDRAW_ALIGN_CENTER_OFFSET) as *mut u8;
        if !writable_range(
            field,
            TEXTDRAW_ALIGN_RIGHT_OFFSET + 1 - TEXTDRAW_ALIGN_CENTER_OFFSET,
        ) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_unaligned(field, u8::from(alignment == 2));
            std::ptr::write_unaligned(
                field.add(TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_ALIGN_CENTER_OFFSET),
                u8::from(alignment == 1),
            );
            std::ptr::write_unaligned(
                field.add(TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_ALIGN_CENTER_OFFSET),
                u8::from(alignment == 3),
            );
        }
        Ok(())
    }

    pub(super) fn set_textdraw_model_style(
        self,
        id: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<(), DirectClientError> {
        const ROTATION: usize = TEXTDRAW_DATA_OFFSET + 0x47;
        const ZOOM: usize = TEXTDRAW_DATA_OFFSET + 0x53;
        const COLOUR1: usize = TEXTDRAW_DATA_OFFSET + 0x57;
        const COLOUR2: usize = TEXTDRAW_DATA_OFFSET + 0x59;
        if !rotation.x.is_finite()
            || !rotation.y.is_finite()
            || !rotation.z.is_finite()
            || !zoom.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let object = self.textdraw_object(id)?;
        let field = (object + ROTATION) as *mut u8;
        if !writable_range(field, COLOUR2 + 2 - ROTATION) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::write_unaligned(field.cast::<f32>(), rotation.x);
            std::ptr::write_unaligned(field.add(4).cast::<f32>(), rotation.y);
            std::ptr::write_unaligned(field.add(8).cast::<f32>(), rotation.z);
            std::ptr::write_unaligned(field.add(ZOOM - ROTATION).cast::<f32>(), zoom);
            std::ptr::write_unaligned(field.add(COLOUR1 - ROTATION).cast::<u16>(), colour1);
            std::ptr::write_unaligned(field.add(COLOUR2 - ROTATION).cast::<u16>(), colour2);
        }
        Ok(())
    }

    pub(super) fn set_textdraw_string(self, id: u16, text: &[u8]) -> Result<(), DirectClientError> {
        if text.len() > MAX_TEXTDRAW_CREATE_TEXT_BYTES || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let object = self.textdraw_object(id)? as *mut c_void;
        let mut text = text.to_vec();
        text.push(0);
        let set_text: TextdrawSetTextFn =
            unsafe { mem::transmute(self.module_base + TEXTDRAW_SET_TEXT_RVA) };
        unsafe { set_text(object, text.as_ptr()) };
        Ok(())
    }

    pub(super) fn textdraw(self, id: u16) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        if !self.textdraw_exists(id)? {
            return Ok(None);
        }
        let object = self.textdraw_object(id)?;
        if !readable_range(object as *const u8, TEXTDRAW_MODEL_COLOUR2_OFFSET + 2) {
            return Err(DirectClientError::NotReady);
        }
        let f = |offset| {
            unsafe { read_unaligned::<f32>(object + offset) }
                .filter(|v| v.is_finite())
                .ok_or(DirectClientError::NotReady)
        };
        let b = |offset| match unsafe { read_unaligned::<u8>(object + offset) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        };
        let rotation = unsafe { read_vector3(object + TEXTDRAW_ROTATION_OFFSET) }
            .filter(|v| v.x.is_finite() && v.y.is_finite() && v.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            bounded_c_string(
                (object + TEXTDRAW_STRING_OFFSET) as *const u8,
                MAX_TEXTDRAW_STRING_BYTES + 1,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextdrawSnapshot {
            pool_index: id,
            text,
            letter_width: f(TEXTDRAW_LETTER_WIDTH_OFFSET)?,
            letter_height: f(TEXTDRAW_LETTER_WIDTH_OFFSET + 4)?,
            letter_colour: unsafe {
                read_unaligned::<u32>(object + TEXTDRAW_LETTER_WIDTH_OFFSET + 8)
            }
            .ok_or(DirectClientError::NotReady)?,
            x: f(TEXTDRAW_X_OFFSET)?,
            y: f(TEXTDRAW_Y_OFFSET)?,
            shadow: unsafe { read_unaligned::<u8>(object + TEXTDRAW_SHADOW_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            outline: unsafe { read_unaligned::<u8>(object + TEXTDRAW_OUTLINE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            background_colour: unsafe {
                read_unaligned::<u32>(object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET)
            }
            .ok_or(DirectClientError::NotReady)?,
            style: unsafe { read_unaligned::<i32>(object + TEXTDRAW_STYLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            proportional: b(TEXTDRAW_PROPORTIONAL_OFFSET)?,
            align_left: b(TEXTDRAW_ALIGN_LEFT_OFFSET)?,
            align_center: b(TEXTDRAW_ALIGN_CENTER_OFFSET)?,
            align_right: b(TEXTDRAW_ALIGN_RIGHT_OFFSET)?,
            box_enabled: b(TEXTDRAW_BOX_ENABLED_OFFSET)?,
            box_width: f(TEXTDRAW_BOX_WIDTH_OFFSET)?,
            box_height: f(TEXTDRAW_BOX_HEIGHT_OFFSET)?,
            box_colour: unsafe { read_unaligned::<u32>(object + TEXTDRAW_BOX_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            model_id: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_ID_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            rotation,
            zoom: f(TEXTDRAW_ZOOM_OFFSET)?,
            model_colour1: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR1_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            model_colour2: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR2_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
        }))
    }

    fn textdraw_pool_slot(self, id: u16) -> Result<(usize, usize), DirectClientError> {
        if id >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|p| *p != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(pools as *const u8, NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + 4) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|p| *p != 0)
            .ok_or(DirectClientError::NotReady)?;
        let slot = pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(id) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, slot + mem::size_of::<usize>() - pool) {
            return Err(DirectClientError::NotReady);
        }
        Ok((pool, slot))
    }

    fn textdraw_object(self, id: u16) -> Result<usize, DirectClientError> {
        let (pool, slot) = self.textdraw_pool_slot(id)?;
        if unsafe { read_unaligned::<i32>(pool + usize::from(id) * 4) } != Some(1) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { read_unaligned::<usize>(slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)
    }
    fn write_textdraw(self, id: u16, offset: usize, bytes: &[u8]) -> Result<(), DirectClientError> {
        let field = (self.textdraw_object(id)? + offset) as *mut u8;
        if !writable_range(field, bytes.len()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), field, bytes.len());
        }
        Ok(())
    }

    fn input(self) -> Option<*mut c_void> {
        let input: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(INPUT_SINGLETON_RVA)?) }?.cast();
        (!input.is_null() && readable_range(input.cast(), INPUT_CACHE_READABLE_SIZE))
            .then_some(input)
    }

    fn dialog(self) -> Option<*mut c_void> {
        let dialog: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(DIALOG_SINGLETON_RVA)?) }?.cast();
        (!dialog.is_null() && readable_range(dialog.cast(), DIALOG_ACTIVE_READABLE_SIZE))
            .then_some(dialog)
    }

    fn scoreboard(self) -> Option<*mut c_void> {
        let scoreboard: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(SCOREBOARD_SINGLETON_RVA)?) }?
                .cast();
        (!scoreboard.is_null() && readable_range(scoreboard.cast(), SCOREBOARD_READABLE_SIZE))
            .then_some(scoreboard)
    }

    fn chat(self) -> Option<*mut c_void> {
        let chat: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(CHAT_SINGLETON_RVA)?) }?.cast();
        (!chat.is_null() && readable_range(chat.cast(), 1)).then_some(chat)
    }

    fn death_window(self) -> Option<*mut c_void> {
        let window: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(DEATH_WINDOW_SINGLETON_RVA)?) }?
                .cast();
        (!window.is_null() && readable_range(window.cast(), 1)).then_some(window)
    }

    fn game(self) -> Option<*mut c_void> {
        let game: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(GAME_SINGLETON_RVA)?) }?.cast();
        (!game.is_null() && readable_range(game.cast(), GAME_CURSOR_MODE_READABLE_SIZE))
            .then_some(game)
    }

    fn net_game(self) -> Option<*mut c_void> {
        let net_game: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(NET_GAME_SINGLETON_RVA)?) }?.cast();
        (!net_game.is_null() && readable_range(net_game.cast(), NET_GAME_SCALAR_READABLE_SIZE))
            .then_some(net_game)
    }

    /// Shows a client-side R3 dialog through `CDialog::Show` on the game thread.
    pub(super) fn show_dialog(self, request: LocalDialogRequest) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if request.title.contains(&0)
            || request.text.contains(&0)
            || request.button1.contains(&0)
            || request.button2.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let mut title = request.title;
        let mut text = request.text;
        let mut button1 = request.button1;
        let mut button2 = request.button2;
        title.push(0);
        text.push(0);
        button1.push(0);
        button2.push(0);
        let show: DialogShowFn = unsafe { mem::transmute(self.module_base + DIALOG_SHOW_RVA) };
        unsafe {
            show(
                dialog,
                i32::from(request.id),
                request.style.as_raw() as i32,
                title.as_ptr().cast(),
                text.as_ptr().cast(),
                button1.as_ptr().cast(),
                button2.as_ptr().cast(),
                0,
            );
        }
        Ok(())
    }

    /// Closes the currently active R3 dialog with one response-button selection.
    pub(super) fn close_dialog(self, button: u8) -> Result<(), DirectClientError> {
        if button > 1 {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let close: DialogCloseFn = unsafe { mem::transmute(self.module_base + DIALOG_CLOSE_RVA) };
        unsafe { close(dialog, button) };
        Ok(())
    }

    /// Copies the response state observed by the R3 `CDialog::Close` hook
    /// before the native close routine invalidates the controls.
    pub(super) fn dialog_response_on_close(
        self,
        dialog: *mut c_void,
        button: u8,
    ) -> Result<Option<LocalDialogResponseSnapshot>, DirectClientError> {
        if dialog.is_null() || self.dialog() != Some(dialog) {
            return Ok(None);
        }
        if !readable_range(dialog.cast(), DIALOG_SNAPSHOT_READABLE_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        if !self.dialog_is_active()?
            || matches!(
                unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_SERVER_SIDE_OFFSET) },
                Some(1)
            )
        {
            return Ok(None);
        }
        let Some(dialog_id) = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_ID_OFFSET) }
            .and_then(|id| u16::try_from(id).ok())
            .filter(|id| *id != 1)
        else {
            return Ok(None);
        };
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let list_item = if listbox == 0 {
            0
        } else {
            let selected = (listbox + DXUT_LISTBOX_SELECTED_OFFSET) as *const i32;
            if !readable_range(selected.cast(), mem::size_of::<i32>()) {
                return Err(DirectClientError::NotReady);
            }
            unsafe { read_unaligned::<i32>(selected as usize) }
                .ok_or(DirectClientError::NotReady)?
        };
        let input = self.dialog_editbox_text()?.unwrap_or_default();
        Ok(Some(LocalDialogResponseSnapshot {
            dialog_id,
            button,
            list_item,
            input,
        }))
    }

    /// Copies bounded R3 dialog controls and text on the game thread.
    pub(super) fn dialog_state(self) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if !readable_range(dialog.cast(), DIALOG_SNAPSHOT_READABLE_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        if !self.dialog_is_active()? {
            return Ok(None);
        }
        let style = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_TYPE_OFFSET) }
            .and_then(|style| u32::try_from(style).ok())
            .and_then(LocalDialogStyle::from_raw)
            .ok_or(DirectClientError::NotReady)?;
        let id = unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let title = unsafe {
            bounded_c_string(
                (dialog as usize + DIALOG_CAPTION_OFFSET) as *const u8,
                DIALOG_CAPTION_CAPACITY,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let server_side =
            match unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_SERVER_SIDE_OFFSET) } {
                Some(0) => false,
                Some(1) => true,
                _ => return Err(DirectClientError::NotReady),
            };
        let text = self.dialog_text()?;
        let editbox_text = self.dialog_editbox_text()?;
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let (selected_item, list_item_count, listbox_items) = if listbox == 0 {
            (None, None, Vec::new())
        } else {
            let selected = listbox + DXUT_LISTBOX_SELECTED_OFFSET;
            let count = listbox + DXUT_LISTBOX_ITEM_COUNT_OFFSET;
            if !readable_range(selected as *const u8, mem::size_of::<i32>())
                || !readable_range(count as *const u8, mem::size_of::<i32>())
            {
                return Err(DirectClientError::NotReady);
            }
            let selected_item = unsafe { read_unaligned::<i32>(selected) };
            let list_item_count = unsafe { read_unaligned::<i32>(count) }
                .filter(|count| *count >= 0)
                .ok_or(DirectClientError::NotReady)?;
            let mut items = Vec::new();
            for index in 0..usize::try_from(list_item_count)
                .map_err(|_| DirectClientError::NotReady)?
                .min(MAX_DIALOG_LISTBOX_ITEMS)
            {
                items.push(self.dialog_listbox_item_text(index)?);
            }
            (selected_item, Some(list_item_count), items)
        };
        Ok(Some(LocalDialogSnapshot {
            id,
            style,
            title,
            server_side,
            selected_item,
            list_item_count,
            text,
            editbox_text,
            listbox_items,
        }))
    }

    pub(super) fn set_dialog_selected_item(self, selected: i32) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .filter(|listbox| *listbox != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (listbox + DXUT_LISTBOX_SELECTED_OFFSET) as *mut i32;
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, selected) };
        Ok(())
    }

    fn dialog_text(self) -> Result<Vec<u8>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let text = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_TEXT_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if text == 0 {
            return Ok(Vec::new());
        }
        unsafe { bounded_c_string(text as *const u8, MAX_DIALOG_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)
    }

    fn dialog_editbox_text(self) -> Result<Option<Vec<u8>>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox: *mut c_void = unsafe { read_pointer(dialog as usize + DIALOG_EDITBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if editbox.is_null() {
            return Ok(None);
        }
        if !readable_range(editbox.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_text: DxutEditBoxGetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_GET_TEXT_RVA) };
        unsafe { bounded_c_string(get_text(editbox), MAX_DIALOG_EDITBOX_TEXT_BYTES + 1) }
            .map(Some)
            .ok_or(DirectClientError::NotReady)
    }

    fn dialog_listbox_item_text(self, index: usize) -> Result<Vec<u8>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .filter(|listbox| *listbox != 0)
            .ok_or(DirectClientError::NotReady)?;
        let items = unsafe { read_unaligned::<usize>(listbox + DXUT_LISTBOX_ITEMS_OFFSET) }
            .filter(|items| *items != 0)
            .ok_or(DirectClientError::NotReady)?;
        let item = unsafe { read_unaligned::<usize>(items + index * mem::size_of::<usize>()) }
            .filter(|item| *item != 0)
            .ok_or(DirectClientError::NotReady)?;
        unsafe { bounded_c_string(item as *const u8, DXUT_LISTBOX_ITEM_TEXT_CAPACITY) }
            .ok_or(DirectClientError::NotReady)
    }

    /// Selects whether the active R3 dialog should be treated as client-side.
    pub(super) fn set_dialog_client_side(self, client_side: bool) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (dialog as *mut u8)
                .add(DIALOG_SERVER_SIDE_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { std::ptr::write_unaligned(field, i32::from(!client_side)) };
        Ok(())
    }

    /// Replaces text in an active R3 dialog edit box using the pinned DXUT ABI.
    pub(super) fn set_dialog_editbox_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        if text.len() > 128 || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox: *mut c_void = unsafe { read_pointer(dialog as usize + DIALOG_EDITBOX_OFFSET) }
            .filter(|editbox| !editbox.is_null())
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if !readable_range(editbox.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let mut text = text.to_vec();
        text.push(0);
        let set_text: DxutEditBoxSetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_SET_TEXT_RVA) };
        unsafe { set_text(editbox, text.as_ptr().cast(), false) };
        Ok(())
    }

    pub(super) fn dialog_is_ready(self) -> bool {
        self.dialog().is_some()
    }
}

fn argb_to_native_rgba(colour: u32) -> u32 {
    colour.rotate_left(8)
}

fn parse_animation_entry(entry: &[u8]) -> Result<AnimationSnapshot, DirectClientError> {
    let length = entry
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(entry.len());
    let Some(separator) = entry[..length].iter().position(|byte| *byte == b':') else {
        return Err(DirectClientError::NotReady);
    };
    let (name, file) = (&entry[..separator], &entry[separator + 1..length]);
    if name.is_empty() || file.is_empty() || file.contains(&b':') {
        return Err(DirectClientError::NotReady);
    }
    Ok(AnimationSnapshot {
        name: name.to_vec(),
        file: file.to_vec(),
    })
}

fn copy_chat_input_text(
    editbox: *mut c_void,
    get_text: DxutEditBoxGetTextFn,
) -> Result<Vec<u8>, DirectClientError> {
    if editbox.is_null() || !readable_range(editbox.cast(), 1) {
        return Err(DirectClientError::NotReady);
    }
    unsafe { bounded_c_string(get_text(editbox), CHAT_INPUT_TEXT_CAPACITY) }
        .ok_or(DirectClientError::NotReady)
}

fn copy_chat_display_mode(
    chat: *mut c_void,
    get_mode: ChatGetModeFn,
) -> Result<i32, DirectClientError> {
    if chat.is_null() || !readable_range(chat.cast(), 1) {
        return Err(DirectClientError::NotReady);
    }
    let mode = unsafe { get_mode(chat) };
    matches!(mode, 0..=2)
        .then_some(mode)
        .ok_or(DirectClientError::NotReady)
}

fn copy_player_counts(
    pool: *mut c_void,
    get_count: PlayerPoolGetCountFn,
) -> Result<(u16, u16), DirectClientError> {
    let including_npcs = unsafe { get_count(pool, 1) };
    let excluding_npcs = unsafe { get_count(pool, 0) };
    let including_npcs = u16::try_from(including_npcs)
        .ok()
        .filter(|count| *count <= MAX_SAMP_PLAYERS)
        .ok_or(DirectClientError::NotReady)?;
    let excluding_npcs = u16::try_from(excluding_npcs)
        .ok()
        .filter(|count| *count <= including_npcs)
        .ok_or(DirectClientError::NotReady)?;
    Ok((including_npcs, excluding_npcs))
}

fn copy_player_max_id(pool: *mut c_void) -> Result<u16, DirectClientError> {
    unsafe { read_unaligned::<i32>(pool as usize + PLAYER_POOL_LARGEST_ID_OFFSET) }
        .and_then(|id| u16::try_from(id).ok())
        .filter(|id| *id < MAX_SAMP_PLAYERS)
        .ok_or(DirectClientError::NotReady)
}

// The explicitly injected native call targets keep this copy routine testable
// without fabricating a module image; grouping them would obscure the ABI.
#[allow(clippy::too_many_arguments)]
fn copy_player_info(
    pool: *mut c_void,
    id: u16,
    is_connected: PlayerPoolPlayerBooleanFn,
    get_player: PlayerPoolGetRemotePlayerFn,
    does_exist: RemotePlayerDoesExistFn,
    get_name: PlayerPoolGetNameFn,
    get_score: PlayerPoolGetPlayerStatFn,
    get_ping: PlayerPoolGetPlayerStatFn,
    get_colour: RemotePlayerGetColourArgbFn,
    get_status: RemotePlayerGetStatusFn,
) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
    if pool.is_null() || id >= MAX_SAMP_PLAYERS {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { is_connected(pool, id) } {
        0 => Ok(None),
        1 => {
            let remote = unsafe { get_player(pool, id) };
            if remote.is_null() || !readable_range(remote.cast(), 1) {
                return Err(DirectClientError::NotReady);
            }
            let defined = match unsafe { does_exist(remote) } {
                0 => false,
                1 => true,
                _ => return Err(DirectClientError::NotReady),
            };
            let object_slot = (pool as usize)
                .checked_add(PLAYER_POOL_OBJECTS_OFFSET + usize::from(id) * mem::size_of::<usize>())
                .ok_or(DirectClientError::NotReady)?;
            let info = unsafe { read_pointer(object_slot) }.ok_or(DirectClientError::NotReady)?;
            if info.is_null() || !readable_range(info.cast(), PLAYER_INFO_READABLE_SIZE) {
                return Err(DirectClientError::NotReady);
            }
            let is_npc =
                match unsafe { read_unaligned::<i32>(info as usize + PLAYER_INFO_IS_NPC_OFFSET) } {
                    Some(0) => false,
                    Some(1) => true,
                    _ => return Err(DirectClientError::NotReady),
                };
            let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
                .filter(|name| !name.is_empty())
                .ok_or(DirectClientError::NotReady)?;
            Ok(Some(PlayerInfoSnapshot {
                id,
                defined,
                paused: unsafe { get_status(remote) } == 0,
                nickname,
                is_local: false,
                is_npc,
                colour: unsafe { get_colour(remote) },
                score: unsafe { get_score(pool, id) },
                ping: (unsafe { get_ping(pool, id) }).max(0) as u32,
            }))
        }
        _ => Err(DirectClientError::NotReady),
    }
}

fn copy_remote_player_state(
    pool: *mut c_void,
    id: u16,
    is_connected: PlayerPoolPlayerBooleanFn,
    get_player: PlayerPoolGetRemotePlayerFn,
    does_exist: RemotePlayerDoesExistFn,
) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
    if pool.is_null() || id >= MAX_SAMP_PLAYERS {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { is_connected(pool, id) } {
        0 => return Ok(None),
        1 => {}
        _ => return Err(DirectClientError::NotReady),
    }
    let remote = unsafe { get_player(pool, id) };
    if remote.is_null() || !readable_range(remote.cast(), REMOTE_PLAYER_STATE_READABLE_SIZE) {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { does_exist(remote) } {
        0 => return Ok(None),
        1 => {}
        _ => return Err(DirectClientError::NotReady),
    }
    let health =
        unsafe { read_unaligned::<f32>(remote as usize + REMOTE_PLAYER_REPORTED_HEALTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
    let armour =
        unsafe { read_unaligned::<f32>(remote as usize + REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
    let special_action =
        unsafe { read_unaligned::<u8>(remote as usize + REMOTE_PLAYER_SPECIAL_ACTION_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
    let animation =
        unsafe { read_unaligned::<u32>(remote as usize + REMOTE_PLAYER_ANIMATION_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
    Ok(Some(RemotePlayerStateSnapshot {
        id,
        health,
        armour,
        special_action,
        animation_id: animation as u16,
    }))
}

fn copy_remote_player_is_streamed_out(
    pool: *mut c_void,
    id: u16,
    is_connected: PlayerPoolPlayerBooleanFn,
    get_player: PlayerPoolGetRemotePlayerFn,
    does_exist: RemotePlayerDoesExistFn,
) -> Result<Option<bool>, DirectClientError> {
    if pool.is_null() || id >= MAX_SAMP_PLAYERS {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { is_connected(pool, id) } {
        0 => return Ok(None),
        1 => {}
        _ => return Err(DirectClientError::NotReady),
    }
    let remote = unsafe { get_player(pool, id) };
    if remote.is_null() || !readable_range(remote.cast(), mem::size_of::<usize>()) {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { does_exist(remote) } {
        0 => return Ok(None),
        1 => {}
        _ => return Err(DirectClientError::NotReady),
    }
    let ped = unsafe { read_pointer(remote as usize + REMOTE_PLAYER_PED_OFFSET) }
        .ok_or(DirectClientError::NotReady)?;
    if ped.is_null() {
        return Ok(Some(true));
    }
    if !readable_range(
        ped.cast(),
        SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
    ) {
        return Err(DirectClientError::NotReady);
    }
    let game_ped = unsafe { read_pointer(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
        .ok_or(DirectClientError::NotReady)?;
    Ok(Some(game_ped.is_null()))
}

fn copy_remote_onfoot_sync(
    pool: *mut c_void,
    id: u16,
    is_connected: PlayerPoolPlayerBooleanFn,
    get_player: PlayerPoolGetRemotePlayerFn,
    does_exist: RemotePlayerDoesExistFn,
) -> Result<Option<OnFootSyncSnapshot>, DirectClientError> {
    if pool.is_null() || id >= MAX_SAMP_PLAYERS {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { is_connected(pool, id) } {
        0 => return Ok(None),
        1 => {}
        _ => return Err(DirectClientError::NotReady),
    }
    let remote = unsafe { get_player(pool, id) };
    if remote.is_null()
        || !readable_range(
            remote.cast(),
            REMOTE_PLAYER_ONFOOT_OFFSET + ONFOOT_SYNC_SIZE,
        )
    {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { does_exist(remote) } {
        0 => Ok(None),
        1 => copy_onfoot_sync(id, remote as usize + REMOTE_PLAYER_ONFOOT_OFFSET).map(Some),
        _ => Err(DirectClientError::NotReady),
    }
}

fn copy_onfoot_sync(id: u16, address: usize) -> Result<OnFootSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, ONFOOT_SYNC_SIZE) {
        return Err(DirectClientError::NotReady);
    }
    let controller_left_stick_x =
        unsafe { read_unaligned::<i16>(address + ONFOOT_CONTROLLER_LEFT_STICK_X_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
    let controller_left_stick_y =
        unsafe { read_unaligned::<i16>(address + ONFOOT_CONTROLLER_LEFT_STICK_Y_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
    let controller_buttons =
        unsafe { read_unaligned::<i16>(address + ONFOOT_CONTROLLER_BUTTONS_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
    let position = unsafe { read_vector3(address + ONFOOT_POSITION_OFFSET) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)?;
    let quaternion = [
        unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET) },
        unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET + 4) },
        unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET + 8) },
        unsafe { read_unaligned::<f32>(address + ONFOOT_QUATERNION_OFFSET + 12) },
    ];
    let quaternion = quaternion
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .filter(|values| values.iter().all(|value| value.is_finite()))
        .ok_or(DirectClientError::NotReady)?;
    let speed = unsafe { read_vector3(address + ONFOOT_SPEED_OFFSET) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)?;
    let surfing_offset = unsafe { read_vector3(address + ONFOOT_SURFING_OFFSET_OFFSET) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)?;
    Ok(OnFootSyncSnapshot {
        id,
        controller_left_stick_x,
        controller_left_stick_y,
        controller_buttons,
        position,
        quaternion: [quaternion[0], quaternion[1], quaternion[2], quaternion[3]],
        health: unsafe { read_unaligned::<u8>(address + ONFOOT_HEALTH_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        armour: unsafe { read_unaligned::<u8>(address + ONFOOT_ARMOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        weapon: unsafe { read_unaligned::<u8>(address + ONFOOT_WEAPON_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        special_action: unsafe { read_unaligned::<u8>(address + ONFOOT_SPECIAL_ACTION_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        speed,
        surfing_offset,
        surfing_vehicle_id: unsafe {
            read_unaligned::<u16>(address + ONFOOT_SURFING_VEHICLE_ID_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        animation: unsafe { read_unaligned::<u32>(address + ONFOOT_ANIMATION_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
    })
}

fn copy_remote_incar_sync(
    pool: *mut c_void,
    id: u16,
    is_connected: PlayerPoolPlayerBooleanFn,
    get_player: PlayerPoolGetRemotePlayerFn,
    does_exist: RemotePlayerDoesExistFn,
) -> Result<Option<InCarSyncSnapshot>, DirectClientError> {
    if pool.is_null() || id >= MAX_SAMP_PLAYERS {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { is_connected(pool, id) } {
        0 => return Ok(None),
        1 => {}
        _ => return Err(DirectClientError::NotReady),
    }
    let remote = unsafe { get_player(pool, id) };
    if remote.is_null()
        || !readable_range(remote.cast(), REMOTE_PLAYER_INCAR_OFFSET + INCAR_SYNC_SIZE)
    {
        return Err(DirectClientError::NotReady);
    }
    match unsafe { does_exist(remote) } {
        0 => Ok(None),
        1 => copy_incar_sync(id, remote as usize + REMOTE_PLAYER_INCAR_OFFSET).map(Some),
        _ => Err(DirectClientError::NotReady),
    }
}

fn copy_incar_sync(id: u16, address: usize) -> Result<InCarSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, INCAR_SYNC_SIZE) {
        return Err(DirectClientError::NotReady);
    }
    let quaternion = [
        unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET) },
        unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET + 4) },
        unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET + 8) },
        unsafe { read_unaligned::<f32>(address + INCAR_QUATERNION_OFFSET + 12) },
    ];
    let quaternion = quaternion
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .filter(|values| values.iter().all(|value| value.is_finite()))
        .ok_or(DirectClientError::NotReady)?;
    let position = unsafe { read_vector3(address + INCAR_POSITION_OFFSET) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)?;
    let speed = unsafe { read_vector3(address + INCAR_SPEED_OFFSET) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)?;
    let read_bool = |offset| match unsafe { read_unaligned::<u8>(address + offset) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    };
    Ok(InCarSyncSnapshot {
        id,
        vehicle_id: unsafe { read_unaligned::<u16>(address + INCAR_VEHICLE_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        controller_left_stick_x: unsafe {
            read_unaligned::<i16>(address + INCAR_CONTROLLER_LEFT_STICK_X_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        controller_left_stick_y: unsafe {
            read_unaligned::<i16>(address + INCAR_CONTROLLER_LEFT_STICK_Y_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        controller_buttons: unsafe {
            read_unaligned::<i16>(address + INCAR_CONTROLLER_BUTTONS_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        quaternion: [quaternion[0], quaternion[1], quaternion[2], quaternion[3]],
        position,
        speed,
        vehicle_health: unsafe { read_unaligned::<f32>(address + INCAR_VEHICLE_HEALTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?,
        driver_health: unsafe { read_unaligned::<u8>(address + INCAR_DRIVER_HEALTH_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        driver_armour: unsafe { read_unaligned::<u8>(address + INCAR_DRIVER_ARMOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        weapon: unsafe { read_unaligned::<u8>(address + INCAR_WEAPON_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        siren: read_bool(INCAR_SIREN_OFFSET)?,
        landing_gear: read_bool(INCAR_LANDING_GEAR_OFFSET)?,
        trailer_id: unsafe { read_unaligned::<u16>(address + INCAR_TRAILER_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        vehicle_specific: unsafe {
            read_unaligned::<[u8; 4]>(address + INCAR_VEHICLE_SPECIFIC_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
    })
}

fn copy_passenger_sync(
    id: u16,
    address: usize,
) -> Result<PassengerSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, PASSENGER_SYNC_SIZE) {
        return Err(DirectClientError::NotReady);
    }
    let position = unsafe { read_vector3(address + PASSENGER_POSITION_OFFSET) }
        .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
        .ok_or(DirectClientError::NotReady)?;
    Ok(PassengerSyncSnapshot {
        id,
        vehicle_id: unsafe { read_unaligned::<u16>(address + PASSENGER_VEHICLE_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        seat_id: unsafe { read_unaligned::<u8>(address + PASSENGER_SEAT_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        weapon: unsafe { read_unaligned::<u8>(address + PASSENGER_WEAPON_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        health: unsafe { read_unaligned::<u8>(address + PASSENGER_HEALTH_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        armour: unsafe { read_unaligned::<u8>(address + PASSENGER_ARMOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        controller_left_stick_x: unsafe {
            read_unaligned::<i16>(address + PASSENGER_CONTROLLER_LEFT_STICK_X_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        controller_left_stick_y: unsafe {
            read_unaligned::<i16>(address + PASSENGER_CONTROLLER_LEFT_STICK_Y_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        controller_buttons: unsafe {
            read_unaligned::<i16>(address + PASSENGER_CONTROLLER_BUTTONS_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        position,
    })
}

fn copy_trailer_sync(id: u16, address: usize) -> Result<TrailerSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, TRAILER_SYNC_SIZE) {
        return Err(DirectClientError::NotReady);
    }
    let vector = |offset| {
        unsafe { read_vector3(address + offset) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)
    };
    let quaternion = [
        unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET) },
        unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET + 4) },
        unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET + 8) },
        unsafe { read_unaligned::<f32>(address + TRAILER_QUATERNION_OFFSET + 12) },
    ];
    let quaternion = quaternion
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .filter(|values| values.iter().all(|value| value.is_finite()))
        .ok_or(DirectClientError::NotReady)?;
    Ok(TrailerSyncSnapshot {
        id,
        trailer_id: unsafe { read_unaligned::<u16>(address + TRAILER_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        position: vector(TRAILER_POSITION_OFFSET)?,
        quaternion: [quaternion[0], quaternion[1], quaternion[2], quaternion[3]],
        speed: vector(TRAILER_SPEED_OFFSET)?,
        turn_speed: vector(TRAILER_TURN_SPEED_OFFSET)?,
    })
}

fn copy_aim_sync(id: u16, address: usize) -> Result<AimSyncSnapshot, DirectClientError> {
    if !readable_range(address as *const u8, AIM_SYNC_SIZE) {
        return Err(DirectClientError::NotReady);
    }
    let vector = |offset| {
        unsafe { read_vector3(address + offset) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)
    };
    Ok(AimSyncSnapshot {
        id,
        camera_mode: unsafe { read_unaligned::<u8>(address + AIM_CAMERA_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
        aim_first: vector(AIM_FIRST_OFFSET)?,
        aim_position: vector(AIM_POSITION_OFFSET)?,
        aim_z: unsafe { read_unaligned::<f32>(address + AIM_Z_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?,
        zoom_and_weapon_state: unsafe {
            read_unaligned::<u8>(address + AIM_ZOOM_WEAPON_STATE_OFFSET)
        }
        .ok_or(DirectClientError::NotReady)?,
        aspect_ratio: unsafe { read_unaligned::<u8>(address + AIM_ASPECT_RATIO_OFFSET) }
            .ok_or(DirectClientError::NotReady)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn reads_verified_r3_netgame_scalars() {
        let mut module = vec![0_u8; NET_GAME_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut net_game = vec![0_u8; NET_GAME_GAME_STATE_OFFSET + std::mem::size_of::<i32>()];
        let module_base = module.as_mut_ptr() as usize;
        let net_game_pointer = net_game.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module
                    .as_mut_ptr()
                    .add(NET_GAME_SINGLETON_RVA)
                    .cast::<usize>(),
                net_game_pointer as usize,
            );
            ptr::write_unaligned(
                net_game_pointer.add(NET_GAME_PORT_OFFSET).cast::<i32>(),
                7777,
            );
            ptr::write_unaligned(
                net_game_pointer
                    .add(NET_GAME_GAME_STATE_OFFSET)
                    .cast::<i32>(),
                6,
            );
        }
        net_game[NET_GAME_HOST_ADDRESS_OFFSET..NET_GAME_HOST_ADDRESS_OFFSET + 9]
            .copy_from_slice(b"127.0.0.1");
        net_game[NET_GAME_HOSTNAME_OFFSET..NET_GAME_HOSTNAME_OFFSET + 8]
            .copy_from_slice(b"R3 probe");

        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        assert_eq!(profile.game_state(), Ok(6));
        assert_eq!(
            profile.server_info(),
            Ok(ServerInfoSnapshot {
                address: b"127.0.0.1".to_vec(),
                hostname: b"R3 probe".to_vec(),
                port: 7777,
            })
        );
    }

    #[test]
    fn maps_public_r1_game_states_to_r3_native_values() {
        let mut module = vec![0_u8; NET_GAME_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut net_game = vec![0_u8; NET_GAME_GAME_STATE_OFFSET + std::mem::size_of::<i32>()];
        let module_base = module.as_mut_ptr() as usize;
        let net_game_pointer = net_game.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module
                    .as_mut_ptr()
                    .add(NET_GAME_SINGLETON_RVA)
                    .cast::<usize>(),
                net_game_pointer as usize,
            );
        }
        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        for (public, native) in [(0, 0), (9, 1), (13, 2), (14, 5), (15, 6), (18, 11)] {
            profile.set_game_state(public).unwrap();
            assert_eq!(profile.game_state(), Ok(native));
        }
        assert_eq!(profile.set_game_state(1), Err(DirectClientError::NotReady));
    }

    #[test]
    fn rejects_other_entry_points() {
        assert!(R3ClientProfile::verify(0x10000, SAMP_R3_1_ENTRY_POINT).is_some());
        assert!(R3ClientProfile::verify(0x10000, SAMP_R3_1_ENTRY_POINT - 1).is_none());
    }

    #[test]
    fn reads_verified_r3_chat_input_cache() {
        let mut module = vec![0_u8; INPUT_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut input = vec![0_u8; INPUT_CACHE_READABLE_SIZE];
        let module_base = module.as_mut_ptr() as usize;
        let input_pointer = input.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module.as_mut_ptr().add(INPUT_SINGLETON_RVA).cast::<usize>(),
                input_pointer as usize,
            );
            ptr::write_unaligned(
                input_pointer.add(INPUT_COMMAND_COUNT_OFFSET).cast::<i32>(),
                2,
            );
            ptr::write_unaligned(input_pointer.add(INPUT_ENABLED_OFFSET).cast::<i32>(), 1);
        }
        input[INPUT_COMMAND_NAME_OFFSET..INPUT_COMMAND_NAME_OFFSET + 5].copy_from_slice(b"quit\0");
        input[INPUT_COMMAND_NAME_OFFSET + INPUT_COMMAND_NAME_CAPACITY
            ..INPUT_COMMAND_NAME_OFFSET + INPUT_COMMAND_NAME_CAPACITY + 5]
            .copy_from_slice(b"help\0");

        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        assert_eq!(profile.chat_input_is_active(), Ok(true));
        assert_eq!(
            profile.chat_input_commands(),
            Ok(vec![b"quit".to_vec(), b"help".to_vec()])
        );
    }

    #[test]
    fn reads_verified_r3_dialog_active_flag() {
        let mut module = vec![0_u8; DIALOG_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut dialog = vec![0_u8; DIALOG_ACTIVE_READABLE_SIZE];
        let module_base = module.as_mut_ptr() as usize;
        let dialog_pointer = dialog.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module
                    .as_mut_ptr()
                    .add(DIALOG_SINGLETON_RVA)
                    .cast::<usize>(),
                dialog_pointer as usize,
            );
            ptr::write_unaligned(dialog_pointer.add(DIALOG_ACTIVE_OFFSET).cast::<i32>(), 1);
        }

        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        assert_eq!(profile.dialog_is_active(), Ok(true));
    }

    #[test]
    fn reads_verified_r3_scoreboard_enabled_flag() {
        let mut module = vec![0_u8; SCOREBOARD_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut scoreboard = vec![0_u8; SCOREBOARD_READABLE_SIZE];
        let module_base = module.as_mut_ptr() as usize;
        let scoreboard_pointer = scoreboard.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module
                    .as_mut_ptr()
                    .add(SCOREBOARD_SINGLETON_RVA)
                    .cast::<usize>(),
                scoreboard_pointer as usize,
            );
            ptr::write_unaligned(
                scoreboard_pointer
                    .add(SCOREBOARD_ENABLED_OFFSET)
                    .cast::<i32>(),
                1,
            );
        }

        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        assert_eq!(profile.scoreboard_is_open(), Ok(true));
    }

    unsafe extern "thiscall" fn fake_pool_is_connected(_pool: *mut c_void, id: u16) -> i32 {
        i32::from(id == 7)
    }

    unsafe extern "thiscall" fn fake_pool_get_remote_player(
        _pool: *mut c_void,
        _id: u16,
    ) -> *mut c_void {
        static REMOTE: u8 = 0;
        (&raw const REMOTE).cast_mut().cast()
    }

    unsafe extern "thiscall" fn fake_remote_player_does_exist(_remote: *mut c_void) -> i32 {
        1
    }

    unsafe extern "thiscall" fn fake_remote_player_invalid_boolean(_remote: *mut c_void) -> i32 {
        2
    }

    unsafe extern "thiscall" fn fake_player_pool_get_name(
        _pool: *mut c_void,
        _id: u16,
    ) -> *const u8 {
        c"R3 remote".as_ptr().cast()
    }

    unsafe extern "thiscall" fn fake_player_pool_get_score(_pool: *mut c_void, _id: u16) -> i32 {
        42
    }

    unsafe extern "thiscall" fn fake_player_pool_get_ping(_pool: *mut c_void, _id: u16) -> i32 {
        125
    }

    unsafe extern "thiscall" fn fake_remote_player_get_colour(_remote: *mut c_void) -> u32 {
        0xAABB_CCDD
    }

    unsafe extern "thiscall" fn fake_remote_player_get_status(_remote: *mut c_void) -> i32 {
        0
    }

    unsafe extern "thiscall" fn fake_pool_get_remote_as_pool(
        pool: *mut c_void,
        _id: u16,
    ) -> *mut c_void {
        pool
    }

    #[test]
    fn copies_validated_r3_player_info() {
        let mut pool_memory = vec![
            0_u8;
            PLAYER_POOL_OBJECTS_OFFSET
                + (usize::from(7_u16) + 1) * mem::size_of::<usize>()
        ];
        let mut info_memory = vec![0_u8; PLAYER_INFO_READABLE_SIZE];
        let pool = pool_memory.as_mut_ptr().cast();
        unsafe {
            ptr::write_unaligned(
                pool_memory
                    .as_mut_ptr()
                    .add(PLAYER_POOL_OBJECTS_OFFSET + usize::from(7_u16) * mem::size_of::<usize>())
                    .cast::<usize>(),
                info_memory.as_mut_ptr() as usize,
            );
            ptr::write_unaligned(
                info_memory
                    .as_mut_ptr()
                    .add(PLAYER_INFO_IS_NPC_OFFSET)
                    .cast::<i32>(),
                1,
            );
        }

        assert_eq!(
            copy_player_info(
                pool,
                6,
                fake_pool_is_connected,
                fake_pool_get_remote_player,
                fake_remote_player_does_exist,
                fake_player_pool_get_name,
                fake_player_pool_get_score,
                fake_player_pool_get_ping,
                fake_remote_player_get_colour,
                fake_remote_player_get_status,
            ),
            Ok(None)
        );
        let snapshot = copy_player_info(
            pool,
            7,
            fake_pool_is_connected,
            fake_pool_get_remote_player,
            fake_remote_player_does_exist,
            fake_player_pool_get_name,
            fake_player_pool_get_score,
            fake_player_pool_get_ping,
            fake_remote_player_get_colour,
            fake_remote_player_get_status,
        )
        .unwrap()
        .unwrap();
        assert_eq!(snapshot.id, 7);
        assert!(snapshot.defined);
        assert!(snapshot.paused);
        assert_eq!(snapshot.nickname, b"R3 remote");
        assert!(snapshot.is_npc);
        assert_eq!(snapshot.colour, 0xAABB_CCDD);
        assert_eq!(snapshot.score, 42);
        assert_eq!(snapshot.ping, 125);
        assert_eq!(
            copy_player_info(
                pool,
                7,
                fake_pool_is_connected,
                fake_pool_get_remote_player,
                fake_remote_player_invalid_boolean,
                fake_player_pool_get_name,
                fake_player_pool_get_score,
                fake_player_pool_get_ping,
                fake_remote_player_get_colour,
                fake_remote_player_get_status,
            ),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn copies_validated_r3_remote_player_state() {
        let mut remote = vec![0_u8; REMOTE_PLAYER_STATE_READABLE_SIZE];
        unsafe {
            ptr::write_unaligned(
                remote
                    .as_mut_ptr()
                    .add(REMOTE_PLAYER_REPORTED_HEALTH_OFFSET)
                    .cast::<f32>(),
                87.5,
            );
            ptr::write_unaligned(
                remote
                    .as_mut_ptr()
                    .add(REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET)
                    .cast::<f32>(),
                42.0,
            );
            remote[REMOTE_PLAYER_SPECIAL_ACTION_OFFSET] = 3;
            ptr::write_unaligned(
                remote
                    .as_mut_ptr()
                    .add(REMOTE_PLAYER_ANIMATION_OFFSET)
                    .cast::<u32>(),
                0x1234_5678,
            );
        }
        assert_eq!(
            copy_remote_player_state(
                remote.as_mut_ptr().cast(),
                7,
                fake_pool_is_connected,
                fake_pool_get_remote_as_pool,
                fake_remote_player_does_exist,
            ),
            Ok(Some(RemotePlayerStateSnapshot {
                id: 7,
                health: 87.5,
                armour: 42.0,
                special_action: 3,
                animation_id: 0x5678,
            }))
        );
    }

    #[test]
    fn copies_validated_r3_streamed_out_state() {
        let mut remote = vec![0_u8; mem::size_of::<usize>()];
        let mut ped = vec![0_u8; SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>()];
        unsafe {
            ptr::write_unaligned(
                remote.as_mut_ptr().cast::<usize>(),
                ped.as_mut_ptr() as usize,
            );
            ptr::write_unaligned(
                ped.as_mut_ptr()
                    .add(SAMP_PED_GAME_PED_OFFSET)
                    .cast::<usize>(),
                0,
            );
        }
        assert_eq!(
            copy_remote_player_is_streamed_out(
                remote.as_mut_ptr().cast(),
                7,
                fake_pool_is_connected,
                fake_pool_get_remote_as_pool,
                fake_remote_player_does_exist,
            ),
            Ok(Some(true))
        );
    }

    #[test]
    fn copies_validated_r3_onfoot_sync() {
        let mut data = vec![0_u8; ONFOOT_SYNC_SIZE];
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().cast::<i16>(), -12);
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(ONFOOT_CONTROLLER_LEFT_STICK_Y_OFFSET)
                    .cast::<i16>(),
                34,
            );
            ptr::write_unaligned(
                data.as_mut_ptr().add(ONFOOT_POSITION_OFFSET).cast::<f32>(),
                1.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(ONFOOT_POSITION_OFFSET + 4)
                    .cast::<f32>(),
                2.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(ONFOOT_POSITION_OFFSET + 8)
                    .cast::<f32>(),
                3.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(ONFOOT_QUATERNION_OFFSET + 12)
                    .cast::<f32>(),
                1.0,
            );
            data[ONFOOT_HEALTH_OFFSET] = 90;
            data[ONFOOT_ARMOUR_OFFSET] = 40;
            data[ONFOOT_WEAPON_OFFSET] = 24;
            data[ONFOOT_SPECIAL_ACTION_OFFSET] = 3;
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(ONFOOT_SURFING_VEHICLE_ID_OFFSET)
                    .cast::<u16>(),
                77,
            );
            ptr::write_unaligned(
                data.as_mut_ptr().add(ONFOOT_ANIMATION_OFFSET).cast::<u32>(),
                0x1234_5678,
            );
        }
        let snapshot = copy_onfoot_sync(7, data.as_ptr() as usize).unwrap();
        assert_eq!(snapshot.id, 7);
        assert_eq!(snapshot.controller_left_stick_x, -12);
        assert_eq!(snapshot.controller_left_stick_y, 34);
        assert_eq!(snapshot.position.x, 1.0);
        assert_eq!(snapshot.position.y, 2.0);
        assert_eq!(snapshot.position.z, 3.0);
        assert_eq!(snapshot.quaternion, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(snapshot.health, 90);
        assert_eq!(snapshot.armour, 40);
        assert_eq!(snapshot.weapon, 24);
        assert_eq!(snapshot.special_action, 3);
        assert_eq!(snapshot.surfing_vehicle_id, 77);
        assert_eq!(snapshot.animation, 0x1234_5678);
    }

    #[test]
    fn copies_validated_r3_incar_sync() {
        let mut data = vec![0_u8; INCAR_SYNC_SIZE];
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().cast::<u16>(), 321);
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(INCAR_QUATERNION_OFFSET + 12)
                    .cast::<f32>(),
                1.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(INCAR_VEHICLE_HEALTH_OFFSET)
                    .cast::<f32>(),
                999.0,
            );
            data[INCAR_DRIVER_HEALTH_OFFSET] = 80;
            data[INCAR_DRIVER_ARMOUR_OFFSET] = 20;
            data[INCAR_WEAPON_OFFSET] = 31;
            data[INCAR_SIREN_OFFSET] = 1;
            ptr::write_unaligned(
                data.as_mut_ptr().add(INCAR_TRAILER_ID_OFFSET).cast::<u16>(),
                99,
            );
            data[INCAR_VEHICLE_SPECIFIC_OFFSET..INCAR_VEHICLE_SPECIFIC_OFFSET + 4]
                .copy_from_slice(&[1, 2, 3, 4]);
        }
        let snapshot = copy_incar_sync(7, data.as_ptr() as usize).unwrap();
        assert_eq!(snapshot.id, 7);
        assert_eq!(snapshot.vehicle_id, 321);
        assert_eq!(snapshot.quaternion, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(snapshot.vehicle_health, 999.0);
        assert_eq!(snapshot.driver_health, 80);
        assert_eq!(snapshot.driver_armour, 20);
        assert_eq!(snapshot.weapon, 31);
        assert!(snapshot.siren);
        assert!(!snapshot.landing_gear);
        assert_eq!(snapshot.trailer_id, 99);
        assert_eq!(snapshot.vehicle_specific, [1, 2, 3, 4]);
    }

    #[test]
    fn copies_validated_r3_passenger_sync() {
        let mut data = vec![0_u8; PASSENGER_SYNC_SIZE];
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().cast::<u16>(), 321);
            data[PASSENGER_SEAT_ID_OFFSET] = 2;
            data[PASSENGER_WEAPON_OFFSET] = 24;
            data[PASSENGER_HEALTH_OFFSET] = 80;
            data[PASSENGER_ARMOUR_OFFSET] = 40;
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(PASSENGER_POSITION_OFFSET)
                    .cast::<f32>(),
                1.0,
            );
        }
        let snapshot = copy_passenger_sync(7, data.as_ptr() as usize).unwrap();
        assert_eq!(snapshot.id, 7);
        assert_eq!(snapshot.vehicle_id, 321);
        assert_eq!(snapshot.seat_id, 2);
        assert_eq!(snapshot.weapon, 24);
        assert_eq!(snapshot.health, 80);
        assert_eq!(snapshot.armour, 40);
        assert_eq!(snapshot.position.x, 1.0);
    }

    #[test]
    fn copies_validated_r3_trailer_sync() {
        let mut data = vec![0_u8; TRAILER_SYNC_SIZE];
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().cast::<u16>(), 99);
            ptr::write_unaligned(
                data.as_mut_ptr().add(TRAILER_POSITION_OFFSET).cast::<f32>(),
                1.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(TRAILER_QUATERNION_OFFSET + 12)
                    .cast::<f32>(),
                1.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr().add(TRAILER_SPEED_OFFSET).cast::<f32>(),
                2.0,
            );
            ptr::write_unaligned(
                data.as_mut_ptr()
                    .add(TRAILER_TURN_SPEED_OFFSET)
                    .cast::<f32>(),
                3.0,
            );
        }
        let snapshot = copy_trailer_sync(7, data.as_ptr() as usize).unwrap();
        assert_eq!(snapshot.id, 7);
        assert_eq!(snapshot.trailer_id, 99);
        assert_eq!(snapshot.position.x, 1.0);
        assert_eq!(snapshot.quaternion, [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(snapshot.speed.x, 2.0);
        assert_eq!(snapshot.turn_speed.x, 3.0);
    }

    #[test]
    fn copies_validated_r3_aim_sync() {
        let mut data = vec![0_u8; AIM_SYNC_SIZE];
        data[AIM_CAMERA_MODE_OFFSET] = 53;
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().add(AIM_FIRST_OFFSET).cast::<f32>(), 1.0);
            ptr::write_unaligned(
                data.as_mut_ptr().add(AIM_POSITION_OFFSET).cast::<f32>(),
                2.0,
            );
            ptr::write_unaligned(data.as_mut_ptr().add(AIM_Z_OFFSET).cast::<f32>(), 3.0);
        }
        data[AIM_ZOOM_WEAPON_STATE_OFFSET] = 0x87;
        data[AIM_ASPECT_RATIO_OFFSET] = 0x42;
        let snapshot = copy_aim_sync(7, data.as_ptr() as usize).unwrap();
        assert_eq!(snapshot.id, 7);
        assert_eq!(snapshot.camera_mode, 53);
        assert_eq!(snapshot.aim_first.x, 1.0);
        assert_eq!(snapshot.aim_position.x, 2.0);
        assert_eq!(snapshot.aim_z, 3.0);
        assert_eq!(snapshot.zoom_and_weapon_state, 0x87);
        assert_eq!(snapshot.aspect_ratio, 0x42);
    }

    #[test]
    fn reads_verified_r3_cursor_mode() {
        let mut module = vec![0_u8; GAME_SINGLETON_RVA + std::mem::size_of::<usize>()];
        let mut game = vec![0_u8; GAME_CURSOR_MODE_READABLE_SIZE];
        let module_base = module.as_mut_ptr() as usize;
        let game_pointer = game.as_mut_ptr();
        unsafe {
            ptr::write_unaligned(
                module.as_mut_ptr().add(GAME_SINGLETON_RVA).cast::<usize>(),
                game_pointer as usize,
            );
            ptr::write_unaligned(game_pointer.add(GAME_CURSOR_MODE_OFFSET).cast::<i32>(), 3);
        }

        let profile = R3ClientProfile::verify(module_base, SAMP_R3_1_ENTRY_POINT).unwrap();

        assert_eq!(profile.cursor_mode(), Ok(3));
        unsafe {
            ptr::write_unaligned(game_pointer.add(GAME_CURSOR_MODE_OFFSET).cast::<i32>(), 5);
        }
        assert_eq!(profile.cursor_mode(), Err(DirectClientError::NotReady));
    }

    unsafe extern "thiscall" fn fake_editbox_get_text(_editbox: *mut c_void) -> *const u8 {
        c"/r3".as_ptr().cast()
    }

    unsafe extern "thiscall" fn fake_chat_get_mode(_chat: *mut c_void) -> i32 {
        2
    }

    unsafe extern "thiscall" fn fake_chat_get_invalid_mode(_chat: *mut c_void) -> i32 {
        3
    }

    #[test]
    fn copies_validated_r3_chat_display_mode() {
        let chat = 0_u8;
        let chat = (&raw const chat).cast_mut().cast();

        assert_eq!(copy_chat_display_mode(chat, fake_chat_get_mode), Ok(2));
        assert_eq!(
            copy_chat_display_mode(chat, fake_chat_get_invalid_mode),
            Err(DirectClientError::NotReady)
        );
    }

    unsafe extern "thiscall" fn fake_player_pool_get_count(
        _pool: *mut c_void,
        include_npcs: i32,
    ) -> i32 {
        if include_npcs == 1 { 3 } else { 2 }
    }

    unsafe extern "thiscall" fn fake_player_pool_get_invalid_count(
        _pool: *mut c_void,
        _include_npcs: i32,
    ) -> i32 {
        1005
    }

    #[test]
    fn copies_validated_r3_player_pool_scalars() {
        let mut pool = vec![0_u8; PLAYER_POOL_LOCAL_ID_OFFSET + mem::size_of::<u16>()];
        unsafe {
            ptr::write_unaligned(
                pool.as_mut_ptr()
                    .add(PLAYER_POOL_LARGEST_ID_OFFSET)
                    .cast::<i32>(),
                42,
            );
        }

        assert_eq!(
            copy_player_counts(pool.as_mut_ptr().cast(), fake_player_pool_get_count),
            Ok((3, 2))
        );
        assert_eq!(copy_player_max_id(pool.as_mut_ptr().cast()), Ok(42));
    }

    #[test]
    fn rejects_invalid_r3_player_pool_scalars() {
        let mut pool = vec![0_u8; PLAYER_POOL_LOCAL_ID_OFFSET + mem::size_of::<u16>()];
        unsafe {
            ptr::write_unaligned(
                pool.as_mut_ptr()
                    .add(PLAYER_POOL_LARGEST_ID_OFFSET)
                    .cast::<i32>(),
                i32::from(MAX_SAMP_PLAYERS),
            );
        }

        assert_eq!(
            copy_player_counts(pool.as_mut_ptr().cast(), fake_player_pool_get_invalid_count,),
            Err(DirectClientError::NotReady)
        );
        assert_eq!(
            copy_player_max_id(pool.as_mut_ptr().cast()),
            Err(DirectClientError::NotReady)
        );
    }

    #[test]
    fn copies_bounded_r3_chat_input_text() {
        let editbox = 0_u8;

        assert_eq!(
            copy_chat_input_text(
                (&raw const editbox).cast_mut().cast(),
                fake_editbox_get_text,
            ),
            Ok(b"/r3".to_vec())
        );
    }

    #[test]
    fn converts_public_argb_to_r3_native_rgba() {
        assert_eq!(argb_to_native_rgba(0xFF6FCF97), 0x6FCF97FF);
    }
}
