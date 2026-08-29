//! Owned and output C ABI values.

use super::*;

/// C-compatible storage for [`LocalPlayer`].
///
/// This is output-only. `nickname_len` selects the initialized prefix of
/// `nickname`; the buffer has no required terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkLocalPlayerV1 {
    pub id: u16,
    pub nickname_len: u16,
    pub nickname: [u8; 256],
    pub colour: u32,
    pub spawned: u8,
    pub special_action: u8,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
    pub position: Vector3,
    pub velocity: Vector3,
    pub has_vehicle: u8,
    pub _reserved: u8,
    pub vehicle_id: u16,
    pub score: i32,
    pub ping: u32,
}

/// C-compatible storage for an active R1 dialog core snapshot.
///
/// `active` is zero when no dialog is active. When it is one, `title_len`
/// selects the initialized prefix of `title`; the buffer has no required
/// terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkActiveDialogV1 {
    pub active: u8,
    pub style: u8,
    pub server_side: u8,
    pub _reserved: u8,
    pub id: i32,
    pub title_len: u8,
    pub title: [u8; 65],
}

/// Fixed ABI storage for the cached R1 chat-input text. `len` selects the
/// initialized byte prefix; the buffer has no required terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkChatInputTextV1 {
    pub len: u8,
    pub bytes: [u8; 128],
}

/// Fixed ABI storage for one cached R1 dialog listbox item text.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkDialogListItemV1 {
    pub len: u8,
    pub bytes: [u8; MAX_SAMP_DIALOG_LISTBOX_ITEM_BYTES],
}

/// Fixed ABI storage for one coherent active-dialog cache publication.
///
/// `active` is zero when no dialog is active. Otherwise, the length fields
/// select initialized byte prefixes with no required terminators.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkDialogSnapshotV1 {
    pub active: u8,
    pub style: u8,
    pub server_side: u8,
    pub has_editbox: u8,
    pub id: i32,
    pub title_len: u8,
    pub editbox_text_len: u8,
    pub listbox_item_count: u8,
    pub _reserved: u8,
    pub text_len: u16,
    pub _reserved2: [u8; 2],
    pub title: [u8; 65],
    pub editbox_text: [u8; MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES],
    pub text: [u8; MAX_SAMP_DIALOG_TEXT_BYTES],
    pub listbox_items: [SampClientSdkDialogListItemV1; MAX_SAMP_DIALOG_LISTBOX_ITEMS],
}

/// Fixed ABI storage for one owned R1 dialog-close response.
///
/// `available` is zero when no response is pending. When it is one,
/// `input_len` selects the initialized prefix of `input`; the buffer has no
/// required terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SampClientSdkDialogResponseV1 {
    pub available: u8,
    pub button: u8,
    pub input_len: u8,
    pub _reserved: u8,
    pub dialog_id: u16,
    pub _reserved2: u16,
    pub list_item: i32,
    pub input: [u8; MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES],
}

impl Default for SampClientSdkDialogResponseV1 {
    fn default() -> Self {
        Self {
            available: 0,
            button: 0,
            input_len: 0,
            _reserved: 0,
            dialog_id: 0,
            _reserved2: 0,
            list_item: 0,
            input: [0; MAX_SAMP_DIALOG_EDITBOX_TEXT_BYTES],
        }
    }
}

/// C-compatible storage for an owned [`PlayerInfo`] result.
///
/// `exists` is zero for a cached disconnected ID and one for a copied entry.
/// The host always initializes the whole structure; `nickname_len` selects the
/// initialized prefix of `nickname` when `exists` is one.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkPlayerInfoV1 {
    pub exists: u8,
    pub is_local: u8,
    pub is_npc: u8,
    pub _reserved: u8,
    pub id: u16,
    pub nickname_len: u16,
    pub nickname: [u8; 256],
    pub colour: u32,
    pub score: i32,
    pub ping: u32,
}

/// C-compatible storage for an owned remote-player state result.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkRemotePlayerStateV1 {
    pub exists: u8,
    pub special_action: u8,
    pub _reserved: u16,
    pub id: u16,
    pub animation_id: u16,
    pub health: f32,
    pub armour: f32,
}

/// C-compatible storage for an owned R1 streamed-out player marker position.
///
/// `exists` is zero when the latest completed query found no connected player
/// with an active marker. When it is one, `position` is the client marker
/// cache, so it is integer-quantized and may be stale.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkStreamedOutPlayerPositionV1 {
    pub exists: u8,
    pub _reserved: [u8; 3],
    pub position: Vector3,
}

/// C-compatible storage for an owned R1 on-foot synchronization snapshot.
///
/// `exists` is zero when the latest completed query found no defined player.
/// `surfing_vehicle_id` preserves the native `0xFFFF` sentinel, and
/// `animation` preserves the raw native animation bits.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkOnFootSyncV1 {
    pub exists: u8,
    pub health: u8,
    pub armour: u8,
    pub weapon: u8,
    pub special_action: u8,
    pub _reserved: [u8; 3],
    pub id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub _reserved2: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub speed: Vector3,
    pub surfing_offset: Vector3,
    pub surfing_vehicle_id: u16,
    pub _reserved3: u16,
    pub animation: u32,
}

/// C-compatible storage for an owned R1 in-car synchronization snapshot.
///
/// `exists` is zero when the latest completed query found no defined player.
/// `vehicle_id`, `trailer_id`, and `vehicle_specific` preserve the native raw
/// values, including any sentinel or game-specific encoding.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkInCarSyncV1 {
    pub exists: u8,
    pub driver_health: u8,
    pub driver_armour: u8,
    pub weapon: u8,
    pub siren: u8,
    pub landing_gear: u8,
    pub _reserved: [u8; 2],
    pub id: u16,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub _reserved2: u16,
    pub quaternion: [f32; 4],
    pub position: Vector3,
    pub speed: Vector3,
    pub vehicle_health: f32,
    pub trailer_id: u16,
    pub vehicle_specific: [u8; 4],
}

/// C-compatible storage for an owned R1 passenger synchronization snapshot.
///
/// `exists` is zero when the latest completed query found no defined player.
/// `vehicle_id`, `seat_id`, and `weapon` preserve their native raw values.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkPassengerSyncV1 {
    pub exists: u8,
    pub seat_id: u8,
    pub weapon: u8,
    pub health: u8,
    pub armour: u8,
    pub _reserved: [u8; 3],
    pub id: u16,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub _reserved2: u16,
    pub position: Vector3,
}

/// C-compatible storage for an owned R1 trailer synchronization snapshot.
///
/// `exists` is zero when the latest completed query found no defined player.
/// `trailer_id` preserves its native raw value, including any sentinel.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkTrailerSyncV1 {
    pub exists: u8,
    pub _reserved: [u8; 3],
    pub id: u16,
    pub trailer_id: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub speed: Vector3,
    pub turn_speed: Vector3,
}

/// C-compatible storage for an owned R1 aim synchronization snapshot.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkAimSyncV1 {
    pub exists: u8,
    pub camera_mode: u8,
    pub zoom_and_weapon_state: u8,
    pub aspect_ratio: u8,
    pub id: u16,
    pub _reserved: u16,
    pub aim_first: Vector3,
    pub aim_position: Vector3,
    pub aim_z: f32,
}

/// C-compatible storage for an owned [`Gangzone`] result.
///
/// `exists` is zero when the latest completed query found no gangzone. The
/// host initializes all fields in that case; when it is one, the scalar fields
/// hold an R1 game-thread copy.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SampClientSdkGangzoneV1 {
    pub exists: u8,
    pub _reserved: [u8; 3],
    pub id: u16,
    pub _reserved2: u16,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub colour: u32,
    pub alternate_colour: u32,
}

/// C-compatible storage for an owned [`TextLabel`] result.
///
/// `exists` is zero when the latest completed query found no label. When it
/// is one, `text_len` selects the initialized prefix of `text`; the buffer has
/// no required terminator. `0xFFFF` in either attachment field means `None`.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkTextLabelV1 {
    pub exists: u8,
    pub behind_walls: u8,
    pub _reserved: [u8; 2],
    pub id: u16,
    pub attached_player_id: u16,
    pub attached_vehicle_id: u16,
    pub _reserved2: u16,
    pub colour: u32,
    pub position: Vector3,
    pub draw_distance: f32,
    pub text_len: u16,
    pub _reserved3: [u8; 2],
    pub text: [u8; MAX_SAMP_TEXT_LABEL_TEXT_BYTES],
}

/// C-compatible storage for an owned [`TextDraw`] result.
///
/// `exists` is zero when the latest completed query found no textdraw. When
/// it is one, all scalar fields are initialized from one R1 game-thread copy.
/// Flags use canonical zero or one values; colours retain their native R1
/// Direct3D representation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkTextDrawV1 {
    pub exists: u8,
    pub proportional: u8,
    pub align_left: u8,
    pub align_center: u8,
    pub align_right: u8,
    pub box_enabled: u8,
    pub _reserved: [u8; 2],
    pub pool_index: u16,
    pub shadow: u8,
    pub outline: u8,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_colour: u32,
    pub x: f32,
    pub y: f32,
    pub background_colour: u32,
    pub style: i32,
    pub box_width: f32,
    pub box_height: f32,
    pub box_colour: u32,
    pub model_id: u16,
    pub _reserved2: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub model_colour1: u16,
    pub model_colour2: u16,
    pub text_len: u16,
    pub _reserved3: [u8; 2],
    pub text: [u8; MAX_SAMP_TEXTDRAW_STRING_BYTES],
}

/// C-compatible storage for an owned [`ChatEntry`] result.
///
/// `text_len` and `prefix_len` select initialized, non-NUL byte prefixes;
/// neither buffer requires a terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkChatEntryV1 {
    pub id: u16,
    pub text_len: u8,
    pub prefix_len: u8,
    pub text_colour: u32,
    pub prefix_colour: u32,
    pub text: [u8; MAX_SAMP_CHAT_ENTRY_TEXT_BYTES],
    pub prefix: [u8; MAX_SAMP_CHAT_ENTRY_PREFIX_BYTES],
}

/// C-compatible storage for [`ServerInfo`].
///
/// This is output-only. Each length selects the initialized prefix of its
/// corresponding buffer; neither buffer requires a terminator.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkServerInfoV1 {
    pub address_len: u16,
    pub hostname_len: u16,
    pub address: [u8; 257],
    pub hostname: [u8; 257],
    pub port: u16,
}

/// C-compatible storage for [`LocalAnimation`].
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampClientSdkAnimationV1 {
    pub name_len: u8,
    pub file_len: u8,
    pub name: [u8; 36],
    pub file: [u8; 36],
}

macro_rules! impl_zeroed_abi_default {
    ($($type:ty),+ $(,)?) => {
        $(
            impl Default for $type {
                fn default() -> Self {
                    // Every listed ABI storage type contains only fields for
                    // which an all-zero bit pattern is valid.
                    unsafe { core::mem::zeroed() }
                }
            }
        )+
    };
}

impl_zeroed_abi_default!(
    SampClientSdkChatInputTextV1,
    SampClientSdkDialogListItemV1,
    SampClientSdkDialogSnapshotV1,
    SampClientSdkActiveDialogV1,
    SampClientSdkLocalPlayerV1,
    SampClientSdkPlayerInfoV1,
    SampClientSdkChatEntryV1,
    SampClientSdkTextDrawV1,
    SampClientSdkTextLabelV1,
    SampClientSdkServerInfoV1,
    SampClientSdkAnimationV1,
);
