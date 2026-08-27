//! Common byte-aligned outgoing SA-MP RPC codecs.

use crate::types::Vector3;
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, OutgoingRpc, WireCodec,
    WireReadExt, WireWriteExt,
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

/// MoonLoader's `onSendClientJoin` payload (RPC 25).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientJoin {
    pub version: i32,
    pub mod_id: u8,
    pub nickname: Vec<u8>,
    pub challenge_response: i32,
    pub join_auth_key: Vec<u8>,
    pub client_version: Vec<u8>,
    pub challenge_response2: i32,
}

/// MoonLoader's `onSendNPCJoin` payload (RPC 54).
#[derive(Clone, Debug, PartialEq)]
pub struct NpcJoin {
    pub version: i32,
    pub mod_id: u8,
    pub nickname: Vec<u8>,
    pub challenge_response: i32,
}

/// MoonLoader's `onSendVehicleDamaged` payload (RPC 106).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleDamage {
    pub vehicle_id: u16,
    pub panel_damage: i32,
    pub door_damage: i32,
    pub lights: u8,
    pub tires: u8,
}

/// MoonLoader's `onSendEnterEditObject` payload (RPC 27).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnterEditObject {
    pub object_type: i32,
    pub object_id: u16,
    pub model_id: i32,
    pub position: Vector3,
}

/// MoonLoader's `onSendEditAttachedObject` payload (RPC 116).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EditAttachedObject {
    pub response: i32,
    pub index: i32,
    pub model_id: i32,
    pub bone: i32,
    pub position: Vector3,
    pub rotation: Vector3,
    pub scale: Vector3,
    pub color1: i32,
    pub color2: i32,
}

/// MoonLoader's `onSendClientCheckResponse` payload (RPC 103).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientCheckResponse {
    pub request_type: u8,
    pub result1: i32,
    pub result2: u8,
}

/// MoonLoader's `onSendDialogResponse` payload (RPC 62).
#[derive(Clone, Debug, PartialEq)]
pub struct DialogResponse {
    pub dialog_id: u16,
    pub button: u8,
    pub list_item: u16,
    pub input: Vec<u8>,
}

/// MoonLoader's `onSendClickPlayer` payload (RPC 23).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickPlayer {
    pub player_id: u16,
    pub source: u8,
}

/// MoonLoader's `onSendEnterVehicle` payload (RPC 26).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnterVehicle {
    pub vehicle_id: u16,
    pub passenger: bool,
}

/// MoonLoader's `onSendVehicleTuningNotification` payload (RPC 96).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleTuning {
    pub vehicle_id: i32,
    pub param1: i32,
    pub param2: i32,
    pub event: i32,
}

pub struct Empty;
pub struct U8;
pub struct U16;
pub struct I32;
pub struct Vector3Codec;
pub struct DeathNotificationCodec;
pub struct MoneyIncreaseCodec;
pub struct CameraTargetUpdateCodec;
pub struct ClientJoinCodec;
pub struct NpcJoinCodec;
pub struct VehicleDamageCodec;
pub struct EnterEditObjectCodec;
pub struct EditAttachedObjectCodec;
pub struct ClientCheckResponseCodec;
pub struct DialogResponseCodec;
pub struct ClickPlayerCodec;
pub struct EnterVehicleCodec;
pub struct VehicleTuningCodec;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty) => {
        pub type $name = OutgoingRpc<$id, $codec, ExactBytesPolicy>;
        pub const $constant: $name = OutgoingRpc::new();
    };
}

descriptor!(
    SendDeathNotification,
    SEND_DEATH_NOTIFICATION,
    53,
    DeathNotificationCodec
);
descriptor!(SendMapMarker, SEND_MAP_MARKER, 119, Vector3Codec);
descriptor!(SendInteriorChange, SEND_INTERIOR_CHANGE, 118, U8);
descriptor!(
    SendUpdateScoresAndPings,
    SEND_UPDATE_SCORES_AND_PINGS,
    155,
    Empty
);
descriptor!(
    SendMoneyIncrease,
    SEND_MONEY_INCREASE,
    31,
    MoneyIncreaseCodec
);
descriptor!(SendPickedUpWeapon, SEND_PICKED_UP_WEAPON, 97, U16);
descriptor!(SendPickedUpPickup, SEND_PICKED_UP_PICKUP, 131, I32);
descriptor!(
    SendCameraTargetUpdate,
    SEND_CAMERA_TARGET_UPDATE,
    168,
    CameraTargetUpdateCodec
);
descriptor!(SendClientJoin, SEND_CLIENT_JOIN, 25, ClientJoinCodec);
descriptor!(SendNpcJoin, SEND_NPC_JOIN, 54, NpcJoinCodec);
descriptor!(
    SendVehicleDamaged,
    SEND_VEHICLE_DAMAGED,
    106,
    VehicleDamageCodec
);
descriptor!(
    SendEnterEditObject,
    SEND_ENTER_EDIT_OBJECT,
    27,
    EnterEditObjectCodec
);
descriptor!(
    SendEditAttachedObject,
    SEND_EDIT_ATTACHED_OBJECT,
    116,
    EditAttachedObjectCodec
);
descriptor!(SendSpawn, SEND_SPAWN, 52, Empty);
descriptor!(SendRequestClass, SEND_REQUEST_CLASS, 128, I32);
descriptor!(SendRequestSpawn, SEND_REQUEST_SPAWN, 129, Empty);
descriptor!(
    SendServerStatisticsRequest,
    SEND_SERVER_STATISTICS_REQUEST,
    102,
    Empty
);
descriptor!(
    SendClientCheckResponse,
    SEND_CLIENT_CHECK_RESPONSE,
    103,
    ClientCheckResponseCodec
);
descriptor!(
    SendDialogResponse,
    SEND_DIALOG_RESPONSE,
    62,
    DialogResponseCodec
);
descriptor!(SendClickPlayer, SEND_CLICK_PLAYER, 23, ClickPlayerCodec);
descriptor!(SendClickTextDraw, SEND_CLICK_TEXT_DRAW, 83, U16);
descriptor!(SendMenuSelect, SEND_MENU_SELECT, 132, U8);
descriptor!(SendQuitMenu, SEND_QUIT_MENU, 140, Empty);
descriptor!(SendEnterVehicle, SEND_ENTER_VEHICLE, 26, EnterVehicleCodec);
descriptor!(SendExitVehicle, SEND_EXIT_VEHICLE, 154, U16);
descriptor!(SendVehicleDestroyed, SEND_VEHICLE_DESTROYED, 136, U16);
descriptor!(
    SendVehicleTuning,
    SEND_VEHICLE_TUNING,
    96,
    VehicleTuningCodec
);

macro_rules! byte_aligned_codec {
    ($codec:ident, $value:ty, $decode:ident, $encode:ident) => {
        impl WireCodec for $codec {
            type Value = $value;

            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                $decode(reader)
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                $encode(writer, value)
            }
        }
    };
}

macro_rules! byte_aligned_scalar_codec {
    ($codec:ident, $value:ty, $read:ident, $write:ident) => {
        impl WireCodec for $codec {
            type Value = $value;

            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                reader.$read()
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                writer.$write(*value)
            }
        }
    };
}

macro_rules! byte_aligned_vector_codec {
    ($codec:ident, $value:ty, $read:ident, $write:ident) => {
        impl WireCodec for $codec {
            type Value = $value;

            fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
                reader.$read()
            }

            fn encode<W: BitWrite>(
                writer: &mut W,
                value: &Self::Value,
            ) -> Result<(), EncodeError<W::Error>> {
                writer.$write(value)
            }
        }
    };
}

byte_aligned_codec!(Empty, (), read_empty, write_empty);
byte_aligned_scalar_codec!(U8, u8, read_u8, write_u8);
byte_aligned_scalar_codec!(U16, u16, read_u16_le, write_u16_le);
byte_aligned_scalar_codec!(I32, i32, read_i32_le, write_i32_le);
byte_aligned_vector_codec!(Vector3Codec, Vector3, read_vector3_le, write_vector3_le);
byte_aligned_codec!(
    DeathNotificationCodec,
    DeathNotification,
    read_death_notification,
    write_death_notification
);
byte_aligned_codec!(
    MoneyIncreaseCodec,
    MoneyIncrease,
    read_money_increase,
    write_money_increase
);
byte_aligned_codec!(
    CameraTargetUpdateCodec,
    CameraTargetUpdate,
    read_camera_target_update,
    write_camera_target_update
);
byte_aligned_codec!(
    ClientJoinCodec,
    ClientJoin,
    read_client_join,
    write_client_join
);
byte_aligned_codec!(NpcJoinCodec, NpcJoin, read_npc_join, write_npc_join);
byte_aligned_codec!(
    VehicleDamageCodec,
    VehicleDamage,
    read_vehicle_damage,
    write_vehicle_damage
);
byte_aligned_codec!(
    EnterEditObjectCodec,
    EnterEditObject,
    read_enter_edit_object,
    write_enter_edit_object
);
byte_aligned_codec!(
    EditAttachedObjectCodec,
    EditAttachedObject,
    read_edit_attached_object,
    write_edit_attached_object
);
byte_aligned_codec!(
    ClientCheckResponseCodec,
    ClientCheckResponse,
    read_client_check_response,
    write_client_check_response
);
byte_aligned_codec!(
    DialogResponseCodec,
    DialogResponse,
    read_dialog_response,
    write_dialog_response
);
byte_aligned_codec!(
    ClickPlayerCodec,
    ClickPlayer,
    read_click_player,
    write_click_player
);
byte_aligned_codec!(
    EnterVehicleCodec,
    EnterVehicle,
    read_enter_vehicle,
    write_enter_vehicle
);
byte_aligned_codec!(
    VehicleTuningCodec,
    VehicleTuning,
    read_vehicle_tuning,
    write_vehicle_tuning
);

fn read_empty<R: BitRead>(_reader: &mut R) -> Result<(), DecodeError<R::Error>> {
    Ok(())
}

fn write_empty<W: BitWrite>(_writer: &mut W, _value: &()) -> Result<(), EncodeError<W::Error>> {
    Ok(())
}

fn read_death_notification<R: BitRead>(
    reader: &mut R,
) -> Result<DeathNotification, DecodeError<R::Error>> {
    Ok(DeathNotification {
        reason: reader.read_u8()?,
        killer_id: reader.read_u16_le()?,
    })
}

fn write_death_notification<W: BitWrite>(
    writer: &mut W,
    value: &DeathNotification,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.reason)?;
    writer.write_u16_le(value.killer_id)
}

fn read_money_increase<R: BitRead>(reader: &mut R) -> Result<MoneyIncrease, DecodeError<R::Error>> {
    Ok(MoneyIncrease {
        amount: reader.read_i32_le()?,
        increase_type: reader.read_i32_le()?,
    })
}

fn write_money_increase<W: BitWrite>(
    writer: &mut W,
    value: &MoneyIncrease,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.amount)?;
    writer.write_i32_le(value.increase_type)
}

fn read_camera_target_update<R: BitRead>(
    reader: &mut R,
) -> Result<CameraTargetUpdate, DecodeError<R::Error>> {
    Ok(CameraTargetUpdate {
        object_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
        player_id: reader.read_u16_le()?,
        actor_id: reader.read_u16_le()?,
    })
}

fn write_camera_target_update<W: BitWrite>(
    writer: &mut W,
    value: &CameraTargetUpdate,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u16_le(value.player_id)?;
    writer.write_u16_le(value.actor_id)
}

fn read_client_join<R: BitRead>(reader: &mut R) -> Result<ClientJoin, DecodeError<R::Error>> {
    Ok(ClientJoin {
        version: reader.read_i32_le()?,
        mod_id: reader.read_u8()?,
        nickname: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        challenge_response: reader.read_i32_le()?,
        join_auth_key: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        client_version: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        challenge_response2: reader.read_i32_le()?,
    })
}

fn write_client_join<W: BitWrite>(
    writer: &mut W,
    value: &ClientJoin,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.version)?;
    writer.write_u8(value.mod_id)?;
    writer.write_len_prefixed_bytes_u8(&value.nickname, usize::from(u8::MAX))?;
    writer.write_i32_le(value.challenge_response)?;
    writer.write_len_prefixed_bytes_u8(&value.join_auth_key, usize::from(u8::MAX))?;
    writer.write_len_prefixed_bytes_u8(&value.client_version, usize::from(u8::MAX))?;
    writer.write_i32_le(value.challenge_response2)
}

fn read_npc_join<R: BitRead>(reader: &mut R) -> Result<NpcJoin, DecodeError<R::Error>> {
    Ok(NpcJoin {
        version: reader.read_i32_le()?,
        mod_id: reader.read_u8()?,
        nickname: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
        challenge_response: reader.read_i32_le()?,
    })
}

fn write_npc_join<W: BitWrite>(
    writer: &mut W,
    value: &NpcJoin,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.version)?;
    writer.write_u8(value.mod_id)?;
    writer.write_len_prefixed_bytes_u8(&value.nickname, usize::from(u8::MAX))?;
    writer.write_i32_le(value.challenge_response)
}

fn read_vehicle_damage<R: BitRead>(reader: &mut R) -> Result<VehicleDamage, DecodeError<R::Error>> {
    Ok(VehicleDamage {
        vehicle_id: reader.read_u16_le()?,
        panel_damage: reader.read_i32_le()?,
        door_damage: reader.read_i32_le()?,
        lights: reader.read_u8()?,
        tires: reader.read_u8()?,
    })
}

fn write_vehicle_damage<W: BitWrite>(
    writer: &mut W,
    value: &VehicleDamage,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_i32_le(value.panel_damage)?;
    writer.write_i32_le(value.door_damage)?;
    writer.write_u8(value.lights)?;
    writer.write_u8(value.tires)
}

fn read_enter_edit_object<R: BitRead>(
    reader: &mut R,
) -> Result<EnterEditObject, DecodeError<R::Error>> {
    Ok(EnterEditObject {
        object_type: reader.read_i32_le()?,
        object_id: reader.read_u16_le()?,
        model_id: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_enter_edit_object<W: BitWrite>(
    writer: &mut W,
    value: &EnterEditObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.object_type)?;
    writer.write_u16_le(value.object_id)?;
    writer.write_i32_le(value.model_id)?;
    writer.write_vector3_le(&value.position)
}

fn read_edit_attached_object<R: BitRead>(
    reader: &mut R,
) -> Result<EditAttachedObject, DecodeError<R::Error>> {
    Ok(EditAttachedObject {
        response: reader.read_i32_le()?,
        index: reader.read_i32_le()?,
        model_id: reader.read_i32_le()?,
        bone: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
        rotation: reader.read_vector3_le()?,
        scale: reader.read_vector3_le()?,
        color1: reader.read_i32_le()?,
        color2: reader.read_i32_le()?,
    })
}

fn write_edit_attached_object<W: BitWrite>(
    writer: &mut W,
    value: &EditAttachedObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.response)?;
    writer.write_i32_le(value.index)?;
    writer.write_i32_le(value.model_id)?;
    writer.write_i32_le(value.bone)?;
    writer.write_vector3_le(&value.position)?;
    writer.write_vector3_le(&value.rotation)?;
    writer.write_vector3_le(&value.scale)?;
    writer.write_i32_le(value.color1)?;
    writer.write_i32_le(value.color2)
}

fn read_client_check_response<R: BitRead>(
    reader: &mut R,
) -> Result<ClientCheckResponse, DecodeError<R::Error>> {
    Ok(ClientCheckResponse {
        request_type: reader.read_u8()?,
        result1: reader.read_i32_le()?,
        result2: reader.read_u8()?,
    })
}

fn write_client_check_response<W: BitWrite>(
    writer: &mut W,
    value: &ClientCheckResponse,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.request_type)?;
    writer.write_i32_le(value.result1)?;
    writer.write_u8(value.result2)
}

fn read_dialog_response<R: BitRead>(
    reader: &mut R,
) -> Result<DialogResponse, DecodeError<R::Error>> {
    Ok(DialogResponse {
        dialog_id: reader.read_u16_le()?,
        button: reader.read_u8()?,
        list_item: reader.read_u16_le()?,
        input: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_dialog_response<W: BitWrite>(
    writer: &mut W,
    value: &DialogResponse,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.dialog_id)?;
    writer.write_u8(value.button)?;
    writer.write_u16_le(value.list_item)?;
    writer.write_len_prefixed_bytes_u8(&value.input, usize::from(u8::MAX))
}

fn read_click_player<R: BitRead>(reader: &mut R) -> Result<ClickPlayer, DecodeError<R::Error>> {
    Ok(ClickPlayer {
        player_id: reader.read_u16_le()?,
        source: reader.read_u8()?,
    })
}

fn write_click_player<W: BitWrite>(
    writer: &mut W,
    value: &ClickPlayer,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.source)
}

fn read_enter_vehicle<R: BitRead>(reader: &mut R) -> Result<EnterVehicle, DecodeError<R::Error>> {
    Ok(EnterVehicle {
        vehicle_id: reader.read_u16_le()?,
        passenger: reader.read_u8()? != 0,
    })
}

fn write_enter_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &EnterVehicle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(u8::from(value.passenger))
}

fn read_vehicle_tuning<R: BitRead>(reader: &mut R) -> Result<VehicleTuning, DecodeError<R::Error>> {
    Ok(VehicleTuning {
        vehicle_id: reader.read_i32_le()?,
        param1: reader.read_i32_le()?,
        param2: reader.read_i32_le()?,
        event: reader.read_i32_le()?,
    })
}

fn write_vehicle_tuning<W: BitWrite>(
    writer: &mut W,
    value: &VehicleTuning,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.vehicle_id)?;
    writer.write_i32_le(value.param1)?;
    writer.write_i32_le(value.param2)?;
    writer.write_i32_le(value.event)
}
