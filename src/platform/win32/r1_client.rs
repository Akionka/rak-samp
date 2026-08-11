//! Private SA-MP 0.3.7 R1 client profile for direct local helpers.
//!
//! This deliberately does not share [`crate::AddressSet`]: RakNet hook offsets
//! are supported across several clients, while these object layouts and native
//! calls use approved fixed R1 offsets and validate native values at each access.

mod addresses;
mod handles;
pub(crate) mod memory;
mod native_types;
mod players;
mod pools;
mod singletons;
mod textdraws;
mod ui;

use crate::runtime::{
    AimSyncSnapshot, AnimationSnapshot, ChatEntrySnapshot, DirectClientError, GangzoneSnapshot,
    InCarSyncSnapshot, LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest,
    LocalDialogSnapshot, LocalDialogStyle, LocalPlayerSnapshot, OnFootSyncSnapshot,
    PassengerSyncSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot, ServerInfoSnapshot,
    TextLabelSnapshot, TextdrawSnapshot, TrailerSyncSnapshot, Vector3,
};
use addresses::*;
use memory::*;
use native_types::*;
use std::{ffi::c_void, mem, ptr};
const INVALID_ID: u16 = u16::MAX;

/// A narrow R1-only profile whose fields and call targets never cross the
/// plugin ABI. `verify` has to succeed before any profile address is used.
#[derive(Clone, Copy, Debug)]
pub(super) struct R1ClientProfile {
    module_base: usize,
}

impl R1ClientProfile {
    pub(super) fn verify(module_base: usize, entry_point: u32) -> Option<Self> {
        // Build selection is performed by `SampVersion::from_entry_point` before
        // this profile is requested. R1's native bridge therefore uses the
        // approved fixed offsets and validates every pointer, range, capacity,
        // and enum at the point of use instead of gating the whole surface on
        // global executable identity or instruction checks.
        (module_base != 0 && entry_point == SAMP_R1_ENTRY_POINT).then_some(Self { module_base })
    }

    /// Captures the validated R1 player-pool address on the game thread.
    pub(super) fn player_pool(self) -> Result<*mut c_void, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        Ok(pool)
    }

    /// Captures the validated R1 vehicle-pool address on the game thread.
    pub(super) fn vehicle_pool(self) -> Result<*mut c_void, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_vehicle_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_VEHICLE_POOL_RVA) };
        let pool = unsafe { get_vehicle_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        Ok(pool)
    }

    /// Captures the validated R1 local-player object address on the game
    /// thread. The address stays opaque outside the host boundary.
    pub(super) fn local_player_address(self) -> Result<*mut c_void, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_local_player: PlayerPoolGetLocalPlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
        let local = unsafe { get_local_player(pool) };
        if local.is_null() || !readable_range(local.cast(), LOCAL_PLAYER_INCAR_OFFSET + 0x30) {
            return Err(DirectClientError::NotReady);
        }
        Ok(local)
    }

    /// Returns the R1 RakPeer base underlying a captured RakClient interface.
    pub(super) fn rakpeer_address(
        self,
        rakclient: *mut c_void,
    ) -> Result<*mut c_void, DirectClientError> {
        let peer = (rakclient as usize)
            .checked_sub(RAKPEER_SIZE)
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(peer.cast(), RAKPEER_SIZE + 1) {
            return Err(DirectClientError::NotReady);
        }
        Ok(peer)
    }

    pub(super) fn animation_catalog(self) -> Result<Vec<AnimationSnapshot>, DirectClientError> {
        let table = self.module_base + ANIMATION_TABLE_RVA;
        let length = ANIMATION_TABLE_ENTRY_COUNT * ANIMATION_TABLE_ENTRY_SIZE;
        if !readable_range(table as *const u8, length) {
            return Err(DirectClientError::NotReady);
        }
        let entries = unsafe { std::slice::from_raw_parts(table as *const u8, length) };
        entries
            .chunks_exact(ANIMATION_TABLE_ENTRY_SIZE)
            .map(parse_animation_entry)
            .collect()
    }

    pub(super) fn death_window_is_ready(self) -> bool {
        self.death_window().is_some()
    }

    pub(super) fn game_state(self) -> Result<i32, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_state: NetGameGetStateFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_STATE_RVA) };
        Ok(unsafe { get_state(net_game) })
    }

    /// Writes one established R1 CNetGame state from the game-thread command
    /// pump after checking the fixed-layout scalar is writable.
    pub(super) fn set_game_state(self, state: i32) -> Result<(), DirectClientError> {
        if !is_r1_game_state(state) {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let field = unsafe {
            (net_game as *mut u8)
                .add(NET_GAME_GAME_STATE_OFFSET)
                .cast::<i32>()
        };
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, state) };
        Ok(())
    }

    /// Starts the documented R1 reconnect sequence after copying a bounded
    /// server address and port into the validated CNetGame fields.
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
        let host_address = unsafe { (net_game as *mut u8).add(NET_GAME_HOST_ADDRESS_OFFSET) };
        let port_field = unsafe {
            (net_game as *mut u8)
                .add(NET_GAME_PORT_OFFSET)
                .cast::<i32>()
        };
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

    /// Executes SF.lua's R1 disconnect sequence against the captured RakClient
    /// interface, then lets CNetGame reset its state for a later connection.
    pub(super) fn disconnect_with_reason(
        self,
        rak_client: *mut c_void,
        block_duration: u32,
    ) -> Result<(), DirectClientError> {
        if rak_client.is_null() {
            return Err(DirectClientError::NotReady);
        }
        let vtable =
            unsafe { read_pointer(rak_client as usize) }.ok_or(DirectClientError::NotReady)?;
        let disconnect_offset = RAK_CLIENT_DISCONNECT_VTABLE_SLOT
            .checked_mul(mem::size_of::<usize>())
            .ok_or(DirectClientError::NotReady)?;
        if vtable.is_null() || !readable_range(vtable, disconnect_offset + mem::size_of::<usize>())
        {
            return Err(DirectClientError::NotReady);
        }
        let disconnect_address = unsafe { read_pointer(vtable as usize + disconnect_offset) }
            .filter(|address| !address.is_null())
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(disconnect_address, 1) {
            return Err(DirectClientError::NotReady);
        }
        let disconnect: RakClientDisconnectFn = unsafe { mem::transmute(disconnect_address) };
        unsafe { disconnect(rak_client, block_duration, 0) };

        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let shutdown: NetGameNoArgFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_SHUTDOWN_FOR_RESTART_RVA) };
        unsafe { shutdown(net_game) };
        Ok(())
    }

    /// Invokes R1 `SCLocalPlayer::SetSpecialAction` on the game thread.
    pub(super) fn set_local_player_special_action(
        self,
        action: u8,
    ) -> Result<(), DirectClientError> {
        if !matches!(action, 0..=12 | 20..=25 | 68) {
            return Err(DirectClientError::NotReady);
        }
        let local_player = self.local_player_address()?;
        let set_special_action: LocalPlayerSetSpecialActionFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_SET_SPECIAL_ACTION_RVA) };
        unsafe { set_special_action(local_player, action) };
        Ok(())
    }

    /// Invokes the documented R1 local- or remote-player colour setter on the
    /// game thread after resolving the checked player-pool entry.
    pub(super) fn set_player_colour(self, id: u16, colour: u32) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        if !readable_range(
            pool.cast(),
            PLAYER_POOL_LOCAL_ID_OFFSET + mem::size_of::<u16>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id);
        if local_id == Some(id) {
            let local = self.local_player_address()?;
            let set_colour: LocalPlayerSetColourFn =
                unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_SET_COLOUR_RVA) };
            unsafe { set_colour(local, colour) };
            return Ok(());
        }

        let is_connected: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
        if unsafe { is_connected(pool, id) } != 1 {
            return Err(DirectClientError::NotReady);
        }
        let get_player: PlayerPoolGetRemotePlayerFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
        let remote = unsafe { get_player(pool, id) };
        if remote.is_null() || !readable_range(remote.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let set_colour: RemotePlayerSetColourFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_SET_COLOUR_RVA) };
        unsafe { set_colour(remote, colour) };
        Ok(())
    }

    /// Updates R1's local player name through the documented player-pool
    /// method, retaining no client pointer outside this game-thread call.
    pub(super) fn set_local_player_name(self, name: &[u8]) -> Result<(), DirectClientError> {
        if name.len() > MAX_LOCAL_PLAYER_NAME_BYTES || name.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.player_pool()?;
        let name = nul_terminated(name.to_vec());
        let set_name: PlayerPoolSetLocalPlayerNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_SET_LOCAL_PLAYER_NAME_RVA) };
        unsafe { set_name(pool, name.as_ptr().cast()) };
        Ok(())
    }

    /// Invokes R1 `SCLocalPlayer::SendUnoccupiedData` for one checked vehicle
    /// ID and the caller-supplied native seat scalar on the game thread.
    pub(super) fn force_unoccupied_sync(
        self,
        vehicle: u16,
        seat: i32,
    ) -> Result<(), DirectClientError> {
        if vehicle >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let local_player = self.local_player_address()?;
        let send: LocalPlayerSendUnoccupiedDataFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_SEND_UNOCCUPIED_DATA_RVA) };
        unsafe { send(local_player, vehicle, seat) };
        Ok(())
    }

    /// Invokes R1 `SCLocalPlayer::SendAimData` on the game thread.
    pub(super) fn force_aim_sync(self) -> Result<(), DirectClientError> {
        let local_player = self.local_player_address()?;
        let last_update = (local_player as usize + LOCAL_PLAYER_LAST_ANY_UPDATE_OFFSET) as *mut u32;
        if !readable_range(last_update.cast(), mem::size_of::<u32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(last_update, 0) };
        let send: LocalPlayerSendAimDataFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_SEND_AIM_DATA_RVA) };
        unsafe { send(local_player) };
        Ok(())
    }

    /// Invokes R1 `SCLocalPlayer::Spawn` on the game thread.
    pub(super) fn spawn_local_player(self) -> Result<(), DirectClientError> {
        let local_player = self.local_player_address()?;
        let spawn: LocalPlayerSpawnFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_SPAWN_RVA) };
        if unsafe { spawn(local_player) } == 0 {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    /// Updates one R1 send-rate global after validating its selected scalar.
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
        .ok_or(DirectClientError::NotReady)?;
        let port = unsafe { read_unaligned::<i32>(net_game as usize + NET_GAME_PORT_OFFSET) }
            .and_then(|port| u16::try_from(port).ok())
            .filter(|port| *port != 0)
            .ok_or(DirectClientError::NotReady)?;
        Ok(ServerInfoSnapshot {
            address,
            hostname,
            port,
        })
    }

    fn net_game(self) -> Option<*mut c_void> {
        let net_game: *mut c_void =
            unsafe { read_pointer(self.module_base + NET_GAME_SINGLETON_RVA) }?.cast();
        (!net_game.is_null() && readable_range(net_game.cast(), 1)).then_some(net_game)
    }

    fn game(self) -> Option<*mut c_void> {
        let game: *mut c_void =
            unsafe { read_pointer(self.module_base + GAME_SINGLETON_RVA) }?.cast();
        (!game.is_null() && readable_range(game.cast(), GAME_CURSOR_MODE_OFFSET + 4))
            .then_some(game)
    }
}

fn assigned_player_id(id: u16) -> Option<u16> {
    (id != INVALID_ID).then_some(id)
}

const fn is_r1_game_state(state: i32) -> bool {
    matches!(state, 0 | 9 | 13 | 14 | 15 | 18)
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

fn nul_terminated(mut value: Vec<u8>) -> Vec<u8> {
    value.push(0);
    value
}

#[cfg(test)]
mod tests;
