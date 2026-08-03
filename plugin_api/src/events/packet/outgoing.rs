use super::*;

pub const SEND_RCON_COMMAND: Packet<Vec<u8>> =
    Packet::new(RCON_COMMAND_ID, decode_rcon_command, encode_rcon_command);
/// The `onSendAuthenticationResponse` descriptor.
pub const SEND_AUTHENTICATION_RESPONSE: Packet<Vec<u8>> =
    Packet::new(AUTHENTICATION_ID, decode_string8, encode_string8);
pub const SEND_STATS_UPDATE: Packet<StatsUpdate> =
    Packet::new(STATS_UPDATE_ID, decode_stats_update, encode_stats_update);
/// The `onSendWeaponsUpdate` descriptor.
pub const SEND_WEAPONS_UPDATE: Packet<WeaponsUpdate> =
    Packet::new(204, decode_weapons_update, encode_weapons_update);
pub const SEND_PLAYER_SYNC: Packet<PlayerSync> =
    Packet::new(PLAYER_SYNC_ID, decode_player_sync, encode_player_sync);
pub const SEND_VEHICLE_SYNC: Packet<VehicleSync> =
    Packet::new(VEHICLE_SYNC_ID, decode_vehicle_sync, encode_vehicle_sync);
pub const SEND_PASSENGER_SYNC: Packet<PassengerSync> = Packet::new(
    PASSENGER_SYNC_ID,
    decode_passenger_sync,
    encode_passenger_sync,
);
pub const SEND_AIM_SYNC: Packet<AimSync> =
    Packet::new(AIM_SYNC_ID, decode_aim_sync, encode_aim_sync);
pub const SEND_UNOCCUPIED_SYNC: Packet<UnoccupiedSync> = Packet::new(
    UNOCCUPIED_SYNC_ID,
    decode_unoccupied_sync,
    encode_unoccupied_sync,
);
pub const SEND_TRAILER_SYNC: Packet<TrailerSync> =
    Packet::new(TRAILER_SYNC_ID, decode_trailer_sync, encode_trailer_sync);
pub const SEND_BULLET_SYNC: Packet<BulletSync> =
    Packet::new(BULLET_SYNC_ID, decode_bullet_sync, encode_bullet_sync);
pub const SEND_SPECTATOR_SYNC: Packet<SpectatorSync> = Packet::new(
    SPECTATOR_SYNC_ID,
    decode_spectator_sync,
    encode_spectator_sync,
);

macro_rules! packet_helper {
    ($name:ident, $value:ty, $packet:ident, $event_name:literal) => {
        #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an outgoing raw packet callback.")]
        ///
        /// # Safety
        ///
        /// See [`super::super::handle`].
        pub unsafe fn $name(
            api: HostApi,
            raw: *mut RakSampEventV1,
            handler: impl FnOnce($value) -> RpcAction<$value>,
        ) -> Result<RakSampHookAction, EventError> {
            unsafe { handle(api, raw, $packet, handler) }
        }
    };
}

packet_helper!(
    on_send_rcon_command,
    Vec<u8>,
    SEND_RCON_COMMAND,
    "onSendRconCommand"
);
packet_helper!(
    on_send_authentication_response,
    Vec<u8>,
    SEND_AUTHENTICATION_RESPONSE,
    "onSendAuthenticationResponse"
);
packet_helper!(
    on_send_stats_update,
    StatsUpdate,
    SEND_STATS_UPDATE,
    "onSendStatsUpdate"
);
packet_helper!(
    on_send_weapons_update,
    WeaponsUpdate,
    SEND_WEAPONS_UPDATE,
    "onSendWeaponsUpdate"
);
packet_helper!(
    on_send_player_sync,
    PlayerSync,
    SEND_PLAYER_SYNC,
    "onSendPlayerSync"
);
packet_helper!(
    on_send_vehicle_sync,
    VehicleSync,
    SEND_VEHICLE_SYNC,
    "onSendVehicleSync"
);
packet_helper!(
    on_send_passenger_sync,
    PassengerSync,
    SEND_PASSENGER_SYNC,
    "onSendPassengerSync"
);
packet_helper!(on_send_aim_sync, AimSync, SEND_AIM_SYNC, "onSendAimSync");
packet_helper!(
    on_send_unoccupied_sync,
    UnoccupiedSync,
    SEND_UNOCCUPIED_SYNC,
    "onSendUnoccupiedSync"
);
packet_helper!(
    on_send_trailer_sync,
    TrailerSync,
    SEND_TRAILER_SYNC,
    "onSendTrailerSync"
);
packet_helper!(
    on_send_bullet_sync,
    BulletSync,
    SEND_BULLET_SYNC,
    "onSendBulletSync"
);
packet_helper!(
    on_send_spectator_sync,
    SpectatorSync,
    SEND_SPECTATOR_SYNC,
    "onSendSpectatorSync"
);
