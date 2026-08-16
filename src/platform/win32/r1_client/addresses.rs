//! Approved SA-MP 0.3.7 R1 and GTA:SA 1.0 US native addresses.

#![allow(dead_code)] // Legacy sync oracle retained until fixture parity tests move.

pub(super) const DIALOG_SINGLETON_RVA: usize = 0x21A0B8;
pub(super) const DIALOG_SHOW_RVA: usize = 0x6B9C0;
pub(super) const DIALOG_CLOSE_RVA: usize = 0x6C040;
pub(super) const INPUT_SINGLETON_RVA: usize = 0x21A0E8;
pub(super) const INPUT_OPEN_RVA: usize = 0x657E0;
pub(super) const INPUT_CLOSE_RVA: usize = 0x658E0;
pub(super) const INPUT_GET_COMMAND_HANDLER_RVA: usize = 0x65A70;
pub(super) const INPUT_ADD_COMMAND_RVA: usize = 0x65AD0;
pub(super) const INPUT_PROCESS_RVA: usize = 0x65D30;
pub(super) const DXUT_EDIT_BOX_SET_TEXT_RVA: usize = 0x80F60;
pub(super) const DXUT_EDIT_BOX_GET_TEXT_RVA: usize = 0x81030;
pub(super) const CHAT_SINGLETON_RVA: usize = 0x21A0E4;
pub(super) const CHAT_ADD_ENTRY_RVA: usize = 0x64010;
pub(super) const CHAT_GET_MODE_RVA: usize = 0x5D7A0;
pub(super) const SCOREBOARD_SINGLETON_RVA: usize = 0x21A0B4;
pub(super) const DEATH_WINDOW_SINGLETON_RVA: usize = 0x21A0EC;
pub(super) const DEATH_WINDOW_ADD_MESSAGE_RVA: usize = 0x66A10;
pub(super) const NET_GAME_SINGLETON_RVA: usize = 0x21A0F8;
#[cfg(test)]
pub(super) const NET_GAME_GET_STATE_RVA: usize = 0x2E20;
pub(super) const NET_GAME_GET_PLAYER_POOL_RVA: usize = 0x1160;
pub(super) const NET_GAME_GET_VEHICLE_POOL_RVA: usize = 0x1170;
#[cfg(test)]
pub(super) const NET_GAME_SHUTDOWN_FOR_RESTART_RVA: usize = 0xA060;
pub(super) const PLAYER_POOL_GET_LOCAL_PLAYER_RVA: usize = 0x1A30;
#[cfg(test)]
pub(super) const PLAYER_POOL_GET_LOCAL_SCORE_RVA: usize = 0x6A1F0;
#[cfg(test)]
pub(super) const PLAYER_POOL_GET_LOCAL_PING_RVA: usize = 0x6A200;
pub(super) const PLAYER_POOL_IS_CONNECTED_RVA: usize = 0x10B0;
pub(super) const PLAYER_POOL_GET_REMOTE_PLAYER_RVA: usize = 0x10F0;
#[cfg(test)]
pub(super) const PLAYER_POOL_IS_NPC_RVA: usize = 0xB680;
#[cfg(test)]
pub(super) const PLAYER_POOL_GET_NAME_RVA: usize = 0x13CE0;
#[cfg(test)]
pub(super) const PLAYER_POOL_GET_SCORE_RVA: usize = 0x6A190;
#[cfg(test)]
pub(super) const PLAYER_POOL_GET_PING_RVA: usize = 0x6A1C0;
#[cfg(test)]
pub(super) const PLAYER_POOL_GET_COUNT_RVA: usize = 0x10520;
#[cfg(test)]
pub(super) const PLAYER_POOL_SET_LOCAL_PLAYER_NAME_RVA: usize = 0xB3E0;
pub(super) const VEHICLE_POOL_DOES_EXIST_RVA: usize = 0x1140;
#[cfg(test)]
pub(super) const REMOTE_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x12A00;
#[cfg(test)]
pub(super) const REMOTE_PLAYER_SET_COLOUR_RVA: usize = 0x129D0;
pub(super) const REMOTE_PLAYER_DOES_EXIST_RVA: usize = 0x1080;
#[cfg(test)]
pub(super) const REMOTE_PLAYER_GET_STATUS_RVA: usize = 0x12BA0;
pub(super) const LOCAL_PLAYER_GET_PED_RVA: usize = 0x2D60;
#[cfg(test)]
pub(super) const LOCAL_PLAYER_SET_COLOUR_RVA: usize = 0x3D40;
#[cfg(test)]
pub(super) const LOCAL_PLAYER_SET_SPECIAL_ACTION_RVA: usize = 0x30C0;
#[cfg(test)]
pub(super) const LOCAL_PLAYER_SPAWN_RVA: usize = 0x3AD0;
#[cfg(test)]
pub(super) const LOCAL_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x3D90;
pub(super) const LOCAL_PLAYER_SEND_UNOCCUPIED_DATA_RVA: usize = 0x4B30;
pub(super) const LOCAL_PLAYER_SEND_AIM_DATA_RVA: usize = 0x4FF0;
pub(super) const LOCAL_PLAYER_SEND_ONFOOT_DATA_RVA: usize = 0x4D10;
pub(super) const LOCAL_PLAYER_SEND_STATS_RVA: usize = 0x5AF0;
pub(super) const LOCAL_PLAYER_SEND_TRAILER_DATA_RVA: usize = 0x51B0;
pub(super) const LOCAL_PLAYER_SEND_PASSENGER_DATA_RVA: usize = 0x5380;
pub(super) const LOCAL_PLAYER_SEND_INCAR_DATA_RVA: usize = 0x6E30;
pub(super) const LOCAL_PLAYER_UPDATE_WEAPONS_RVA: usize = 0x6080;
pub(super) const ONFOOT_SEND_RATE_RVA: usize = 0xEC0A8;
pub(super) const INCAR_SEND_RATE_RVA: usize = 0xEC0AC;
pub(super) const AIM_SEND_RATE_RVA: usize = 0xEC0B0;
#[cfg(test)]
pub(super) const PED_GET_HEALTH_RVA: usize = 0xA6610;
#[cfg(test)]
pub(super) const PED_GET_ARMOUR_RVA: usize = 0xA6650;
pub(super) const GAME_SINGLETON_RVA: usize = 0x21A10C;
pub(super) const GAME_SET_CURSOR_MODE_RVA: usize = 0x9BD30;
pub(super) const GAME_PROCESS_INPUT_ENABLING_RVA: usize = 0x9BC10;
pub(super) const ANIMATION_TABLE_RVA: usize = 0xF15B0;

// GTA SA 1.0 US `CPools` handle conversions (cdecl): ped/vehicle pointer to
// GTAREF. Cross-checked with DK22Pac/plugin-sdk `plugin_sa/game_sa/CPools.cpp`.
pub(super) const CPOOLS_GET_PED_REF: usize = 0x54FF60;
pub(super) const CPOOLS_GET_VEHICLE_REF: usize = 0x54FFC0;

pub(super) const LABEL_POOL_CREATE_RVA: usize = 0x11C0;
pub(super) const LABEL_POOL_DELETE_RVA: usize = 0x12D0;
pub(super) const TEXTDRAW_POOL_CREATE_RVA: usize = 0x1AE20;
pub(super) const TEXTDRAW_POOL_DELETE_RVA: usize = 0x1AD00;
pub(super) const TEXTDRAW_SET_TEXT_RVA: usize = 0xAC870;
#[cfg(test)]
pub(super) const SAMP_R1_ENTRY_POINT: u32 = 0x31DF13;
