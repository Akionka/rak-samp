use super::*;

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

macro_rules! packet_helper {
    ($name:ident, $value:ty, $packet:ident, $event_name:literal) => {
        #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an incoming raw packet callback.")]
        ///
        /// # Safety
        ///
        /// See [`super::super::handle`].
        #[allow(dead_code)]
        pub(crate) unsafe fn $name(
            api: HostApi,
            raw: *mut SampClientSdkEventV1,
            handler: impl FnOnce($value) -> RpcAction<$value>,
        ) -> Result<SampClientSdkHookAction, EventError> {
            unsafe { handle(api, raw, $packet, handler) }
        }
    };
}

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
