//! The public, owned-value facade over the host ABI.

use crate::{
    CommandReceipt, HostApi, ResolveError, SampClientSdkClientVersion, SampClientSdkHostStatus,
    SampClientSdkResult, SampGameState,
    limits::{
        MAX_SAMP_GANGZONES, MAX_SAMP_OBJECTS, MAX_SAMP_PLAYERS, MAX_SAMP_TEXT_LABELS,
        MAX_SAMP_TEXTDRAWS, MAX_SAMP_VEHICLES,
    },
};
use std::time::Duration;

mod local_player;
mod network;
mod pools;
mod sampfuncs;
mod ui;
pub use local_player::{Anim, Animations, Local, Player, Players};
pub use network::{Net, Server};
pub use pools::{Gangzones, Labels, Objects, Pickups, Textdraws, Vehicles};
pub use sampfuncs::Sampfuncs;
pub use ui::{Chat, ChatInput, Cursor, DeathWindow, Dialogs, Scoreboard};

macro_rules! bounded_id {
    ($name:ident, $maximum:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u16);

        impl $name {
            /// Returns `None` when `raw` is outside the R1 pool range.
            #[must_use]
            pub const fn new(raw: u16) -> Option<Self> {
                if raw < $maximum {
                    Some(Self(raw))
                } else {
                    None
                }
            }

            /// Returns the bounded raw SA-MP pool index.
            #[must_use]
            pub const fn get(self) -> u16 {
                self.0
            }
        }
    };
}

bounded_id!(
    PlayerId,
    MAX_SAMP_PLAYERS,
    "A checked SA-MP player-pool ID."
);
bounded_id!(
    VehicleId,
    MAX_SAMP_VEHICLES,
    "A checked SA-MP vehicle-pool ID."
);
bounded_id!(
    TextLabelId,
    MAX_SAMP_TEXT_LABELS,
    "A checked SA-MP 3D text-label ID."
);
bounded_id!(
    TextdrawId,
    MAX_SAMP_TEXTDRAWS,
    "A checked SA-MP textdraw-pool index."
);
bounded_id!(
    ObjectId,
    MAX_SAMP_OBJECTS,
    "A checked SA-MP object-pool ID."
);
bounded_id!(
    GangzoneId,
    MAX_SAMP_GANGZONES,
    "A checked SA-MP gangzone-pool ID."
);

macro_rules! gta_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u32);

        impl $name {
            /// Returns `None` for the null GTA handle.
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                if raw == 0 { None } else { Some(Self(raw)) }
            }

            /// Returns the raw non-null GTA handle.
            #[must_use]
            pub const fn get(self) -> u32 {
                self.0
            }
        }
    };
}

gta_handle!(
    ObjectHandle,
    "A typed non-null GTA SA object handle (GTAREF)."
);
gta_handle!(
    PickupHandle,
    "A typed non-null GTA SA pickup handle (GTAREF)."
);
gta_handle!(VehicleHandle, "A typed non-null GTA SA vehicle handle.");
gta_handle!(PedHandle, "A typed non-null GTA SA ped handle.");

/// Entry point for safe, copied SA-MP client operations.
#[derive(Clone, Copy)]
pub struct Samp {
    api: HostApi,
}

impl Samp {
    /// Connects to the default `samp_client_sdk.asi` host.
    pub fn connect(timeout: Duration) -> Result<Self, ResolveError> {
        crate::wait_for_default_host(timeout).map(|api| Self { api })
    }

    /// Connects to a named host module. `module_name` must be NUL-terminated.
    pub fn connect_to(module_name: &[u8], timeout: Duration) -> Result<Self, ResolveError> {
        crate::wait_for_host(module_name, timeout).map(|api| Self { api })
    }

    /// Returns the host lifecycle state without accessing client memory.
    #[must_use]
    pub fn status(self) -> SampClientSdkHostStatus {
        self.api.status()
    }

    /// Returns lifecycle and recognized-build predicates without reading
    /// client memory. This groups SF.lua's three historical probe helpers
    /// under one explicit host-status view.
    #[must_use]
    pub const fn probe(self) -> Probe {
        Probe { api: self.api }
    }

    /// Returns the recognized SA-MP client version identity.
    pub fn version(self) -> Result<SampClientSdkClientVersion, SampClientSdkResult> {
        self.api.samp_version()
    }

    /// Returns the cached native game-state scalar for a verified client profile.
    pub fn game_state(self) -> Result<i32, SampClientSdkResult> {
        self.api.samp_game_state()
    }

    /// Queues one validated R1 CNetGame-state write on the game thread.
    pub fn set_game_state(
        self,
        state: SampGameState,
    ) -> Result<CommandReceipt<()>, SampClientSdkResult> {
        self.api.submit_samp_game_state(state)
    }

    #[must_use]
    pub fn net(self) -> Net {
        Net::from_api(self.api)
    }

    /// Returns the optional SAMPFUNCS interop view.
    #[must_use]
    pub fn sampfuncs(self) -> Sampfuncs {
        Sampfuncs::from_api(self.api)
    }

    #[must_use]
    pub fn server(self) -> Server {
        Server::from_api(self.api)
    }

    #[must_use]
    pub fn local(self) -> Local {
        Local::from_api(self.api)
    }

    #[must_use]
    pub fn players(self) -> Players {
        Players::from_api(self.api)
    }

    #[must_use]
    pub fn textdraws(self) -> Textdraws {
        Textdraws::from_api(self.api)
    }

    #[must_use]
    pub fn labels(self) -> Labels {
        Labels::from_api(self.api)
    }

    #[must_use]
    pub fn objects(self) -> Objects {
        Objects::from_api(self.api)
    }

    #[must_use]
    pub fn pickups(self) -> Pickups {
        Pickups::from_api(self.api)
    }

    #[must_use]
    pub fn vehicles(self) -> Vehicles {
        Vehicles::from_api(self.api)
    }

    #[must_use]
    pub fn gangzones(self) -> Gangzones {
        Gangzones::from_api(self.api)
    }

    #[must_use]
    pub fn dialogs(self) -> Dialogs {
        Dialogs::from_api(self.api)
    }

    #[must_use]
    pub fn chat(self) -> Chat {
        Chat::from_api(self.api)
    }

    #[must_use]
    pub fn chat_input(self) -> ChatInput {
        ChatInput::from_api(self.api)
    }

    #[must_use]
    pub fn cursor(self) -> Cursor {
        Cursor::from_api(self.api)
    }

    #[must_use]
    pub fn scoreboard(self) -> Scoreboard {
        Scoreboard::from_api(self.api)
    }

    #[must_use]
    pub fn anim(self) -> Anim {
        Anim::from_api(self.api)
    }

    pub(crate) const fn api(self) -> HostApi {
        self.api
    }

    #[cfg(test)]
    pub(crate) const fn from_api(api: HostApi) -> Self {
        Self { api }
    }
}

/// Safe host and recognized-build probes.
#[derive(Clone, Copy)]
pub struct Probe {
    api: HostApi,
}

impl Probe {
    /// Returns whether the host has attached to a recognized `samp.dll`.
    #[must_use]
    pub fn is_samp_loaded(self) -> bool {
        self.api.is_samp_loaded()
    }

    /// Returns whether the SDK recognizes the loaded SA-MP build.
    #[must_use]
    pub fn is_sampfuncs_lua_loaded(self) -> bool {
        self.api.samp_version().is_ok()
    }

    /// Returns whether `SAMPFUNCS.asi` itself is loaded in this process.
    ///
    /// This is distinct from [`Self::is_sampfuncs_lua_loaded`], the historical
    /// compatibility probe for the recognized SA-MP client build.
    #[must_use]
    pub fn is_sampfuncs_loaded(self) -> bool {
        self.api.sampfuncs_loaded()
    }

    /// Returns whether the recognized client and its RakClient hooks are ready.
    #[must_use]
    pub fn is_samp_available(self) -> bool {
        self.api.is_samp_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_ids_reject_pool_bounds() {
        assert_eq!(
            PlayerId::new(MAX_SAMP_PLAYERS - 1).map(PlayerId::get),
            Some(1003)
        );
        assert_eq!(PlayerId::new(MAX_SAMP_PLAYERS), None);
        assert_eq!(VehicleId::new(MAX_SAMP_VEHICLES), None);
        assert_eq!(TextLabelId::new(MAX_SAMP_TEXT_LABELS), None);
        assert_eq!(TextdrawId::new(MAX_SAMP_TEXTDRAWS), None);
        assert_eq!(ObjectId::new(MAX_SAMP_OBJECTS), None);
        assert_eq!(GangzoneId::new(MAX_SAMP_GANGZONES), None);
    }

    #[test]
    fn gta_handles_reject_the_null_value() {
        assert_eq!(ObjectHandle::new(0), None);
        assert_eq!(PickupHandle::new(0), None);
        assert_eq!(VehicleHandle::new(0), None);
        assert_eq!(PedHandle::new(0), None);
        assert_eq!(ObjectHandle::new(42).map(ObjectHandle::get), Some(42));
        assert_eq!(PickupHandle::new(42).map(PickupHandle::get), Some(42));
        assert_eq!(VehicleHandle::new(42).map(VehicleHandle::get), Some(42));
        assert_eq!(PedHandle::new(42).map(PedHandle::get), Some(42));
    }

    #[test]
    fn handle_lookups_route_through_the_mock_abi_and_round_trip() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let object_id = ObjectId::new(7).unwrap();
        let object_handle = samp.objects().handle(object_id).unwrap().unwrap();
        assert_eq!(object_handle.get(), 0x1007);
        assert_eq!(object_handle.to_id(samp).unwrap(), Some(object_id));

        let pickup_handle = samp.pickups().handle(7).unwrap().unwrap();
        assert_eq!(pickup_handle.get(), 0x2007);
        assert_eq!(pickup_handle.to_id(samp).unwrap(), Some(7));

        let vehicle_id = VehicleId::new(7).unwrap();
        let vehicle_handle = samp.vehicles().handle(vehicle_id).unwrap().unwrap();
        assert_eq!(vehicle_handle.get(), 0x3007);
        assert_eq!(vehicle_handle.to_id(samp).unwrap(), Some(vehicle_id));

        let player_id = PlayerId::new(7).unwrap();
        let ped_handle = samp
            .players()
            .player(player_id)
            .ped_handle()
            .unwrap()
            .unwrap();
        assert_eq!(ped_handle.get(), 0x4007);
        assert_eq!(ped_handle.to_id(samp).unwrap(), Some(player_id));
    }

    #[test]
    fn facade_reads_route_to_the_mock_abi() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        assert!(samp.probe().is_samp_loaded());
        assert!(samp.probe().is_sampfuncs_lua_loaded());
        assert!(samp.probe().is_sampfuncs_loaded());
        assert!(samp.probe().is_samp_available());
        assert_eq!(samp.version(), Ok(SampClientSdkClientVersion::R1));
        assert_eq!(samp.game_state(), Ok(14));
        assert!(samp.sampfuncs().is_loaded());
        assert_eq!(samp.sampfuncs().log_console(b"facade test"), Ok(()));
        assert_eq!(samp.server().info().map(|info| info.port), Ok(7777));
        assert_eq!(samp.local().player().map(|player| player.id()), Ok(42));
        assert_eq!(samp.players().count(true), Ok(3));
        assert_eq!(
            samp.players().player(PlayerId::new(7).unwrap()).nickname(),
            Ok(Some(b"remote".to_vec()))
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(7).unwrap())
                .streamed_out_position(),
            Ok(Some(crate::Vector3 {
                x: 100.0,
                y: -200.0,
                z: 15.0,
            }))
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(8).unwrap())
                .streamed_out_position(),
            Ok(None)
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(7).unwrap())
                .onfoot_sync()
                .map(|sync| sync.map(|sync| (sync.position, sync.animation))),
            Ok(Some((
                crate::Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                0x1234_5678,
            )))
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(8).unwrap())
                .onfoot_sync(),
            Ok(None)
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(7).unwrap())
                .vehicle_sync()
                .map(|sync| sync.map(|sync| (sync.vehicle_id, sync.siren, sync.vehicle_health))),
            Ok(Some((411, true, 900.0)))
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(8).unwrap())
                .vehicle_sync(),
            Ok(None)
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(7).unwrap())
                .passenger_sync()
                .map(|sync| sync.map(|sync| (sync.vehicle_id, sync.seat_id, sync.position))),
            Ok(Some((
                411,
                2,
                crate::Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
            )))
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(8).unwrap())
                .passenger_sync(),
            Ok(None)
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(7).unwrap())
                .trailer_sync()
                .map(|sync| sync.map(|sync| (sync.trailer_id, sync.turn_speed))),
            Ok(Some((
                123,
                crate::Vector3 {
                    x: 7.0,
                    y: 8.0,
                    z: 9.0
                }
            )))
        );
        assert_eq!(
            samp.players()
                .player(PlayerId::new(8).unwrap())
                .trailer_sync(),
            Ok(None)
        );
        assert_eq!(
            samp.textdraws().exists(TextdrawId::new(7).unwrap()),
            Ok(true)
        );
        assert_eq!(
            samp.textdraws()
                .get(TextdrawId::new(7).unwrap())
                .map(|value| value.map(|value| (value.letter_style(), value.position()))),
            Ok(Some(((1.0, 2.0, 0xFF11_2233), (3.0, 4.0))))
        );
        assert_eq!(samp.labels().exists(TextLabelId::new(7).unwrap()), Ok(true));
        assert_eq!(
            samp.labels()
                .delete(TextLabelId::new(7).unwrap())
                .map(|receipt| receipt.id()),
            Ok(36)
        );
        assert_eq!(
            samp.labels()
                .create_at(
                    TextLabelId::new(7).unwrap(),
                    b"fixture",
                    0xFF11_2233,
                    crate::Vector3 {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0
                    },
                    25.0,
                    true,
                    Some(PlayerId::new(8).unwrap()),
                    None,
                )
                .map(|receipt| receipt.id()),
            Ok(39)
        );
        let mut created = samp
            .labels()
            .create(
                b"fixture",
                0xFF11_2233,
                crate::Vector3 {
                    x: 1.0,
                    y: 2.0,
                    z: 3.0,
                },
                25.0,
                true,
                Some(PlayerId::new(8).unwrap()),
                None,
            )
            .unwrap();
        assert_eq!(created.id(), 42);
        assert_eq!(created.try_take(), Ok(Some(TextLabelId::new(7).unwrap())));
        let mut updated = samp
            .labels()
            .set_text(TextLabelId::new(7).unwrap(), b"updated")
            .unwrap();
        assert_eq!(updated.id(), 43);
        assert_eq!(updated.try_take(), Ok(Some(())));
        assert!(matches!(
            samp.labels().set_text(TextLabelId::new(7).unwrap(), b""),
            Err(SampClientSdkResult::InvalidArgument)
        ));
        assert_eq!(samp.dialogs().list_item_count(), Ok(3));
        assert_eq!(
            samp.dialogs().last_response(),
            Ok(Some(crate::LocalDialogResponse {
                dialog_id: 7,
                button: 1,
                list_item: 2,
                input: b"fixture".to_vec(),
            }))
        );
        assert_eq!(
            samp.chat().entry(7).map(|entry| (entry.text, entry.prefix)),
            Ok((b"fixture".to_vec(), b"prefix".to_vec()))
        );
        assert_eq!(samp.objects().exists(ObjectId::new(7).unwrap()), Ok(true));
        assert_eq!(samp.vehicles().exists(VehicleId::new(7).unwrap()), Ok(true));
        assert_eq!(samp.chat_input().is_active(), Ok(false));
        assert_eq!(
            samp.dialogs().active().map(|dialog| dialog.map(|dialog| (
                dialog.id(),
                dialog.style(),
                dialog.caption().to_vec(),
                dialog.is_client_side(),
                dialog.text().to_vec(),
                dialog.editbox_text().map(<[u8]>::to_vec),
                dialog.items().to_vec()
            ))),
            Ok(Some((
                7,
                crate::LocalDialogStyle::Input,
                b"fixture".to_vec(),
                true,
                b"fixture".to_vec(),
                Some(b"fixture".to_vec()),
                vec![b"fixture".to_vec(); 3]
            )))
        );
        assert_eq!(
            samp.anim().get(0).map(|animation| animation.name),
            Ok(b"AIRPORT".to_vec())
        );
        assert_eq!(samp.anim().find(b"AIRPORT", b"THRW_BARL_THRW"), Ok(Some(0)));
        assert_eq!(samp.net().rpc_name(61), Some("ShowDialog"));
        assert_eq!(samp.net().packet_name(207), Some("PLAYER_SYNC"));
        assert_eq!(
            samp.net()
                .encode_string(b"ok")
                .map(|value| value.len_bits()),
            Ok(32)
        );
        let mut stream = samp_protocol::BitStream::from_bits([0b1010_0000], 3).unwrap();
        assert_eq!(
            samp.net().decode_string(&mut stream),
            Ok(b"fixture".to_vec())
        );
        let _ = samp.pickups();
    }

    #[test]
    fn local_protocol_actions_delegate_to_the_receipt_bearing_network_path() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let vehicle = VehicleId::new(7).unwrap();
        for mut receipt in [
            samp.local().request_class(3).unwrap(),
            samp.local().send_interior_change(1).unwrap(),
            samp.local().send_spawn().unwrap(),
            samp.local().send_enter_vehicle(vehicle, false).unwrap(),
            samp.local().send_exit_vehicle(vehicle).unwrap(),
        ] {
            assert_eq!(receipt.id(), 4);
            assert_eq!(receipt.try_take(), Ok(Some(())));
        }
    }

    #[test]
    fn game_state_change_returns_an_owned_completion_receipt() {
        let samp = Samp::from_api(crate::events::test_support::test_api());
        let mut receipt = samp.set_game_state(SampGameState::Connected).unwrap();
        assert_eq!(receipt.id(), 10);
        assert_eq!(receipt.try_take(), Ok(Some(())));
    }

    #[test]
    fn local_player_mutations_and_send_rate_return_owned_completion_receipts() {
        let samp = Samp::from_api(crate::events::test_support::test_api());

        let mut spawn = samp.local().spawn().unwrap();
        assert_eq!(spawn.id(), 11);
        assert_eq!(spawn.try_take(), Ok(Some(())));

        let mut special_action = samp
            .local()
            .set_special_action(crate::SpecialAction::HandsUp)
            .unwrap();
        assert_eq!(special_action.id(), 12);
        assert_eq!(special_action.try_take(), Ok(Some(())));

        let mut send_rate = samp
            .net()
            .set_send_rate(crate::SendRateKind::Aim, 25)
            .unwrap();
        assert_eq!(send_rate.id(), 13);
        assert_eq!(send_rate.try_take(), Ok(Some(())));

        let mut colour = samp
            .players()
            .player(PlayerId::new(7).unwrap())
            .set_colour(0xFF00_00FF)
            .unwrap();
        assert_eq!(colour.id(), 21);
        assert_eq!(colour.try_take(), Ok(Some(())));

        let mut nickname = samp.local().set_nickname(b"fixture").unwrap();
        assert_eq!(nickname.id(), 22);
        assert_eq!(nickname.try_take(), Ok(Some(())));

        let mut unoccupied = samp
            .local()
            .force_unoccupied_sync(VehicleId::new(7).unwrap(), 1)
            .unwrap();
        assert_eq!(unoccupied.id(), 23);
        assert_eq!(unoccupied.try_take(), Ok(Some(())));
        let mut aim = samp.local().force_aim_sync().unwrap();
        assert_eq!(aim.id(), 23);
        assert_eq!(aim.try_take(), Ok(Some(())));
        let mut onfoot = samp.local().force_onfoot_sync().unwrap();
        assert_eq!(onfoot.id(), 24);
        assert_eq!(onfoot.try_take(), Ok(Some(())));
        let mut stats = samp.local().force_stats_sync().unwrap();
        assert_eq!(stats.id(), 25);
        assert_eq!(stats.try_take(), Ok(Some(())));
        let mut trailer = samp
            .local()
            .force_trailer_sync(VehicleId::new(7).unwrap())
            .unwrap();
        assert_eq!(trailer.id(), 26);
        assert_eq!(trailer.try_take(), Ok(Some(())));
        let mut vehicle = samp
            .local()
            .force_vehicle_sync(VehicleId::new(7).unwrap())
            .unwrap();
        assert_eq!(vehicle.id(), 27);
        assert_eq!(vehicle.try_take(), Ok(Some(())));
        let mut passenger = samp
            .local()
            .force_passenger_sync(VehicleId::new(7).unwrap(), 1)
            .unwrap();
        assert_eq!(passenger.id(), 28);
        assert_eq!(passenger.try_take(), Ok(Some(())));
        let mut weapons = samp.local().force_weapons_sync().unwrap();
        assert_eq!(weapons.id(), 29);
        assert_eq!(weapons.try_take(), Ok(Some(())));
    }
}
