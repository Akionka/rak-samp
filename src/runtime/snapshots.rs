use super::LocalDialogStyle;

/// Host-owned data copied from the verified R1 game-thread client state.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LocalPlayerSnapshot {
    pub(crate) id: u16,
    pub(crate) nickname: Vec<u8>,
    pub(crate) colour: u32,
    pub(crate) spawned: bool,
    pub(crate) health: f32,
    pub(crate) armour: f32,
    pub(crate) position: Vector3,
    pub(crate) velocity: Vector3,
    pub(crate) special_action: u8,
    pub(crate) animation_id: u16,
    pub(crate) vehicle_id: Option<u16>,
    pub(crate) score: i32,
    pub(crate) ping: u32,
}

/// Host-owned metadata copied from one active R1 dialog. The dynamic dialog
/// text, editbox contents, and listbox item strings are bounded copies made on
/// the verified game thread; no native or DXUT pointer crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalDialogSnapshot {
    pub(crate) id: i32,
    pub(crate) style: LocalDialogStyle,
    pub(crate) title: Vec<u8>,
    pub(crate) server_side: bool,
    pub(crate) selected_item: Option<i32>,
    pub(crate) list_item_count: Option<i32>,
    /// Bounded copy of the active dialog's dynamically allocated text.
    pub(crate) text: Vec<u8>,
    /// Bounded copy of the active dialog's editbox text. `None` marks dialogs
    /// without an editbox (for example a message box).
    pub(crate) editbox_text: Option<Vec<u8>>,
    /// Bounded copies of the active dialog's listbox item strings.
    pub(crate) listbox_items: Vec<Vec<u8>>,
}

/// Host-owned data captured immediately before an eligible R1 dialog closes.
/// It owns every byte so the native dialog and DXUT controls can be released as
/// soon as the close call continues.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalDialogResponseSnapshot {
    pub(crate) dialog_id: u16,
    pub(crate) button: u8,
    pub(crate) list_item: i32,
    pub(crate) input: Vec<u8>,
}

/// Host-owned directory data copied for either the local or one remote R1
/// player. It deliberately omits every native and GTA pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlayerInfoSnapshot {
    pub(crate) id: u16,
    pub(crate) defined: bool,
    pub(crate) paused: bool,
    pub(crate) nickname: Vec<u8>,
    pub(crate) is_local: bool,
    pub(crate) is_npc: bool,
    pub(crate) colour: u32,
    pub(crate) score: i32,
    pub(crate) ping: u32,
}

/// Host-owned volatile state copied from one remote R1 player record on the
/// verified game thread. It deliberately contains no player, ped, or GTA
/// pointer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RemotePlayerStateSnapshot {
    pub(crate) id: u16,
    pub(crate) health: f32,
    pub(crate) armour: f32,
    pub(crate) special_action: u8,
    pub(crate) animation_id: u16,
}

/// Host-owned R1 on-foot synchronization data copied on the verified game
/// thread. Controller and animation fields retain their native raw bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct OnFootSyncSnapshot {
    pub(crate) id: u16,
    pub(crate) controller_left_stick_x: i16,
    pub(crate) controller_left_stick_y: i16,
    pub(crate) controller_buttons: i16,
    pub(crate) position: Vector3,
    pub(crate) quaternion: [f32; 4],
    pub(crate) health: u8,
    pub(crate) armour: u8,
    pub(crate) weapon: u8,
    pub(crate) special_action: u8,
    pub(crate) speed: Vector3,
    pub(crate) surfing_offset: Vector3,
    pub(crate) surfing_vehicle_id: u16,
    pub(crate) animation: u32,
}

/// Host-owned R1 in-car synchronization data copied on the verified game
/// thread. Vehicle IDs and specific bytes retain their native raw values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct InCarSyncSnapshot {
    pub(crate) id: u16,
    pub(crate) vehicle_id: u16,
    pub(crate) controller_left_stick_x: i16,
    pub(crate) controller_left_stick_y: i16,
    pub(crate) controller_buttons: i16,
    pub(crate) quaternion: [f32; 4],
    pub(crate) position: Vector3,
    pub(crate) speed: Vector3,
    pub(crate) vehicle_health: f32,
    pub(crate) driver_health: u8,
    pub(crate) driver_armour: u8,
    pub(crate) weapon: u8,
    pub(crate) siren: bool,
    pub(crate) landing_gear: bool,
    pub(crate) trailer_id: u16,
    pub(crate) vehicle_specific: [u8; 4],
}

/// Host-owned R1 passenger synchronization data copied on the verified game
/// thread. Vehicle, seat, and weapon values retain their native raw values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PassengerSyncSnapshot {
    pub(crate) id: u16,
    pub(crate) vehicle_id: u16,
    pub(crate) seat_id: u8,
    pub(crate) weapon: u8,
    pub(crate) health: u8,
    pub(crate) armour: u8,
    pub(crate) controller_left_stick_x: i16,
    pub(crate) controller_left_stick_y: i16,
    pub(crate) controller_buttons: i16,
    pub(crate) position: Vector3,
}

/// Host-owned R1 trailer synchronization data copied on the verified game
/// thread. The trailer ID retains its native raw value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TrailerSyncSnapshot {
    pub(crate) id: u16,
    pub(crate) trailer_id: u16,
    pub(crate) position: Vector3,
    pub(crate) quaternion: [f32; 4],
    pub(crate) speed: Vector3,
    pub(crate) turn_speed: Vector3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AimSyncSnapshot {
    pub(crate) id: u16,
    pub(crate) camera_mode: u8,
    pub(crate) aim_first: Vector3,
    pub(crate) aim_position: Vector3,
    pub(crate) aim_z: f32,
    pub(crate) zoom_and_weapon_state: u8,
    pub(crate) aspect_ratio: u8,
}

/// Host-owned gangzone data copied from the verified R1 game thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GangzoneSnapshot {
    pub(crate) id: u16,
    pub(crate) left: f32,
    pub(crate) bottom: f32,
    pub(crate) right: f32,
    pub(crate) top: f32,
    pub(crate) colour: u32,
    pub(crate) alternate_colour: u32,
}

/// Host-owned data copied from one R1 3D text-label record on the verified
/// game thread. The dynamically allocated native text is copied within its
/// protocol-bounded lifetime; no native pointer crosses this boundary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextLabelSnapshot {
    pub(crate) id: u16,
    pub(crate) text: Vec<u8>,
    pub(crate) colour: u32,
    pub(crate) position: Vector3,
    pub(crate) draw_distance: f32,
    pub(crate) behind_walls: bool,
    pub(crate) attached_player_id: Option<u16>,
    pub(crate) attached_vehicle_id: Option<u16>,
}

/// Host-owned data copied from one R1 textdraw record on the verified game
/// thread. The fixed native display-string buffer is copied before this value
/// crosses the private profile boundary.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextdrawSnapshot {
    pub(crate) pool_index: u16,
    pub(crate) text: Vec<u8>,
    pub(crate) letter_width: f32,
    pub(crate) letter_height: f32,
    pub(crate) letter_colour: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) shadow: u8,
    pub(crate) outline: u8,
    pub(crate) background_colour: u32,
    pub(crate) style: i32,
    pub(crate) proportional: bool,
    pub(crate) align_left: bool,
    pub(crate) align_center: bool,
    pub(crate) align_right: bool,
    pub(crate) box_enabled: bool,
    pub(crate) box_width: f32,
    pub(crate) box_height: f32,
    pub(crate) box_colour: u32,
    pub(crate) model_id: u16,
    pub(crate) rotation: Vector3,
    pub(crate) zoom: f32,
    pub(crate) model_colour1: u16,
    pub(crate) model_colour2: u16,
}

/// Host-owned data copied from one R1 fixed chat-history entry on the
/// verified game thread. No native chat or UI pointer crosses this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChatEntrySnapshot {
    pub(crate) id: u16,
    pub(crate) text: Vec<u8>,
    pub(crate) prefix: Vec<u8>,
    pub(crate) text_colour: u32,
    pub(crate) prefix_colour: u32,
}

/// Host-owned current-server metadata copied from the verified R1 game thread.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ServerInfoSnapshot {
    pub(crate) address: Vec<u8>,
    pub(crate) hostname: Vec<u8>,
    pub(crate) port: u16,
}

/// One owned entry from R1's fixed animation-name table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AnimationSnapshot {
    pub(crate) name: Vec<u8>,
    pub(crate) file: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vector3 {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}
