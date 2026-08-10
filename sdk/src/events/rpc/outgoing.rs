//! Outgoing client-to-server RPC helpers.

pub mod chat;
pub mod connection;
pub mod damage;
pub mod object;

use self::damage::{ActorDamage, SEND_GIVE_ACTOR_DAMAGE, SEND_VEHICLE_DAMAGED, VehicleDamage};

use crate::events::core::{PayloadWriter, handle};
use crate::{
    HostApi, SampClientSdkEventV1, SampClientSdkHookAction,
    events::{Event, EventError, Rpc, RpcAction, Vector3},
};

/// MoonLoader's `onSendDialogResponse` payload (RPC 62).
#[derive(Clone, Debug, PartialEq)]
pub struct DialogResponse {
    pub dialog_id: u16,
    pub button: u8,
    pub list_item: u16,
    pub input: Vec<u8>,
}

/// MoonLoader's `onSendEnterVehicle` payload (RPC 26).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnterVehicle {
    pub vehicle_id: u16,
    pub passenger: bool,
}

/// MoonLoader's `onSendDeathNotification` payload (RPC 53).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeathNotification {
    pub reason: u8,
    pub killer_id: u16,
}

/// MoonLoader's `onSendClickPlayer` payload (RPC 23).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickPlayer {
    pub player_id: u16,
    pub source: u8,
}

/// MoonLoader's `onSendVehicleTuningNotification` payload (RPC 96).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleTuning {
    pub vehicle_id: i32,
    pub param1: i32,
    pub param2: i32,
    pub event: i32,
}

/// MoonLoader's `onSendClientCheckResponse` payload (RPC 103).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCheckResponse {
    pub request_type: u8,
    pub result1: i32,
    pub result2: u8,
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

/// The `onSendDialogResponse` descriptor.
pub const SEND_DIALOG_RESPONSE: Rpc<DialogResponse> =
    Rpc::new(62, decode_dialog_response, encode_dialog_response);
/// The `onSendEnterVehicle` descriptor.
pub const SEND_ENTER_VEHICLE: Rpc<EnterVehicle> =
    Rpc::new(26, decode_enter_vehicle, encode_enter_vehicle);
/// The `onSendExitVehicle` descriptor.
pub const SEND_EXIT_VEHICLE: Rpc<u16> = Rpc::new(154, decode_u16, encode_u16);
/// The `onSendSpawn` descriptor.
pub const SEND_SPAWN: Rpc<()> = Rpc::new(52, decode_empty, encode_empty);
/// The `onSendDeathNotification` descriptor.
pub const SEND_DEATH_NOTIFICATION: Rpc<DeathNotification> =
    Rpc::new(53, decode_death_notification, encode_death_notification);
/// The `onSendMapMarker` descriptor.
pub const SEND_MAP_MARKER: Rpc<Vector3> = Rpc::new(119, decode_vector3, encode_vector3);
/// The `onSendClickPlayer` descriptor.
pub const SEND_CLICK_PLAYER: Rpc<ClickPlayer> =
    Rpc::new(23, decode_click_player, encode_click_player);
/// The `onSendInteriorChange` descriptor.
pub const SEND_INTERIOR_CHANGE: Rpc<u8> = Rpc::new(118, decode_u8, encode_u8);
/// The `onSendRequestClass` descriptor.
pub const SEND_REQUEST_CLASS: Rpc<i32> = Rpc::new(128, decode_i32, encode_i32);
/// The `onSendRequestSpawn` descriptor.
pub const SEND_REQUEST_SPAWN: Rpc<()> = Rpc::new(129, decode_empty, encode_empty);
/// The `onSendMenuSelect` descriptor.
pub const SEND_MENU_SELECT: Rpc<u8> = Rpc::new(132, decode_u8, encode_u8);
/// The `onSendVehicleDestroyed` descriptor.
pub const SEND_VEHICLE_DESTROYED: Rpc<u16> = Rpc::new(136, decode_u16, encode_u16);
/// The `onSendClickTextDraw` descriptor.
pub const SEND_CLICK_TEXT_DRAW: Rpc<u16> = Rpc::new(83, decode_u16, encode_u16);
/// The `onSendUpdateScoresAndPings` descriptor.
pub const SEND_UPDATE_SCORES_AND_PINGS: Rpc<()> = Rpc::new(155, decode_empty, encode_empty);
/// The `onSendClientJoin` descriptor.
/// The `onSendEnterEditObject` descriptor.
/// The `onSendMoneyIncreaseNotification` descriptor.
pub const SEND_MONEY_INCREASE: Rpc<MoneyIncrease> =
    Rpc::new(31, decode_money_increase, encode_money_increase);
/// The `onSendNPCJoin` descriptor.
/// The `onSendVehicleTuningNotification` descriptor.
pub const SEND_VEHICLE_TUNING: Rpc<VehicleTuning> =
    Rpc::new(96, decode_vehicle_tuning, encode_vehicle_tuning);
/// The `onSendPickedUpWeapon` descriptor.
pub const SEND_PICKED_UP_WEAPON: Rpc<u16> = Rpc::new(97, decode_u16, encode_u16);
/// The `onSendServerStatisticsRequest` descriptor.
pub const SEND_SERVER_STATISTICS_REQUEST: Rpc<()> = Rpc::new(102, decode_empty, encode_empty);
/// The `onSendClientCheckResponse` descriptor.
pub const SEND_CLIENT_CHECK_RESPONSE: Rpc<ClientCheckResponse> = Rpc::new(
    103,
    decode_client_check_response,
    encode_client_check_response,
);
/// The `onSendEditAttachedObject` descriptor.
/// The `onSendEditObject` descriptor.
/// The `onSendPickedUpPickup` descriptor.
pub const SEND_PICKED_UP_PICKUP: Rpc<i32> = Rpc::new(131, decode_i32, encode_i32);
/// The `onSendQuitMenu` descriptor.
pub const SEND_QUIT_MENU: Rpc<()> = Rpc::new(140, decode_empty, encode_empty);
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
    on_send_dialog_response,
    DialogResponse,
    SEND_DIALOG_RESPONSE,
    "onSendDialogResponse"
);
rpc_helper!(
    on_send_enter_vehicle,
    EnterVehicle,
    SEND_ENTER_VEHICLE,
    "onSendEnterVehicle"
);
rpc_helper!(
    on_send_exit_vehicle,
    u16,
    SEND_EXIT_VEHICLE,
    "onSendExitVehicle"
);
rpc_helper!(on_send_spawn, (), SEND_SPAWN, "onSendSpawn");
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
    on_send_click_player,
    ClickPlayer,
    SEND_CLICK_PLAYER,
    "onSendClickPlayer"
);
rpc_helper!(
    on_send_interior_change,
    u8,
    SEND_INTERIOR_CHANGE,
    "onSendInteriorChange"
);
rpc_helper!(
    on_send_request_class,
    i32,
    SEND_REQUEST_CLASS,
    "onSendRequestClass"
);
rpc_helper!(
    on_send_request_spawn,
    (),
    SEND_REQUEST_SPAWN,
    "onSendRequestSpawn"
);
rpc_helper!(
    on_send_menu_select,
    u8,
    SEND_MENU_SELECT,
    "onSendMenuSelect"
);
rpc_helper!(
    on_send_vehicle_destroyed,
    u16,
    SEND_VEHICLE_DESTROYED,
    "onSendVehicleDestroyed"
);
rpc_helper!(
    on_send_click_text_draw,
    u16,
    SEND_CLICK_TEXT_DRAW,
    "onSendClickTextDraw"
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
    on_send_vehicle_tuning,
    VehicleTuning,
    SEND_VEHICLE_TUNING,
    "onSendVehicleTuningNotification"
);
rpc_helper!(
    on_send_picked_up_weapon,
    u16,
    SEND_PICKED_UP_WEAPON,
    "onSendPickedUpWeapon"
);
rpc_helper!(
    on_send_server_statistics_request,
    (),
    SEND_SERVER_STATISTICS_REQUEST,
    "onSendServerStatisticsRequest"
);
rpc_helper!(
    on_send_client_check_response,
    ClientCheckResponse,
    SEND_CLIENT_CHECK_RESPONSE,
    "onSendClientCheckResponse"
);
rpc_helper!(
    on_send_vehicle_damaged,
    VehicleDamage,
    SEND_VEHICLE_DAMAGED,
    "onSendVehicleDamaged"
);
rpc_helper!(
    on_send_picked_up_pickup,
    i32,
    SEND_PICKED_UP_PICKUP,
    "onSendPickedUpPickup"
);
rpc_helper!(on_send_quit_menu, (), SEND_QUIT_MENU, "onSendQuitMenu");
rpc_helper!(
    on_send_camera_target_update,
    CameraTargetUpdate,
    SEND_CAMERA_TARGET_UPDATE,
    "onSendCameraTargetUpdate"
);
rpc_helper!(
    on_send_give_actor_damage,
    ActorDamage,
    SEND_GIVE_ACTOR_DAMAGE,
    "onSendGiveActorDamage"
);

fn decode_dialog_response(event: &mut Event<'_>) -> Result<DialogResponse, EventError> {
    Ok(DialogResponse {
        dialog_id: event.read_u16()?,
        button: event.read_u8()?,
        list_item: event.read_u16()?,
        input: event.read_string8()?,
    })
}

fn encode_dialog_response(value: DialogResponse) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.dialog_id);
    writer.u8(value.button);
    writer.u16(value.list_item);
    writer.string8(&value.input)?;
    Ok(writer.finish())
}

fn decode_enter_vehicle(event: &mut Event<'_>) -> Result<EnterVehicle, EventError> {
    Ok(EnterVehicle {
        vehicle_id: event.read_u16()?,
        passenger: event.read_u8()? != 0,
    })
}

fn encode_enter_vehicle(value: EnterVehicle) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.vehicle_id);
    writer.u8(u8::from(value.passenger));
    Ok(writer.finish())
}

fn decode_click_player(event: &mut Event<'_>) -> Result<ClickPlayer, EventError> {
    Ok(ClickPlayer {
        player_id: event.read_u16()?,
        source: event.read_u8()?,
    })
}

fn encode_click_player(value: ClickPlayer) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u16(value.player_id);
    writer.u8(value.source);
    Ok(writer.finish())
}

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

fn decode_vehicle_tuning(event: &mut Event<'_>) -> Result<VehicleTuning, EventError> {
    Ok(VehicleTuning {
        vehicle_id: decode_i32(event)?,
        param1: decode_i32(event)?,
        param2: decode_i32(event)?,
        event: decode_i32(event)?,
    })
}

fn encode_vehicle_tuning(value: VehicleTuning) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u32(value.vehicle_id as u32);
    writer.u32(value.param1 as u32);
    writer.u32(value.param2 as u32);
    writer.u32(value.event as u32);
    Ok(writer.finish())
}

fn decode_client_check_response(event: &mut Event<'_>) -> Result<ClientCheckResponse, EventError> {
    Ok(ClientCheckResponse {
        request_type: event.read_u8()?,
        result1: decode_i32(event)?,
        result2: event.read_u8()?,
    })
}

fn encode_client_check_response(value: ClientCheckResponse) -> Result<Vec<u8>, EventError> {
    let mut writer = PayloadWriter::new();
    writer.u8(value.request_type);
    writer.u32(value.result1 as u32);
    writer.u8(value.result2);
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
