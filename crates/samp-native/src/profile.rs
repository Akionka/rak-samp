use crate::SampVersion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeProfile {
    pub module_base: usize,
    pub spec: &'static ProfileSpec,
}

impl NativeProfile {
    /// Selects a data-only native profile for an exact supported executable identity.
    pub fn select(module_base: usize, version: SampVersion, entry_point: u32) -> Option<Self> {
        let spec = super::profiles::for_identity(version, entry_point)?;
        (module_base != 0).then_some(Self { module_base, spec })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileSpec {
    pub identity: ProfileIdentity,
    pub net_game: NetGameSpec,
    pub pools: PoolSpec,
    pub players: PlayerSpec,
    pub sync: SyncSpec,
    pub ui: UiSpec,
    pub text_labels: TextLabelSpec,
    pub textdraws: TextdrawSpec,
    pub handles: HandleSpec,
    pub strategies: ProfileStrategies,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileIdentity {
    pub name: &'static str,
    pub version: SampVersion,
    pub entry_point: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetGameSpec {
    pub singleton_rva: NativeRva,
    pub rak_client_offset: Option<FieldOffset>,
    pub get_state_rva: Option<NativeRva>,
    pub get_player_pool_rva: NativeRva,
    pub get_vehicle_pool_rva: NativeRva,
    pub shutdown_for_restart_rva: NativeRva,
    pub host_address_offset: FieldOffset,
    pub hostname_offset: FieldOffset,
    pub port_offset: FieldOffset,
    pub game_state_offset: FieldOffset,
    pub server_settings_offset: FieldOffset,
    pub pools_offset: FieldOffset,
    pub pools: NetGamePoolSpec,
    pub host_string_capacity: NativeSize,
    pub rak_client_disconnect_vtable_slot: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetGamePoolSpec {
    pub player_offset: Option<FieldOffset>,
    pub vehicle_offset: Option<FieldOffset>,
    pub object_offset: FieldOffset,
    pub gangzone_offset: FieldOffset,
    pub text_label_offset: FieldOffset,
    pub textdraw_offset: FieldOffset,
    pub pickup_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolSpec {
    pub limits: PoolLimits,
    pub player: PlayerPoolLayout,
    pub vehicle: VehiclePoolLayout,
    pub object: ObjectPoolLayout,
    pub pickup: PickupPoolLayout,
    pub text_label: TextLabelPoolLayout,
    pub textdraw: TextdrawPoolLayout,
    pub gangzone: GangzonePoolLayout,
    pub entity_handle_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PoolLimits {
    pub players: NativeLimit,
    pub vehicles: NativeLimit,
    pub objects: NativeLimit,
    pub text_labels: NativeLimit,
    pub textdraws: NativeLimit,
    pub gangzones: NativeLimit,
    pub pickups: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerPoolLayout {
    pub largest_id_offset: FieldOffset,
    pub local_id_offset: FieldOffset,
    pub objects_offset: Option<FieldOffset>,
    pub player_info: Option<PlayerInfoLayout>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerInfoLayout {
    pub npc_offset: FieldOffset,
    pub readable_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VehiclePoolLayout {
    pub not_empty_offset: FieldOffset,
    pub game_objects_offset: FieldOffset,
    pub does_exist_rva: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectPoolLayout {
    pub not_empty_offset: FieldOffset,
    pub objects_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PickupPoolLayout {
    pub handles_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLabelPoolLayout {
    pub not_empty_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextdrawPoolLayout {
    pub not_empty_offset: FieldOffset,
    pub objects_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GangzonePoolLayout {
    pub not_empty_offset: FieldOffset,
    pub left_offset: FieldOffset,
    pub bottom_offset: FieldOffset,
    pub right_offset: FieldOffset,
    pub top_offset: FieldOffset,
    pub colour_offset: FieldOffset,
    pub alternate_colour_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerSpec {
    pub pool_rvas: PlayerPoolRvas,
    pub remote_rvas: RemotePlayerRvas,
    pub local_rvas: LocalPlayerRvas,
    pub ped_rvas: PedRvas,
    pub local: LocalPlayerLayout,
    pub remote: RemotePlayerLayout,
    pub local_player_name_capacity: NativeSize,
    pub animation: AnimationTableSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlayerPoolRvas {
    pub get_local_player: NativeRva,
    pub get_local_score: NativeRva,
    pub get_local_ping: NativeRva,
    pub is_connected: NativeRva,
    pub get_remote_player: NativeRva,
    pub is_npc: Option<NativeRva>,
    pub get_name: NativeRva,
    pub get_score: NativeRva,
    pub get_ping: NativeRva,
    pub get_count: NativeRva,
    pub set_local_player_name: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemotePlayerRvas {
    pub get_colour_argb: NativeRva,
    pub set_colour: NativeRva,
    pub does_exist: NativeRva,
    pub get_status: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalPlayerRvas {
    pub get_ped: Option<NativeRva>,
    pub get_colour_argb: NativeRva,
    pub set_colour: NativeRva,
    pub set_special_action: NativeRva,
    pub spawn: NativeRva,
    pub send_unoccupied_data: NativeRva,
    pub send_aim_data: NativeRva,
    pub send_onfoot_data: NativeRva,
    pub send_stats: NativeRva,
    pub send_trailer_data: NativeRva,
    pub send_passenger_data: NativeRva,
    pub send_incar_data: NativeRva,
    pub update_weapons: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PedRvas {
    pub get_health: NativeRva,
    pub get_armour: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalPlayerLayout {
    pub ped_offset: Option<FieldOffset>,
    pub active_offset: FieldOffset,
    pub current_vehicle_offset: FieldOffset,
    pub onfoot_offset: FieldOffset,
    pub passenger_offset: FieldOffset,
    pub trailer_offset: FieldOffset,
    pub incar_offset: FieldOffset,
    pub aim_offset: FieldOffset,
    pub last_any_update_offset: FieldOffset,
    pub onfoot: LocalOnFootLayout,
    pub incar: LocalInCarLayout,
    pub game_ped_offset: FieldOffset,
    pub readable_size: Option<NativeSize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalOnFootLayout {
    pub position_offset: FieldOffset,
    pub speed_offset: FieldOffset,
    pub special_action_offset: FieldOffset,
    pub animation_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalInCarLayout {
    pub position_offset: FieldOffset,
    pub speed_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RemotePlayerLayout {
    pub ped_offset: Option<FieldOffset>,
    pub special_action_offset: FieldOffset,
    pub onfoot_offset: FieldOffset,
    pub incar_offset: FieldOffset,
    pub trailer_offset: FieldOffset,
    pub passenger_offset: FieldOffset,
    pub aim_offset: FieldOffset,
    pub reported_armour_offset: FieldOffset,
    pub reported_health_offset: FieldOffset,
    pub animation_offset: FieldOffset,
    pub state_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnimationTableSpec {
    pub rva: NativeRva,
    pub entry_count: NativeLimit,
    pub entry_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncSpec {
    pub send_rates: SyncSendRateRvas,
    pub onfoot: OnFootSyncLayout,
    pub incar: InCarSyncLayout,
    pub passenger: PassengerSyncLayout,
    pub trailer: TrailerSyncLayout,
    pub aim: AimSyncLayout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncSendRateRvas {
    pub onfoot: NativeRva,
    pub incar: NativeRva,
    pub aim: NativeRva,
}

macro_rules! offset_layout {
    ($name:ident { $($field:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name {
            pub size: NativeSize,
            $(pub $field: FieldOffset,)+
        }
    };
}

offset_layout!(OnFootSyncLayout {
    controller_left_stick_x,
    controller_left_stick_y,
    controller_buttons,
    position,
    quaternion,
    health,
    armour,
    weapon,
    special_action,
    speed,
    surfing_offset,
    surfing_vehicle_id,
    animation,
});
offset_layout!(InCarSyncLayout {
    vehicle_id,
    controller_left_stick_x,
    controller_left_stick_y,
    controller_buttons,
    quaternion,
    position,
    speed,
    vehicle_health,
    driver_health,
    driver_armour,
    weapon,
    siren,
    landing_gear,
    trailer_id,
    vehicle_specific,
});
offset_layout!(PassengerSyncLayout {
    vehicle_id,
    seat_id,
    weapon,
    health,
    armour,
    controller_left_stick_x,
    controller_left_stick_y,
    controller_buttons,
    position,
});
offset_layout!(TrailerSyncLayout {
    id,
    position,
    quaternion,
    speed,
    turn_speed
});
offset_layout!(AimSyncLayout {
    camera_mode,
    first,
    position,
    z,
    zoom_weapon_state,
    aspect_ratio
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiSpec {
    pub dialog: DialogSpec,
    pub input: InputSpec,
    pub chat: ChatSpec,
    pub scoreboard: ScoreboardSpec,
    pub death_window: DeathWindowSpec,
    pub game: GameSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialogSpec {
    pub singleton_rva: NativeRva,
    pub show_rva: NativeRva,
    pub close_rva: NativeRva,
    pub active_offset: FieldOffset,
    pub dialog_type_offset: FieldOffset,
    pub id_offset: FieldOffset,
    pub listbox_offset: FieldOffset,
    pub editbox_offset: FieldOffset,
    pub text_offset: FieldOffset,
    pub caption_offset: FieldOffset,
    pub caption_capacity: NativeSize,
    pub server_side_offset: FieldOffset,
    pub listbox: DialogListboxSpec,
    pub max_text_bytes: NativeSize,
    pub max_editbox_text_bytes: NativeSize,
    pub max_listbox_items: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialogListboxSpec {
    pub selected_offset: FieldOffset,
    pub items_offset: FieldOffset,
    pub item_count_offset: FieldOffset,
    pub item_text_offset: FieldOffset,
    pub item_text_capacity: NativeSize,
    pub item_data_offset: FieldOffset,
    pub item_active_rect_offset: FieldOffset,
    pub item_visible_offset: FieldOffset,
    pub item_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InputSpec {
    pub singleton_rva: NativeRva,
    pub open_rva: NativeRva,
    pub close_rva: NativeRva,
    pub get_command_handler_rva: NativeRva,
    pub add_command_rva: NativeRva,
    pub process_rva: NativeRva,
    pub edit_box_set_text_rva: Option<NativeRva>,
    pub edit_box_get_text_rva: Option<NativeRva>,
    pub enabled_offset: FieldOffset,
    pub edit_box_offset: FieldOffset,
    pub command_proc_offset: FieldOffset,
    pub command_name_offset: FieldOffset,
    pub command_name_capacity: NativeSize,
    pub command_count_offset: FieldOffset,
    pub max_text_bytes: NativeSize,
    pub max_commands: NativeLimit,
    pub max_command_name_bytes: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChatSpec {
    pub singleton_rva: NativeRva,
    pub add_entry_rva: NativeRva,
    pub get_mode_rva: NativeRva,
    pub display_mode_offset: FieldOffset,
    pub entries_offset: FieldOffset,
    pub entry_size: NativeSize,
    pub prefix_offset: FieldOffset,
    pub prefix_capacity: NativeSize,
    pub text_offset: FieldOffset,
    pub text_capacity: NativeSize,
    pub text_colour_offset: FieldOffset,
    pub prefix_colour_offset: FieldOffset,
    pub max_entries: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScoreboardSpec {
    pub singleton_rva: NativeRva,
    pub enabled_offset: FieldOffset,
    pub readable_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeathWindowSpec {
    pub singleton_rva: Option<NativeRva>,
    pub add_message_rva: Option<NativeRva>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameSpec {
    pub singleton_rva: NativeRva,
    pub set_cursor_mode_rva: NativeRva,
    pub process_input_enabling_rva: NativeRva,
    pub cursor_mode_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextLabelSpec {
    pub create_rva: NativeRva,
    pub delete_rva: NativeRva,
    pub size: NativeSize,
    pub text_offset: FieldOffset,
    pub colour_offset: FieldOffset,
    pub position_offset: FieldOffset,
    pub draw_distance_offset: FieldOffset,
    pub behind_walls_offset: FieldOffset,
    pub attached_player_offset: FieldOffset,
    pub attached_vehicle_offset: FieldOffset,
    pub text_capacity: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextdrawSpec {
    pub create_rva: NativeRva,
    pub delete_rva: NativeRva,
    pub text_setter_rva: NativeRva,
    pub native_size: NativeSize,
    pub string_offset: FieldOffset,
    pub create_text_capacity: NativeSize,
    pub stored_string_capacity: NativeSize,
    pub data_offset: FieldOffset,
    pub transmit: TextdrawTransmitLayout,
    pub data: TextdrawDataLayout,
}

offset_layout!(TextdrawTransmitLayout { x, y });
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextdrawDataLayout {
    pub width: FieldOffset,
    pub height: FieldOffset,
    pub colour: FieldOffset,
    pub align_center: FieldOffset,
    pub box_enabled: FieldOffset,
    pub box_width: FieldOffset,
    pub box_height: FieldOffset,
    pub box_colour: FieldOffset,
    pub proportional: FieldOffset,
    pub background_colour: FieldOffset,
    pub shadow: FieldOffset,
    pub outline: FieldOffset,
    pub align_left: FieldOffset,
    pub align_right: FieldOffset,
    pub style: FieldOffset,
    pub x: FieldOffset,
    pub y: FieldOffset,
    pub model_id: FieldOffset,
    pub rotation: FieldOffset,
    pub zoom: FieldOffset,
    pub model_colour1: FieldOffset,
    pub model_colour2: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandleSpec {
    pub rakpeer_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileStrategies {
    pub game_state_codec: GameStateCodec,
    pub local_player_source: LocalPlayerSource,
    pub pool_getter_abi: PoolGetterAbi,
    pub booleans: NativeBooleanPolicies,
    pub force_sync_reset: ForceSyncReset,
    pub list_item_text_layout: ListItemTextLayout,
    pub textdraw_calls: TextdrawCallStrategy,
}

/// Native boolean encodings grouped by the field families that consume them.
///
/// `LocalPlayerLayout::active_offset` is intentionally absent: it is a
/// non-zero activity marker, not a strict native boolean.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeBooleanPolicies {
    pub pool_occupancy: NativeBoolean,
    pub player_is_npc: NativeBoolean,
    pub dialog_active: NativeBoolean,
    pub dialog_server_side: NativeBoolean,
    pub input_enabled: NativeBoolean,
    pub label_behind_walls: NativeBoolean,
    pub textdraw_flags: NativeBoolean,
    pub vehicle_sync_flags: NativeBoolean,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStateCodec {
    Identity,
    Classic,
}

impl GameStateCodec {
    #[must_use]
    pub const fn encode(self, state: i32) -> Option<i32> {
        match self {
            Self::Identity => match state {
                0 | 9 | 13 | 14 | 15 | 18 => Some(state),
                _ => None,
            },
            Self::Classic => match state {
                0 => Some(0),
                9 => Some(1),
                13 => Some(2),
                14 => Some(5),
                15 => Some(6),
                18 => Some(11),
                _ => None,
            },
        }
    }

    #[must_use]
    pub const fn decode(self, state: i32) -> Option<i32> {
        match self {
            Self::Identity => match state {
                0 | 9 | 13 | 14 | 15 | 18 => Some(state),
                _ => None,
            },
            Self::Classic => match state {
                0 => Some(0),
                1 => Some(9),
                2 => Some(13),
                5 => Some(14),
                6 => Some(15),
                11 => Some(18),
                _ => None,
            },
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalPlayerSource {
    PlayerPoolGetter,
    NetGameField,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoolGetterAbi {
    R1,
    Classic,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeBoolean {
    ValidatedI32,
    ValidatedU8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForceSyncReset {
    ClearLastAnyUpdate,
    ProfileSpecific,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListItemTextLayout {
    DxutComboBoxItem,
    DirectPointer,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextdrawCallStrategy {
    NativeMethods,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeRva(usize);
impl NativeRva {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
    pub const fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FieldOffset(usize);
impl FieldOffset {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
    pub const fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSize(usize);
impl NativeSize {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
    pub const fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeLimit(usize);
impl NativeLimit {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
    pub const fn get(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_value_newtypes_preserve_their_categories() {
        assert_eq!(NativeRva::new(0xAC870).get(), 0xAC870);
        assert_eq!(FieldOffset::new(0x3BD).get(), 0x3BD);
        assert_eq!(NativeSize::new(0x9D6).get(), 0x9D6);
        assert_eq!(NativeLimit::new(2_304).get(), 2_304);
    }

    #[test]
    fn strategies_retain_confirmed_behavioral_differences() {
        assert_eq!(GameStateCodec::Identity, GameStateCodec::Identity);
        assert_eq!(
            LocalPlayerSource::PlayerPoolGetter,
            LocalPlayerSource::PlayerPoolGetter
        );
        assert_eq!(NativeBoolean::ValidatedI32, NativeBoolean::ValidatedI32);
        assert_eq!(NativeBoolean::ValidatedU8, NativeBoolean::ValidatedU8);
        assert_eq!(
            ForceSyncReset::ClearLastAnyUpdate,
            ForceSyncReset::ClearLastAnyUpdate
        );
        assert_eq!(
            ListItemTextLayout::DxutComboBoxItem,
            ListItemTextLayout::DxutComboBoxItem
        );
        assert_eq!(
            TextdrawCallStrategy::NativeMethods,
            TextdrawCallStrategy::NativeMethods
        );
    }

    #[test]
    fn selection_rejects_a_zero_module_base() {
        assert!(NativeProfile::select(0, SampVersion::R1, 0x31DF13).is_none());
    }
}
