use crate::client::SampVersion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeClientProfile {
    pub(crate) module_base: usize,
    pub(crate) spec: &'static ProfileSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProfileSpec {
    pub(crate) identity: ProfileIdentity,
    pub(crate) net_game: NetGameSpec,
    pub(crate) pools: PoolSpec,
    pub(crate) players: PlayerSpec,
    pub(crate) sync: SyncSpec,
    pub(crate) ui: UiSpec,
    pub(crate) text_labels: TextLabelSpec,
    pub(crate) textdraws: TextdrawSpec,
    pub(crate) handles: HandleSpec,
    pub(crate) strategies: ProfileStrategies,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProfileIdentity {
    pub(crate) name: &'static str,
    pub(crate) version: SampVersion,
    pub(crate) entry_point: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetGameSpec {
    pub(crate) singleton_rva: NativeRva,
    pub(crate) get_state_rva: Option<NativeRva>,
    pub(crate) get_player_pool_rva: NativeRva,
    pub(crate) get_vehicle_pool_rva: NativeRva,
    pub(crate) shutdown_for_restart_rva: NativeRva,
    pub(crate) host_address_offset: FieldOffset,
    pub(crate) hostname_offset: FieldOffset,
    pub(crate) port_offset: FieldOffset,
    pub(crate) game_state_offset: FieldOffset,
    pub(crate) server_settings_offset: FieldOffset,
    pub(crate) pools_offset: FieldOffset,
    pub(crate) pools: NetGamePoolSpec,
    pub(crate) host_string_capacity: NativeSize,
    pub(crate) rak_client_disconnect_vtable_slot: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NetGamePoolSpec {
    pub(crate) player_offset: Option<FieldOffset>,
    pub(crate) vehicle_offset: Option<FieldOffset>,
    pub(crate) object_offset: FieldOffset,
    pub(crate) gangzone_offset: FieldOffset,
    pub(crate) text_label_offset: FieldOffset,
    pub(crate) textdraw_offset: FieldOffset,
    pub(crate) pickup_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PoolSpec {
    pub(crate) limits: PoolLimits,
    pub(crate) player: PlayerPoolLayout,
    pub(crate) vehicle: VehiclePoolLayout,
    pub(crate) object: ObjectPoolLayout,
    pub(crate) pickup: PickupPoolLayout,
    pub(crate) text_label: TextLabelPoolLayout,
    pub(crate) textdraw: TextdrawPoolLayout,
    pub(crate) gangzone: GangzonePoolLayout,
    pub(crate) entity_handle_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PoolLimits {
    pub(crate) players: NativeLimit,
    pub(crate) vehicles: NativeLimit,
    pub(crate) objects: NativeLimit,
    pub(crate) text_labels: NativeLimit,
    pub(crate) textdraws: NativeLimit,
    pub(crate) gangzones: NativeLimit,
    pub(crate) pickups: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPoolLayout {
    pub(crate) largest_id_offset: FieldOffset,
    pub(crate) local_id_offset: FieldOffset,
    pub(crate) objects_offset: Option<FieldOffset>,
    pub(crate) player_info: Option<PlayerInfoLayout>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerInfoLayout {
    pub(crate) npc_offset: FieldOffset,
    pub(crate) readable_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VehiclePoolLayout {
    pub(crate) not_empty_offset: FieldOffset,
    pub(crate) game_objects_offset: FieldOffset,
    pub(crate) does_exist_rva: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ObjectPoolLayout {
    pub(crate) not_empty_offset: FieldOffset,
    pub(crate) objects_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PickupPoolLayout {
    pub(crate) handles_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextLabelPoolLayout {
    pub(crate) not_empty_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextdrawPoolLayout {
    pub(crate) not_empty_offset: FieldOffset,
    pub(crate) objects_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GangzonePoolLayout {
    pub(crate) not_empty_offset: FieldOffset,
    pub(crate) left_offset: FieldOffset,
    pub(crate) bottom_offset: FieldOffset,
    pub(crate) right_offset: FieldOffset,
    pub(crate) top_offset: FieldOffset,
    pub(crate) colour_offset: FieldOffset,
    pub(crate) alternate_colour_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSpec {
    pub(crate) pool_rvas: PlayerPoolRvas,
    pub(crate) remote_rvas: RemotePlayerRvas,
    pub(crate) local_rvas: LocalPlayerRvas,
    pub(crate) ped_rvas: PedRvas,
    pub(crate) local: LocalPlayerLayout,
    pub(crate) remote: RemotePlayerLayout,
    pub(crate) local_player_name_capacity: NativeSize,
    pub(crate) animation: AnimationTableSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerPoolRvas {
    pub(crate) get_local_player: NativeRva,
    pub(crate) get_local_score: NativeRva,
    pub(crate) get_local_ping: NativeRva,
    pub(crate) is_connected: NativeRva,
    pub(crate) get_remote_player: NativeRva,
    pub(crate) is_npc: Option<NativeRva>,
    pub(crate) get_name: NativeRva,
    pub(crate) get_score: NativeRva,
    pub(crate) get_ping: NativeRva,
    pub(crate) get_count: NativeRva,
    pub(crate) set_local_player_name: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RemotePlayerRvas {
    pub(crate) get_colour_argb: NativeRva,
    pub(crate) set_colour: NativeRva,
    pub(crate) does_exist: NativeRva,
    pub(crate) get_status: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalPlayerRvas {
    pub(crate) get_ped: Option<NativeRva>,
    pub(crate) get_colour_argb: NativeRva,
    pub(crate) set_colour: NativeRva,
    pub(crate) set_special_action: NativeRva,
    pub(crate) spawn: NativeRva,
    pub(crate) send_unoccupied_data: NativeRva,
    pub(crate) send_aim_data: NativeRva,
    pub(crate) send_onfoot_data: NativeRva,
    pub(crate) send_stats: NativeRva,
    pub(crate) send_trailer_data: NativeRva,
    pub(crate) send_passenger_data: NativeRva,
    pub(crate) send_incar_data: NativeRva,
    pub(crate) update_weapons: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PedRvas {
    pub(crate) get_health: NativeRva,
    pub(crate) get_armour: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalPlayerLayout {
    pub(crate) ped_offset: Option<FieldOffset>,
    pub(crate) active_offset: FieldOffset,
    pub(crate) current_vehicle_offset: FieldOffset,
    pub(crate) onfoot_offset: FieldOffset,
    pub(crate) passenger_offset: FieldOffset,
    pub(crate) trailer_offset: FieldOffset,
    pub(crate) incar_offset: FieldOffset,
    pub(crate) aim_offset: FieldOffset,
    pub(crate) last_any_update_offset: FieldOffset,
    pub(crate) onfoot: LocalOnFootLayout,
    pub(crate) incar: LocalInCarLayout,
    pub(crate) game_ped_offset: FieldOffset,
    pub(crate) readable_size: Option<NativeSize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalOnFootLayout {
    pub(crate) position_offset: FieldOffset,
    pub(crate) speed_offset: FieldOffset,
    pub(crate) special_action_offset: FieldOffset,
    pub(crate) animation_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LocalInCarLayout {
    pub(crate) position_offset: FieldOffset,
    pub(crate) speed_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RemotePlayerLayout {
    pub(crate) special_action_offset: FieldOffset,
    pub(crate) onfoot_offset: FieldOffset,
    pub(crate) incar_offset: FieldOffset,
    pub(crate) trailer_offset: FieldOffset,
    pub(crate) passenger_offset: FieldOffset,
    pub(crate) aim_offset: FieldOffset,
    pub(crate) reported_armour_offset: FieldOffset,
    pub(crate) reported_health_offset: FieldOffset,
    pub(crate) animation_offset: FieldOffset,
    pub(crate) state_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AnimationTableSpec {
    pub(crate) rva: NativeRva,
    pub(crate) entry_count: NativeLimit,
    pub(crate) entry_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SyncSpec {
    pub(crate) send_rates: SyncSendRateRvas,
    pub(crate) onfoot: OnFootSyncLayout,
    pub(crate) incar: InCarSyncLayout,
    pub(crate) passenger: PassengerSyncLayout,
    pub(crate) trailer: TrailerSyncLayout,
    pub(crate) aim: AimSyncLayout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SyncSendRateRvas {
    pub(crate) onfoot: NativeRva,
    pub(crate) incar: NativeRva,
    pub(crate) aim: NativeRva,
}

macro_rules! offset_layout {
    ($name:ident { $($field:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub(crate) struct $name {
            pub(crate) size: NativeSize,
            $(pub(crate) $field: FieldOffset,)+
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
pub(crate) struct UiSpec {
    pub(crate) dialog: DialogSpec,
    pub(crate) input: InputSpec,
    pub(crate) chat: ChatSpec,
    pub(crate) scoreboard: ScoreboardSpec,
    pub(crate) death_window: DeathWindowSpec,
    pub(crate) game: GameSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DialogSpec {
    pub(crate) singleton_rva: NativeRva,
    pub(crate) show_rva: NativeRva,
    pub(crate) close_rva: NativeRva,
    pub(crate) active_offset: FieldOffset,
    pub(crate) dialog_type_offset: FieldOffset,
    pub(crate) id_offset: FieldOffset,
    pub(crate) listbox_offset: FieldOffset,
    pub(crate) editbox_offset: FieldOffset,
    pub(crate) text_offset: FieldOffset,
    pub(crate) caption_offset: FieldOffset,
    pub(crate) caption_capacity: NativeSize,
    pub(crate) server_side_offset: FieldOffset,
    pub(crate) listbox: DialogListboxSpec,
    pub(crate) max_text_bytes: NativeSize,
    pub(crate) max_editbox_text_bytes: NativeSize,
    pub(crate) max_listbox_items: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DialogListboxSpec {
    pub(crate) selected_offset: FieldOffset,
    pub(crate) items_offset: FieldOffset,
    pub(crate) item_count_offset: FieldOffset,
    pub(crate) item_text_offset: FieldOffset,
    pub(crate) item_text_capacity: NativeSize,
    pub(crate) item_data_offset: FieldOffset,
    pub(crate) item_active_rect_offset: FieldOffset,
    pub(crate) item_visible_offset: FieldOffset,
    pub(crate) item_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InputSpec {
    pub(crate) singleton_rva: NativeRva,
    pub(crate) open_rva: NativeRva,
    pub(crate) close_rva: NativeRva,
    pub(crate) get_command_handler_rva: NativeRva,
    pub(crate) add_command_rva: NativeRva,
    pub(crate) process_rva: NativeRva,
    pub(crate) edit_box_set_text_rva: Option<NativeRva>,
    pub(crate) edit_box_get_text_rva: Option<NativeRva>,
    pub(crate) enabled_offset: FieldOffset,
    pub(crate) edit_box_offset: FieldOffset,
    pub(crate) command_proc_offset: FieldOffset,
    pub(crate) command_name_offset: FieldOffset,
    pub(crate) command_name_capacity: NativeSize,
    pub(crate) command_count_offset: FieldOffset,
    pub(crate) max_text_bytes: NativeSize,
    pub(crate) max_commands: NativeLimit,
    pub(crate) max_command_name_bytes: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ChatSpec {
    pub(crate) singleton_rva: NativeRva,
    pub(crate) add_entry_rva: NativeRva,
    pub(crate) get_mode_rva: NativeRva,
    pub(crate) display_mode_offset: FieldOffset,
    pub(crate) entries_offset: FieldOffset,
    pub(crate) entry_size: NativeSize,
    pub(crate) prefix_offset: FieldOffset,
    pub(crate) prefix_capacity: NativeSize,
    pub(crate) text_offset: FieldOffset,
    pub(crate) text_capacity: NativeSize,
    pub(crate) text_colour_offset: FieldOffset,
    pub(crate) prefix_colour_offset: FieldOffset,
    pub(crate) max_entries: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScoreboardSpec {
    pub(crate) singleton_rva: NativeRva,
    pub(crate) enabled_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeathWindowSpec {
    pub(crate) singleton_rva: Option<NativeRva>,
    pub(crate) add_message_rva: Option<NativeRva>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GameSpec {
    pub(crate) singleton_rva: NativeRva,
    pub(crate) set_cursor_mode_rva: NativeRva,
    pub(crate) process_input_enabling_rva: NativeRva,
    pub(crate) cursor_mode_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextLabelSpec {
    pub(crate) create_rva: NativeRva,
    pub(crate) delete_rva: NativeRva,
    pub(crate) size: NativeSize,
    pub(crate) text_offset: FieldOffset,
    pub(crate) colour_offset: FieldOffset,
    pub(crate) position_offset: FieldOffset,
    pub(crate) draw_distance_offset: FieldOffset,
    pub(crate) behind_walls_offset: FieldOffset,
    pub(crate) attached_player_offset: FieldOffset,
    pub(crate) attached_vehicle_offset: FieldOffset,
    pub(crate) text_capacity: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextdrawSpec {
    pub(crate) create_rva: NativeRva,
    pub(crate) delete_rva: NativeRva,
    pub(crate) text_setter_rva: NativeRva,
    pub(crate) native_size: NativeSize,
    pub(crate) string_offset: FieldOffset,
    pub(crate) create_text_capacity: NativeSize,
    pub(crate) stored_string_capacity: NativeSize,
    pub(crate) data_offset: FieldOffset,
    pub(crate) transmit: TextdrawTransmitLayout,
    pub(crate) data: TextdrawDataLayout,
}

offset_layout!(TextdrawTransmitLayout { x, y });
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextdrawDataLayout {
    pub(crate) width: FieldOffset,
    pub(crate) height: FieldOffset,
    pub(crate) colour: FieldOffset,
    pub(crate) align_center: FieldOffset,
    pub(crate) box_enabled: FieldOffset,
    pub(crate) box_width: FieldOffset,
    pub(crate) box_height: FieldOffset,
    pub(crate) box_colour: FieldOffset,
    pub(crate) proportional: FieldOffset,
    pub(crate) background_colour: FieldOffset,
    pub(crate) shadow: FieldOffset,
    pub(crate) outline: FieldOffset,
    pub(crate) align_left: FieldOffset,
    pub(crate) align_right: FieldOffset,
    pub(crate) style: FieldOffset,
    pub(crate) x: FieldOffset,
    pub(crate) y: FieldOffset,
    pub(crate) model_id: FieldOffset,
    pub(crate) rotation: FieldOffset,
    pub(crate) zoom: FieldOffset,
    pub(crate) model_colour1: FieldOffset,
    pub(crate) model_colour2: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HandleSpec {
    pub(crate) rakpeer_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProfileStrategies {
    pub(crate) game_state_codec: GameStateCodec,
    pub(crate) local_player_source: LocalPlayerSource,
    pub(crate) i32_boolean: NativeBoolean,
    pub(crate) u8_boolean: NativeBoolean,
    pub(crate) force_sync_reset: ForceSyncReset,
    pub(crate) list_item_text_layout: ListItemTextLayout,
    pub(crate) textdraw_calls: TextdrawCallStrategy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GameStateCodec {
    Identity,
    Classic,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LocalPlayerSource {
    PlayerPoolGetter,
    NetGameField,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeBoolean {
    ValidatedI32,
    ValidatedU8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ForceSyncReset {
    ClearLastAnyUpdate,
    ProfileSpecific,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ListItemTextLayout {
    DxutComboBoxItem,
    DirectPointer,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TextdrawCallStrategy {
    NativeMethods,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeRva(usize);
impl NativeRva {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }
    pub(crate) const fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FieldOffset(usize);
impl FieldOffset {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }
    pub(crate) const fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeSize(usize);
impl NativeSize {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }
    pub(crate) const fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeLimit(usize);
impl NativeLimit {
    pub(crate) const fn new(value: usize) -> Self {
        Self(value)
    }
    pub(crate) const fn get(self) -> usize {
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
}
