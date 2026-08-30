use crate::LocalDialogStyle;

/// Host-owned data copied from the verified R1 game-thread client state.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalPlayerSnapshot {
    pub id: u16,
    pub nickname: Vec<u8>,
    pub colour: u32,
    pub spawned: bool,
    pub health: f32,
    pub armour: f32,
    pub position: Vector3,
    pub velocity: Vector3,
    pub special_action: u8,
    pub animation_id: u16,
    pub vehicle_id: Option<u16>,
    pub score: i32,
    pub ping: u32,
}

/// Host-owned metadata copied from one active R1 dialog. The dynamic dialog
/// text, editbox contents, and listbox item strings are bounded copies made on
/// the verified game thread; no native or DXUT pointer crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDialogSnapshot {
    pub id: i32,
    pub style: LocalDialogStyle,
    pub title: Vec<u8>,
    pub server_side: bool,
    pub selected_item: Option<i32>,
    pub list_item_count: Option<i32>,
    /// Bounded copy of the active dialog's dynamically allocated text.
    pub text: Vec<u8>,
    /// Bounded copy of the active dialog's editbox text. `None` marks dialogs
    /// without an editbox (for example a message box).
    pub editbox_text: Option<Vec<u8>>,
    /// Bounded copies of the active dialog's listbox item strings.
    pub listbox_items: Vec<Vec<u8>>,
}

/// Host-owned data captured immediately before an eligible R1 dialog closes.
/// It owns every byte so the native dialog and DXUT controls can be released as
/// soon as the close call continues.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalDialogResponseSnapshot {
    pub dialog_id: u16,
    pub button: u8,
    pub list_item: i32,
    pub input: Vec<u8>,
}

/// Host-owned directory data copied for either the local or one remote R1
/// player. It deliberately omits every native and GTA pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerInfoSnapshot {
    pub id: u16,
    pub defined: bool,
    pub paused: bool,
    pub nickname: Vec<u8>,
    pub is_local: bool,
    pub is_npc: bool,
    pub colour: u32,
    pub score: i32,
    pub ping: u32,
}

/// Host-owned volatile state copied from one remote R1 player record on the
/// verified game thread. It deliberately contains no player, ped, or GTA
/// pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RemotePlayerStateSnapshot {
    pub id: u16,
    pub health: f32,
    pub armour: f32,
    pub special_action: u8,
    pub animation_id: u16,
}

/// Host-owned R1 on-foot synchronization data copied on the verified game
/// thread. Controller and animation fields retain their native raw bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OnFootSyncSnapshot {
    pub id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub health: u8,
    pub armour: u8,
    pub weapon: u8,
    pub special_action: u8,
    pub speed: Vector3,
    pub surfing_offset: Vector3,
    pub surfing_vehicle_id: u16,
    pub animation: u32,
}

/// Host-owned R1 in-car synchronization data copied on the verified game
/// thread. Vehicle IDs and specific bytes retain their native raw values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InCarSyncSnapshot {
    pub id: u16,
    pub vehicle_id: u16,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub quaternion: [f32; 4],
    pub position: Vector3,
    pub speed: Vector3,
    pub vehicle_health: f32,
    pub driver_health: u8,
    pub driver_armour: u8,
    pub weapon: u8,
    pub siren: bool,
    pub landing_gear: bool,
    pub trailer_id: u16,
    pub vehicle_specific: [u8; 4],
}

/// Host-owned R1 passenger synchronization data copied on the verified game
/// thread. Vehicle, seat, and weapon values retain their native raw values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PassengerSyncSnapshot {
    pub id: u16,
    pub vehicle_id: u16,
    pub seat_id: u8,
    pub weapon: u8,
    pub health: u8,
    pub armour: u8,
    pub controller_left_stick_x: i16,
    pub controller_left_stick_y: i16,
    pub controller_buttons: i16,
    pub position: Vector3,
}

/// Host-owned R1 trailer synchronization data copied on the verified game
/// thread. The trailer ID retains its native raw value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailerSyncSnapshot {
    pub id: u16,
    pub trailer_id: u16,
    pub position: Vector3,
    pub quaternion: [f32; 4],
    pub speed: Vector3,
    pub turn_speed: Vector3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimSyncSnapshot {
    pub id: u16,
    pub camera_mode: u8,
    pub aim_first: Vector3,
    pub aim_position: Vector3,
    pub aim_z: f32,
    pub zoom_and_weapon_state: u8,
    pub aspect_ratio: u8,
}

/// Host-owned gangzone data copied from the verified R1 game thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GangzoneSnapshot {
    pub id: u16,
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub colour: u32,
    pub alternate_colour: u32,
}

/// Host-owned data copied from one R1 3D text-label record on the verified
/// game thread. The dynamically allocated native text is copied within its
/// protocol-bounded lifetime; no native pointer crosses this boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct TextLabelSnapshot {
    pub id: u16,
    pub text: Vec<u8>,
    pub colour: u32,
    pub position: Vector3,
    pub draw_distance: f32,
    pub behind_walls: bool,
    pub attached_player_id: Option<u16>,
    pub attached_vehicle_id: Option<u16>,
}

/// Host-owned data copied from one R1 textdraw record on the verified game
/// thread. The fixed native display-string buffer is copied before this value
/// crosses the private profile boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct TextdrawSnapshot {
    pub pool_index: u16,
    pub text: Vec<u8>,
    pub letter_width: f32,
    pub letter_height: f32,
    pub letter_colour: u32,
    pub x: f32,
    pub y: f32,
    pub shadow: u8,
    pub outline: u8,
    pub background_colour: u32,
    pub style: i32,
    pub proportional: bool,
    pub align_left: bool,
    pub align_center: bool,
    pub align_right: bool,
    pub box_enabled: bool,
    pub box_width: f32,
    pub box_height: f32,
    pub box_colour: u32,
    pub model_id: u16,
    pub rotation: Vector3,
    pub zoom: f32,
    pub model_colour1: u16,
    pub model_colour2: u16,
}

/// Host-owned data copied from one R1 fixed chat-history entry on the
/// verified game thread. No native chat or UI pointer crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChatEntrySnapshot {
    pub id: u16,
    pub text: Vec<u8>,
    pub prefix: Vec<u8>,
    pub text_colour: u32,
    pub prefix_colour: u32,
}

/// Host-owned current-server metadata copied from the verified R1 game thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerInfoSnapshot {
    pub address: Vec<u8>,
    pub hostname: Vec<u8>,
    pub port: u16,
}

/// One owned entry from R1's fixed animation-name table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnimationSnapshot {
    pub name: Vec<u8>,
    pub file: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
