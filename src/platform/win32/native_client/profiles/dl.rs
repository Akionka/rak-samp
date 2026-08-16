//! Immutable SA-MP 0.3.DL-R1 profile data.

use super::super::profile::*;
use super::r3::R3_SPEC;
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

const DL_LIMITS: PoolLimits = PoolLimits {
    players: limit(1004),
    vehicles: limit(2000),
    objects: limit(2100),
    text_labels: limit(2048),
    textdraws: limit(2304),
    gangzones: limit(1024),
    pickups: limit(4096),
};

pub(crate) const DL_SPEC: ProfileSpec = ProfileSpec {
    identity: ProfileIdentity {
        name: "SA-MP 0.3.DL-R1",
        version: SampVersion::Dl,
        entry_point: 0x0FDB60,
    },
    net_game: NetGameSpec {
        singleton_rva: rva(0x2ACA24),
        shutdown_for_restart_rva: rva(0xA230),
        pools: NetGamePoolSpec {
            pickup_offset: offset(0x10),
            object_offset: offset(0x14),
            gangzone_offset: offset(0x18),
            text_label_offset: offset(0x1C),
            textdraw_offset: offset(0x20),
            ..R3_SPEC.net_game.pools
        },
        ..R3_SPEC.net_game
    },
    pools: PoolSpec {
        limits: DL_LIMITS,
        player: PlayerPoolLayout {
            local_id_offset: offset(0),
            largest_id_offset: offset(0x22),
            objects_offset: Some(offset(0x26)),
            player_info: Some(PlayerInfoLayout {
                npc_offset: offset(4),
                readable_size: size(0x2C),
            }),
        },
        object: ObjectPoolLayout {
            objects_offset: offset(0x20D4),
            ..R3_SPEC.pools.object
        },
        ..R3_SPEC.pools
    },
    players: PlayerSpec {
        pool_rvas: PlayerPoolRvas {
            get_local_player: rva(0x1A80),
            get_count: rva(0x138C0),
            get_name: rva(0x170D0),
            get_local_score: rva(0x6E2E0),
            get_local_ping: rva(0x6E2F0),
            get_score: rva(0x6E290),
            get_ping: rva(0x6E2B0),
            set_local_player_name: rva(0xB490),
            ..R3_SPEC.players.pool_rvas
        },
        remote_rvas: RemotePlayerRvas {
            get_colour_argb: rva(0x15E30),
            set_colour: rva(0x15E00),
            get_status: rva(0x15FD0),
            ..R3_SPEC.players.remote_rvas
        },
        local_rvas: LocalPlayerRvas {
            get_colour_argb: rva(0x3E20),
            set_colour: rva(0x3DE0),
            set_special_action: rva(0x3110),
            spawn: rva(0x3A70),
            send_unoccupied_data: rva(0x4BD0),
            send_aim_data: rva(0x5090),
            send_onfoot_data: rva(0x4DB0),
            send_stats: rva(0x5B50),
            send_trailer_data: rva(0x5240),
            send_passenger_data: rva(0x5400),
            send_incar_data: rva(0x6E80),
            update_weapons: rva(0x60D0),
            ..R3_SPEC.players.local_rvas
        },
        ped_rvas: PedRvas {
            get_health: rva(0xAB970),
            get_armour: rva(0xAB9B0),
        },
        local: LocalPlayerLayout {
            ped_offset: Some(offset(0)),
            trailer_offset: offset(4),
            onfoot_offset: offset(0x3A),
            passenger_offset: offset(0x7E),
            incar_offset: offset(0x96),
            aim_offset: offset(0xD5),
            last_any_update_offset: offset(0x110),
            readable_size: Some(size(0x102)),
            ..R3_SPEC.players.local
        },
        remote: RemotePlayerLayout {
            ped_offset: Some(offset(4)),
            special_action_offset: offset(0x18),
            passenger_offset: offset(0x24),
            onfoot_offset: offset(0x3C),
            incar_offset: offset(0x80),
            trailer_offset: offset(0xBF),
            aim_offset: offset(0xF5),
            ..R3_SPEC.players.remote
        },
        animation: AnimationTableSpec {
            rva: rva(0x1419D0),
            ..R3_SPEC.players.animation
        },
        ..R3_SPEC.players
    },
    sync: SyncSpec {
        send_rates: SyncSendRateRvas {
            onfoot: rva(0x13C0A8),
            incar: rva(0x13C0AC),
            aim: rva(0x13C0B0),
        },
        ..R3_SPEC.sync
    },
    ui: UiSpec {
        dialog: DialogSpec {
            singleton_rva: rva(0x2AC9E0),
            close_rva: rva(0x700D0),
            ..R3_SPEC.ui.dialog
        },
        input: InputSpec {
            singleton_rva: rva(0x2ACA14),
            open_rva: rva(0x68EC0),
            close_rva: rva(0x68FC0),
            get_command_handler_rva: rva(0x69150),
            add_command_rva: rva(0x691B0),
            process_rva: rva(0x69410),
            edit_box_set_text_rva: Some(rva(0x85000)),
            edit_box_get_text_rva: Some(rva(0x850D0)),
            ..R3_SPEC.ui.input
        },
        chat: ChatSpec {
            singleton_rva: rva(0x2ACA10),
            add_entry_rva: rva(0x67650),
            get_mode_rva: rva(0x60D30),
            ..R3_SPEC.ui.chat
        },
        death_window: DeathWindowSpec {
            singleton_rva: Some(rva(0x2ACA18)),
            add_message_rva: Some(rva(0x6A0F0)),
        },
        game: GameSpec {
            singleton_rva: rva(0x2ACA3C),
            set_cursor_mode_rva: rva(0xA0530),
            process_input_enabling_rva: rva(0xA0410),
            ..R3_SPEC.ui.game
        },
        scoreboard: ScoreboardSpec {
            singleton_rva: rva(0x2AC9DC),
            ..R3_SPEC.ui.scoreboard
        },
    },
    text_labels: TextLabelSpec {
        create_rva: rva(0x11D0),
        delete_rva: rva(0x12E0),
        ..R3_SPEC.text_labels
    },
    textdraws: TextdrawSpec {
        create_rva: rva(0x1E3D0),
        delete_rva: rva(0x1E2B0),
        text_setter_rva: rva(0xB2B60),
        ..R3_SPEC.textdraws
    },
    handles: R3_SPEC.handles,
    strategies: R3_SPEC.strategies,
};

#[cfg(test)]
mod tests {
    use super::DL_SPEC;
    use crate::client::SampVersion;

    #[test]
    fn dl_spec_records_its_distinct_identity_and_object_limit() {
        assert_eq!(DL_SPEC.identity.version, SampVersion::Dl);
        assert_eq!(DL_SPEC.identity.entry_point, 0x0FDB60);
        assert_eq!(DL_SPEC.net_game.singleton_rva.get(), 0x2ACA24);
        assert_eq!(DL_SPEC.pools.limits.objects.get(), 2100);
        assert_eq!(DL_SPEC.players.local.last_any_update_offset.get(), 0x110);
        assert_eq!(DL_SPEC.textdraws.text_setter_rva.get(), 0xB2B60);
    }
}
