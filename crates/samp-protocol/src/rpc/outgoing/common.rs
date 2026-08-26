//! Common byte-aligned outgoing SA-MP RPC codecs.

use crate::rpc::incoming::Vector3;
use crate::{BitRead, BitWrite, DecodeError, EncodeError, Rpc, TrailingPolicy, WireCodec};

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
        pub type $name = Rpc<$id, $codec>;
        pub const $constant: $name = Rpc::new();
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

            const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

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

byte_aligned_codec!(Empty, (), read_empty, write_empty);
byte_aligned_codec!(U8, u8, read_u8, write_u8);
byte_aligned_codec!(U16, u16, read_u16, write_u16);
byte_aligned_codec!(I32, i32, read_i32, write_i32);
byte_aligned_codec!(Vector3Codec, Vector3, read_vector3, write_vector3);
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

fn read_u8<R: BitRead>(reader: &mut R) -> Result<u8, DecodeError<R::Error>> {
    Ok(read_fixed::<R, 1>(reader)?[0])
}

fn write_u8<W: BitWrite>(writer: &mut W, value: &u8) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &[*value])
}

fn read_u16<R: BitRead>(reader: &mut R) -> Result<u16, DecodeError<R::Error>> {
    Ok(u16::from_le_bytes(read_fixed::<R, 2>(reader)?))
}

fn write_u16<W: BitWrite>(writer: &mut W, value: &u16) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_i32<R: BitRead>(reader: &mut R) -> Result<i32, DecodeError<R::Error>> {
    Ok(i32::from_le_bytes(read_fixed::<R, 4>(reader)?))
}

fn write_i32<W: BitWrite>(writer: &mut W, value: &i32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_le_bytes())
}

fn read_f32<R: BitRead>(reader: &mut R) -> Result<f32, DecodeError<R::Error>> {
    Ok(f32::from_bits(u32::from_le_bytes(read_fixed::<R, 4>(
        reader,
    )?)))
}

fn write_f32<W: BitWrite>(writer: &mut W, value: f32) -> Result<(), EncodeError<W::Error>> {
    write_bytes(writer, &value.to_bits().to_le_bytes())
}

fn read_vector3<R: BitRead>(reader: &mut R) -> Result<Vector3, DecodeError<R::Error>> {
    Ok(Vector3 {
        x: read_f32(reader)?,
        y: read_f32(reader)?,
        z: read_f32(reader)?,
    })
}

fn write_vector3<W: BitWrite>(
    writer: &mut W,
    value: &Vector3,
) -> Result<(), EncodeError<W::Error>> {
    write_f32(writer, value.x)?;
    write_f32(writer, value.y)?;
    write_f32(writer, value.z)
}

fn read_death_notification<R: BitRead>(
    reader: &mut R,
) -> Result<DeathNotification, DecodeError<R::Error>> {
    Ok(DeathNotification {
        reason: read_u8(reader)?,
        killer_id: read_u16(reader)?,
    })
}

fn write_death_notification<W: BitWrite>(
    writer: &mut W,
    value: &DeathNotification,
) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, &value.reason)?;
    write_u16(writer, &value.killer_id)
}

fn read_money_increase<R: BitRead>(reader: &mut R) -> Result<MoneyIncrease, DecodeError<R::Error>> {
    Ok(MoneyIncrease {
        amount: read_i32(reader)?,
        increase_type: read_i32(reader)?,
    })
}

fn write_money_increase<W: BitWrite>(
    writer: &mut W,
    value: &MoneyIncrease,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.amount)?;
    write_i32(writer, &value.increase_type)
}

fn read_camera_target_update<R: BitRead>(
    reader: &mut R,
) -> Result<CameraTargetUpdate, DecodeError<R::Error>> {
    Ok(CameraTargetUpdate {
        object_id: read_u16(reader)?,
        vehicle_id: read_u16(reader)?,
        player_id: read_u16(reader)?,
        actor_id: read_u16(reader)?,
    })
}

fn write_camera_target_update<W: BitWrite>(
    writer: &mut W,
    value: &CameraTargetUpdate,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.object_id)?;
    write_u16(writer, &value.vehicle_id)?;
    write_u16(writer, &value.player_id)?;
    write_u16(writer, &value.actor_id)
}

fn read_client_join<R: BitRead>(reader: &mut R) -> Result<ClientJoin, DecodeError<R::Error>> {
    Ok(ClientJoin {
        version: read_i32(reader)?,
        mod_id: read_u8(reader)?,
        nickname: read_string8(reader)?,
        challenge_response: read_i32(reader)?,
        join_auth_key: read_string8(reader)?,
        client_version: read_string8(reader)?,
        challenge_response2: read_i32(reader)?,
    })
}

fn write_client_join<W: BitWrite>(
    writer: &mut W,
    value: &ClientJoin,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.version)?;
    write_u8(writer, &value.mod_id)?;
    write_string8(writer, &value.nickname)?;
    write_i32(writer, &value.challenge_response)?;
    write_string8(writer, &value.join_auth_key)?;
    write_string8(writer, &value.client_version)?;
    write_i32(writer, &value.challenge_response2)
}

fn read_npc_join<R: BitRead>(reader: &mut R) -> Result<NpcJoin, DecodeError<R::Error>> {
    Ok(NpcJoin {
        version: read_i32(reader)?,
        mod_id: read_u8(reader)?,
        nickname: read_string8(reader)?,
        challenge_response: read_i32(reader)?,
    })
}

fn write_npc_join<W: BitWrite>(
    writer: &mut W,
    value: &NpcJoin,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.version)?;
    write_u8(writer, &value.mod_id)?;
    write_string8(writer, &value.nickname)?;
    write_i32(writer, &value.challenge_response)
}

fn read_vehicle_damage<R: BitRead>(reader: &mut R) -> Result<VehicleDamage, DecodeError<R::Error>> {
    Ok(VehicleDamage {
        vehicle_id: read_u16(reader)?,
        panel_damage: read_i32(reader)?,
        door_damage: read_i32(reader)?,
        lights: read_u8(reader)?,
        tires: read_u8(reader)?,
    })
}

fn write_vehicle_damage<W: BitWrite>(
    writer: &mut W,
    value: &VehicleDamage,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.vehicle_id)?;
    write_i32(writer, &value.panel_damage)?;
    write_i32(writer, &value.door_damage)?;
    write_u8(writer, &value.lights)?;
    write_u8(writer, &value.tires)
}

fn read_enter_edit_object<R: BitRead>(
    reader: &mut R,
) -> Result<EnterEditObject, DecodeError<R::Error>> {
    Ok(EnterEditObject {
        object_type: read_i32(reader)?,
        object_id: read_u16(reader)?,
        model_id: read_i32(reader)?,
        position: read_vector3(reader)?,
    })
}

fn write_enter_edit_object<W: BitWrite>(
    writer: &mut W,
    value: &EnterEditObject,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.object_type)?;
    write_u16(writer, &value.object_id)?;
    write_i32(writer, &value.model_id)?;
    write_vector3(writer, &value.position)
}

fn read_edit_attached_object<R: BitRead>(
    reader: &mut R,
) -> Result<EditAttachedObject, DecodeError<R::Error>> {
    Ok(EditAttachedObject {
        response: read_i32(reader)?,
        index: read_i32(reader)?,
        model_id: read_i32(reader)?,
        bone: read_i32(reader)?,
        position: read_vector3(reader)?,
        rotation: read_vector3(reader)?,
        scale: read_vector3(reader)?,
        color1: read_i32(reader)?,
        color2: read_i32(reader)?,
    })
}

fn write_edit_attached_object<W: BitWrite>(
    writer: &mut W,
    value: &EditAttachedObject,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.response)?;
    write_i32(writer, &value.index)?;
    write_i32(writer, &value.model_id)?;
    write_i32(writer, &value.bone)?;
    write_vector3(writer, &value.position)?;
    write_vector3(writer, &value.rotation)?;
    write_vector3(writer, &value.scale)?;
    write_i32(writer, &value.color1)?;
    write_i32(writer, &value.color2)
}

fn read_client_check_response<R: BitRead>(
    reader: &mut R,
) -> Result<ClientCheckResponse, DecodeError<R::Error>> {
    Ok(ClientCheckResponse {
        request_type: read_u8(reader)?,
        result1: read_i32(reader)?,
        result2: read_u8(reader)?,
    })
}

fn write_client_check_response<W: BitWrite>(
    writer: &mut W,
    value: &ClientCheckResponse,
) -> Result<(), EncodeError<W::Error>> {
    write_u8(writer, &value.request_type)?;
    write_i32(writer, &value.result1)?;
    write_u8(writer, &value.result2)
}

fn read_dialog_response<R: BitRead>(
    reader: &mut R,
) -> Result<DialogResponse, DecodeError<R::Error>> {
    Ok(DialogResponse {
        dialog_id: read_u16(reader)?,
        button: read_u8(reader)?,
        list_item: read_u16(reader)?,
        input: read_string8(reader)?,
    })
}

fn write_dialog_response<W: BitWrite>(
    writer: &mut W,
    value: &DialogResponse,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.dialog_id)?;
    write_u8(writer, &value.button)?;
    write_u16(writer, &value.list_item)?;
    write_string8(writer, &value.input)
}

fn read_click_player<R: BitRead>(reader: &mut R) -> Result<ClickPlayer, DecodeError<R::Error>> {
    Ok(ClickPlayer {
        player_id: read_u16(reader)?,
        source: read_u8(reader)?,
    })
}

fn write_click_player<W: BitWrite>(
    writer: &mut W,
    value: &ClickPlayer,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.player_id)?;
    write_u8(writer, &value.source)
}

fn read_enter_vehicle<R: BitRead>(reader: &mut R) -> Result<EnterVehicle, DecodeError<R::Error>> {
    Ok(EnterVehicle {
        vehicle_id: read_u16(reader)?,
        passenger: read_u8(reader)? != 0,
    })
}

fn write_enter_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &EnterVehicle,
) -> Result<(), EncodeError<W::Error>> {
    write_u16(writer, &value.vehicle_id)?;
    write_u8(writer, &u8::from(value.passenger))
}

fn read_vehicle_tuning<R: BitRead>(reader: &mut R) -> Result<VehicleTuning, DecodeError<R::Error>> {
    Ok(VehicleTuning {
        vehicle_id: read_i32(reader)?,
        param1: read_i32(reader)?,
        param2: read_i32(reader)?,
        event: read_i32(reader)?,
    })
}

fn write_vehicle_tuning<W: BitWrite>(
    writer: &mut W,
    value: &VehicleTuning,
) -> Result<(), EncodeError<W::Error>> {
    write_i32(writer, &value.vehicle_id)?;
    write_i32(writer, &value.param1)?;
    write_i32(writer, &value.param2)?;
    write_i32(writer, &value.event)
}

fn read_string8<R: BitRead>(reader: &mut R) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let length = usize::from(read_u8(reader)?);
    read_bytes(reader, length)
}

fn write_string8<W: BitWrite>(writer: &mut W, value: &[u8]) -> Result<(), EncodeError<W::Error>> {
    if value.len() > u8::MAX as usize {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.len(),
            limit: u8::MAX as usize,
        });
    }
    write_u8(writer, &(value.len() as u8))?;
    write_bytes(writer, value)
}

fn read_fixed<R: BitRead, const LENGTH: usize>(
    reader: &mut R,
) -> Result<[u8; LENGTH], DecodeError<R::Error>> {
    let bit_len = LENGTH * u8::BITS as usize;
    ensure_available(reader, bit_len)?;
    let bytes = reader
        .read_left_aligned_bits(bit_len)
        .map_err(DecodeError::Source)?;
    match bytes.try_into() {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(DecodeError::OutOfBounds {
            requested_bits: bit_len,
            available_bits: 0,
        }),
    }
}

fn read_bytes<R: BitRead>(reader: &mut R, length: usize) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let bit_len = length * u8::BITS as usize;
    ensure_available(reader, bit_len)?;
    reader
        .read_left_aligned_bits(bit_len)
        .map_err(DecodeError::Source)
}

fn ensure_available<R: BitRead>(
    reader: &R,
    requested_bits: usize,
) -> Result<(), DecodeError<R::Error>> {
    let available_bits = reader.remaining_bits();
    if requested_bits > available_bits {
        return Err(DecodeError::OutOfBounds {
            requested_bits,
            available_bits,
        });
    }
    Ok(())
}

fn write_bytes<W: BitWrite>(writer: &mut W, bytes: &[u8]) -> Result<(), EncodeError<W::Error>> {
    writer
        .write_left_aligned_bits(bytes, bytes.len() * u8::BITS as usize)
        .map_err(EncodeError::Source)
}
