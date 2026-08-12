//! Private SA-MP 0.3.7 R3-1 profile for verified read-only caches.
//!
//! This deliberately covers only copied server/game, local-player, narrow
//! chat-input, and dialog-active snapshots. Broader R3 UI, remote-player, pool,
//! raw-address, and mutation helpers remain unavailable until each family has
//! an independent fixture and live validation.

use super::r1_client::memory::{
    bounded_c_string, read_pointer, read_unaligned, read_vector3, readable_range,
};
use crate::runtime::{DirectClientError, LocalPlayerSnapshot, ServerInfoSnapshot};
use std::{ffi::c_void, mem};

const SAMP_R3_1_ENTRY_POINT: u32 = 0x0C_C4_D0;
const NET_GAME_SINGLETON_RVA: usize = 0x26_E8_DC;
const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x30;
const NET_GAME_HOSTNAME_OFFSET: usize = 0x131;
const NET_GAME_PORT_OFFSET: usize = 0x235;
const NET_GAME_GAME_STATE_OFFSET: usize = 0x3CD;
const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
const NET_GAME_SCALAR_READABLE_SIZE: usize = NET_GAME_GAME_STATE_OFFSET + mem::size_of::<i32>();
const NET_GAME_GET_PLAYER_POOL_RVA: usize = 0x1160;
const PLAYER_POOL_GET_LOCAL_PLAYER_RVA: usize = 0x1A30;
const PLAYER_POOL_GET_LOCAL_SCORE_RVA: usize = 0x6E140;
const PLAYER_POOL_GET_LOCAL_PING_RVA: usize = 0x6E150;
const PLAYER_POOL_GET_NAME_RVA: usize = 0x16F00;
const LOCAL_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x3DA0;
const PED_GET_HEALTH_RVA: usize = 0xAB4C0;
const PED_GET_ARMOUR_RVA: usize = 0xAB500;
const PLAYER_POOL_LOCAL_ID_OFFSET: usize = 0x2F1C;
const LOCAL_PLAYER_INCAR_OFFSET: usize = 0x04;
const LOCAL_PLAYER_ONFOOT_OFFSET: usize = 0x98;
const LOCAL_PLAYER_ACTIVE_OFFSET: usize = 0xF4;
const LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET: usize = 0xFC;
const LOCAL_PLAYER_SNAPSHOT_READABLE_SIZE: usize = LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET + 2;
const SAMP_PED_GAME_PED_OFFSET: usize = 0x2A4;
const INVALID_ID: u16 = u16::MAX;
const MAX_SAMP_PLAYERS: u16 = 1004;
const ONFOOT_POSITION_OFFSET: usize = 0x06;
const ONFOOT_SPECIAL_ACTION_OFFSET: usize = 0x25;
const ONFOOT_SPEED_OFFSET: usize = 0x26;
const ONFOOT_ANIMATION_OFFSET: usize = 0x40;
const INCAR_POSITION_OFFSET: usize = 0x18;
const INCAR_SPEED_OFFSET: usize = 0x24;
const INPUT_SINGLETON_RVA: usize = 0x26_E8_CC;
const INPUT_EDIT_BOX_OFFSET: usize = 0x08;
const INPUT_COMMAND_NAME_OFFSET: usize = 0x24C;
const INPUT_COMMAND_NAME_CAPACITY: usize = 33;
const INPUT_COMMAND_COUNT_OFFSET: usize = 0x14DC;
const INPUT_ENABLED_OFFSET: usize = 0x14E0;
const INPUT_CACHE_READABLE_SIZE: usize = INPUT_ENABLED_OFFSET + mem::size_of::<i32>();
const MAX_CHAT_COMMANDS: usize = 144;
const CHAT_INPUT_TEXT_CAPACITY: usize = 129;
const DXUT_EDIT_BOX_GET_TEXT_RVA: usize = 0x84F40;
const DIALOG_SINGLETON_RVA: usize = 0x26_E8_98;
const DIALOG_ACTIVE_OFFSET: usize = 0x28;
const DIALOG_ACTIVE_READABLE_SIZE: usize = DIALOG_ACTIVE_OFFSET + mem::size_of::<i32>();

type NetGameGetPlayerPoolFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalStatFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolGetNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type LocalPlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
type DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;

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

    /// Captures the R3-1 CNetGame state with a guarded scalar read.
    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let field = (net_game as usize)
            .checked_add(NET_GAME_GAME_STATE_OFFSET)
            .ok_or(DirectClientError::NotReady)?;
        unsafe { read_unaligned::<i32>(field) }.ok_or(DirectClientError::NotReady)
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

    /// Copies the R3-1 chat-input enabled flag without invoking its UI methods.
    pub(super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(input as usize + INPUT_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
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

    /// Copies the R3-1 dialog active flag without reading dialog controls.
    pub(super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(dialog as usize + DIALOG_ACTIVE_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    fn player_pool(self) -> Result<*mut c_void, DirectClientError> {
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

    fn net_game(self) -> Option<*mut c_void> {
        let net_game: *mut c_void =
            unsafe { read_pointer(self.module_base.checked_add(NET_GAME_SINGLETON_RVA)?) }?.cast();
        (!net_game.is_null() && readable_range(net_game.cast(), NET_GAME_SCALAR_READABLE_SIZE))
            .then_some(net_game)
    }
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

    unsafe extern "thiscall" fn fake_editbox_get_text(_editbox: *mut c_void) -> *const u8 {
        c"/r3".as_ptr().cast()
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
}
