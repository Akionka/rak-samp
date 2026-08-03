use super::*;

/// The `onAuthenticationRequest` descriptor.
pub const AUTHENTICATION_REQUEST: Packet<Vec<u8>> =
    Packet::new(AUTHENTICATION_ID, decode_string8, encode_string8);
/// The `onConnectionRequestAccepted` descriptor.
pub const CONNECTION_ACCEPTED: Packet<ConnectionAccepted> = Packet::new(
    CONNECTION_REQUEST_ACCEPTED_ID,
    decode_connection_accepted,
    encode_connection_accepted,
);
/// The `onConnectionLost` descriptor.
pub const CONNECTION_LOST: Packet<()> = Packet::new(CONNECTION_LOST_ID, decode_empty, encode_empty);
/// The `onConnectionBanned` descriptor.
pub const CONNECTION_BANNED: Packet<()> =
    Packet::new(CONNECTION_BANNED_ID, decode_empty, encode_empty);
/// The `onConnectionAttemptFailed` descriptor.
pub const CONNECTION_ATTEMPT_FAILED: Packet<()> =
    Packet::new(CONNECTION_ATTEMPT_FAILED_ID, decode_empty, encode_empty);
/// The `onConnectionNoFreeSlot` descriptor.
pub const CONNECTION_NO_FREE_SLOT: Packet<()> =
    Packet::new(NO_FREE_INCOMING_CONNECTIONS_ID, decode_empty, encode_empty);
/// The `onConnectionPasswordInvalid` descriptor.
pub const CONNECTION_PASSWORD_INVALID: Packet<()> =
    Packet::new(INVALID_PASSWORD_ID, decode_empty, encode_empty);
/// The `onConnectionClosed` descriptor.
pub const CONNECTION_CLOSED: Packet<()> =
    Packet::new(DISCONNECTION_NOTIFICATION_ID, decode_empty, encode_empty);
/// The compressed R1 remote-player sync descriptor.
pub const PLAYER_SYNC: Packet<RemotePlayerSync> = Packet::new_bits(
    PLAYER_SYNC_ID,
    decode_remote_player_sync,
    encode_remote_player_sync,
);
/// The compressed R1 remote-vehicle sync descriptor.
pub const VEHICLE_SYNC: Packet<RemoteVehicleSync> = Packet::new_bits(
    VEHICLE_SYNC_ID,
    decode_remote_vehicle_sync,
    encode_remote_vehicle_sync,
);
/// The variable-length R1 marker-sync descriptor.
pub const MARKERS_SYNC: Packet<MarkersSync> =
    Packet::new_bits(MARKERS_SYNC_ID, decode_markers_sync, encode_markers_sync);
pub const AIM_SYNC: Packet<RemoteSync<AimSync>> =
    Packet::new(AIM_SYNC_ID, decode_remote_aim_sync, encode_remote_aim_sync);
pub const BULLET_SYNC: Packet<RemoteSync<BulletSync>> = Packet::new(
    BULLET_SYNC_ID,
    decode_remote_bullet_sync,
    encode_remote_bullet_sync,
);
pub const UNOCCUPIED_SYNC: Packet<RemoteSync<UnoccupiedSync>> = Packet::new(
    UNOCCUPIED_SYNC_ID,
    decode_remote_unoccupied_sync,
    encode_remote_unoccupied_sync,
);
pub const TRAILER_SYNC: Packet<RemoteSync<TrailerSync>> = Packet::new(
    TRAILER_SYNC_ID,
    decode_remote_trailer_sync,
    encode_remote_trailer_sync,
);
pub const PASSENGER_SYNC: Packet<RemoteSync<PassengerSync>> = Packet::new(
    PASSENGER_SYNC_ID,
    decode_remote_passenger_sync,
    encode_remote_passenger_sync,
);

macro_rules! packet_helper {
    ($name:ident, $value:ty, $packet:ident, $event_name:literal) => {
        #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an incoming raw packet callback.")]
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

packet_helper!(on_aim_sync, RemoteSync<AimSync>, AIM_SYNC, "onAimSync");
packet_helper!(
    on_authentication_request,
    Vec<u8>,
    AUTHENTICATION_REQUEST,
    "onAuthenticationRequest"
);
packet_helper!(
    on_connection_accepted,
    ConnectionAccepted,
    CONNECTION_ACCEPTED,
    "onConnectionRequestAccepted"
);
packet_helper!(on_connection_lost, (), CONNECTION_LOST, "onConnectionLost");
packet_helper!(
    on_connection_banned,
    (),
    CONNECTION_BANNED,
    "onConnectionBanned"
);
packet_helper!(
    on_connection_attempt_failed,
    (),
    CONNECTION_ATTEMPT_FAILED,
    "onConnectionAttemptFailed"
);
packet_helper!(
    on_connection_no_free_slot,
    (),
    CONNECTION_NO_FREE_SLOT,
    "onConnectionNoFreeSlot"
);
packet_helper!(
    on_connection_password_invalid,
    (),
    CONNECTION_PASSWORD_INVALID,
    "onConnectionPasswordInvalid"
);
packet_helper!(
    on_connection_closed,
    (),
    CONNECTION_CLOSED,
    "onConnectionClosed"
);
packet_helper!(
    on_player_sync,
    RemotePlayerSync,
    PLAYER_SYNC,
    "onPlayerSync"
);
packet_helper!(
    on_vehicle_sync,
    RemoteVehicleSync,
    VEHICLE_SYNC,
    "onVehicleSync"
);
packet_helper!(on_markers_sync, MarkersSync, MARKERS_SYNC, "onMarkersSync");
packet_helper!(
    on_bullet_sync,
    RemoteSync<BulletSync>,
    BULLET_SYNC,
    "onBulletSync"
);
packet_helper!(
    on_unoccupied_sync,
    RemoteSync<UnoccupiedSync>,
    UNOCCUPIED_SYNC,
    "onUnoccupiedSync"
);
packet_helper!(
    on_trailer_sync,
    RemoteSync<TrailerSync>,
    TRAILER_SYNC,
    "onTrailerSync"
);
packet_helper!(
    on_passenger_sync,
    RemoteSync<PassengerSync>,
    PASSENGER_SYNC,
    "onPassengerSync"
);
