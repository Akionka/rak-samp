//! Outgoing client-to-server RPC helpers.

pub mod connection;
pub mod damage;
pub mod object;
pub mod session;
pub mod ui;
pub mod vehicle;

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, Rpc, RpcAction, Vector3},
};

/// MoonLoader's `onSendDeathNotification` payload (RPC 53).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeathNotification {
    pub reason: u8,
    pub killer_id: u16,
}

/// MoonLoader's `onSendMoneyIncreaseNotification` payload (RPC 31).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoneyIncrease {
    pub amount: i32,
    pub increase_type: i32,
}

/// MoonLoader's `onSendCameraTargetUpdate` payload (RPC 168).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraTargetUpdate {
    pub object_id: u16,
    pub vehicle_id: u16,
    pub player_id: u16,
    pub actor_id: u16,
}

/// The `onSendDeathNotification` descriptor.
pub const SEND_DEATH_NOTIFICATION: Rpc<DeathNotification> =
    Rpc::new(53, decode_death_notification, encode_death_notification);
/// The `onSendMapMarker` descriptor.
pub const SEND_MAP_MARKER: Rpc<Vector3> = Rpc::new(119, decode_vector3, encode_vector3);
/// The `onSendInteriorChange` descriptor.
pub const SEND_INTERIOR_CHANGE: Rpc<u8> = Rpc::new(118, decode_u8, encode_u8);
/// The `onSendUpdateScoresAndPings` descriptor.
pub const SEND_UPDATE_SCORES_AND_PINGS: Rpc<()> = Rpc::new(155, decode_empty, encode_empty);
/// The `onSendClientJoin` descriptor.
/// The `onSendEnterEditObject` descriptor.
/// The `onSendMoneyIncreaseNotification` descriptor.
pub const SEND_MONEY_INCREASE: Rpc<MoneyIncrease> =
    Rpc::new(31, decode_money_increase, encode_money_increase);
/// The `onSendNPCJoin` descriptor.
/// The `onSendPickedUpWeapon` descriptor.
pub const SEND_PICKED_UP_WEAPON: Rpc<u16> = Rpc::new(97, decode_u16, encode_u16);
/// The `onSendEditAttachedObject` descriptor.
/// The `onSendEditObject` descriptor.
/// The `onSendPickedUpPickup` descriptor.
pub const SEND_PICKED_UP_PICKUP: Rpc<i32> = Rpc::new(131, decode_i32, encode_i32);
/// The `onSendCameraTargetUpdate` descriptor.
pub const SEND_CAMERA_TARGET_UPDATE: Rpc<CameraTargetUpdate> = Rpc::new(
    168,
    decode_camera_target_update,
    encode_camera_target_update,
);

macro_rules! rpc_helper {
    ($name:ident, $value:ty, $rpc:ident, $event_name:literal) => {
        #[doc = concat!("Handles MoonLoader's `", $event_name, "` from an outgoing raw RPC callback.")]
        ///
        /// # Safety
        ///
        /// See [`crate::events::handle`].
        #[allow(dead_code)]
        pub(crate) unsafe fn $name(
            api: HostApi,
            raw: *mut SampClientSdkEventV1,
            handler: impl FnOnce($value) -> RpcAction<$value>,
        ) -> Result<SampClientSdkHookAction, EventError> {
            unsafe { handle(api, raw, $rpc, handler) }
        }
    };
}

rpc_helper!(
    on_send_death_notification,
    DeathNotification,
    SEND_DEATH_NOTIFICATION,
    "onSendDeathNotification"
);
rpc_helper!(
    on_send_map_marker,
    Vector3,
    SEND_MAP_MARKER,
    "onSendMapMarker"
);
rpc_helper!(
    on_send_interior_change,
    u8,
    SEND_INTERIOR_CHANGE,
    "onSendInteriorChange"
);
rpc_helper!(
    on_send_update_scores_and_pings,
    (),
    SEND_UPDATE_SCORES_AND_PINGS,
    "onSendUpdateScoresAndPings"
);
rpc_helper!(
    on_send_money_increase,
    MoneyIncrease,
    SEND_MONEY_INCREASE,
    "onSendMoneyIncreaseNotification"
);
rpc_helper!(
    on_send_picked_up_weapon,
    u16,
    SEND_PICKED_UP_WEAPON,
    "onSendPickedUpWeapon"
);
rpc_helper!(
    on_send_picked_up_pickup,
    i32,
    SEND_PICKED_UP_PICKUP,
    "onSendPickedUpPickup"
);
rpc_helper!(
    on_send_camera_target_update,
    CameraTargetUpdate,
    SEND_CAMERA_TARGET_UPDATE,
    "onSendCameraTargetUpdate"
);
fn decode_money_increase(event: &mut Event<'_>) -> Result<MoneyIncrease, EventError> {
    Ok(MoneyIncrease {
        amount: decode_i32(event)?,
        increase_type: decode_i32(event)?,
    })
}

fn encode_money_increase(value: MoneyIncrease) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.amount as u32);
    writer.u32(value.increase_type as u32);
    Ok(writer.finish())
}

fn decode_camera_target_update(event: &mut Event<'_>) -> Result<CameraTargetUpdate, EventError> {
    Ok(CameraTargetUpdate {
        object_id: event.read_u16()?,
        vehicle_id: event.read_u16()?,
        player_id: event.read_u16()?,
        actor_id: event.read_u16()?,
    })
}

fn encode_camera_target_update(value: CameraTargetUpdate) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.object_id);
    writer.u16(value.vehicle_id);
    writer.u16(value.player_id);
    writer.u16(value.actor_id);
    Ok(writer.finish())
}

fn decode_u8(event: &mut Event<'_>) -> Result<u8, EventError> {
    event.read_u8()
}

fn encode_u8(value: u8) -> Result<Vec<u8>, EventError> {
    Ok(vec![value])
}

fn decode_i32(event: &mut Event<'_>) -> Result<i32, EventError> {
    Ok(event.read_u32()? as i32)
}

fn encode_i32(value: i32) -> Result<Vec<u8>, EventError> {
    Ok(value.to_le_bytes().to_vec())
}

fn decode_u16(event: &mut Event<'_>) -> Result<u16, EventError> {
    event.read_u16()
}

fn encode_u16(value: u16) -> Result<Vec<u8>, EventError> {
    Ok(value.to_le_bytes().to_vec())
}

fn decode_empty(_event: &mut Event<'_>) -> Result<(), EventError> {
    Ok(())
}

fn encode_empty(_value: ()) -> Result<Vec<u8>, EventError> {
    Ok(Vec::new())
}

fn decode_death_notification(event: &mut Event<'_>) -> Result<DeathNotification, EventError> {
    Ok(DeathNotification {
        reason: event.read_u8()?,
        killer_id: event.read_u16()?,
    })
}

fn encode_death_notification(value: DeathNotification) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.reason);
    writer.u16(value.killer_id);
    Ok(writer.finish())
}

fn decode_vector3(event: &mut Event<'_>) -> Result<Vector3, EventError> {
    Ok(Vector3 {
        x: event.read_f32()?,
        y: event.read_f32()?,
        z: event.read_f32()?,
    })
}

fn encode_vector3(value: Vector3) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.vector3(value);
    Ok(writer.finish())
}
