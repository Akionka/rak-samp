use super::memory::NativeVector3;
use std::ffi::c_void;

pub(super) type DialogShowFn = unsafe extern "thiscall" fn(
    *mut c_void,
    i32,
    i32,
    *const i8,
    *const i8,
    *const i8,
    *const i8,
    i32,
);
pub(super) type DialogCloseFn = unsafe extern "thiscall" fn(*mut c_void, u8);
pub(super) type ChatAddEntryFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
pub(super) type InputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type InputGetCommandHandlerFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8) -> *const c_void;
pub(super) type InputAddCommandFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, unsafe extern "cdecl" fn(*const i8));
pub(super) type DxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);
pub(super) type DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const i8;
pub(super) type ChatGetModeFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
pub(super) type DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
pub(super) type NetGameGetPlayerPoolFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
pub(super) type TextdrawPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16);
pub(super) type TextdrawPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, i32, *mut c_void, *const u8) -> *mut c_void;
pub(super) type TextdrawSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const u8);
pub(super) type LabelPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, u16, *const u8, u32, NativeVector3, f32, u8, u16, u16);
pub(super) type LabelPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
pub(super) type PlayerPoolGetLocalPlayerFn =
    unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
pub(super) type PlayerPoolGetLocalScoreFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
pub(super) type PlayerPoolGetLocalPingFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
pub(super) type PlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
pub(super) type PlayerPoolGetRemotePlayerFn =
    unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
pub(super) type PlayerPoolGetPlayerNameFn =
    unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
pub(super) type PlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
pub(super) type PlayerPoolSetLocalPlayerNameFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8);
pub(super) type LocalPlayerGetPedFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
pub(super) type LocalPlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
pub(super) type LocalPlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
pub(super) type LocalPlayerSetSpecialActionFn = unsafe extern "thiscall" fn(*mut c_void, u8);
pub(super) type LocalPlayerSpawnFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
pub(super) type LocalPlayerSendUnoccupiedDataFn =
    unsafe extern "thiscall" fn(*mut c_void, u16, i32);
pub(super) type LocalPlayerSendAimDataFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type LocalPlayerSendOnfootDataFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type LocalPlayerSendStatsFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type LocalPlayerSendTrailerDataFn = unsafe extern "thiscall" fn(*mut c_void, u16);
pub(super) type LocalPlayerSendPassengerDataFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type LocalPlayerSendIncarDataFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type LocalPlayerUpdateWeaponsFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type GameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
pub(super) type GameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);
pub(super) type RemotePlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
pub(super) type RemotePlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
pub(super) type RemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
pub(super) type RemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
pub(super) type PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
pub(super) type CpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;
