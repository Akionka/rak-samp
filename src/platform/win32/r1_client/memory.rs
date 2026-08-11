//! Fixed R1 native layouts and guarded memory access helpers.

use crate::runtime::{DirectClientError, Vector3};
#[cfg(test)]
use std::ffi::c_void;
use std::mem;
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY,
    PAGE_GUARD, PAGE_NOACCESS, PAGE_READWRITE, PAGE_WRITECOPY, VirtualQuery,
};

pub(super) const CHAT_DISPLAY_MODE_OFFSET: usize = 0x08;
pub(super) const CHAT_ENTRIES_OFFSET: usize = 0x132;
pub(super) const CHAT_ENTRY_SIZE: usize = 0xFC;
pub(super) const CHAT_ENTRY_PREFIX_OFFSET: usize = 0x04;
pub(super) const CHAT_ENTRY_PREFIX_CAPACITY: usize = 28;
pub(super) const CHAT_ENTRY_TEXT_OFFSET: usize = 0x20;
pub(super) const CHAT_ENTRY_TEXT_CAPACITY: usize = 144;
pub(super) const CHAT_ENTRY_TEXT_COLOUR_OFFSET: usize = 0xF4;
pub(super) const CHAT_ENTRY_PREFIX_COLOUR_OFFSET: usize = 0xF8;
pub(super) const MAX_CHAT_ENTRIES: u16 = 100;
pub(super) const RAKPEER_SIZE: usize = 0xDDE;
pub(super) const ANIMATION_TABLE_ENTRY_COUNT: usize = 1812;
pub(super) const ANIMATION_TABLE_ENTRY_SIZE: usize = 36;
pub(super) const MAX_SAMP_PLAYERS: u16 = 1004;
pub(super) const MAX_SAMP_VEHICLES: u16 = 2000;
pub(super) const MAX_SAMP_TEXT_LABELS: u16 = 2048;
pub(super) const MAX_SAMP_TEXTDRAWS: u16 = 2304;
pub(super) const MAX_SAMP_OBJECTS: u16 = 1000;
pub(super) const MAX_SAMP_GANGZONES: u16 = 1024;
pub(super) const MAX_SAMP_PICKUPS: u16 = 4096;
pub(super) const MAX_LOCAL_PLAYER_NAME_BYTES: usize = 255;

pub(super) const PLAYER_POOL_LOCAL_ID_OFFSET: usize = 0x04;
pub(super) const PLAYER_POOL_LARGEST_ID_OFFSET: usize = 0x00;
pub(super) const VEHICLE_POOL_NOT_EMPTY_OFFSET: usize = 0x3074;
pub(super) const VEHICLE_POOL_GAME_OBJECTS_OFFSET: usize = 0x4FB4;
pub(super) const OBJECT_POOL_NOT_EMPTY_OFFSET: usize = 0x04;
pub(super) const OBJECT_POOL_OBJECTS_OFFSET: usize = 0xFA4;
pub(super) const PICKUP_POOL_HANDLES_OFFSET: usize = 0x04;
pub(super) const ENTITY_HANDLE_OFFSET: usize = 0x44;
pub(super) const NET_GAME_POOLS_OFFSET: usize = 0x3CD;
pub(super) const NET_GAME_POOLS_LABEL_POOL_OFFSET: usize = 0x0C;
pub(super) const NET_GAME_POOLS_TEXTDRAW_POOL_OFFSET: usize = 0x10;
pub(super) const NET_GAME_POOLS_OBJECT_POOL_OFFSET: usize = 0x04;
pub(super) const NET_GAME_POOLS_GANGZONE_POOL_OFFSET: usize = 0x08;
pub(super) const LABEL_POOL_NOT_EMPTY_OFFSET: usize = 0xE800;
pub(super) const LABEL_TEXT_OFFSET: usize = 0x00;
pub(super) const LABEL_COLOUR_OFFSET: usize = 0x04;
pub(super) const LABEL_POSITION_OFFSET: usize = 0x08;
pub(super) const LABEL_DRAW_DISTANCE_OFFSET: usize = 0x14;
pub(super) const LABEL_BEHIND_WALLS_OFFSET: usize = 0x18;
pub(super) const LABEL_ATTACHED_PLAYER_OFFSET: usize = 0x19;
pub(super) const LABEL_ATTACHED_VEHICLE_OFFSET: usize = 0x1B;
pub(super) const LABEL_SIZE: usize = 0x1D;
pub(super) const MAX_TEXT_LABEL_TEXT_BYTES: usize = 4_095;
pub(super) const MAX_TEXTDRAW_STRING_BYTES: usize = 1_601;
pub(super) const TEXTDRAW_STRING_OFFSET: usize = 801;
pub(super) const TEXTDRAW_POOL_NOT_EMPTY_OFFSET: usize = 0;
pub(super) const TEXTDRAW_POOL_OBJECTS_OFFSET: usize = 0x2400;
pub(super) const GANGZONE_POOL_NOT_EMPTY_OFFSET: usize = 0x1000;
pub(super) const GANGZONE_LEFT_OFFSET: usize = 0x00;
pub(super) const GANGZONE_BOTTOM_OFFSET: usize = 0x04;
pub(super) const GANGZONE_RIGHT_OFFSET: usize = 0x08;
pub(super) const GANGZONE_TOP_OFFSET: usize = 0x0C;
pub(super) const GANGZONE_COLOUR_OFFSET: usize = 0x10;
pub(super) const GANGZONE_ALTERNATE_COLOUR_OFFSET: usize = 0x14;
pub(super) const REMOTE_PLAYER_SPECIAL_ACTION_OFFSET: usize = 0xBB;
pub(super) const REMOTE_PLAYER_ONFOOT_OFFSET: usize = 0xC8;
pub(super) const REMOTE_PLAYER_INCAR_OFFSET: usize = 0x10C;
pub(super) const REMOTE_PLAYER_PASSENGER_OFFSET: usize = 0x181;
pub(super) const REMOTE_PLAYER_TRAILER_OFFSET: usize = 0x14B;
pub(super) const REMOTE_PLAYER_REPORTED_ARMOUR_OFFSET: usize = 0x1B8;
pub(super) const REMOTE_PLAYER_REPORTED_HEALTH_OFFSET: usize = 0x1BC;
pub(super) const REMOTE_PLAYER_ANIMATION_OFFSET: usize = 0x1C0;
pub(super) const REMOTE_PLAYER_STATE_SIZE: usize = REMOTE_PLAYER_ANIMATION_OFFSET + 4;

pub(super) const ONFOOT_SYNC_SIZE: usize = 68;
pub(super) const ONFOOT_CONTROLLER_LEFT_STICK_X_OFFSET: usize = 0x00;
pub(super) const ONFOOT_CONTROLLER_LEFT_STICK_Y_OFFSET: usize = 0x02;
pub(super) const ONFOOT_CONTROLLER_BUTTONS_OFFSET: usize = 0x04;
pub(super) const ONFOOT_POSITION_OFFSET: usize = 0x06;
pub(super) const ONFOOT_QUATERNION_OFFSET: usize = 0x12;
pub(super) const ONFOOT_HEALTH_OFFSET: usize = 0x22;
pub(super) const ONFOOT_ARMOUR_OFFSET: usize = 0x23;
pub(super) const ONFOOT_WEAPON_OFFSET: usize = 0x24;
pub(super) const ONFOOT_SPECIAL_ACTION_OFFSET: usize = 0x25;
pub(super) const ONFOOT_SPEED_OFFSET: usize = 0x26;
pub(super) const ONFOOT_SURFING_OFFSET_OFFSET: usize = 0x32;
pub(super) const ONFOOT_SURFING_VEHICLE_ID_OFFSET: usize = 0x3E;
pub(super) const ONFOOT_ANIMATION_OFFSET: usize = 0x40;

pub(super) const INCAR_SYNC_SIZE: usize = 63;
pub(super) const INCAR_VEHICLE_ID_OFFSET: usize = 0x00;
pub(super) const INCAR_CONTROLLER_LEFT_STICK_X_OFFSET: usize = 0x02;
pub(super) const INCAR_CONTROLLER_LEFT_STICK_Y_OFFSET: usize = 0x04;
pub(super) const INCAR_CONTROLLER_BUTTONS_OFFSET: usize = 0x06;
pub(super) const INCAR_QUATERNION_OFFSET: usize = 0x08;
pub(super) const INCAR_POSITION_OFFSET: usize = 0x18;
pub(super) const INCAR_SPEED_OFFSET: usize = 0x24;
pub(super) const INCAR_VEHICLE_HEALTH_OFFSET: usize = 0x30;
pub(super) const INCAR_DRIVER_HEALTH_OFFSET: usize = 0x34;
pub(super) const INCAR_DRIVER_ARMOUR_OFFSET: usize = 0x35;
pub(super) const INCAR_WEAPON_OFFSET: usize = 0x36;
pub(super) const INCAR_SIREN_OFFSET: usize = 0x37;
pub(super) const INCAR_LANDING_GEAR_OFFSET: usize = 0x38;
pub(super) const INCAR_TRAILER_ID_OFFSET: usize = 0x39;
pub(super) const INCAR_VEHICLE_SPECIFIC_OFFSET: usize = 0x3B;

pub(super) const PASSENGER_SYNC_SIZE: usize = 24;
pub(super) const PASSENGER_VEHICLE_ID_OFFSET: usize = 0x00;
pub(super) const PASSENGER_SEAT_ID_OFFSET: usize = 0x02;
pub(super) const PASSENGER_WEAPON_OFFSET: usize = 0x03;
pub(super) const PASSENGER_HEALTH_OFFSET: usize = 0x04;
pub(super) const PASSENGER_ARMOUR_OFFSET: usize = 0x05;
pub(super) const PASSENGER_CONTROLLER_LEFT_STICK_X_OFFSET: usize = 0x06;
pub(super) const PASSENGER_CONTROLLER_LEFT_STICK_Y_OFFSET: usize = 0x08;
pub(super) const PASSENGER_CONTROLLER_BUTTONS_OFFSET: usize = 0x0A;
pub(super) const PASSENGER_POSITION_OFFSET: usize = 0x0C;

pub(super) const TRAILER_SYNC_SIZE: usize = 54;
pub(super) const TRAILER_ID_OFFSET: usize = 0x00;
pub(super) const TRAILER_POSITION_OFFSET: usize = 0x02;
pub(super) const TRAILER_QUATERNION_OFFSET: usize = 0x0E;
pub(super) const TRAILER_SPEED_OFFSET: usize = 0x1E;
pub(super) const TRAILER_TURN_SPEED_OFFSET: usize = 0x2A;

// These packed CNetGame fields are cross-checked by the independently written
// fixture. `GetGameState`'s signed R1 target reads offset 0x3BD from this same
// layout, which anchors the packed field sequence.
pub(super) const NET_GAME_HOST_ADDRESS_OFFSET: usize = 0x20;
pub(super) const NET_GAME_HOSTNAME_OFFSET: usize = 0x121;
pub(super) const NET_GAME_PORT_OFFSET: usize = 0x225;
pub(super) const NET_GAME_GAME_STATE_OFFSET: usize = 0x3BD;
#[cfg(test)]
pub(super) const NET_GAME_SERVER_SETTINGS_OFFSET: usize = 0x3C5;
pub(super) const NET_GAME_POOLS_PICKUP_POOL_OFFSET: usize = 0x20;
pub(super) const NET_GAME_HOST_STRING_CAPACITY: usize = 257;
pub(super) const RAK_CLIENT_DISCONNECT_VTABLE_SLOT: usize = 2;
pub(super) const SCOREBOARD_ENABLED_OFFSET: usize = 0x00;
pub(super) const GAME_CURSOR_MODE_OFFSET: usize = 0x55;
pub(super) const DIALOG_ACTIVE_OFFSET: usize = 0x28;
pub(super) const DIALOG_TYPE_OFFSET: usize = 0x2C;
pub(super) const DIALOG_ID_OFFSET: usize = 0x30;
pub(super) const DIALOG_LISTBOX_OFFSET: usize = 0x20;
pub(super) const DIALOG_EDITBOX_OFFSET: usize = 0x24;
pub(super) const DIALOG_TEXT_OFFSET: usize = 0x34;
pub(super) const DIALOG_CAPTION_OFFSET: usize = 0x40;
pub(super) const DIALOG_CAPTION_CAPACITY: usize = 65;
pub(super) const DIALOG_SERVER_SIDE_OFFSET: usize = 0x81;
pub(super) const DXUT_LISTBOX_SELECTED_OFFSET: usize = 0x143;
pub(super) const DXUT_LISTBOX_ITEMS_OFFSET: usize = 0x14C;
pub(super) const DXUT_LISTBOX_ITEM_COUNT_OFFSET: usize = 0x150;
pub(super) const DXUT_LISTBOX_ITEM_TEXT_OFFSET: usize = 0x00;
pub(super) const DXUT_LISTBOX_ITEM_TEXT_CAPACITY: usize = 256;
#[cfg(test)]
pub(super) const DXUT_LISTBOX_ITEM_DATA_OFFSET: usize = 0x100;
#[cfg(test)]
pub(super) const DXUT_LISTBOX_ITEM_ACTIVE_RECT_OFFSET: usize = 0x104;
#[cfg(test)]
pub(super) const DXUT_LISTBOX_ITEM_VISIBLE_OFFSET: usize = 0x114;
#[cfg(test)]
pub(super) const DXUT_LISTBOX_ITEM_SIZE: usize = 0x118;
pub(super) const MAX_DIALOG_TEXT_BYTES: usize = 4_096;
pub(super) const MAX_DIALOG_EDITBOX_TEXT_BYTES: usize = 128;
pub(super) const MAX_DIALOG_LISTBOX_ITEMS: usize = 100;
pub(super) const INPUT_ENABLED_OFFSET: usize = 0x14E0;
pub(super) const INPUT_EDIT_BOX_OFFSET: usize = 0x08;
pub(super) const MAX_CHAT_INPUT_TEXT_BYTES: usize = 128;
pub(super) const MAX_CHAT_COMMANDS: usize = 144;
pub(super) const MAX_CHAT_COMMAND_NAME_BYTES: usize = 32;
pub(super) const INPUT_COMMAND_PROC_OFFSET: usize = 0x0C;
pub(super) const INPUT_COMMAND_NAME_OFFSET: usize = 0x24C;
pub(super) const INPUT_COMMAND_NAME_CAPACITY: usize = MAX_CHAT_COMMAND_NAME_BYTES + 1;
pub(super) const INPUT_COMMAND_COUNT_OFFSET: usize = 0x14DC;
pub(super) const TEXTDRAW_DATA_OFFSET: usize = 0x963;
pub(super) const TEXTDRAW_LETTER_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET;
pub(super) const TEXTDRAW_LETTER_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x04;
pub(super) const TEXTDRAW_LETTER_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x08;
pub(super) const TEXTDRAW_ALIGN_CENTER_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0D;
pub(super) const TEXTDRAW_BOX_ENABLED_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0E;
pub(super) const TEXTDRAW_BOX_WIDTH_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x0F;
pub(super) const TEXTDRAW_BOX_HEIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x13;
pub(super) const TEXTDRAW_BOX_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x17;
pub(super) const TEXTDRAW_PROPORTIONAL_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1B;
pub(super) const TEXTDRAW_BACKGROUND_COLOUR_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x1C;
pub(super) const TEXTDRAW_SHADOW_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x20;
pub(super) const TEXTDRAW_OUTLINE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x21;
pub(super) const TEXTDRAW_ALIGN_LEFT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x22;
pub(super) const TEXTDRAW_ALIGN_RIGHT_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x23;
pub(super) const TEXTDRAW_STYLE_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x24;
pub(super) const TEXTDRAW_X_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x28;
pub(super) const TEXTDRAW_Y_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x2C;
pub(super) const TEXTDRAW_MODEL_ID_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x45;
pub(super) const TEXTDRAW_ROTATION_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x47;
pub(super) const TEXTDRAW_ZOOM_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x53;
pub(super) const TEXTDRAW_MODEL_COLOUR1_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x57;
pub(super) const TEXTDRAW_MODEL_COLOUR2_OFFSET: usize = TEXTDRAW_DATA_OFFSET + 0x59;

pub(super) const LOCAL_PLAYER_ACTIVE_OFFSET: usize = 0x0C;
pub(super) const LOCAL_PLAYER_CURRENT_VEHICLE_OFFSET: usize = 0x14;
pub(super) const LOCAL_PLAYER_ONFOOT_OFFSET: usize = 0x18;
pub(super) const LOCAL_PLAYER_INCAR_OFFSET: usize = 0xAA;
pub(super) const LOCAL_PLAYER_PASSENGER_OFFSET: usize = 0x5C;
pub(super) const LOCAL_PLAYER_TRAILER_OFFSET: usize = 0x74;
pub(super) const LOCAL_PLAYER_ONFOOT_POSITION_OFFSET: usize = ONFOOT_POSITION_OFFSET;
pub(super) const LOCAL_PLAYER_ONFOOT_SPEED_OFFSET: usize = ONFOOT_SPEED_OFFSET;
pub(super) const LOCAL_PLAYER_ONFOOT_SPECIAL_ACTION_OFFSET: usize = ONFOOT_SPECIAL_ACTION_OFFSET;
pub(super) const LOCAL_PLAYER_ONFOOT_ANIMATION_OFFSET: usize = ONFOOT_ANIMATION_OFFSET;
pub(super) const LOCAL_PLAYER_INCAR_POSITION_OFFSET: usize = 0x18;
pub(super) const LOCAL_PLAYER_INCAR_SPEED_OFFSET: usize = 0x24;

// `CPed` inherits a 0x48-byte `CEntity`, then owns its accessory arrays before
// its GTA-ped pointer.
pub(super) const SAMP_PED_GAME_PED_OFFSET: usize = 0x2A4;

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
pub(super) struct NativeDxutComboBoxItem {
    pub(super) str_text: [u8; DXUT_LISTBOX_ITEM_TEXT_CAPACITY],
    pub(super) data: *mut c_void,
    pub(super) active_rect: windows_sys::Win32::Foundation::RECT,
    pub(super) visible: bool,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub(super) struct NativeVector3 {
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

pub(super) unsafe fn read_pointer(address: usize) -> Option<*mut u8> {
    unsafe { read_unaligned::<usize>(address) }.map(|value| value as *mut u8)
}

pub(super) unsafe fn read_unaligned<T: Copy>(address: usize) -> Option<T> {
    readable_range(address as *const u8, mem::size_of::<T>())
        .then(|| unsafe { (address as *const T).read_unaligned() })
}

pub(super) unsafe fn read_vector3(address: usize) -> Option<Vector3> {
    Some(Vector3 {
        x: unsafe { read_unaligned::<f32>(address) }?,
        y: unsafe { read_unaligned::<f32>(address.checked_add(4)?) }?,
        z: unsafe { read_unaligned::<f32>(address.checked_add(8)?) }?,
    })
}

pub(super) fn read_r1_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<i32>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

pub(super) fn read_u8_bool(address: usize) -> Result<bool, DirectClientError> {
    match unsafe { read_unaligned::<u8>(address) } {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => Err(DirectClientError::NotReady),
    }
}

pub(crate) unsafe fn bounded_c_string(pointer: *const u8, maximum: usize) -> Option<Vec<u8>> {
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

pub(super) unsafe fn bounded_dxut_listbox_item_text(pointer: *const u8) -> Option<Vec<u8>> {
    unsafe { bounded_c_string(pointer, DXUT_LISTBOX_ITEM_TEXT_CAPACITY) }
}

pub(super) fn readable_range(address: *const u8, length: usize) -> bool {
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

pub(super) fn writable_range(address: *const u8, length: usize) -> bool {
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
