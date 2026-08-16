use super::super::profile::*;
use crate::client::SampVersion;

const fn rva(value: usize) -> NativeRva {
    NativeRva::new(value)
}
const fn offset(value: usize) -> FieldOffset {
    FieldOffset::new(value)
}
const fn size(value: usize) -> NativeSize {
    NativeSize::new(value)
}
const fn limit(value: usize) -> NativeLimit {
    NativeLimit::new(value)
}

pub(crate) const R1_SPEC: ProfileSpec = ProfileSpec {
    identity: ProfileIdentity {
        name: "SA-MP 0.3.7 R1",
        version: SampVersion::R1,
        entry_point: 0x31DF13,
    },
    net_game: NetGameSpec {
        singleton_rva: rva(0x21A0F8),
        get_state_rva: rva(0x2E20),
        get_player_pool_rva: rva(0x1160),
        get_vehicle_pool_rva: rva(0x1170),
        shutdown_for_restart_rva: rva(0xA060),
        host_address_offset: offset(0x20),
        hostname_offset: offset(0x121),
        port_offset: offset(0x225),
        game_state_offset: offset(0x3BD),
        server_settings_offset: offset(0x3C5),
        pools_offset: offset(0x3CD),
        pools: NetGamePoolSpec {
            object_offset: offset(0x04),
            gangzone_offset: offset(0x08),
            text_label_offset: offset(0x0C),
            textdraw_offset: offset(0x10),
            pickup_offset: offset(0x20),
        },
        host_string_capacity: size(257),
        rak_client_disconnect_vtable_slot: 2,
    },
    pools: PoolSpec {
        limits: PoolLimits {
            players: limit(1004),
            vehicles: limit(2000),
            objects: limit(1000),
            text_labels: limit(2048),
            textdraws: limit(2304),
            gangzones: limit(1024),
            pickups: limit(4096),
        },
        player: PlayerPoolLayout {
            largest_id_offset: offset(0),
            local_id_offset: offset(4),
        },
        vehicle: VehiclePoolLayout {
            not_empty_offset: offset(0x3074),
            game_objects_offset: offset(0x4FB4),
            does_exist_rva: rva(0x1140),
        },
        object: ObjectPoolLayout {
            not_empty_offset: offset(0x04),
            objects_offset: offset(0xFA4),
        },
        pickup: PickupPoolLayout {
            handles_offset: offset(0x04),
        },
        text_label: TextLabelPoolLayout {
            not_empty_offset: offset(0xE800),
        },
        textdraw: TextdrawPoolLayout {
            not_empty_offset: offset(0),
            objects_offset: offset(0x2400),
        },
        gangzone: GangzonePoolLayout {
            not_empty_offset: offset(0x1000),
            left_offset: offset(0),
            bottom_offset: offset(4),
            right_offset: offset(8),
            top_offset: offset(0xC),
            colour_offset: offset(0x10),
            alternate_colour_offset: offset(0x14),
        },
        entity_handle_offset: offset(0x44),
    },
    players: PlayerSpec {
        pool_rvas: PlayerPoolRvas {
            get_local_player: rva(0x1A30),
            get_local_score: rva(0x6A1F0),
            get_local_ping: rva(0x6A200),
            is_connected: rva(0x10B0),
            get_remote_player: rva(0x10F0),
            is_npc: rva(0xB680),
            get_name: rva(0x13CE0),
            get_score: rva(0x6A190),
            get_ping: rva(0x6A1C0),
            get_count: rva(0x10520),
            set_local_player_name: rva(0xB3E0),
        },
        remote_rvas: RemotePlayerRvas {
            get_colour_argb: rva(0x12A00),
            set_colour: rva(0x129D0),
            does_exist: rva(0x1080),
            get_status: rva(0x12BA0),
        },
        local_rvas: LocalPlayerRvas {
            get_ped: rva(0x2D60),
            get_colour_argb: rva(0x3D90),
            set_colour: rva(0x3D40),
            set_special_action: rva(0x30C0),
            spawn: rva(0x3AD0),
            send_unoccupied_data: rva(0x4B30),
            send_aim_data: rva(0x4FF0),
            send_onfoot_data: rva(0x4D10),
            send_stats: rva(0x5AF0),
            send_trailer_data: rva(0x51B0),
            send_passenger_data: rva(0x5380),
            send_incar_data: rva(0x6E30),
            update_weapons: rva(0x6080),
        },
        ped_rvas: PedRvas {
            get_health: rva(0xA6610),
            get_armour: rva(0xA6650),
        },
        local: LocalPlayerLayout {
            active_offset: offset(0xC),
            current_vehicle_offset: offset(0x14),
            onfoot_offset: offset(0x18),
            passenger_offset: offset(0x5C),
            trailer_offset: offset(0x74),
            incar_offset: offset(0xAA),
            aim_offset: offset(0xE9),
            last_any_update_offset: offset(0x1D8),
            onfoot: LocalOnFootLayout {
                position_offset: offset(0x06),
                speed_offset: offset(0x26),
                special_action_offset: offset(0x25),
                animation_offset: offset(0x40),
            },
            incar: LocalInCarLayout {
                position_offset: offset(0x18),
                speed_offset: offset(0x24),
            },
            game_ped_offset: offset(0x2A4),
        },
        remote: RemotePlayerLayout {
            special_action_offset: offset(0xBB),
            onfoot_offset: offset(0xC8),
            incar_offset: offset(0x10C),
            trailer_offset: offset(0x14B),
            passenger_offset: offset(0x181),
            aim_offset: offset(0x199),
            reported_armour_offset: offset(0x1B8),
            reported_health_offset: offset(0x1BC),
            animation_offset: offset(0x1C0),
            state_size: size(0x1C4),
        },
        local_player_name_capacity: size(255),
        animation: AnimationTableSpec {
            rva: rva(0xF15B0),
            entry_count: limit(1812),
            entry_size: size(36),
        },
    },
    sync: SyncSpec {
        send_rates: SyncSendRateRvas {
            onfoot: rva(0xEC0A8),
            incar: rva(0xEC0AC),
            aim: rva(0xEC0B0),
        },
        onfoot: OnFootSyncLayout {
            size: size(68),
            controller_left_stick_x: offset(0),
            controller_left_stick_y: offset(2),
            controller_buttons: offset(4),
            position: offset(6),
            quaternion: offset(0x12),
            health: offset(0x22),
            armour: offset(0x23),
            weapon: offset(0x24),
            special_action: offset(0x25),
            speed: offset(0x26),
            surfing_offset: offset(0x32),
            surfing_vehicle_id: offset(0x3E),
            animation: offset(0x40),
        },
        incar: InCarSyncLayout {
            size: size(63),
            vehicle_id: offset(0),
            controller_left_stick_x: offset(2),
            controller_left_stick_y: offset(4),
            controller_buttons: offset(6),
            quaternion: offset(8),
            position: offset(0x18),
            speed: offset(0x24),
            vehicle_health: offset(0x30),
            driver_health: offset(0x34),
            driver_armour: offset(0x35),
            weapon: offset(0x36),
            siren: offset(0x37),
            landing_gear: offset(0x38),
            trailer_id: offset(0x39),
            vehicle_specific: offset(0x3B),
        },
        passenger: PassengerSyncLayout {
            size: size(24),
            vehicle_id: offset(0),
            seat_id: offset(2),
            weapon: offset(3),
            health: offset(4),
            armour: offset(5),
            controller_left_stick_x: offset(6),
            controller_left_stick_y: offset(8),
            controller_buttons: offset(0xA),
            position: offset(0xC),
        },
        trailer: TrailerSyncLayout {
            size: size(54),
            id: offset(0),
            position: offset(2),
            quaternion: offset(0xE),
            speed: offset(0x1E),
            turn_speed: offset(0x2A),
        },
        aim: AimSyncLayout {
            size: size(31),
            camera_mode: offset(0),
            first: offset(1),
            position: offset(0xD),
            z: offset(0x19),
            zoom_weapon_state: offset(0x1D),
            aspect_ratio: offset(0x1E),
        },
    },
    ui: UiSpec {
        dialog: DialogSpec {
            singleton_rva: rva(0x21A0B8),
            show_rva: rva(0x6B9C0),
            close_rva: rva(0x6C040),
            active_offset: offset(0x28),
            dialog_type_offset: offset(0x2C),
            id_offset: offset(0x30),
            listbox_offset: offset(0x20),
            editbox_offset: offset(0x24),
            text_offset: offset(0x34),
            caption_offset: offset(0x40),
            caption_capacity: size(65),
            server_side_offset: offset(0x81),
            listbox: DialogListboxSpec {
                selected_offset: offset(0x143),
                items_offset: offset(0x14C),
                item_count_offset: offset(0x150),
                item_text_offset: offset(0),
                item_text_capacity: size(256),
                item_data_offset: offset(0x100),
                item_active_rect_offset: offset(0x104),
                item_visible_offset: offset(0x114),
                item_size: size(0x118),
            },
            max_text_bytes: size(4096),
            max_editbox_text_bytes: size(128),
            max_listbox_items: limit(100),
        },
        input: InputSpec {
            singleton_rva: rva(0x21A0E8),
            open_rva: rva(0x657E0),
            close_rva: rva(0x658E0),
            get_command_handler_rva: rva(0x65A70),
            add_command_rva: rva(0x65AD0),
            process_rva: rva(0x65D30),
            edit_box_set_text_rva: rva(0x80F60),
            edit_box_get_text_rva: rva(0x81030),
            enabled_offset: offset(0x14E0),
            edit_box_offset: offset(8),
            command_proc_offset: offset(0xC),
            command_name_offset: offset(0x24C),
            command_name_capacity: size(33),
            command_count_offset: offset(0x14DC),
            max_text_bytes: size(128),
            max_commands: limit(144),
            max_command_name_bytes: size(32),
        },
        chat: ChatSpec {
            singleton_rva: rva(0x21A0E4),
            add_entry_rva: rva(0x64010),
            get_mode_rva: rva(0x5D7A0),
            display_mode_offset: offset(8),
            entries_offset: offset(0x132),
            entry_size: size(0xFC),
            prefix_offset: offset(4),
            prefix_capacity: size(28),
            text_offset: offset(0x20),
            text_capacity: size(144),
            text_colour_offset: offset(0xF4),
            prefix_colour_offset: offset(0xF8),
            max_entries: limit(100),
        },
        scoreboard: ScoreboardSpec {
            singleton_rva: rva(0x21A0B4),
            enabled_offset: offset(0),
        },
        death_window: DeathWindowSpec {
            singleton_rva: rva(0x21A0EC),
            add_message_rva: rva(0x66A10),
        },
        game: GameSpec {
            singleton_rva: rva(0x21A10C),
            set_cursor_mode_rva: rva(0x9BD30),
            process_input_enabling_rva: rva(0x9BC10),
            cursor_mode_offset: offset(0x55),
        },
    },
    text_labels: TextLabelSpec {
        create_rva: rva(0x11C0),
        delete_rva: rva(0x12D0),
        size: size(0x1D),
        text_offset: offset(0),
        colour_offset: offset(4),
        position_offset: offset(8),
        draw_distance_offset: offset(0x14),
        behind_walls_offset: offset(0x18),
        attached_player_offset: offset(0x19),
        attached_vehicle_offset: offset(0x1B),
        text_capacity: size(4095),
    },
    textdraws: TextdrawSpec {
        create_rva: rva(0x1AE20),
        delete_rva: rva(0x1AD00),
        text_setter_rva: rva(0xAC870),
        native_size: size(0x9D6),
        string_offset: offset(801),
        create_text_capacity: size(800),
        stored_string_capacity: size(1601),
        data_offset: offset(0x963),
        transmit: TextdrawTransmitLayout {
            size: size(0x3F),
            x: offset(0x21),
            y: offset(0x25),
        },
        data: TextdrawDataLayout {
            width: offset(0),
            height: offset(4),
            colour: offset(8),
            align_center: offset(0xD),
            box_enabled: offset(0xE),
            box_width: offset(0xF),
            box_height: offset(0x13),
            box_colour: offset(0x17),
            proportional: offset(0x1B),
            background_colour: offset(0x1C),
            shadow: offset(0x20),
            outline: offset(0x21),
            align_left: offset(0x22),
            align_right: offset(0x23),
            style: offset(0x24),
            x: offset(0x28),
            y: offset(0x2C),
            model_id: offset(0x45),
            rotation: offset(0x47),
            zoom: offset(0x53),
            model_colour1: offset(0x57),
            model_colour2: offset(0x59),
        },
    },
    handles: HandleSpec {
        rakpeer_size: size(0xDDE),
    },
    strategies: ProfileStrategies {
        game_state_codec: GameStateCodec::Identity,
        local_player_source: LocalPlayerSource::PlayerPoolGetter,
        i32_boolean: NativeBoolean::ValidatedI32,
        u8_boolean: NativeBoolean::ValidatedU8,
        force_sync_reset: ForceSyncReset::ClearLastAnyUpdate,
        list_item_text_layout: ListItemTextLayout::DxutComboBoxItem,
        textdraw_calls: TextdrawCallStrategy::NativeMethods,
    },
};

#[cfg(test)]
mod tests {
    use super::R1_SPEC;
    use crate::client::SampVersion;

    #[test]
    fn r1_spec_pins_identity_strategies_and_critical_values() {
        assert_eq!(R1_SPEC.identity.version, SampVersion::R1);
        assert_eq!(R1_SPEC.identity.entry_point, 0x31DF13);
        assert_eq!(R1_SPEC.net_game.singleton_rva.get(), 0x21A0F8);
        assert_eq!(R1_SPEC.net_game.game_state_offset.get(), 0x3BD);
        assert_eq!(R1_SPEC.textdraws.text_setter_rva.get(), 0xAC870);
        assert_eq!(R1_SPEC.textdraws.string_offset.get(), 801);
        assert_eq!(R1_SPEC.sync.onfoot.size.get(), 68);
        assert_eq!(R1_SPEC.pools.limits.players.get(), 1004);
    }
}
