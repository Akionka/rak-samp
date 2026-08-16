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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PoolSpec {
    pub(crate) player_limit: NativeLimit,
    pub(crate) vehicle_limit: NativeLimit,
    pub(crate) object_limit: NativeLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSpec {
    pub(crate) local_player_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SyncSpec {
    pub(crate) last_any_update_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSpec {
    pub(crate) dialog_list_item_text_offset: FieldOffset,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextLabelSpec {
    pub(crate) text_capacity: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextdrawSpec {
    pub(crate) text_capacity: NativeSize,
    pub(crate) text_setter_rva: NativeRva,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HandleSpec {
    pub(crate) pool_entry_size: NativeSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProfileStrategies {
    pub(crate) game_state_codec: GameStateCodec,
    pub(crate) local_player_source: LocalPlayerSource,
    pub(crate) native_boolean: NativeBoolean,
    pub(crate) force_sync_reset: ForceSyncReset,
    pub(crate) list_item_text_layout: ListItemTextLayout,
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
    Byte,
    ValidatedI32,
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
pub(crate) struct NativeRva(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FieldOffset(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeSize(usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeLimit(usize);

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE_SPEC: ProfileSpec = ProfileSpec {
        identity: ProfileIdentity {
            name: "test",
            version: SampVersion::R1,
            entry_point: 0x31DF13,
        },
        net_game: NetGameSpec {
            singleton_rva: NativeRva(0x1000),
        },
        pools: PoolSpec {
            player_limit: NativeLimit(1_000),
            vehicle_limit: NativeLimit(2_000),
            object_limit: NativeLimit(3_000),
        },
        players: PlayerSpec {
            local_player_offset: FieldOffset(0x20),
        },
        sync: SyncSpec {
            last_any_update_offset: FieldOffset(0x24),
        },
        ui: UiSpec {
            dialog_list_item_text_offset: FieldOffset(0x28),
        },
        text_labels: TextLabelSpec {
            text_capacity: NativeSize(144),
        },
        textdraws: TextdrawSpec {
            text_capacity: NativeSize(800),
            text_setter_rva: NativeRva(0xAC870),
        },
        handles: HandleSpec {
            pool_entry_size: NativeSize(4),
        },
        strategies: ProfileStrategies {
            game_state_codec: GameStateCodec::Identity,
            local_player_source: LocalPlayerSource::PlayerPoolGetter,
            native_boolean: NativeBoolean::Byte,
            force_sync_reset: ForceSyncReset::ClearLastAnyUpdate,
            list_item_text_layout: ListItemTextLayout::DxutComboBoxItem,
        },
    };

    #[test]
    fn profile_spec_constructs_each_nested_subsystem() {
        let profile = NativeClientProfile {
            module_base: 0x400000,
            spec: &PROFILE_SPEC,
        };

        assert_eq!(profile.module_base, 0x400000);
        assert_eq!(profile.spec.identity, PROFILE_SPEC.identity);
        assert_eq!(profile.spec.net_game.singleton_rva, NativeRva(0x1000));
        assert_eq!(profile.spec.pools.player_limit, NativeLimit(1_000));
        assert_eq!(profile.spec.players.local_player_offset, FieldOffset(0x20));
        assert_eq!(profile.spec.sync.last_any_update_offset, FieldOffset(0x24));
        assert_eq!(
            profile.spec.ui.dialog_list_item_text_offset,
            FieldOffset(0x28)
        );
        assert_eq!(profile.spec.text_labels.text_capacity, NativeSize(144));
        assert_eq!(profile.spec.textdraws.text_setter_rva, NativeRva(0xAC870));
        assert_eq!(profile.spec.handles.pool_entry_size, NativeSize(4));
    }

    #[test]
    fn profile_strategies_retain_confirmed_behavioral_differences() {
        let classic = ProfileStrategies {
            game_state_codec: GameStateCodec::Classic,
            local_player_source: LocalPlayerSource::NetGameField,
            native_boolean: NativeBoolean::ValidatedI32,
            force_sync_reset: ForceSyncReset::ProfileSpecific,
            list_item_text_layout: ListItemTextLayout::DirectPointer,
        };

        assert_eq!(classic.game_state_codec, GameStateCodec::Classic);
        assert_eq!(classic.local_player_source, LocalPlayerSource::NetGameField);
        assert_eq!(classic.native_boolean, NativeBoolean::ValidatedI32);
        assert_eq!(classic.force_sync_reset, ForceSyncReset::ProfileSpecific);
        assert_eq!(
            classic.list_item_text_layout,
            ListItemTextLayout::DirectPointer
        );
    }
}
