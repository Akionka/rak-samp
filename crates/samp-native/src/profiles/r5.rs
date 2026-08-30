//! Immutable SA-MP 0.3.7 R5-1 profile data.
//!
//! This module must not inherit from `R3_SPEC`; shared values are repeated or
//! named as local constants so every R5 field has an explicit source.

use super::super::profile::*;
use crate::SampVersion;

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

const COMMON_LIMITS: PoolLimits = PoolLimits {
    players: limit(1004),
    vehicles: limit(2000),
    objects: limit(1000),
    text_labels: limit(2048),
    textdraws: limit(2304),
    gangzones: limit(1024),
    pickups: limit(4096),
};

pub const R5_SPEC: ProfileSpec = ProfileSpec {
    identity: ProfileIdentity {
        name: "SA-MP 0.3.7 R5-1",
        version: SampVersion::R5_1,
        entry_point: 0x0CBC90,
    },
    net_game: NetGameSpec {
        singleton_rva: rva(0x26EB94),
        rak_client_offset: Some(offset(0)),
        get_state_rva: None,
        get_player_pool_rva: rva(0x1170),
        get_vehicle_pool_rva: rva(0x1180),
        shutdown_for_restart_rva: rva(0xA540),
        host_address_offset: offset(0x30),
        hostname_offset: offset(0x131),
        port_offset: offset(0x235),
        game_state_offset: offset(0x3CD),
        server_settings_offset: offset(0x3D5),
        pools_offset: offset(0x3DE),
        pools: NetGamePoolSpec {
            player_offset: Some(offset(4)),
            vehicle_offset: Some(offset(0)),
            pickup_offset: offset(8),
            object_offset: offset(0xC),
            gangzone_offset: offset(0x14),
            text_label_offset: offset(0x18),
            textdraw_offset: offset(0x1C),
        },
        host_string_capacity: size(257),
        rak_client_disconnect_vtable_slot: 2,
    },
    pools: PoolSpec {
        limits: COMMON_LIMITS,
        player: PlayerPoolLayout {
            largest_id_offset: offset(0x2F3A),
            local_id_offset: offset(4),
            objects_offset: Some(offset(0x1F8A)),
            player_info: Some(PlayerInfoLayout {
                npc_offset: offset(8),
                readable_size: size(0x30),
            }),
        },
        vehicle: VehiclePoolLayout {
            not_empty_offset: offset(0x3074),
            game_objects_offset: offset(0x4FB4),
            does_exist_rva: rva(0x1150),
        },
        object: ObjectPoolLayout {
            not_empty_offset: offset(4),
            objects_offset: offset(0xFA4),
        },
        pickup: PickupPoolLayout {
            handles_offset: offset(4),
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
            get_local_player: rva(0x1A40),
            get_local_score: rva(0x6E8B0),
            get_local_ping: rva(0x6E8C0),
            is_connected: rva(0x10B0),
            get_remote_player: rva(0x10F0),
            is_npc: None,
            get_name: rva(0x175C0),
            get_score: rva(0x6E850),
            get_ping: rva(0x6E880),
            get_count: rva(0x139F0),
            set_local_player_name: rva(0xB8A0),
        },
        remote_rvas: RemotePlayerRvas {
            get_colour_argb: rva(0x16180),
            set_colour: rva(0x16150),
            does_exist: rva(0x1080),
            get_status: rva(0x16330),
        },
        local_rvas: LocalPlayerRvas {
            get_ped: None,
            get_colour_argb: rva(0x3F20),
            set_colour: rva(0x3ED0),
            set_special_action: rva(0x30F0),
            spawn: rva(0x3C20),
            send_unoccupied_data: rva(0x4D30),
            send_aim_data: rva(0x5210),
            send_onfoot_data: rva(0x4F00),
            send_stats: rva(0x5D00),
            send_trailer_data: rva(0x53D0),
            send_passenger_data: rva(0x5590),
            send_incar_data: rva(0x7080),
            update_weapons: rva(0x6290),
        },
        ped_rvas: PedRvas {
            get_health: rva(0xABD50),
            get_armour: rva(0xABD90),
        },
        local: LocalPlayerLayout {
            ped_offset: Some(offset(0x104)),
            active_offset: offset(0xF0),
            current_vehicle_offset: offset(0xF8),
            onfoot_offset: offset(0x94),
            passenger_offset: offset(0xD8),
            trailer_offset: offset(0x5E),
            incar_offset: offset(0),
            aim_offset: offset(0x3F),
            last_any_update_offset: offset(0x13F),
            onfoot: LocalOnFootLayout {
                position_offset: offset(6),
                speed_offset: offset(0x26),
                special_action_offset: offset(0x25),
                animation_offset: offset(0x40),
            },
            incar: LocalInCarLayout {
                position_offset: offset(0x18),
                speed_offset: offset(0x24),
            },
            game_ped_offset: offset(0x2A4),
            readable_size: Some(size(0x108)),
        },
        remote: RemotePlayerLayout {
            ped_offset: Some(offset(0x1DD)),
            special_action_offset: offset(0xC),
            onfoot_offset: offset(0xC5),
            incar_offset: offset(0x19),
            trailer_offset: offset(0x58),
            passenger_offset: offset(0xAD),
            aim_offset: offset(0x8E),
            reported_armour_offset: offset(0x1AC),
            reported_health_offset: offset(0x1B0),
            animation_offset: offset(0x1B4),
            state_size: size(0x1B8),
        },
        local_player_name_capacity: size(255),
        animation: AnimationTableSpec {
            rva: rva(0x1039E8),
            entry_count: limit(1812),
            entry_size: size(36),
        },
    },
    sync: SyncSpec {
        send_rates: SyncSendRateRvas {
            onfoot: rva(0xFE0A8),
            incar: rva(0xFE0AC),
            aim: rva(0xFE0B0),
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
            singleton_rva: rva(0x26EB50),
            show_rva: rva(0x6FFB0),
            close_rva: rva(0x70630),
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
            singleton_rva: rva(0x26EB84),
            open_rva: rva(0x69480),
            close_rva: rva(0x69580),
            get_command_handler_rva: rva(0x69710),
            add_command_rva: rva(0x69770),
            process_rva: rva(0x699D0),
            edit_box_set_text_rva: Some(rva(0x85580)),
            edit_box_get_text_rva: Some(rva(0x85650)),
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
            singleton_rva: rva(0x26EB80),
            add_entry_rva: rva(0x67BE0),
            get_mode_rva: rva(0x612B0),
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
            singleton_rva: rva(0x26EB4C),
            enabled_offset: offset(0),
            readable_size: size(0x44),
        },
        death_window: DeathWindowSpec {
            singleton_rva: Some(rva(0x26EB88)),
            add_message_rva: Some(rva(0x6A6B0)),
        },
        game: GameSpec {
            singleton_rva: rva(0x26EBAC),
            set_cursor_mode_rva: rva(0xA06F0),
            process_input_enabling_rva: rva(0xA05D0),
            cursor_mode_offset: offset(0x61),
        },
    },
    text_labels: TextLabelSpec {
        create_rva: rva(0x11D0),
        delete_rva: rva(0x12E0),
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
        create_rva: rva(0x1E910),
        delete_rva: rva(0x1E7F0),
        text_setter_rva: rva(0xB2F60),
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
        game_state_codec: GameStateCodec::Classic,
        local_player_source: LocalPlayerSource::PlayerPoolGetter,
        pool_getter_abi: PoolGetterAbi::Classic,
        booleans: NativeBooleanPolicies {
            pool_occupancy: NativeBoolean::ValidatedI32,
            player_is_npc: NativeBoolean::ValidatedI32,
            dialog_active: NativeBoolean::ValidatedI32,
            dialog_server_side: NativeBoolean::ValidatedI32,
            input_enabled: NativeBoolean::ValidatedI32,
            label_behind_walls: NativeBoolean::ValidatedU8,
            textdraw_flags: NativeBoolean::ValidatedU8,
            vehicle_sync_flags: NativeBoolean::ValidatedU8,
        },
        force_sync_reset: ForceSyncReset::ClearLastAnyUpdate,
        list_item_text_layout: ListItemTextLayout::DirectPointer,
        textdraw_calls: TextdrawCallStrategy::NativeMethods,
    },
};

#[cfg(test)]
mod tests {
    use super::R5_SPEC;
    use crate::SampVersion;

    #[test]
    fn r5_spec_materializes_its_distinct_values() {
        assert_eq!(R5_SPEC.identity.version, SampVersion::R5_1);
        assert_eq!(R5_SPEC.identity.entry_point, 0x0CBC90);
        assert_eq!(R5_SPEC.net_game.singleton_rva.get(), 0x26EB94);
        assert_eq!(
            R5_SPEC.net_game.rak_client_offset.map(|value| value.get()),
            Some(0)
        );
        assert_eq!(R5_SPEC.pools.player.largest_id_offset.get(), 0x2F3A);
        assert_eq!(
            R5_SPEC.players.local.ped_offset.map(|value| value.get()),
            Some(0x104)
        );
        assert_eq!(
            R5_SPEC.players.remote.ped_offset.map(|value| value.get()),
            Some(0x1DD)
        );
        assert_eq!(R5_SPEC.textdraws.text_setter_rva.get(), 0xB2F60);
    }
}
