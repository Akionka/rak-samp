use crate::events::core::{ProtocolEventError, handle_protocol};
use crate::events::{Event, ProtocolAction};
use crate::{HostApi, SampClientSdkEventV1, SampClientSdkHookAction};

macro_rules! protocol_packet_helper {
    ($name:ident, $descriptor:path, $value:ty, $event_name:literal) => {
        #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an incoming raw Packet callback.")]
        ///
        /// # Safety
        ///
        /// See [`crate::events::handle`].
        #[allow(dead_code)]
        pub(crate) unsafe fn $name(
            api: HostApi,
            raw: *mut SampClientSdkEventV1,
            handler: impl FnOnce($value) -> ProtocolAction<$value>,
        ) -> Result<SampClientSdkHookAction, ProtocolEventError> {
            let mut event = unsafe { Event::from_callback(api, raw) }.map_err(|error| {
                ProtocolEventError::DecodeSource(error)
            })?;
            handle_protocol::<$descriptor>(&mut event, handler)
        }
    };
}

protocol_packet_helper!(
    on_player_sync,
    samp_protocol::packet::r1::RemotePlayerSyncPacket,
    samp_protocol::packet::r1::RemotePlayerSync,
    "onPlayerSync"
);
protocol_packet_helper!(
    on_vehicle_sync,
    samp_protocol::packet::r1::RemoteVehicleSyncPacket,
    samp_protocol::packet::r1::RemoteVehicleSync,
    "onVehicleSync"
);
protocol_packet_helper!(
    on_markers_sync,
    samp_protocol::packet::r1::MarkersSyncPacket,
    samp_protocol::packet::r1::MarkersSync,
    "onMarkersSync"
);
