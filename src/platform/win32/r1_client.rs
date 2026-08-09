//! Private SA-MP 0.3.7 R1 client profile for direct local helpers.
//!
//! This deliberately does not share [`crate::AddressSet`]: RakNet hook offsets
//! are supported across several clients, while these object layouts and native
//! calls use approved fixed R1 offsets and validate native values at each access.

use crate::runtime::{
    AnimationSnapshot, ChatEntrySnapshot, DirectClientError, GangzoneSnapshot,
    LocalChatMessageRequest, LocalDeathMessageRequest, LocalDialogRequest, LocalDialogSnapshot,
    LocalDialogStyle, LocalPlayerSnapshot, PlayerInfoSnapshot, RemotePlayerStateSnapshot,
    ServerInfoSnapshot, TextLabelSnapshot, TextdrawSnapshot, Vector3,
};
use std::{ffi::c_void, mem, ptr};
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
    PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE, PAGE_WRITECOPY, VirtualQuery,
};

const SAMP_R1_ENTRY_POINT: u32 = 0x31DF13;

const DIALOG_SINGLETON_RVA: usize = 0x21A0B8;
const DIALOG_SHOW_RVA: usize = 0x6B9C0;
const DIALOG_CLOSE_RVA: usize = 0x6C040;
const INPUT_SINGLETON_RVA: usize = 0x21A0E8;
const INPUT_OPEN_RVA: usize = 0x657E0;
const INPUT_CLOSE_RVA: usize = 0x658E0;
const INPUT_PROCESS_RVA: usize = 0x65D30;
const DXUT_EDIT_BOX_SET_TEXT_RVA: usize = 0x80F60;
const DXUT_EDIT_BOX_GET_TEXT_RVA: usize = 0x81030;
const CHAT_SINGLETON_RVA: usize = 0x21A0E4;
const CHAT_ADD_ENTRY_RVA: usize = 0x64010;
const CHAT_GET_MODE_RVA: usize = 0x5D7A0;
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
const SCOREBOARD_SINGLETON_RVA: usize = 0x21A0B4;
const DEATH_WINDOW_SINGLETON_RVA: usize = 0x21A0EC;
const DEATH_WINDOW_ADD_MESSAGE_RVA: usize = 0x66A10;
const NET_GAME_SINGLETON_RVA: usize = 0x21A0F8;
const NET_GAME_GET_STATE_RVA: usize = 0x2E20;
const NET_GAME_GET_PLAYER_POOL_RVA: usize = 0x1160;
const NET_GAME_GET_VEHICLE_POOL_RVA: usize = 0x1170;
const NET_GAME_SHUTDOWN_FOR_RESTART_RVA: usize = 0xA060;
const PLAYER_POOL_GET_LOCAL_PLAYER_RVA: usize = 0x1A30;
const PLAYER_POOL_GET_LOCAL_SCORE_RVA: usize = 0x6A1F0;
const PLAYER_POOL_GET_LOCAL_PING_RVA: usize = 0x6A200;
const PLAYER_POOL_IS_CONNECTED_RVA: usize = 0x10B0;
const PLAYER_POOL_GET_REMOTE_PLAYER_RVA: usize = 0x10F0;
const PLAYER_POOL_IS_NPC_RVA: usize = 0xB680;
const PLAYER_POOL_GET_NAME_RVA: usize = 0x13CE0;
const PLAYER_POOL_GET_SCORE_RVA: usize = 0x6A190;
const PLAYER_POOL_GET_PING_RVA: usize = 0x6A1C0;
const PLAYER_POOL_GET_COUNT_RVA: usize = 0x10520;
const PLAYER_POOL_SET_LOCAL_PLAYER_NAME_RVA: usize = 0xB3E0;
const VEHICLE_POOL_DOES_EXIST_RVA: usize = 0x1140;
const REMOTE_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x12A00;
const REMOTE_PLAYER_SET_COLOUR_RVA: usize = 0x129D0;
const REMOTE_PLAYER_DOES_EXIST_RVA: usize = 0x1080;
const REMOTE_PLAYER_GET_STATUS_RVA: usize = 0x12BA0;
const LOCAL_PLAYER_GET_PED_RVA: usize = 0x2D60;
const LOCAL_PLAYER_GET_COLOUR_ARGB_RVA: usize = 0x3D90;
const LOCAL_PLAYER_SET_COLOUR_RVA: usize = 0x3D40;
const LOCAL_PLAYER_SET_SPECIAL_ACTION_RVA: usize = 0x30C0;
const LOCAL_PLAYER_SPAWN_RVA: usize = 0x3AD0;
const LOCAL_PLAYER_SEND_UNOCCUPIED_DATA_RVA: usize = 0x4B30;
const ONFOOT_SEND_RATE_RVA: usize = 0xEC0A8;
const INCAR_SEND_RATE_RVA: usize = 0xEC0AC;
const AIM_SEND_RATE_RVA: usize = 0xEC0B0;
const RAKPEER_SIZE: usize = 0xDDE;
const PED_GET_HEALTH_RVA: usize = 0xA6610;
const PED_GET_ARMOUR_RVA: usize = 0xA6650;
const GAME_SINGLETON_RVA: usize = 0x21A10C;
const GAME_SET_CURSOR_MODE_RVA: usize = 0x9BD30;
const GAME_PROCESS_INPUT_ENABLING_RVA: usize = 0x9BC10;
const ANIMATION_TABLE_RVA: usize = 0xF15B0;
const ANIMATION_TABLE_ENTRY_COUNT: usize = 1812;
const ANIMATION_TABLE_ENTRY_SIZE: usize = 36;
const MAX_SAMP_PLAYERS: u16 = 1004;
const MAX_SAMP_VEHICLES: u16 = 2000;
const MAX_SAMP_TEXT_LABELS: u16 = 2048;
const MAX_SAMP_TEXTDRAWS: u16 = 2304;
const MAX_SAMP_OBJECTS: u16 = 1000;
const MAX_SAMP_GANGZONES: u16 = 1024;
const MAX_SAMP_PICKUPS: u16 = 4096;
const MAX_LOCAL_PLAYER_NAME_BYTES: usize = 255;

const PLAYER_POOL_LOCAL_ID_OFFSET: usize = 0x04;
const PLAYER_POOL_LARGEST_ID_OFFSET: usize = 0x00;
const VEHICLE_POOL_NOT_EMPTY_OFFSET: usize = 0x3074;
const VEHICLE_POOL_GAME_OBJECTS_OFFSET: usize = 0x4FB4;
const OBJECT_POOL_NOT_EMPTY_OFFSET: usize = 0x04;
const OBJECT_POOL_OBJECTS_OFFSET: usize = 0xFA4;
const PICKUP_POOL_HANDLES_OFFSET: usize = 0x04;
const ENTITY_HANDLE_OFFSET: usize = 0x44;
// GTA SA 1.0 US `CPools` handle conversions (cdecl): ped/vehicle pointer to
// GTAREF. Cross-checked with DK22Pac/plugin-sdk `plugin_sa/game_sa/CPools.cpp`.
const CPOOLS_GET_PED_REF: usize = 0x54FF60;
const CPOOLS_GET_VEHICLE_REF: usize = 0x54FFC0;
const NET_GAME_POOLS_OFFSET: usize = 0x3CD;
const NET_GAME_POOLS_LABEL_POOL_OFFSET: usize = 0x0C;
const NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET: usize = 0x10;
const NET_GAME_POOLS_OBJECT_POOL_OFFSET: usize = 0x04;
const NET_GAME_POOLS_GANGZONE_POOL_OFFSET: usize = 0x08;
const LABEL_POOL_NOT_EMPTY_OFFSET: usize = 0xE800;
const LABEL_POOL_CREATE_RVA: usize = 0x11C0;
const LABEL_POOL_DELETE_RVA: usize = 0x12D0;
const LABEL_TEXT_OFFSET: usize = 0x00;
const LABEL_COLOUR_OFFSET: usize = 0x04;
const LABEL_POSITION_OFFSET: usize = 0x08;
const LABEL_DRAW_DISTANCE_OFFSET: usize = 0x14;
const LABEL_BEHIND_WALLS_OFFSET: usize = 0x18;
const LABEL_ATTACHED_PLAYER_OFFSET: usize = 0x19;
const LABEL_ATTACHED_VEHICLE_OFFSET: usize = 0x1B;
const LABEL_SIZE: usize = 0x1D;
const MAX_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
const MAX_TEXTDRAW_STRING_BYTES: usize = 1_601;
const TEXTDRAW_STRING_OFFSET: usize = 801;
const TEXTDRAW_POOL_NOT_EMPTY_OFFSET: usize = 0;
const TEXTDRAW_POOL_OBJECTS_OFFSET: usize = 0x2400;
const TEXTDRAW_POOL_DELETE_RVA: usize = 0x1AD00;
const GANGZONE_POOL_NOT_EMPTY_OFFSET: usize = 0x1000;
const GANGZONE_LEFT_OFFSET: usize = 0x00;
const GANGZONE_BOTTOM_OFFSET: usize = 0x04;
const GANGZONE_RIGHT_OFFSET: usize = 0x08;
const GANGZONE_TOP_OFFSET: usize = 0x0C;
const GANGZONE_COLOUR_OFFSET: usize = 0x10;
const GANGZONE_ALTERNATE_COLOUR_OFFSET: usize = 0x14;
const REMOTE_PLAYER_SPECIAL_ACTION_OFFSET: usize = 0xBB;
const REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET: usize = 0x1B8;
const REMOTE_PLAYER_REPORTED_HEALTH_OFFSET: usize = 0x1BC;
const REMOTE_PLAYER_ANIMATION_OFFSET: usize = 0x1C0;
const REMOTE_PLAYER_STATE_SIZE: usize = REMOTE_PLAYER_ANIMATION_OFFSET + 4;
// These packed CNetGame fields are cross-checked by the independently written
// fixture. `GetGameState`'s signed R1 target reads offset 0x3BD from this same
// layout, which anchors the packed field sequence.
const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x20;
const NET_GAME_HOSTNAME_OFFSET: usize = 0x121;
const NET_GAME_PORT_OFFSET: usize = 0x225;
const NET_GAME_GAME_STATE_OFFSET: usize = 0x3BD;
#[cfg(test)]
const NET_GAME_SERVER_SETTINGS_OFFSET: usize = 0x3C5;
const NET_GAME_POOLS_PICKUP_POOL_OFFSET: usize = 0x20;
const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
const RAK_CLIENT_DISCONNECT_VTABLE_SLOT: usize = 2;
const SCOREBOARD_ENABLED_OFFSET: usize = 0x00;
const GAME_CURSOR_MODE_OFFSET: usize = 0x55;
const DIALOG_ACTIVE_OFFSET: usize = 0x28;
const DIALOG_TYPE_OFFSET: usize = 0x2C;
const DIALOG_ID_OFFSET: usize = 0x30;
const DIALOG_LISTBOX_OFFSET: usize = 0x20;
const DIALOG_EDITBOX_OFFSET: usize = 0x24;
const DIALOG_TEXT_OFFSET: usize = 0x34;
const DIALOG_CAPTION_OFFSET: usize = 0x40;
const DIALOG_CAPTION_CAPACITY: usize = 65;
const DIALOG_SERVER_SIDE_OFFSET: usize = 0x81;
const DXUT_LISTBOX_SELECTED_OFFSET: usize = 0x143;
const DXUT_LISTBOX_ITEMS_OFFSET: usize = 0x14C;
const DXUT_LISTBOX_ITEM_COUNT_OFFSET: usize = 0x150;
const DXUT_LISTBOX_ITEM_TEXT_OFFSET: usize = 0x00;
const DXUT_LISTBOX_ITEM_TEXT_CAPACITY: usize = 256;
#[cfg(test)]
const DXUT_LISTBOX_ITEM_DATA_OFFSET: usize = 0x100;
#[cfg(test)]
const DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET: usize = 0x104;
#[cfg(test)]
const DXUT_LISTBOX_ITEM_VISIBLE_OFFSET: usize = 0x114;
#[cfg(test)]
const DXUT_LISTBOX_ITEM_SIZE: usize = 0x118;
const MAX_DIALOG_TEXT_BYTES: usize = 4_096;
const MAX_DIALOG_EDITBOX_TEXT_BYTES: usize = 128;
const MAX_DIALOG_LISTBOX_ITEMS: usize = 100;
const INPUT_ENABLED_OFFSET: usize = 0x14E0;
const INPUT_EDIT_BOX_OFFSET: usize = 0x08;
const MAX_CHAT_INPUT_TEXT_BYTES: usize = 128;
const TEXTDRAW_DATA_OFFSET: usize = 0x963;
const TEXTDRAW_LETTER_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET;
const TEXTDRAW_LETTER_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x04;
const TEXTDRAW_LETTER_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x08;
const TEXTDRAW_ALIGN_CENTER_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0D;
const TEXTDRAW_BOX_ENABLED_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0E;
const TEXTDRAW_BOX_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0F;
const TEXTDRAW_BOX_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x13;
const TEXTDRAW_BOX_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x17;
const TEXTDRAW_PROPORTIONAL_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1B;
const TEXTDRAW_BACKGROUND_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1C;
const TEXTDRAW_SHADOW_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x20;
const TEXTDRAW_OUTLINE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x21;
const TEXTDRAW_ALIGN_LEFT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x22;
const TEXTDRAW_ALIGN_RIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x23;
const TEXTDRAW_STYLE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x24;
const TEXTDRAW_X_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x28;
const TEXTDRAW_Y_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x2C;
const TEXTDRAW_MODEL_ID_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x45;
const TEXTDRAW_ROTATION_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x47;
const TEXTDRAW_ZOOM_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x53;
const TEXTDRAW_MODEL_COLOUR1_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x57;
const TEXTDRAW_MODEL_COLOUR2_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x59;

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

/// Rust mirror of the native `DXUTComboBoxItem` layout declared by SF.lua at
/// the pinned commit (`SFlua/cdef/dxut.lua`) and asserted against the
/// independent C++ fixture using the real windef `RECT`:
///
/// ```c
/// struct DXUTComboBoxItem {
///     char   strText[256];
///     void*  pData;
///     SCRect rcActive;   // == RECT from windef.h
///     bool   bVisible;
/// };
/// ```
///
/// The host reads only `str_text`; the remaining fields pin the overall
/// default-aligned packing so a future consumer cannot silently disagree with
/// the fixture. This type never crosses the plugin ABI.
#[cfg(test)]
#[repr(C)]
#[derive(Clone, Copy)]
struct NativeDxutComboBoxItem {
    str_text: [u8; DXUT_LISTBOX_ITEM_TEXT_CAPACITY],
    data: *mut c_void,
    active_rect: windows_sys::Win32::Foundation::RECT,
    visible: bool,
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

    /// Invokes R1 `CDialog::Close` with one response-button selection.
    pub(super) fn close_dialog(self, button: u8) -> Result<(), DirectClientError> {
        if button > 1 {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let close: DialogCloseFn = unsafe { mem::transmute(self.module_base + DIALOG_CLOSE_RVA) };
        unsafe { close(dialog, button) };
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

    /// Replaces one fixed R1 chat-history entry on the game thread.
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
            ptr::write_bytes(
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                0,
                CHAT_ENTRY_PREFIX_CAPACITY,
            );
            ptr::write_bytes(
                entry.add(CHAT_ENTRY_TEXT_OFFSET),
                0,
                CHAT_ENTRY_TEXT_CAPACITY,
            );
            ptr::copy_nonoverlapping(
                prefix.as_ptr(),
                entry.add(CHAT_ENTRY_PREFIX_OFFSET),
                prefix.len(),
            );
            ptr::copy_nonoverlapping(text.as_ptr(), entry.add(CHAT_ENTRY_TEXT_OFFSET), text.len());
            ptr::write_unaligned(
                entry.add(CHAT_ENTRY_TEXT_COLOUR_OFFSET).cast::<u32>(),
                text_colour,
            );
            ptr::write_unaligned(
                entry.add(CHAT_ENTRY_PREFIX_COLOUR_OFFSET).cast::<u32>(),
                prefix_colour,
            );
        }
        Ok(())
    }

    /// Copies one fixed R1 chat-history entry on the game thread.
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

    /// Writes one established R1 `CChat::m_nMode` value from the game-thread
    /// command pump.
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
        unsafe { ptr::write_unaligned(field, mode) };
        Ok(())
    }

    pub(super) fn cursor_mode(self) -> Result<i32, DirectClientError> {
        let game = self.game().ok_or(DirectClientError::NotReady)?;
        let mode = unsafe { read_unaligned::<i32>(game as usize + GAME_CURSOR_MODE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        matches!(mode, 0..=4)
            .then_some(mode)
            .ok_or(DirectClientError::NotReady)
    }

    /// Invokes the validated R1 cursor-mode transition from the game-thread
    /// command pump.
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

    /// Implements SF.lua's R1 cursor toggle, including input re-enabling when
    /// the cursor is hidden.
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

    pub(super) fn scoreboard_is_open(self) -> Result<bool, DirectClientError> {
        let scoreboard = self.scoreboard().ok_or(DirectClientError::NotReady)?;
        match unsafe { read_unaligned::<i32>(scoreboard as usize + SCOREBOARD_ENABLED_OFFSET) } {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Writes the R1 scoreboard-enabled field from the game-thread command
    /// pump after proving that the complete native field remains writable.
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
        unsafe { ptr::write_unaligned(field, i32::from(open)) };
        Ok(())
    }

    pub(super) fn dialog_is_active(self) -> Result<bool, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)
    }

    /// Sets whether the active R1 dialog is client-side. The native field has
    /// inverse semantics: it stores whether the dialog is server-side.
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
        unsafe { ptr::write_unaligned(field, i32::from(!client_side)) };
        Ok(())
    }

    /// Writes the selected index of an active R1 list dialog on the game thread.
    pub(super) fn set_dialog_selected_item(self, selected: i32) -> Result<(), DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            dialog.cast(),
            DIALOG_LISTBOX_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .filter(|value| *value != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (listbox + DXUT_LISTBOX_SELECTED_OFFSET) as *mut i32;
        if !writable_range(field.cast(), mem::size_of::<i32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, selected) };
        Ok(())
    }

    /// Copies bounded metadata and dynamic text from an active R1 dialog on
    /// the game thread. All text and item strings are bounded copies; no
    /// native or DXUT pointer crosses this boundary.
    pub(super) fn dialog_state(self) -> Result<Option<LocalDialogSnapshot>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            dialog.cast(),
            DIALOG_SERVER_SIDE_OFFSET + mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(dialog as usize + DIALOG_ACTIVE_OFFSET)? {
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
        let server_side = read_r1_bool(dialog as usize + DIALOG_SERVER_SIDE_OFFSET)?;
        let text = self.dialog_text()?;
        let editbox_text = self.dialog_editbox_text()?;
        let listbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_LISTBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let (selected_item, list_item_count, listbox_items) = if listbox == 0 {
            (None, None, Vec::new())
        } else {
            let selected = (listbox + DXUT_LISTBOX_SELECTED_OFFSET) as *const i32;
            let item_count = (listbox + DXUT_LISTBOX_ITEM_COUNT_OFFSET) as *const i32;
            if !readable_range(selected.cast(), mem::size_of::<i32>())
                || !readable_range(item_count.cast(), mem::size_of::<i32>())
            {
                return Err(DirectClientError::NotReady);
            }
            let selected_item = unsafe { read_unaligned::<i32>(selected as usize) };
            let list_item_count = unsafe { read_unaligned::<i32>(item_count as usize) }
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

    /// Copies the bounded R1 dialog body text on the game thread. The native
    /// `m_szText` pointer is validated and read through a bounded copy.
    pub(super) fn dialog_text(self) -> Result<Vec<u8>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let text = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_TEXT_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if text == 0 {
            return Ok(Vec::new());
        }
        unsafe { bounded_c_string(text as *const u8, MAX_DIALOG_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)
    }

    /// Copies the bounded R1 dialog editbox text on the game thread. Dialogs
    /// without an editbox report `None` rather than failing the snapshot.
    pub(super) fn dialog_editbox_text(self) -> Result<Option<Vec<u8>>, DirectClientError> {
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_EDITBOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if editbox == 0 {
            return Ok(None);
        }
        if !readable_range(editbox as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_text: DxutEditBoxGetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_GET_TEXT_RVA) };
        unsafe {
            bounded_c_string(
                get_text(editbox as *mut c_void).cast(),
                MAX_DIALOG_EDITBOX_TEXT_BYTES + 1,
            )
        }
        .map(Some)
        .ok_or(DirectClientError::NotReady)
    }

    /// Replaces the R1 dialog editbox text through its native DXUT method.
    pub(super) fn set_dialog_editbox_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        if text.len() > MAX_DIALOG_EDITBOX_TEXT_BYTES || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let dialog = self.dialog().ok_or(DirectClientError::NotReady)?;
        let editbox = unsafe { read_unaligned::<usize>(dialog as usize + DIALOG_EDITBOX_OFFSET) }
            .filter(|editbox| *editbox != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(editbox as *const u8, 1) {
            return Err(DirectClientError::NotReady);
        }
        let text = nul_terminated(text.to_vec());
        let set_text: DxutEditBoxSetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_SET_TEXT_RVA) };
        unsafe { set_text(editbox as *mut c_void, text.as_ptr().cast(), false) };
        Ok(())
    }

    /// Copies one bounded R1 dialog listbox item string on the game thread.
    pub(super) fn dialog_listbox_item_text(
        self,
        index: usize,
    ) -> Result<Vec<u8>, DirectClientError> {
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
        unsafe {
            bounded_dxut_listbox_item_text((item + DXUT_LISTBOX_ITEM_TEXT_OFFSET) as *const u8)
        }
        .ok_or(DirectClientError::NotReady)
    }

    pub(super) fn chat_input_is_active(self) -> Result<bool, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        read_r1_bool(input as usize + INPUT_ENABLED_OFFSET)
    }

    /// Updates the R1 chat edit box through its native DXUT method.
    pub(super) fn set_chat_input_text(self, text: &[u8]) -> Result<(), DirectClientError> {
        if text.len() > MAX_CHAT_INPUT_TEXT_BYTES || text.contains(&0) {
            return Err(DirectClientError::NotReady);
        }
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let edit_box: *mut c_void = unsafe { read_pointer(input as usize + INPUT_EDIT_BOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if edit_box.is_null() || !readable_range(edit_box.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let text = nul_terminated(text.to_vec());
        let set_text: DxutEditBoxSetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_SET_TEXT_RVA) };
        unsafe { set_text(edit_box, text.as_ptr().cast(), false) };
        Ok(())
    }

    /// Copies the current R1 chat edit-box text while running on the game
    /// thread; callers publish the owned bytes through the cache.
    pub(super) fn chat_input_text(self) -> Result<Vec<u8>, DirectClientError> {
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let edit_box: *mut c_void = unsafe { read_pointer(input as usize + INPUT_EDIT_BOX_OFFSET) }
            .ok_or(DirectClientError::NotReady)?
            .cast();
        if edit_box.is_null() || !readable_range(edit_box.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_text: DxutEditBoxGetTextFn =
            unsafe { mem::transmute(self.module_base + DXUT_EDIT_BOX_GET_TEXT_RVA) };
        unsafe { bounded_c_string(get_text(edit_box).cast(), MAX_CHAT_INPUT_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)
    }

    /// Opens or closes R1's chat input through its native transition methods.
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

    /// Replaces the R1 chat-input text and dispatches its native command path.
    pub(super) fn process_chat_input(self, text: &[u8]) -> Result<(), DirectClientError> {
        self.set_chat_input_text(text)?;
        let input = self.input().ok_or(DirectClientError::NotReady)?;
        let process: InputNoArgFn = unsafe { mem::transmute(self.module_base + INPUT_PROCESS_RVA) };
        unsafe { process(input) };
        Ok(())
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

    /// Copies one remote player through bounded fixed-offset R1 accessors.
    /// It is invoked only by the host's game-thread pump; no client pointer
    /// survives this method.
    pub(super) fn player_info(
        self,
        id: u16,
    ) -> Result<Option<PlayerInfoSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
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
        if remote.is_null() || !readable_range(remote.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let is_npc: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_NPC_RVA) };
        let get_name: PlayerPoolGetPlayerNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
        let get_score: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_SCORE_RVA) };
        let get_ping: PlayerPoolGetPlayerStatFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_PING_RVA) };
        let get_colour: RemotePlayerGetColourArgbFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_COLOUR_ARGB_RVA) };
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        let get_status: RemotePlayerGetStatusFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_GET_STATUS_RVA) };
        let is_npc = match unsafe { is_npc(pool, id) } {
            0 => false,
            1 => true,
            _ => return Err(DirectClientError::NotReady),
        };
        let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
            .filter(|name| !name.is_empty())
            .ok_or(DirectClientError::NotReady)?;

        Ok(Some(PlayerInfoSnapshot {
            id,
            defined: match unsafe { does_exist(remote) } {
                0 => false,
                1 => true,
                _ => return Err(DirectClientError::NotReady),
            },
            paused: unsafe { get_status(remote) } == 0,
            nickname,
            is_local: false,
            is_npc,
            colour: unsafe { get_colour(remote) },
            score: unsafe { get_score(pool, id) },
            ping: (unsafe { get_ping(pool, id) }).max(0) as u32,
        }))
    }

    /// Copies the volatile fields maintained by R1's remote-player update and
    /// process paths. This runs only on the host game-thread pump.
    pub(super) fn remote_player_state(
        self,
        id: u16,
    ) -> Result<Option<RemotePlayerStateSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
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
        if remote.is_null() || !readable_range(remote.cast(), REMOTE_PLAYER_STATE_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: RemotePlayerDoesExistFn =
            unsafe { mem::transmute(self.module_base + REMOTE_PLAYER_DOES_EXIST_RVA) };
        match unsafe { does_exist(remote) } {
            0 => return Ok(None),
            1 => {}
            _ => return Err(DirectClientError::NotReady),
        }
        let health = unsafe {
            read_unaligned::<f32>(remote as usize + REMOTE_PLAYER_REPORTED_HEALTH_OFFSET)
        }
        .filter(|value| value.is_finite())
        .ok_or(DirectClientError::NotReady)?;
        let armour = unsafe {
            read_unaligned::<f32>(remote as usize + REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET)
        }
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

    /// Reads both R1 `CPlayerPool::GetCount` modes on the game-thread pump.
    /// The resulting scalar pair is published by the host; no pool layout or
    /// pointer crosses this private profile boundary.
    pub(super) fn player_counts(self) -> Result<(u16, u16), DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let get_count: PlayerPoolGetCountFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_COUNT_RVA) };
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

    pub(super) fn player_max_id(self) -> Result<u16, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null()
            || !readable_range(
                pool.cast(),
                PLAYER_POOL_LARGEST_ID_OFFSET + mem::size_of::<i32>(),
            )
        {
            return Err(DirectClientError::NotReady);
        }
        let largest_id =
            unsafe { read_unaligned::<i32>(pool as usize + PLAYER_POOL_LARGEST_ID_OFFSET) }
                .and_then(|id| u16::try_from(id).ok())
                .filter(|id| *id < MAX_SAMP_PLAYERS)
                .ok_or(DirectClientError::NotReady)?;
        Ok(largest_id)
    }

    /// Reads one R1 vehicle-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(super) fn vehicle_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_vehicle_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_VEHICLE_POOL_RVA) };
        let pool = unsafe { get_vehicle_pool(net_game) };
        let checked_len =
            VEHICLE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if pool.is_null() || !readable_range(pool.cast(), checked_len) {
            return Err(DirectClientError::NotReady);
        }
        let does_exist: PlayerPoolPlayerBooleanFn =
            unsafe { mem::transmute(self.module_base + VEHICLE_POOL_DOES_EXIST_RVA) };
        match unsafe { does_exist(pool, id) } {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DirectClientError::NotReady),
        }
    }

    /// Reads one R1 3D text-label-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(super) fn text_label_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            LABEL_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(pool + LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>())
    }

    /// Invokes the documented R1 label-pool delete method on the game thread.
    pub(super) fn delete_text_label(self, id: u16) -> Result<(), DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
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
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let delete: LabelPoolDeleteFn =
            unsafe { mem::transmute(self.module_base + LABEL_POOL_DELETE_RVA) };
        if unsafe { delete(pool, id) } == 0 {
            return Err(DirectClientError::NotReady);
        }
        Ok(())
    }

    /// Invokes the documented R1 label-pool create method on the game thread.
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
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let text = nul_terminated(text.to_vec());
        let create: LabelPoolCreateFn =
            unsafe { mem::transmute(self.module_base + LABEL_POOL_CREATE_RVA) };
        unsafe {
            create(
                pool,
                id,
                text.as_ptr(),
                colour,
                NativeVector3::from(position),
                draw_distance,
                u8::from(behind_walls),
                attached_player_id,
                attached_vehicle_id,
            );
        }
        Ok(())
    }

    /// Copies one R1 3D text-label record on the game-thread pump. The native
    /// string allocation is read only after its matching pool flag is true,
    /// bounded by the R1 encoded-string limit, and copied before this method
    /// returns. No native pointer crosses the private profile boundary.
    pub(super) fn text_label(
        self,
        id: u16,
    ) -> Result<Option<TextLabelSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_TEXT_LABELS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_LABEL_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_LABEL_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            LABEL_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + LABEL_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let label = pool + usize::from(id) * LABEL_SIZE;
        if !readable_range(label as *const u8, LABEL_SIZE) {
            return Err(DirectClientError::NotReady);
        }
        let text = unsafe { read_unaligned::<usize>(label + LABEL_TEXT_OFFSET) }
            .filter(|text| *text != 0)
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe { bounded_c_string(text as *const u8, MAX_TEXT_LABEL_TEXT_BYTES + 1) }
            .ok_or(DirectClientError::NotReady)?;
        let colour = unsafe { read_unaligned::<u32>(label + LABEL_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let position = unsafe { read_vector3(label + LABEL_POSITION_OFFSET) }
            .filter(|position| {
                position.x.is_finite() && position.y.is_finite() && position.z.is_finite()
            })
            .ok_or(DirectClientError::NotReady)?;
        let draw_distance = unsafe { read_unaligned::<f32>(label + LABEL_DRAW_DISTANCE_OFFSET) }
            .filter(|draw_distance| draw_distance.is_finite())
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
            colour,
            position,
            draw_distance,
            behind_walls,
            attached_player_id: (attached_player != u16::MAX).then_some(attached_player),
            attached_vehicle_id: (attached_vehicle != u16::MAX).then_some(attached_vehicle),
        }))
    }

    /// Reads one R1 textdraw-pool existence flag on the game-thread pump.
    /// The raw pool index covers the 2,048 global and 256 local slots. Only
    /// the copied boolean crosses the private profile boundary.
    pub(super) fn textdraw_exists(self, pool_index: u16) -> Result<bool, DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )
    }

    /// Invokes the documented R1 textdraw-pool deletion method on the game
    /// thread after resolving the private pool pointer.
    pub(super) fn delete_textdraw(self, pool_index: u16) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)? as *mut c_void;
        if !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let delete: TextdrawPoolDeleteFn =
            unsafe { mem::transmute(self.module_base + TEXTDRAW_POOL_DELETE_RVA) };
        unsafe { delete(pool, pool_index) };
        Ok(())
    }

    /// Updates one existing R1 textdraw's finite screen position on the game
    /// thread. The fixture-backed pool, object, and two scalar fields are
    /// validated before the direct write.
    pub(super) fn set_textdraw_position(
        self,
        pool_index: u16,
        x: f32,
        y: f32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !x.is_finite() || !y.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_X_OFFSET) as *mut f32;
        if !writable_range(field.cast(), 2 * mem::size_of::<f32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field, x);
            ptr::write_unaligned(field.add(1), y);
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's finite letter dimensions and colour.
    pub(super) fn set_textdraw_letter_style(
        self,
        pool_index: u16,
        width: f32,
        height: f32,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_LETTER_WIDTH_OFFSET) as *mut u8;
        if !writable_range(field, mem::size_of::<f32>() * 2 + mem::size_of::<u32>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<f32>(), width);
            ptr::write_unaligned(field.add(mem::size_of::<f32>()).cast::<f32>(), height);
            ptr::write_unaligned(field.add(mem::size_of::<f32>() * 2).cast::<u32>(), colour);
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's proportional flag on the game thread.
    pub(super) fn set_textdraw_proportional(
        self,
        pool_index: u16,
        proportional: bool,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_PROPORTIONAL_OFFSET) as *mut u8;
        if !writable_range(field, mem::size_of::<u8>()) {
            return Err(DirectClientError::NotReady);
        }
        unsafe { ptr::write_unaligned(field, u8::from(proportional)) };
        Ok(())
    }

    /// Updates one existing R1 textdraw's shadow and background colour.
    pub(super) fn set_textdraw_shadow(
        self,
        pool_index: u16,
        shadow: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) as *mut u8;
        let len = TEXTDRAW_SHADOW_OFFSET + mem::size_of::<u8>() - TEXTDRAW_BACKGROUND_COLOUR_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<u32>(), colour);
            ptr::write_unaligned(
                field.add(TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_BACKGROUND_COLOUR_OFFSET),
                shadow,
            );
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's outline and background colour.
    pub(super) fn set_textdraw_outline(
        self,
        pool_index: u16,
        outline: u8,
        colour: u32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) as *mut u8;
        let len =
            TEXTDRAW_OUTLINE_OFFSET + mem::size_of::<u8>() - TEXTDRAW_BACKGROUND_COLOUR_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<u32>(), colour);
            ptr::write_unaligned(
                field.add(TEXTDRAW_OUTLINE_OFFSET - TEXTDRAW_BACKGROUND_COLOUR_OFFSET),
                outline,
            );
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's finite box dimensions and colour.
    pub(super) fn set_textdraw_box(
        self,
        pool_index: u16,
        enabled: bool,
        colour: u32,
        width: f32,
        height: f32,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !width.is_finite() || !height.is_finite() {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_BOX_ENABLED_OFFSET) as *mut u8;
        let len = TEXTDRAW_BOX_COLOUR_OFFSET + mem::size_of::<u32>() - TEXTDRAW_BOX_ENABLED_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field, u8::from(enabled));
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<f32>(),
                width,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<f32>(),
                height,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_BOX_ENABLED_OFFSET)
                    .cast::<u32>(),
                colour,
            );
        }
        Ok(())
    }

    /// Updates one existing R1 textdraw's one-of-three alignment flags.
    pub(super) fn set_textdraw_alignment(
        self,
        pool_index: u16,
        alignment: u8,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS || !(1..=3).contains(&alignment) {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_ALIGN_CENTER_OFFSET) as *mut u8;
        let len = TEXTDRAW_ALIGN_RIGHT_OFFSET + 1 - TEXTDRAW_ALIGN_CENTER_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field, u8::from(alignment == 2));
            ptr::write_unaligned(
                field.add(TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_ALIGN_CENTER_OFFSET),
                u8::from(alignment == 1),
            );
            ptr::write_unaligned(
                field.add(TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_ALIGN_CENTER_OFFSET),
                u8::from(alignment == 3),
            );
        }
        Ok(())
    }

    /// Replaces one existing R1 textdraw's bounded display string.
    pub(super) fn set_textdraw_model_style(
        self,
        pool_index: u16,
        rotation: Vector3,
        zoom: f32,
        colour1: u16,
        colour2: u16,
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS
            || !rotation.x.is_finite()
            || !rotation.y.is_finite()
            || !rotation.z.is_finite()
            || !zoom.is_finite()
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end))
            || !read_r1_bool(
                pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET
                    + usize::from(pool_index) * mem::size_of::<i32>(),
            )?
        {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let field = (object + TEXTDRAW_ROTATION_OFFSET) as *mut u8;
        let len = TEXTDRAW_MODEL_COLOUR2_OFFSET + mem::size_of::<u16>() - TEXTDRAW_ROTATION_OFFSET;
        if !writable_range(field, len) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_unaligned(field.cast::<f32>(), rotation.x);
            ptr::write_unaligned(field.add(4).cast::<f32>(), rotation.y);
            ptr::write_unaligned(field.add(8).cast::<f32>(), rotation.z);
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_ZOOM_OFFSET - TEXTDRAW_ROTATION_OFFSET)
                    .cast::<f32>(),
                zoom,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_MODEL_COLOUR1_OFFSET - TEXTDRAW_ROTATION_OFFSET)
                    .cast::<u16>(),
                colour1,
            );
            ptr::write_unaligned(
                field
                    .add(TEXTDRAW_MODEL_COLOUR2_OFFSET - TEXTDRAW_ROTATION_OFFSET)
                    .cast::<u16>(),
                colour2,
            );
        }
        Ok(())
    }

    /// Replaces one existing R1 textdraw's bounded display string.
    pub(super) fn set_textdraw_string(
        self,
        pool_index: u16,
        text: &[u8],
    ) -> Result<(), DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS
            || text.len() > MAX_TEXTDRAW_STRING_BYTES
            || text.contains(&0)
        {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        let objects_end =
            TEXTDRAW_POOL_OBJECTS_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, flags_end.max(objects_end)) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Err(DirectClientError::NotReady);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let destination = (object + 801) as *mut u8;
        if !writable_range(destination, MAX_TEXTDRAW_STRING_BYTES + 1) {
            return Err(DirectClientError::NotReady);
        }
        unsafe {
            ptr::write_bytes(destination, 0, MAX_TEXTDRAW_STRING_BYTES + 1);
            ptr::copy_nonoverlapping(text.as_ptr(), destination, text.len());
        }
        Ok(())
    }

    /// Copies one R1 numeric textdraw record on the game-thread pump. The raw
    /// index preserves the native 2,048-global then 256-local pool order. No
    /// textdraw/pool pointer or unproven display-string buffer crosses the
    /// private profile boundary.
    pub(super) fn textdraw(
        self,
        pool_index: u16,
    ) -> Result<Option<TextdrawSnapshot>, DirectClientError> {
        if pool_index >= MAX_SAMP_TEXTDRAWS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let flags_end =
            TEXTDRAW_POOL_NOT_EMPTY_OFFSET + (usize::from(pool_index) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, flags_end) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + TEXTDRAW_POOL_NOT_EMPTY_OFFSET + usize::from(pool_index) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let object_slot =
            pool + TEXTDRAW_POOL_OBJECTS_OFFSET + usize::from(pool_index) * mem::size_of::<usize>();
        let object = unsafe { read_unaligned::<usize>(object_slot) }
            .filter(|object| *object != 0)
            .ok_or(DirectClientError::NotReady)?;
        let last_field_end = TEXTDRAW_MODEL_COLOUR2_OFFSET + mem::size_of::<u16>();
        if !readable_range(object as *const u8, last_field_end) {
            return Err(DirectClientError::NotReady);
        }
        let letter_width = unsafe { read_unaligned::<f32>(object + TEXTDRAW_LETTER_WIDTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let letter_height =
            unsafe { read_unaligned::<f32>(object + TEXTDRAW_LETTER_HEIGHT_OFFSET) }
                .filter(|value| value.is_finite())
                .ok_or(DirectClientError::NotReady)?;
        let letter_colour =
            unsafe { read_unaligned::<u32>(object + TEXTDRAW_LETTER_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let x = unsafe { read_unaligned::<f32>(object + TEXTDRAW_X_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let y = unsafe { read_unaligned::<f32>(object + TEXTDRAW_Y_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_width = unsafe { read_unaligned::<f32>(object + TEXTDRAW_BOX_WIDTH_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_height = unsafe { read_unaligned::<f32>(object + TEXTDRAW_BOX_HEIGHT_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let box_colour = unsafe { read_unaligned::<u32>(object + TEXTDRAW_BOX_COLOUR_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let background_colour =
            unsafe { read_unaligned::<u32>(object + TEXTDRAW_BACKGROUND_COLOUR_OFFSET) }
                .ok_or(DirectClientError::NotReady)?;
        let style = unsafe { read_unaligned::<i32>(object + TEXTDRAW_STYLE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let model_id = unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_ID_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        let rotation = unsafe { read_vector3(object + TEXTDRAW_ROTATION_OFFSET) }
            .filter(|value| value.x.is_finite() && value.y.is_finite() && value.z.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let zoom = unsafe { read_unaligned::<f32>(object + TEXTDRAW_ZOOM_OFFSET) }
            .filter(|value| value.is_finite())
            .ok_or(DirectClientError::NotReady)?;
        let text = unsafe {
            bounded_c_string(
                (object + TEXTDRAW_STRING_OFFSET) as *const u8,
                MAX_TEXTDRAW_STRING_BYTES + 1,
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        Ok(Some(TextdrawSnapshot {
            pool_index,
            text,
            letter_width,
            letter_height,
            letter_colour,
            x,
            y,
            shadow: unsafe { read_unaligned::<u8>(object + TEXTDRAW_SHADOW_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            outline: unsafe { read_unaligned::<u8>(object + TEXTDRAW_OUTLINE_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            background_colour,
            style,
            proportional: read_u8_bool(object + TEXTDRAW_PROPORTIONAL_OFFSET)?,
            align_left: read_u8_bool(object + TEXTDRAW_ALIGN_LEFT_OFFSET)?,
            align_center: read_u8_bool(object + TEXTDRAW_ALIGN_CENTER_OFFSET)?,
            align_right: read_u8_bool(object + TEXTDRAW_ALIGN_RIGHT_OFFSET)?,
            box_enabled: read_u8_bool(object + TEXTDRAW_BOX_ENABLED_OFFSET)?,
            box_width,
            box_height,
            box_colour,
            model_id,
            rotation,
            zoom,
            model_colour1: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR1_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
            model_colour2: unsafe { read_unaligned::<u16>(object + TEXTDRAW_MODEL_COLOUR2_OFFSET) }
                .ok_or(DirectClientError::NotReady)?,
        }))
    }

    /// Reads one R1 object-pool existence flag on the game-thread pump.
    /// Only the copied boolean crosses the private profile boundary.
    pub(super) fn object_exists(self, id: u16) -> Result<bool, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_OBJECT_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            OBJECT_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        read_r1_bool(pool + OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>())
    }

    /// Copies one R1 object-pool handle (GTAREF) on the game thread. The
    /// handle is the `SCEntity::m_handle` field of the object's SAMP wrapper.
    pub(super) fn object_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_OBJECTS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_OBJECT_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_OBJECT_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            OBJECT_POOL_OBJECTS_OFFSET + (usize::from(id) + 1) * mem::size_of::<usize>();
        if !readable_range(pool as *const u8, checked_len)
            || !read_r1_bool(
                pool + OBJECT_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
            )?
        {
            return Ok(None);
        }
        let object = unsafe {
            read_unaligned::<usize>(
                pool + OBJECT_POOL_OBJECTS_OFFSET + usize::from(id) * mem::size_of::<usize>(),
            )
        }
        .filter(|object| *object != 0)
        .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            object as *const u8,
            ENTITY_HANDLE_OFFSET + mem::size_of::<i32>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let handle = unsafe { read_unaligned::<i32>(object + ENTITY_HANDLE_OFFSET) }
            .ok_or(DirectClientError::NotReady)?;
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 object pool for a matching GTAREF on the game thread.
    pub(super) fn object_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_OBJECTS {
            if self.object_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one R1 pickup-pool handle (GTAREF) on the game thread.
    pub(super) fn pickup_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
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
        let checked_len =
            PICKUP_POOL_HANDLES_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        let handle = unsafe {
            read_unaligned::<i32>(
                pool + PICKUP_POOL_HANDLES_OFFSET + usize::from(id) * mem::size_of::<i32>(),
            )
        }
        .ok_or(DirectClientError::NotReady)?;
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 pickup pool for a matching GTAREF on the game thread.
    pub(super) fn pickup_id_by_handle(self, handle: i32) -> Result<Option<u16>, DirectClientError> {
        for id in 0..MAX_SAMP_PICKUPS {
            if self.pickup_handle(id)? == Some(handle) {
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Copies one R1 vehicle GTA handle (GTAREF) on the game thread by
    /// converting the validated `m_pGameObject` pointer through the fixed
    /// GTA SA `CPools::GetVehicleRef` target.
    pub(super) fn vehicle_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_VEHICLES {
            return Err(DirectClientError::NotReady);
        }
        let pool = self.vehicle_pool()?;
        let checked_len =
            VEHICLE_POOL_GAME_OBJECTS_OFFSET + (usize::from(id) + 1) * mem::size_of::<usize>();
        if !readable_range(pool.cast(), checked_len)
            || !read_r1_bool(
                pool as usize
                    + VEHICLE_POOL_NOT_EMPTY_OFFSET
                    + usize::from(id) * mem::size_of::<i32>(),
            )?
        {
            return Ok(None);
        }
        let game_object = unsafe {
            read_unaligned::<usize>(
                pool as usize
                    + VEHICLE_POOL_GAME_OBJECTS_OFFSET
                    + usize::from(id) * mem::size_of::<usize>(),
            )
        }
        .filter(|game_object| *game_object != 0)
        .ok_or(DirectClientError::NotReady)?;
        let get_vehicle_ref: CpoolRefFn = unsafe { mem::transmute(CPOOLS_GET_VEHICLE_REF) };
        let handle = unsafe { get_vehicle_ref(game_object as *mut c_void) };
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 vehicle pool for a matching GTA handle on the game thread.
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

    /// Copies one R1 player-pool GTA ped handle (GTAREF) on the game thread.
    ///
    /// The local player resolves through `CLocalPlayer::GetPed` → `m_pGamePed`
    /// and the fixed GTA SA `CPools::GetPedRef`; remote players resolve through
    /// `CRemotePlayer.m_pPed` → `m_pGamePed` → `GetPedRef`.
    pub(super) fn player_ped_handle(self, id: u16) -> Result<Option<i32>, DirectClientError> {
        if id >= MAX_SAMP_PLAYERS {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id)
                .ok_or(DirectClientError::NotReady)?;
        let game_ped = if id == local_id {
            let get_local_player: PlayerPoolGetLocalPlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_LOCAL_PLAYER_RVA) };
            let local = unsafe { get_local_player(pool) };
            if local.is_null() || !readable_range(local.cast(), 1) {
                return Err(DirectClientError::NotReady);
            }
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
            unsafe { read_unaligned::<usize>(ped as usize + SAMP_PED_GAME_PED_OFFSET) }
        } else {
            let is_connected: PlayerPoolPlayerBooleanFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_IS_CONNECTED_RVA) };
            if unsafe { is_connected(pool, id) } != 1 {
                return Ok(None);
            }
            let get_player: PlayerPoolGetRemotePlayerFn =
                unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_REMOTE_PLAYER_RVA) };
            let remote = unsafe { get_player(pool, id) };
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
        };
        let game_ped = game_ped
            .filter(|game_ped| *game_ped != 0)
            .ok_or(DirectClientError::NotReady)?;
        let get_ped_ref: CpoolRefFn = unsafe { mem::transmute(CPOOLS_GET_PED_REF) };
        let handle = unsafe { get_ped_ref(game_ped as *mut c_void) };
        if handle != 0 {
            Ok(Some(handle))
        } else {
            Ok(None)
        }
    }

    /// Scans the R1 player pool for a matching GTA ped handle on the game
    /// thread. The local player is checked first, matching SF.lua's
    /// `sampGetPlayerIdByCharHandle`.
    pub(super) fn player_id_by_ped_handle(
        self,
        handle: i32,
    ) -> Result<Option<u16>, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }
        let local_id =
            unsafe { read_unaligned::<u16>(pool as usize + PLAYER_POOL_LOCAL_ID_OFFSET) }
                .and_then(assigned_player_id)
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

    /// Copies one R1 gangzone record on the game-thread pump. No client or
    /// GTA pointer crosses the private profile boundary.
    pub(super) fn gangzone(self, id: u16) -> Result<Option<GangzoneSnapshot>, DirectClientError> {
        if id >= MAX_SAMP_GANGZONES {
            return Err(DirectClientError::NotReady);
        }
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let pools = unsafe { read_unaligned::<usize>(net_game as usize + NET_GAME_POOLS_OFFSET) }
            .filter(|pools| *pools != 0)
            .ok_or(DirectClientError::NotReady)?;
        if !readable_range(
            pools as *const u8,
            NET_GAME_POOLS_GANGZONE_POOL_OFFSET + mem::size_of::<usize>(),
        ) {
            return Err(DirectClientError::NotReady);
        }
        let pool = unsafe { read_unaligned::<usize>(pools + NET_GAME_POOLS_GANGZONE_POOL_OFFSET) }
            .filter(|pool| *pool != 0)
            .ok_or(DirectClientError::NotReady)?;
        let checked_len =
            GANGZONE_POOL_NOT_EMPTY_OFFSET + (usize::from(id) + 1) * mem::size_of::<i32>();
        if !readable_range(pool as *const u8, checked_len) {
            return Err(DirectClientError::NotReady);
        }
        if !read_r1_bool(
            pool + GANGZONE_POOL_NOT_EMPTY_OFFSET + usize::from(id) * mem::size_of::<i32>(),
        )? {
            return Ok(None);
        }
        let gangzone =
            unsafe { read_unaligned::<usize>(pool + usize::from(id) * mem::size_of::<usize>()) }
                .filter(|gangzone| *gangzone != 0)
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

    pub(super) fn local_player(self) -> Result<LocalPlayerSnapshot, DirectClientError> {
        let net_game = self.net_game().ok_or(DirectClientError::NotReady)?;
        let get_player_pool: NetGameGetPlayerPoolFn =
            unsafe { mem::transmute(self.module_base + NET_GAME_GET_PLAYER_POOL_RVA) };
        let pool = unsafe { get_player_pool(net_game) };
        if pool.is_null() || !readable_range(pool.cast(), 1) {
            return Err(DirectClientError::NotReady);
        }

        let local = self.local_player_address()?;

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

        let get_name: PlayerPoolGetPlayerNameFn =
            unsafe { mem::transmute(self.module_base + PLAYER_POOL_GET_NAME_RVA) };
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

        let nickname = unsafe { bounded_c_string(get_name(pool, id), 256) }
            .ok_or(DirectClientError::NotReady)?;
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
type DialogCloseFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type ChatAddEntryFn = unsafe extern "thiscall" fn(*mut c_void, i32, *const i8, *const i8, u32, u32);
type InputNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type DxutEditBoxSetTextFn = unsafe extern "thiscall" fn(*mut c_void, *const i8, bool);
type DxutEditBoxGetTextFn = unsafe extern "thiscall" fn(*mut c_void) -> *const i8;
type ChatGetModeFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type DeathWindowAddMessageFn =
    unsafe extern "thiscall" fn(*mut c_void, *const i8, *const i8, u32, u32, u8);
type NetGameGetStateFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type NetGameGetPlayerPoolFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type NetGameNoArgFn = unsafe extern "thiscall" fn(*mut c_void);
type RakClientDisconnectFn = unsafe extern "thiscall" fn(*mut c_void, u32, u8);
type TextdrawPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16);
type LabelPoolCreateFn =
    unsafe extern "thiscall" fn(*mut c_void, u16, *const u8, u32, NativeVector3, f32, u8, u16, u16);
type LabelPoolDeleteFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;

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
type PlayerPoolGetLocalPlayerFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type PlayerPoolGetLocalScoreFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolGetLocalPingFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PlayerPoolPlayerBooleanFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type PlayerPoolGetRemotePlayerFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *mut c_void;
type PlayerPoolGetPlayerNameFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> *const u8;
type PlayerPoolGetPlayerStatFn = unsafe extern "thiscall" fn(*mut c_void, u16) -> i32;
type PlayerPoolGetCountFn = unsafe extern "thiscall" fn(*mut c_void, i32) -> i32;
type PlayerPoolSetLocalPlayerNameFn = unsafe extern "thiscall" fn(*mut c_void, *const i8);
type LocalPlayerGetPedFn = unsafe extern "thiscall" fn(*mut c_void) -> *mut c_void;
type LocalPlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type LocalPlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type LocalPlayerSetSpecialActionFn = unsafe extern "thiscall" fn(*mut c_void, u8);
type LocalPlayerSpawnFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type LocalPlayerSendUnoccupiedDataFn = unsafe extern "thiscall" fn(*mut c_void, u16, i32);
type GameSetCursorModeFn = unsafe extern "thiscall" fn(*mut c_void, i32, i32);
type GameProcessInputEnablingFn = unsafe extern "thiscall" fn(*mut c_void);
type RemotePlayerGetColourArgbFn = unsafe extern "thiscall" fn(*mut c_void) -> u32;
type RemotePlayerSetColourFn = unsafe extern "thiscall" fn(*mut c_void, u32);
type RemotePlayerDoesExistFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type RemotePlayerGetStatusFn = unsafe extern "thiscall" fn(*mut c_void) -> i32;
type PedGetStatFn = unsafe extern "thiscall" fn(*mut c_void) -> f32;
type CpoolRefFn = unsafe extern "cdecl" fn(*mut c_void) -> i32;

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

const fn is_r1_game_state(state: i32) -> bool {
    matches!(state, 0 | 9 | 13 | 14 | 15 | 18)
}

fn read_u8_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<u8>(address) } {
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

unsafe fn bounded_dxut_listbox_item_text(pointer: *const u8) -> Option<Vec<u8>> {
    unsafe { bounded_c_string(pointer, DXUT_LISTBOX_ITEM_TEXT_CAPACITY) }
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

fn writable_range(address: *const u8, length: usize) -> bool {
    if !readable_range(address, length) {
        return false;
    }
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
    let protection = unsafe { info.assume_init() }.Protect & 0xFF;
    matches!(
        protection,
        PAGE_READWRITE | PAGE_WRITECOPY | PAGE_EXECUTE_READWRITE | PAGE_EXECUTE_WRITECOPY
    )
}

#[cfg(test)]
mod tests {
    use super::{
        CHAT_ENTRIES_OFFSET, CHAT_ENTRY_SIZE, DIALOG_ACTIVE_OFFSET, DIALOG_CAPTION_OFFSET,
        DIALOG_EDITBOX_OFFSET, DIALOG_ID_OFFSET, DIALOG_LISTBOX_OFFSET, DIALOG_SERVER_SIDE_OFFSET,
        DIALOG_TEXT_OFFSET, DIALOG_TYPE_OFFSET, DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET,
        DXUT_LISTBOX_ITEM_COUNT_OFFSET, DXUT_LISTBOX_ITEM_DATA_OFFSET, DXUT_LISTBOX_ITEM_SIZE,
        DXUT_LISTBOX_ITEM_TEXT_CAPACITY, DXUT_LISTBOX_ITEM_TEXT_OFFSET,
        DXUT_LISTBOX_ITEM_VISIBLE_OFFSET, DXUT_LISTBOX_ITEMS_OFFSET, DXUT_LISTBOX_SELECTED_OFFSET,
        ENTITY_HANDLE_OFFSET, GAME_CURSOR_MODE_OFFSET, GANGZONE_POOL_NOT_EMPTY_OFFSET,
        INPUT_ENABLED_OFFSET, LABEL_ATTACHED_PLAYER_OFFSET, LABEL_ATTACHED_VEHICLE_OFFSET,
        LABEL_BEHIND_WALLS_OFFSET, LABEL_COLOUR_OFFSET, LABEL_DRAW_DISTANCE_OFFSET,
        LABEL_POOL_NOT_EMPTY_OFFSET, LABEL_POSITION_OFFSET, LABEL_SIZE, LABEL_TEXT_OFFSET,
        LOCAL_PLAYER_ACTIVE_OFFSET, LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET, LOCAL_PLAYER_INCAR_OFFSET,
        LOCAL_PLAYER_INCAR_POSITION_OFFSET, LOCAL_PLAYER_INCAR_SPEED_OFFSET,
        LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET, LOCAL_PLAYER_ONFOOT_OFFSET,
        LOCAL_PLAYER_ONFOOT_POSITION_OFFSET, LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET,
        LOCAL_PLAYER_ONFOOT_SPEED_OFFSET, MAX_TEXT_LABEL_TEXT_BYTES, NET_GAME_GAME_STATE_OFFSET,
        NET_GAME_HOST_ADDRESS_OFFSET, NET_GAME_HOSTNAME_OFFSET,
        NET_GAME_POOLS_GANGZONE_POOL_OFFSET, NET_GAME_POOLS_LABEL_POOL_OFFSET,
        NET_GAME_POOLS_OBJECT_POOL_OFFSET, NET_GAME_POOLS_OFFSET,
        NET_GAME_POOLS_PICKUP_POOL_OFFSET, NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET,
        NET_GAME_PORT_OFFSET, NET_GAME_SERVER_SETTINGS_OFFSET, NativeDxutComboBoxItem,
        OBJECT_POOL_NOT_EMPTY_OFFSET, OBJECT_POOL_OBJECTS_OFFSET, PICKUP_POOL_HANDLES_OFFSET,
        PLAYER_POOL_LARGEST_ID_OFFSET, PLAYER_POOL_LOCAL_ID_OFFSET, SAMP_PED_GAME_PED_OFFSET,
        SCOREBOARD_ENABLED_OFFSET, TEXTDRAW_ALIGN_CENTER_OFFSET, TEXTDRAW_ALIGN_LEFT_OFFSET,
        TEXTDRAW_ALIGN_RIGHT_OFFSET, TEXTDRAW_BACKGROUND_COLOUR_OFFSET, TEXTDRAW_BOX_COLOUR_OFFSET,
        TEXTDRAW_BOX_ENABLED_OFFSET, TEXTDRAW_BOX_HEIGHT_OFFSET, TEXTDRAW_BOX_WIDTH_OFFSET,
        TEXTDRAW_DATA_OFFSET, TEXTDRAW_LETTER_COLOUR_OFFSET, TEXTDRAW_LETTER_HEIGHT_OFFSET,
        TEXTDRAW_LETTER_WIDTH_OFFSET, TEXTDRAW_MODEL_COLOUR1_OFFSET, TEXTDRAW_MODEL_COLOUR2_OFFSET,
        TEXTDRAW_MODEL_ID_OFFSET, TEXTDRAW_OUTLINE_OFFSET, TEXTDRAW_POOL_NOT_EMPTY_OFFSET,
        TEXTDRAW_POOL_OBJECTS_OFFSET, TEXTDRAW_PROPORTIONAL_OFFSET, TEXTDRAW_ROTATION_OFFSET,
        TEXTDRAW_SHADOW_OFFSET, TEXTDRAW_STYLE_OFFSET, TEXTDRAW_X_OFFSET, TEXTDRAW_Y_OFFSET,
        TEXTDRAW_ZOOM_OFFSET, VEHICLE_POOL_GAME_OBJECTS_OFFSET, VEHICLE_POOL_NOT_EMPTY_OFFSET,
        assigned_player_id, bounded_c_string, bounded_dxut_listbox_item_text, mem, nul_terminated,
    };

    unsafe extern "C" {
        fn samp_client_sdk_fixture_r1_onfoot_size() -> usize;
        fn samp_client_sdk_fixture_r1_incar_size() -> usize;
        fn samp_client_sdk_fixture_r1_local_player_prefix_size() -> usize;
        fn samp_client_sdk_fixture_r1_local_active_offset() -> usize;
        fn samp_client_sdk_fixture_r1_local_current_vehicle_offset() -> usize;
        fn samp_client_sdk_fixture_r1_local_onfoot_offset() -> usize;
        fn samp_client_sdk_fixture_r1_onfoot_position_offset() -> usize;
        fn samp_client_sdk_fixture_r1_onfoot_speed_offset() -> usize;
        fn samp_client_sdk_fixture_r1_onfoot_special_action_offset() -> usize;
        fn samp_client_sdk_fixture_r1_onfoot_animation_offset() -> usize;
        fn samp_client_sdk_fixture_r1_incar_position_offset() -> usize;
        fn samp_client_sdk_fixture_r1_incar_speed_offset() -> usize;
        fn samp_client_sdk_fixture_r1_ped_game_ped_offset() -> usize;
        fn samp_client_sdk_fixture_r1_player_pool_local_id_offset() -> usize;
        fn samp_client_sdk_fixture_r1_player_pool_largest_id_offset() -> usize;
        fn samp_client_sdk_fixture_r1_vehicle_pool_not_empty_offset() -> usize;
        fn samp_client_sdk_fixture_r1_vehicle_pool_game_objects_offset() -> usize;
        fn samp_client_sdk_fixture_r1_object_pool_objects_offset() -> usize;
        fn samp_client_sdk_fixture_r1_pickup_pool_handles_offset() -> usize;
        fn samp_client_sdk_fixture_r1_entity_handle_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_host_address_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_hostname_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_port_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_game_state_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_server_settings_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_pools_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_pools_label_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_pools_text_draw_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_pools_object_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_pools_gang_zone_offset() -> usize;
        fn samp_client_sdk_fixture_r1_net_game_pools_pickup_offset() -> usize;
        fn samp_client_sdk_fixture_r1_label_pool_not_empty_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_size() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_text_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_colour_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_position_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_draw_distance_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_behind_walls_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_attached_player_offset() -> usize;
        fn samp_client_sdk_fixture_r1_text_label_attached_vehicle_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_pool_not_empty_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_pool_objects_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_data_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_letter_width_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_letter_height_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_letter_colour_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_align_center_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_box_enabled_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_box_width_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_box_height_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_box_colour_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_proportional_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_background_colour_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_shadow_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_outline_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_align_left_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_align_right_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_style_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_x_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_y_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_model_id_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_rotation_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_zoom_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_model_colour1_offset() -> usize;
        fn samp_client_sdk_fixture_r1_textdraw_model_colour2_offset() -> usize;
        fn samp_client_sdk_fixture_r1_object_pool_not_empty_offset() -> usize;
        fn samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset() -> usize;
        fn samp_client_sdk_fixture_r1_gangzone_size() -> usize;
        fn samp_client_sdk_fixture_r1_game_cursor_mode_offset() -> usize;
        fn samp_client_sdk_fixture_r1_scoreboard_enabled_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_active_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_listbox_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_editbox_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_text_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_listbox_selected_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_listbox_items_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_listbox_item_count_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_combobox_item_text_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_combobox_item_text_capacity() -> usize;
        fn samp_client_sdk_fixture_dxut_combobox_item_data_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_combobox_item_active_rect_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_combobox_item_visible_offset() -> usize;
        fn samp_client_sdk_fixture_dxut_combobox_item_size() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_type_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_id_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_caption_offset() -> usize;
        fn samp_client_sdk_fixture_r1_dialog_server_side_offset() -> usize;
        fn samp_client_sdk_fixture_r1_input_enabled_offset() -> usize;
        fn samp_client_sdk_fixture_r1_chat_entries_offset() -> usize;
        fn samp_client_sdk_fixture_r1_chat_entry_size() -> usize;
    }

    #[test]
    fn r1_sync_offsets_match_the_independent_x86_fixture() {
        unsafe {
            assert_eq!(samp_client_sdk_fixture_r1_onfoot_size(), 68);
            assert_eq!(samp_client_sdk_fixture_r1_incar_size(), 63);
            assert_eq!(samp_client_sdk_fixture_r1_local_player_prefix_size(), 92);
            assert_eq!(
                samp_client_sdk_fixture_r1_local_active_offset(),
                LOCAL_PLAYER_ACTIVE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_local_current_vehicle_offset(),
                LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_local_onfoot_offset(),
                LOCAL_PLAYER_ONFOOT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_onfoot_position_offset(),
                LOCAL_PLAYER_ONFOOT_POSITION_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_onfoot_speed_offset(),
                LOCAL_PLAYER_ONFOOT_SPEED_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_onfoot_special_action_offset(),
                LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_onfoot_animation_offset(),
                LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET
            );
            assert_eq!(
                LOCAL_PLAYER_ONFOOT_OFFSET + 68 + 24 + 54,
                LOCAL_PLAYER_INCAR_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_incar_position_offset(),
                LOCAL_PLAYER_INCAR_POSITION_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_incar_speed_offset(),
                LOCAL_PLAYER_INCAR_SPEED_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_ped_game_ped_offset(),
                SAMP_PED_GAME_PED_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_player_pool_local_id_offset(),
                PLAYER_POOL_LOCAL_ID_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_player_pool_largest_id_offset(),
                PLAYER_POOL_LARGEST_ID_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_vehicle_pool_not_empty_offset(),
                VEHICLE_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_vehicle_pool_game_objects_offset(),
                VEHICLE_POOL_GAME_OBJECTS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_object_pool_objects_offset(),
                OBJECT_POOL_OBJECTS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_pickup_pool_handles_offset(),
                PICKUP_POOL_HANDLES_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_entity_handle_offset(),
                ENTITY_HANDLE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_host_address_offset(),
                NET_GAME_HOST_ADDRESS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_hostname_offset(),
                NET_GAME_HOSTNAME_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_port_offset(),
                NET_GAME_PORT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_game_state_offset(),
                NET_GAME_GAME_STATE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_server_settings_offset(),
                NET_GAME_SERVER_SETTINGS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_pools_offset(),
                NET_GAME_POOLS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_pools_label_offset(),
                NET_GAME_POOLS_LABEL_POOL_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_pools_text_draw_offset(),
                NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_pools_object_offset(),
                NET_GAME_POOLS_OBJECT_POOL_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_pools_gang_zone_offset(),
                NET_GAME_POOLS_GANGZONE_POOL_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_net_game_pools_pickup_offset(),
                NET_GAME_POOLS_PICKUP_POOL_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_label_pool_not_empty_offset(),
                LABEL_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(samp_client_sdk_fixture_r1_text_label_size(), LABEL_SIZE);
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_text_offset(),
                LABEL_TEXT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_colour_offset(),
                LABEL_COLOUR_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_position_offset(),
                LABEL_POSITION_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_draw_distance_offset(),
                LABEL_DRAW_DISTANCE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_behind_walls_offset(),
                LABEL_BEHIND_WALLS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_attached_player_offset(),
                LABEL_ATTACHED_PLAYER_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_text_label_attached_vehicle_offset(),
                LABEL_ATTACHED_VEHICLE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_textdraw_pool_not_empty_offset(),
                TEXTDRAW_POOL_NOT_EMPTY_OFFSET
            );
            let textdraw_offsets = [
                (
                    samp_client_sdk_fixture_r1_textdraw_pool_objects_offset(),
                    TEXTDRAW_POOL_OBJECTS_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_data_offset(),
                    TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_letter_width_offset(),
                    TEXTDRAW_LETTER_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_letter_height_offset(),
                    TEXTDRAW_LETTER_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_letter_colour_offset(),
                    TEXTDRAW_LETTER_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_align_center_offset(),
                    TEXTDRAW_ALIGN_CENTER_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_box_enabled_offset(),
                    TEXTDRAW_BOX_ENABLED_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_box_width_offset(),
                    TEXTDRAW_BOX_WIDTH_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_box_height_offset(),
                    TEXTDRAW_BOX_HEIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_box_colour_offset(),
                    TEXTDRAW_BOX_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_proportional_offset(),
                    TEXTDRAW_PROPORTIONAL_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_background_colour_offset(),
                    TEXTDRAW_BACKGROUND_COLOUR_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_shadow_offset(),
                    TEXTDRAW_SHADOW_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_outline_offset(),
                    TEXTDRAW_OUTLINE_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_align_left_offset(),
                    TEXTDRAW_ALIGN_LEFT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_align_right_offset(),
                    TEXTDRAW_ALIGN_RIGHT_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_style_offset(),
                    TEXTDRAW_STYLE_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_x_offset(),
                    TEXTDRAW_X_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_y_offset(),
                    TEXTDRAW_Y_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_model_id_offset(),
                    TEXTDRAW_MODEL_ID_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_rotation_offset(),
                    TEXTDRAW_ROTATION_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_zoom_offset(),
                    TEXTDRAW_ZOOM_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_model_colour1_offset(),
                    TEXTDRAW_MODEL_COLOUR1_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
                (
                    samp_client_sdk_fixture_r1_textdraw_model_colour2_offset(),
                    TEXTDRAW_MODEL_COLOUR2_OFFSET - TEXTDRAW_DATA_OFFSET,
                ),
            ];
            for (actual, expected) in textdraw_offsets {
                assert_eq!(actual, expected);
            }
            assert_eq!(
                samp_client_sdk_fixture_r1_object_pool_not_empty_offset(),
                OBJECT_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(samp_client_sdk_fixture_r1_gangzone_size(), 0x18);
            assert_eq!(
                samp_client_sdk_fixture_r1_gangzone_pool_not_empty_offset(),
                GANGZONE_POOL_NOT_EMPTY_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_game_cursor_mode_offset(),
                GAME_CURSOR_MODE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_scoreboard_enabled_offset(),
                SCOREBOARD_ENABLED_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_active_offset(),
                DIALOG_ACTIVE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_listbox_offset(),
                DIALOG_LISTBOX_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_editbox_offset(),
                DIALOG_EDITBOX_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_text_offset(),
                DIALOG_TEXT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_listbox_selected_offset(),
                DXUT_LISTBOX_SELECTED_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_listbox_items_offset(),
                DXUT_LISTBOX_ITEMS_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_listbox_item_count_offset(),
                DXUT_LISTBOX_ITEM_COUNT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_combobox_item_text_offset(),
                DXUT_LISTBOX_ITEM_TEXT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_combobox_item_text_capacity(),
                DXUT_LISTBOX_ITEM_TEXT_CAPACITY
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_combobox_item_data_offset(),
                DXUT_LISTBOX_ITEM_DATA_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_combobox_item_active_rect_offset(),
                DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_combobox_item_visible_offset(),
                DXUT_LISTBOX_ITEM_VISIBLE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_dxut_combobox_item_size(),
                DXUT_LISTBOX_ITEM_SIZE
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_type_offset(),
                DIALOG_TYPE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_id_offset(),
                DIALOG_ID_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_caption_offset(),
                DIALOG_CAPTION_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_dialog_server_side_offset(),
                DIALOG_SERVER_SIDE_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_input_enabled_offset(),
                INPUT_ENABLED_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_chat_entries_offset(),
                CHAT_ENTRIES_OFFSET
            );
            assert_eq!(
                samp_client_sdk_fixture_r1_chat_entry_size(),
                CHAT_ENTRY_SIZE
            );
        }
    }

    #[test]
    fn native_dialog_strings_are_terminated_only_after_copying() {
        assert_eq!(nul_terminated(b"dialog".to_vec()), b"dialog\0");
    }

    #[test]
    fn bounded_label_copy_accepts_the_full_r1_text_limit() {
        let mut text = vec![b'x'; MAX_TEXT_LABEL_TEXT_BYTES];
        text.push(0);
        assert_eq!(
            unsafe { bounded_c_string(text.as_ptr(), MAX_TEXT_LABEL_TEXT_BYTES + 1) },
            Some(vec![b'x'; MAX_TEXT_LABEL_TEXT_BYTES])
        );
        assert_eq!(
            unsafe {
                bounded_c_string(
                    text[..MAX_TEXT_LABEL_TEXT_BYTES].as_ptr(),
                    MAX_TEXT_LABEL_TEXT_BYTES,
                )
            },
            None
        );
    }

    #[test]
    fn native_dxut_combobox_item_mirror_matches_the_fixture_layout() {
        assert_eq!(
            mem::offset_of!(NativeDxutComboBoxItem, str_text),
            DXUT_LISTBOX_ITEM_TEXT_OFFSET
        );
        assert_eq!(
            mem::offset_of!(NativeDxutComboBoxItem, data),
            DXUT_LISTBOX_ITEM_DATA_OFFSET
        );
        assert_eq!(
            mem::offset_of!(NativeDxutComboBoxItem, active_rect),
            DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET
        );
        assert_eq!(
            mem::offset_of!(NativeDxutComboBoxItem, visible),
            DXUT_LISTBOX_ITEM_VISIBLE_OFFSET
        );
        assert_eq!(
            mem::size_of::<NativeDxutComboBoxItem>(),
            DXUT_LISTBOX_ITEM_SIZE
        );
        assert_eq!(mem::size_of::<windows_sys::Win32::Foundation::RECT>(), 16);
        assert_eq!(mem::align_of::<NativeDxutComboBoxItem>(), 4);
    }

    #[test]
    fn listbox_item_text_read_stays_inside_the_native_text_field() {
        let mut item = NativeDxutComboBoxItem {
            str_text: [b'x'; DXUT_LISTBOX_ITEM_TEXT_CAPACITY],
            data: std::ptr::null_mut(),
            active_rect: windows_sys::Win32::Foundation::RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            },
            visible: false,
        };

        assert_eq!(
            unsafe { bounded_dxut_listbox_item_text(item.str_text.as_ptr()) },
            None
        );
        item.str_text[DXUT_LISTBOX_ITEM_TEXT_CAPACITY - 1] = 0;
        assert_eq!(
            unsafe { bounded_dxut_listbox_item_text(item.str_text.as_ptr()) },
            Some(vec![b'x'; DXUT_LISTBOX_ITEM_TEXT_CAPACITY - 1])
        );
    }

    #[test]
    fn unassigned_local_player_id_is_not_a_snapshot() {
        assert_eq!(assigned_player_id(u16::MAX), None);
        assert_eq!(assigned_player_id(42), Some(42));
    }
}
