//! Private SA-MP 0.3.7 R1 client profile for direct local helpers.
//!
//! This deliberately does not share [`crate::AddressSet`]: RakNet hook offsets
//! are supported across several clients, while these object layouts and native
//! calls are safe only for the one fingerprinted R1 profile below.

use crate::runtime::{
    AnimationSnapshot, DirectClientError, LocalChatMessageRequest, LocalDeathMessageRequest,
    LocalDialogRequest, LocalPlayerSnapshot, ServerInfoSnapshot, Vector3,
};
use std::{ffi::c_void, mem};
use windows_sys::Win32::System::{
    LibraryLoader::GetModuleHandleA,
    Memory::{MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, VirtualQuery},
};

const SAMP_R1_TIMESTAMP: u32 = 0x5542_F47A;
const SAMP_R1_ENTRY_POINT: u32 = 0x31DF13;

// GTA San Andreas 1.0 US (the standard 14,383,616-byte executable) has this
// in-memory PE identity. The compact executable and every later game build
// have a different SizeOfImage and are intentionally rejected.
const GTA_SA_10_US_IMAGE_BASE: u32 = 0x0040_0000;
const GTA_SA_10_US_IMAGE_SIZE: u32 = 0x0117_7000;
const GTA_SA_10_US_ENTRY_POINT: u32 = 0x0042_4570;

const DIALOG_SINGLETON_RVA: usize = 0x21A0B8;
const DIALOG_SHOW_RVA: usize = 0x6B9C0;
const INPUT_SINGLETON_RVA: usize = 0x21A0E8;
const INPUT_OPEN_RVA: usize = 0x657E0;
const INPUT_CLOSE_RVA: usize = 0x658E0;
const CHAT_SINGLETON_RVA: usize = 0x21A0E4;
const CHAT_ADD_ENTRY_RVA: usize = 0x64010;
const CHAT_GET_MODE_RVA: usize = 0x5D7A0;
const SCOREBOARD_SINGLETON_RVA: usize = 0x21A0B4;
const SCOREBOARD_CLOSE_RVA: usize = 0x6A320;
const SCOREBOARD_ENABLE_RVA: usize = 0x6AD30;
const DEATH_WINDOW_SINGLETON_RVA: usize = 0x21A0EC;
const DEATH_WINDOW_ADD_ENTRY_RVA: usize = 0x66930;
const DEATH_WINDOW_ADD_MESSAGE_RVA: usize = 0x66A10;
const NET_GAME_SINGLETON_RVA: usize = 0x21A0F8;
const NET_GAME_GET_STATE_RVA: usize = 0x2E20;
const NET_GAME_GET_PLAYER_POOL_RVA: usize = 0x1160;
const PLAYER_POOL_GET_LOCAL_PLAYER_RVA: usize = 0x1A30;
const PLAYER_POOL_GET_LOCAL_NAME_RVA: usize = 0x13CD0;
const PLAYER_POOL_GET_LOCAL_SCORE_RVA: usize = 0x6A1F0;
const PLAYER_POOL_GET_LOCAL_PING_RVA: usize = 0x6A200;
const LOCAL_PLAYER_GET_PED_RVA: usize = 0x2D60;
const LOCAL_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x3D90;
const PED_GET_HEALTH_RVA: usize = 0xA6610;
const PED_GET_ARMOUR_RVA: usize = 0xA6650;
const GAME_SINGLETON_RVA: usize = 0x21A10C;
const GAME_PROCESS_INPUT_ENABLING_RVA: usize = 0x9BC10;
const ANIMATION_TABLE_RVA: usize = 0xF15B0;
const ANIMATION_TABLE_ENTRY_COUNT: usize = 1812;
const ANIMATION_TABLE_ENTRY_SIZE: usize = 36;

const PLAYER_POOL_LOCAL_ID_OFFSET: usize = 0x04;
// These packed CNetGame fields are cross-checked by the independently written
// fixture. `GetGameState`'s signed R1 target reads offset 0x3BD from this same
// layout, which anchors the packed field sequence.
const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x20;
const NET_GAME_HOSTNAME_OFFSET: usize = 0x121;
const NET_GAME_PORT_OFFSET: usize = 0x225;
const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
const SCOREBOARD_ENABLED_OFFSET: usize = 0x00;
const GAME_CURSOR_MODE_OFFSET: usize = 0x55;
const DIALOG_ACTIVE_OFFSET: usize = 0x28;
const INPUT_ENABLED_OFFSET: usize = 0x14E0;

// First 16 bytes of SA-MP 0.3.7 R1's `CDialog::Show` at `DIALOG_SHOW_RVA`.
// The function uses a frame-less prologue; do not substitute the common
// `55 8B EC` prologue here, or the valid R1 profile will be rejected.
const DIALOG_SHOW_SIGNATURE: [u8; 16] = [
    0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB, 0x8B, 0xF1,
];

// The `CDialog::Show` active-state comparison immediately follows the original
// show-target signature. Verify it separately to pin the copied `m_bIsActive`
// read without widening the existing show-call signature.
const DIALOG_SHOW_ACTIVE_SIGNATURE: [u8; 22] = [
    0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB, 0x8B, 0xF1,
    0x7D, 0x17, 0x39, 0x5E, 0x28, 0x0F,
];

// `CInput::Open` and `Close` both read the packed `m_bEnabled` flag at
// offset 0x14E0 before proceeding with their UI work. They are evidence only:
// the safe helper below copies the flag and never invokes either mutation.
const INPUT_OPEN_SIGNATURE: [u8; 16] = [
    0x83, 0xEC, 0x10, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0, 0x0F, 0x85,
];
const INPUT_CLOSE_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0, 0x74, 0x39, 0x8B, 0x4E, 0x08,
];

// First 16 bytes of SA-MP 0.3.7 R1's `CChat::AddEntry` at
// `CHAT_ADD_ENTRY_RVA`. The target's x86 prologue moves `this` from ECX into
// EBP, slides the 100-entry ring, and then consumes the five stack arguments.
const CHAT_ADD_ENTRY_SIGNATURE: [u8; 16] = [
    0x55, 0x56, 0x8B, 0xE9, 0x57, 0x8D, 0xBD, 0x32, 0x01, 0x00, 0x00, 0x8D, 0xB5, 0x2E, 0x02, 0x00,
];

// `CChat::GetMode` is a leaf R1 accessor: `mov eax, [ecx + 8]; ret`. Keep
// the exact code signature rather than reading the field directly so the
// private layout remains behind the fingerprinted native profile.
const CHAT_GET_MODE_SIGNATURE: [u8; 4] = [0x8B, 0x41, 0x08, 0xC3];

// `CScoreboard::Close` and `Enable` both start by comparing the packed
// `m_bIsEnabled` field at offset zero. Together they anchor the copied boolean
// read below without turning the field into a public client layout.
const SCOREBOARD_CLOSE_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x74, 0x3C, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74, 0x35, 0xC6,
];
const SCOREBOARD_ENABLE_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x75, 0x43, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74, 0x3C, 0xC6,
];

// `CGame::ProcessInputEnabling` loads `m_nCursorMode` from offset 0x55 before
// checking the associated input-enable state. Its exact R1 signature anchors
// the narrow copied cursor-mode field below.
const GAME_PROCESS_INPUT_ENABLING_SIGNATURE: [u8; 16] = [
    0x56, 0x8B, 0xF1, 0x8B, 0x46, 0x55, 0x57, 0x33, 0xFF, 0x3B, 0xC7, 0x0F, 0x85, 0x07, 0x01, 0x00,
];

// R1 stores 1,812 fixed 36-byte `group:name` animation entries in a static
// table. Its complete first entry fingerprints the data format before the
// game-thread pump makes an owned copy.
const ANIMATION_TABLE_SIGNATURE: [u8; 36] = [
    0x41, 0x49, 0x52, 0x50, 0x4F, 0x52, 0x54, 0x3A, 0x54, 0x48, 0x52, 0x57, 0x5F, 0x42, 0x41, 0x52,
    0x4C, 0x5F, 0x54, 0x48, 0x52, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

// `CDeathWindow::AddMessage` is an R1 thunk to `AddEntry`; verify both its
// five-byte relative jump and the start of the final target before enabling
// the direct death-window helper.
const DEATH_WINDOW_ADD_MESSAGE_SIGNATURE: [u8; 5] = [0xE9, 0x1B, 0xFF, 0xFF, 0xFF];
const DEATH_WINDOW_ADD_ENTRY_SIGNATURE: [u8; 16] = [
    0x8B, 0xD1, 0xE8, 0x49, 0xF6, 0xFF, 0xFF, 0x8A, 0x44, 0x24, 0x14, 0x8B, 0x4C, 0x24, 0x10, 0x88,
];

// `CNetGame::GetGameState` returns the client's native state enum by value.
// Keep this signature separate from the dialog target: callers expose the
// value only as an opaque scalar, rather than depending on enum names from an
// unversioned client header.
const NET_GAME_GET_STATE_SIGNATURE: [u8; 7] = [0x8B, 0x81, 0xBD, 0x03, 0x00, 0x00, 0xC3];

const LOCAL_PLAYER_ACTIVE_OFFSET: usize = 0x0C;
const LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET: usize = 0x14;
const LOCAL_PLAYER_ONFOOT_OFFSET: usize = 0x18;
const LOCAL_PLAYER_INCAR_OFFSET: usize = 0xAA;
const LOCAL_PLAYER_ONFOOT_POSITION_OFFSET: usize = 0x06;
const LOCAL_PLAYER_ONFOOT_SPEED_OFFSET: usize = 0x26;
const LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET: usize = 0x25;
const LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET: usize = 0x40;
const LOCAL_PLAYER_INCAR_POSITION_OFFSET: usize = 0x18;
const LOCAL_PLAYER_INCAR_SPEED_OFFSET: usize = 0x24;
// `CPed` inherits a 0x48-byte `CEntity`, then owns its accessory arrays before
// its GTA-ped pointer.
const SAMP_PED_GAME_PED_OFFSET: usize = 0x2A4;
const INVALID_ID: u16 = u16::MAX;

/// A narrow R1-only profile whose fields and call targets never cross the
/// plugin ABI. `verify` has to succeed before any profile address is used.
#[derive(Clone, Copy, Debug)]
pub(super) struct R1ClientProfile {
    module_base: usize,
}

impl R1ClientProfile {
    pub(super) fn verify(module_base: usize, entry_point: u32) -> Option<Self> {
        (entry_point == SAMP_R1_ENTRY_POINT
            && unsafe { samp_r1_pe_matches(module_base) }
            && unsafe { gta_sa_10_us_matches() }
            && unsafe { r1_targets_match(module_base) })
        .then_some(Self { module_base })
    }

    pub(super) fn show_dialog(self, request: LocalDialogRequest) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;

        let title = nul_terminated(request.title);
        let text = nul_terminated(request.text);
        let button1 = nul_terminated(request.button1);
        let button2 = nul_terminated(request.button2);
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

    pub(super) fn show_chat_message(
        self,
        request: LocalChatMessageRequest,
    ) -> Result<(), DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let text = nul_terminated(request.text);
        let prefix = nul_terminated(request.prefix);
        let add_entry: ChatAddEntryFn =
            unsafe { mem::transmute(self.module_base + CHAT_ADD_ENTRY_RVA) };
        unsafe {
            add_entry(
                chat,
                request.style.as_raw(),
                text.as_ptr().cast(),
                prefix.as_ptr().cast(),
                request.text_colour,
                request.prefix_colour,
            );
        }
        Ok(())
    }

    pub(super) fn show_death_message(
        self,
        request: LocalDeathMessageRequest,
    ) -> Result<(), DirectClientError> {
        let death_window = self.death_window().ok_or(DirectClientError::NotReady)?;
        let killer = nul_terminated(request.killer);
        let victim = nul_terminated(request.victim);
        let add_message: DeathWindowAddMessageFn =
            unsafe { mem::transmute(self.module_base + DEATH_WINDOW_ADD_MESSAGE_RVA) };
        unsafe {
            add_message(
                death_window,
                killer.as_ptr().cast(),
                victim.as_ptr().cast(),
                request.killer_colour,
                request.victim_colour,
                request.weapon,
            );
        }
        Ok(())
    }

    pub(super) fn dialog_is_ready(self) -> bool {
        self.dialog().is_some()
    }

    pub(super) fn chat_is_ready(self) -> bool {
        self.chat().is_some()
    }

    pub(super) fn chat_display_mode(self) -> Result<i32, DirectClientError> {
        let chat = self.chat().ok_or(DirectClientError::NotReady)?;
        let get_mode: ChatGetModeFn =
            unsafe { mem::transmute(self.module_base + CHAT_GET_MODE_RVA) };
        let mode = unsafe { get_mode(chat) };
        matches!(mode, 0..=2)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe { read_unaligned::<i32>(game as usize + GAME_CURSOR_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(scoreboard as usize + SCOREBOARD_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    pub(super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)
    }

    pub(super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(input as usize + INPUT_ENABLED_OFFSET)
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

    fn dialog(self) -> Option<*mut c_void> {
        let dialog: *mut c_void =
            unsafe { read_pointer(self.module_base + DIALOG_SINGLETON_RVA) }?.cast();
        (!dialog.is_null() && readable_range(dialog.cast(), 1)).then_some(dialog)
    }

    fn chat(self) -> Option<*mut c_void> {
        let chat: *mut c_void =
            unsafe { read_pointer(self.module_base + CHAT_SINGLETON_RVA) }?.cast();
        (!chat.is_null() && readable_range(chat.cast(), 1)).then_some(chat)
    }

    fn scoreboard(self) -> Option<*mut c_void> {
        let scoreboard: *mut c_void =
            unsafe { read_pointer(self.module_base + SCOREBOARD_SINGLETON_RVA) }?.cast();
        (!scoreboard.is_null() && readable_range(scoreboard.cast(), 4)).then_some(scoreboard)
    }

    fn input(self) -> Option<*mut c_void> {
        let input: *mut c_void =
            unsafe { read_pointer(self.module_base + INPUT_SINGLETON_RVA) }?.cast();
        (!input.is_null() && readable_range(input.cast(), INPUT_ENABLED_OFFSET + 4))
            .then_some(input)
    }

    fn death_window(self) -> Option<*mut c_void> {
        let death_window: *mut c_void =
            unsafe { read_pointer(self.module_base + DEATH_WINDOW_SINGLETON_RVA) }?.cast();
        (!death_window.is_null() && readable_range(death_window.cast(), 1)).then_some(death_window)
    }

    pub(super) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
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

        let id = unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
            .and_then(assigned_player_id)
            .ok_or(DirectClientError::NotReady)?;

        let get_ped: LocalPlayerGetPedFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_PED_RVA) };
        let ped = unsafe { get_ped(local) };
        if ped.is_null()
            || !readable_range(
                ped.cast(),
                SAMP_PED_GAME_PED_OFFSET + mem::size_of::<usize>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let game_ped = unsafe { read_pointer(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if game_ped.is_null() || !readable_range(game_ped.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let get_name: PlayerPoolGetLocalNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_NAME_RVA) };
        let get_score: PlayerPoolGetLocalScoreFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_SCORE_RVA) };
        let get_ping: PlayerPoolGetLocalPingFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PING_RVA) };
        let get_colour: LocalPlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + LOCAL_PLAYER_GET_COLOUR_ARGB_RVA) };
        let get_health: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_HEALTH_RVA) };
        let get_armour: PedGetStatFn =
            unsafe { mem::transmute(self.module_base + PED_GET_ARMOUR_RVA) };

        let nickname =
            unsafe { bounded_c_string(get_name(pool), 256) }.ok_or(DirectClientError::NotReady)?;
        let current_vehicle =
            unsafe { read_unaligned::<u16>(local as usize + LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let vehicle_id = (current_vehicle != INVALID_ID).then_some(current_vehicle);
        let (position, velocity) = if vehicle_id.is_some() {
            (
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_INCAR_OFFSET
                            + LOCAL_PLAYER_INCAR_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_INCAR_OFFSET
                            + LOCAL_PLAYER_INCAR_SPEED_OFFSET,
                    )
                },
            )
        } else {
            (
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_ONFOOT_OFFSET
                            + LOCAL_PLAYER_ONFOOT_POSITION_OFFSET,
                    )
                },
                unsafe {
                    read_vector3(
                        local as usize
                            + LOCAL_PLAYER_ONFOOT_OFFSET
                            + LOCAL_PLAYER_ONFOOT_SPEED_OFFSET,
                    )
                },
            )
        };
        let position = position.ok_or(DirectClientError::NotReady)?;
        let velocity = velocity.ok_or(DirectClientError::NotReady)?;
        let spawned = unsafe { read_unaligned::<u32>(local as usize + LOCAL_PLAYER_ACTIVE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            != 0;
        let special_action = unsafe {
            read_unaligned::<u8>(
                local as usize
                    + LOCAL_PLAYER_ONFOOT_OFFSET
                    + LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        let animation = unsafe {
            read_unaligned::<u32>(
                local as usize + LOCAL_PLAYER_ONFOOT_OFFSET + LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET,
            )
        }
        .ok_or(DirectClientError::NotReady)?;

        Ok(LocalPlayerSnapshot {
            id,
            nickname,
            colour: unsafe { get_colour(local) },
            spawned,
            health: unsafe { get_health(ped) },
            armour: unsafe { get_armour(ped) },
            position,
            velocity,
            special_action,
            animation_id: animation as u16,
            vehicle_id,
            score: unsafe { get_score(pool) },
            ping: (unsafe { get_ping(pool) }).max(0) as u32,
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
type ChatAddEntryFn = unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type ChatGetModeFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type NetGameGetStateFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type NetGameGetPlayerPoolFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalNameFn = unsafe extern "thiscall" fn(*mut c_void) -> *const u8;
type PlayerPoolGetLocalScoreFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolGetLocalPingFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type LocalPlayerGetPedFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type LocalPlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;

unsafe fn samp_r1_pe_matches(module_base: usize) -> bool {
    let Some(nt_header) = (unsafe { pe_header(module_base) }) else {
        return false;
    };
    (unsafe { nt_header.add(8).cast::<u32>().read_unaligned() } == SAMP_R1_TIMESTAMP)
        && (unsafe { nt_header.add(40).cast::<u32>().read_unaligned() } == SAMP_R1_ENTRY_POINT)
}

unsafe fn gta_sa_10_us_matches() -> bool {
    let module = unsafe { GetModuleHandleA(c"gta_sa.exe".as_ptr().cast()) };
    if module.is_null() {
        return false;
    }
    let Some(nt_header) = (unsafe { pe_header(module as usize) }) else {
        return false;
    };
    let machine = unsafe { nt_header.add(4).cast::<u16>().read_unaligned() };
    let image_base = unsafe { nt_header.add(52).cast::<u32>().read_unaligned() };
    let image_size = unsafe { nt_header.add(80).cast::<u32>().read_unaligned() };
    let entry_point = unsafe { nt_header.add(40).cast::<u32>().read_unaligned() };
    machine == 0x014C
        && image_base == GTA_SA_10_US_IMAGE_BASE
        && image_size == GTA_SA_10_US_IMAGE_SIZE
        && entry_point == GTA_SA_10_US_ENTRY_POINT
        && unsafe { plausible_code(module as usize + entry_point as usize) }
}

unsafe fn r1_targets_match(module_base: usize) -> bool {
    // The prologue is the R1 CDialog::Show call target; verify its signature
    // and ensure every additional native entry is mapped executable code before
    // publishing the profile. A mismatch leaves direct helpers unsupported.
    let show = module_base + DIALOG_SHOW_RVA;
    code_matches(show, &DIALOG_SHOW_SIGNATURE)
        && code_matches(module_base + CHAT_ADD_ENTRY_RVA, &CHAT_ADD_ENTRY_SIGNATURE)
        && code_matches(module_base + CHAT_GET_MODE_RVA, &CHAT_GET_MODE_SIGNATURE)
        && code_matches(module_base + DIALOG_SHOW_RVA, &DIALOG_SHOW_ACTIVE_SIGNATURE)
        && code_matches(module_base + INPUT_OPEN_RVA, &INPUT_OPEN_SIGNATURE)
        && code_matches(module_base + INPUT_CLOSE_RVA, &INPUT_CLOSE_SIGNATURE)
        && code_matches(
            module_base + SCOREBOARD_CLOSE_RVA,
            &SCOREBOARD_CLOSE_SIGNATURE,
        )
        && code_matches(
            module_base + SCOREBOARD_ENABLE_RVA,
            &SCOREBOARD_ENABLE_SIGNATURE,
        )
        && code_matches(
            module_base + GAME_PROCESS_INPUT_ENABLING_RVA,
            &GAME_PROCESS_INPUT_ENABLING_SIGNATURE,
        )
        && bytes_match(
            module_base + ANIMATION_TABLE_RVA,
            &ANIMATION_TABLE_SIGNATURE,
        )
        && code_matches(
            module_base + DEATH_WINDOW_ADD_MESSAGE_RVA,
            &DEATH_WINDOW_ADD_MESSAGE_SIGNATURE,
        )
        && code_matches(
            module_base + DEATH_WINDOW_ADD_ENTRY_RVA,
            &DEATH_WINDOW_ADD_ENTRY_SIGNATURE,
        )
        && code_matches(
            module_base + NET_GAME_GET_STATE_RVA,
            &NET_GAME_GET_STATE_SIGNATURE,
        )
        && [
            NET_GAME_GET_PLAYER_POOL_RVA,
            PLAYER_POOL_GET_LOCAL_PLAYER_RVA,
            PLAYER_POOL_GET_LOCAL_NAME_RVA,
            PLAYER_POOL_GET_LOCAL_SCORE_RVA,
            PLAYER_POOL_GET_LOCAL_PING_RVA,
            LOCAL_PLAYER_GET_PED_RVA,
            LOCAL_PLAYER_GET_COLOUR_ARGB_RVA,
            PED_GET_HEALTH_RVA,
            PED_GET_ARMOUR_RVA,
        ]
        .into_iter()
        .all(|rva| unsafe { plausible_code(module_base + rva) })
}

fn code_matches(address: usize, signature: &[u8]) -> bool {
    bytes_match(address, signature)
}

fn bytes_match(address: usize, signature: &[u8]) -> bool {
    readable_range(address as *const u8, signature.len())
        && unsafe { std::slice::from_raw_parts(address as *const u8, signature.len()) } == signature
}

unsafe fn pe_header(base: usize) -> Option<*const u8> {
    let image = base as *const u8;
    if !readable_range(image, 0x40) || (unsafe { image.cast::<u16>().read_unaligned() } != 0x5A4D) {
        return None;
    }
    let nt_offset = unsafe { image.add(0x3C).cast::<u32>().read_unaligned() } as usize;
    if nt_offset > 0x1000 || !readable_range(unsafe { image.add(nt_offset) }, 84) {
        return None;
    }
    let nt_header = unsafe { image.add(nt_offset) };
    ((unsafe { nt_header.cast::<u32>().read_unaligned() } == 0x0000_4550)
        && (unsafe { nt_header.add(24).cast::<u16>().read_unaligned() } == 0x10B))
        .then_some(nt_header)
}

unsafe fn read_pointer(address: usize) -> Option<*mut u8> {
    unsafe { read_unaligned::<usize>(address) }.map(|value| value as *mut u8)
}

unsafe fn read_unaligned<T: Copy>(address: usize) -> Option<T> {
    readable_range(address as *const u8, mem::size_of::<T>())
        .then(|| unsafe { (address as *const T).read_unaligned() })
}

unsafe fn read_vector3(address: usize) -> Option<Vector3> {
    Some(Vector3 {
        x: unsafe { read_unaligned::<f32>(address) }?,
        y: unsafe { read_unaligned::<f32>(address.checked_add(4)?) }?,
        z: unsafe { read_unaligned::<f32>(address.checked_add(8)?) }?,
    })
}

fn read_r1_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<i32>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
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

unsafe fn bounded_c_string(pointer: *const u8, maximum: usize) -> Option<Vec<u8>> {
    if pointer.is_null() {
        return None;
    }
    let mut output = Vec::new();
    for index in 0..maximum {
        let byte = unsafe { read_unaligned::<u8>((pointer as usize).checked_add(index)?) }?;
        if byte == 0 {
            return Some(output);
        }
        output.push(byte);
    }
    None
}

fn nul_terminated(mut value: Vec<u8>) -> Vec<u8> {
    value.push(0);
    value
}

fn readable_range(address: *const u8, length: usize) -> bool {
    if address.is_null() || length == 0 {
        return length == 0;
    }
    let Some(end) = (address as usize).checked_add(length) else {
        return false;
    };
    let mut info = mem::MaybeUninit::<MEMORY_BASIC_INFORMATION>::zeroed();
    let queried = unsafe {
        VirtualQuery(
            address.cast(),
            info.as_mut_ptr(),
            mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if queried == 0 {
        return false;
    }
    let info = unsafe { info.assume_init() };
    let Some(region_end) = (info.BaseAddress as usize).checked_add(info.RegionSize) else {
        return false;
    };
    info.State == MEM_COMMIT
        && info.Protect & (PAGE_GUARD | PAGE_NOACCESS) == 0
        && end <= region_end
}

unsafe fn plausible_code(address: usize) -> bool {
    if !readable_range(address as *const u8, 3) {
        return false;
    }
    let bytes = unsafe { std::slice::from_raw_parts(address as *const u8, 3) };
    !bytes.iter().all(|byte| *byte == 0 || *byte == 0xCC) && !matches!(bytes[0], 0xC2 | 0xC3 | 0xCC)
}

#[cfg(test)]
mod tests {
    use super::{
        ANIMATION_TABLE_SIGNATURE, CHAT_ADD_ENTRY_SIGNATURE, CHAT_GET_MODE_SIGNATURE,
        DEATH_WINDOW_ADD_ENTRY_SIGNATURE, DEATH_WINDOW_ADD_MESSAGE_SIGNATURE, DIALOG_ACTIVE_OFFSET,
        DIALOG_SHOW_ACTIVE_SIGNATURE, DIALOG_SHOW_SIGNATURE, GAME_CURSOR_MODE_OFFSET,
        GAME_PROCESS_INPUT_ENABLING_SIGNATURE, INPUT_CLOSE_SIGNATURE, INPUT_ENABLED_OFFSET,
        INPUT_OPEN_SIGNATURE, LOCAL_PLAYER_ACTIVE_OFFSET, LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET,
        LOCAL_PLAYER_INCAR_OFFSET, LOCAL_PLAYER_INCAR_POSITION_OFFSET,
        LOCAL_PLAYER_INCAR_SPEED_OFFSET, LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET,
        LOCAL_PLAYER_ONFOOT_OFFSET, LOCAL_PLAYER_ONFOOT_POSITION_OFFSET,
        LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET, LOCAL_PLAYER_ONFOOT_SPEED_OFFSET,
        NET_GAME_GET_STATE_SIGNATURE, NET_GAME_HOST_ADDRESS_OFFSET, NET_GAME_HOSTNAME_OFFSET,
        NET_GAME_PORT_OFFSET, PLAYER_POOL_LOCAL_ID_OFFSET, SAMP_PED_GAME_PED_OFFSET,
        SCOREBOARD_CLOSE_SIGNATURE, SCOREBOARD_ENABLE_SIGNATURE, SCOREBOARD_ENABLED_OFFSET,
        assigned_player_id, nul_terminated, parse_animation_entry,
    };

    unsafe extern "C" {
        fn rak_samp_fixture_r1_onfoot_size() -> usize;
        fn rak_samp_fixture_r1_incar_size() -> usize;
        fn rak_samp_fixture_r1_local_player_prefix_size() -> usize;
        fn rak_samp_fixture_r1_local_active_offset() -> usize;
        fn rak_samp_fixture_r1_local_current_vehicle_offset() -> usize;
        fn rak_samp_fixture_r1_local_onfoot_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_position_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_speed_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_special_action_offset() -> usize;
        fn rak_samp_fixture_r1_onfoot_animation_offset() -> usize;
        fn rak_samp_fixture_r1_incar_position_offset() -> usize;
        fn rak_samp_fixture_r1_incar_speed_offset() -> usize;
        fn rak_samp_fixture_r1_ped_game_ped_offset() -> usize;
        fn rak_samp_fixture_r1_player_pool_local_id_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_host_address_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_hostname_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_port_offset() -> usize;
        fn rak_samp_fixture_r1_net_game_game_state_offset() -> usize;
        fn rak_samp_fixture_r1_game_cursor_mode_offset() -> usize;
        fn rak_samp_fixture_r1_scoreboard_enabled_offset() -> usize;
        fn rak_samp_fixture_r1_dialog_active_offset() -> usize;
        fn rak_samp_fixture_r1_input_enabled_offset() -> usize;
    }

    #[test]
    fn r1_sync_offsets_match_the_independent_x86_fixture() {
        unsafe {
            assert_eq!(rak_samp_fixture_r1_onfoot_size(), 68);
            assert_eq!(rak_samp_fixture_r1_incar_size(), 63);
            assert_eq!(rak_samp_fixture_r1_local_player_prefix_size(), 92);
            assert_eq!(
                rak_samp_fixture_r1_local_active_offset(),
                LOCAL_PLAYER_ACTIVE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_local_current_vehicle_offset(),
                LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_local_onfoot_offset(),
                LOCAL_PLAYER_ONFOOT_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_position_offset(),
                LOCAL_PLAYER_ONFOOT_POSITION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_speed_offset(),
                LOCAL_PLAYER_ONFOOT_SPEED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_special_action_offset(),
                LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_onfoot_animation_offset(),
                LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET
            );
            assert_eq!(
                LOCAL_PLAYER_ONFOOT_OFFSET + 68 + 24 + 54,
                LOCAL_PLAYER_INCAR_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_incar_position_offset(),
                LOCAL_PLAYER_INCAR_POSITION_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_incar_speed_offset(),
                LOCAL_PLAYER_INCAR_SPEED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_ped_game_ped_offset(),
                SAMP_PED_GAME_PED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_player_pool_local_id_offset(),
                PLAYER_POOL_LOCAL_ID_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_host_address_offset(),
                NET_GAME_HOST_ADDRESS_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_hostname_offset(),
                NET_GAME_HOSTNAME_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_net_game_port_offset(),
                NET_GAME_PORT_OFFSET
            );
            assert_eq!(rak_samp_fixture_r1_net_game_game_state_offset(), 0x3BD);
            assert_eq!(
                rak_samp_fixture_r1_game_cursor_mode_offset(),
                GAME_CURSOR_MODE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_scoreboard_enabled_offset(),
                SCOREBOARD_ENABLED_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_dialog_active_offset(),
                DIALOG_ACTIVE_OFFSET
            );
            assert_eq!(
                rak_samp_fixture_r1_input_enabled_offset(),
                INPUT_ENABLED_OFFSET
            );
        }
    }

    #[test]
    fn native_dialog_strings_are_terminated_only_after_copying() {
        assert_eq!(nul_terminated(b"dialog".to_vec()), b"dialog\0");
    }

    #[test]
    fn dialog_show_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(
            DIALOG_SHOW_SIGNATURE,
            [
                0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB,
                0x8B, 0xF1,
            ]
        );
    }

    #[test]
    fn dialog_and_input_state_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            DIALOG_SHOW_ACTIVE_SIGNATURE,
            [
                0x83, 0xEC, 0x10, 0x53, 0x56, 0x57, 0x8B, 0x7C, 0x24, 0x20, 0x33, 0xDB, 0x3B, 0xFB,
                0x8B, 0xF1, 0x7D, 0x17, 0x39, 0x5E, 0x28, 0x0F,
            ]
        );
        assert_eq!(
            INPUT_OPEN_SIGNATURE,
            [
                0x83, 0xEC, 0x10, 0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0,
                0x0F, 0x85,
            ]
        );
        assert_eq!(
            INPUT_CLOSE_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x8B, 0x86, 0xE0, 0x14, 0x00, 0x00, 0x85, 0xC0, 0x74, 0x39, 0x8B,
                0x4E, 0x08,
            ]
        );
    }

    #[test]
    fn animation_table_signature_and_parser_match_the_fingerprinted_r1_data() {
        assert_eq!(
            ANIMATION_TABLE_SIGNATURE,
            [
                0x41, 0x49, 0x52, 0x50, 0x4F, 0x52, 0x54, 0x3A, 0x54, 0x48, 0x52, 0x57, 0x5F, 0x42,
                0x41, 0x52, 0x4C, 0x5F, 0x54, 0x48, 0x52, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ]
        );
        assert_eq!(
            parse_animation_entry(&ANIMATION_TABLE_SIGNATURE),
            Ok(crate::runtime::AnimationSnapshot {
                name: b"AIRPORT".to_vec(),
                file: b"THRW_BARL_THRW".to_vec(),
            })
        );
    }

    #[test]
    fn chat_add_entry_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(
            CHAT_ADD_ENTRY_SIGNATURE,
            [
                0x55, 0x56, 0x8B, 0xE9, 0x57, 0x8D, 0xBD, 0x32, 0x01, 0x00, 0x00, 0x8D, 0xB5, 0x2E,
                0x02, 0x00,
            ]
        );
    }

    #[test]
    fn chat_get_mode_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(CHAT_GET_MODE_SIGNATURE, [0x8B, 0x41, 0x08, 0xC3]);
    }

    #[test]
    fn scoreboard_and_cursor_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            SCOREBOARD_CLOSE_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x74, 0x3C, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74,
                0x35, 0xC6,
            ]
        );
        assert_eq!(
            SCOREBOARD_ENABLE_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x83, 0x3E, 0x00, 0x75, 0x43, 0x8B, 0x46, 0x34, 0x85, 0xC0, 0x74,
                0x3C, 0xC6,
            ]
        );
        assert_eq!(
            GAME_PROCESS_INPUT_ENABLING_SIGNATURE,
            [
                0x56, 0x8B, 0xF1, 0x8B, 0x46, 0x55, 0x57, 0x33, 0xFF, 0x3B, 0xC7, 0x0F, 0x85, 0x07,
                0x01, 0x00,
            ]
        );
    }

    #[test]
    fn death_window_signatures_match_the_fingerprinted_r1_targets() {
        assert_eq!(
            DEATH_WINDOW_ADD_MESSAGE_SIGNATURE,
            [0xE9, 0x1B, 0xFF, 0xFF, 0xFF]
        );
        assert_eq!(
            DEATH_WINDOW_ADD_ENTRY_SIGNATURE,
            [
                0x8B, 0xD1, 0xE8, 0x49, 0xF6, 0xFF, 0xFF, 0x8A, 0x44, 0x24, 0x14, 0x8B, 0x4C, 0x24,
                0x10, 0x88,
            ]
        );
    }

    #[test]
    fn net_game_state_signature_matches_the_fingerprinted_r1_target() {
        assert_eq!(
            NET_GAME_GET_STATE_SIGNATURE,
            [0x8B, 0x81, 0xBD, 0x03, 0x00, 0x00, 0xC3]
        );
    }

    #[test]
    fn unassigned_local_player_id_is_not_a_snapshot() {
        assert_eq!(assigned_player_id(u16::MAX), None);
        assert_eq!(assigned_player_id(42), Some(42));
    }
}
