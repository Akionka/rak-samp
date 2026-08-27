//! The fixed incoming RPC batch from camera attachment through vehicle exit.

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, ExactBytesPolicy, IncomingRpc, WireCodec,
    WireReadExt, WireWriteExt, limits::MAX_STRING32_BYTES, types::Vector2, types::Vector3,
};

/// MoonLoader's `onSetPlayerFightingStyle` payload (RPC 89).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerFightingStyle {
    pub player_id: u16,
    pub style_id: u8,
}

/// MoonLoader's `onSetVehicleVelocity` payload (RPC 91).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleVelocity {
    pub turn: bool,
    pub velocity: Vector3,
}

/// MoonLoader's `onCreatePickup` payload (RPC 95).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pickup {
    pub id: i32,
    pub model: i32,
    pub pickup_type: i32,
    pub position: Vector3,
}

/// MoonLoader's `onMoveObject` payload (RPC 99).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoveObject {
    pub object_id: u16,
    pub from_position: Vector3,
    pub destination: Vector3,
    pub speed: f32,
    pub rotation: Vector3,
}

/// MoonLoader's `onTextDrawSetString` payload (RPC 105).
#[derive(Clone, Debug, PartialEq)]
pub struct TextDrawString {
    pub textdraw_id: u16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onCreateGangZone` payload (RPC 108).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GangZone {
    pub zone_id: u16,
    pub square_start: Vector2,
    pub square_end: Vector2,
    pub color: i32,
}

/// MoonLoader's `onSetVehicleNumberPlate` payload (RPC 123).
#[derive(Clone, Debug, PartialEq)]
pub struct VehicleNumberPlate {
    pub vehicle_id: u16,
    pub text: Vec<u8>,
}

/// MoonLoader's `onSpectatePlayer` / `onSpectateVehicle` payload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spectate {
    pub target_id: u16,
    pub camera_type: u8,
}

/// MoonLoader's `onSetWeaponAmmo` payload (RPC 145).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponAmmo {
    pub weapon_id: u8,
    pub ammo: u16,
}

/// MoonLoader's `onAttachTrailerToVehicle` payload (RPC 148).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrailerAttachment {
    pub trailer_id: u16,
    pub vehicle_id: u16,
}

/// MoonLoader's `onSetCameraLookAt` payload (RPC 158).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraLookAt {
    pub position: Vector3,
    pub cut_type: u8,
}

/// MoonLoader's `onSetVehicleParams` payload (RPC 161).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VehicleParams {
    pub vehicle_id: u16,
    pub objective: bool,
    pub doors_locked: bool,
}

/// MoonLoader's `onPlayerEnterVehicle` payload (RPC 26).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerEnterVehicle {
    pub player_id: u16,
    pub vehicle_id: u16,
    pub passenger: bool,
}

/// MoonLoader's `onPlayerExitVehicle` payload (RPC 154).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerExitVehicle {
    pub player_id: u16,
    pub vehicle_id: u16,
}

pub struct PlayerFightingStyleCodec;
pub struct VehicleVelocityCodec;
pub struct PickupCodec;
pub struct MoveObjectCodec;
pub struct TextDrawStringCodec;
pub struct GangZoneCodec;
pub struct U16I32Codec;
pub struct VehicleNumberPlateCodec;
pub struct SpectateCodec;
pub struct WeaponAmmoCodec;
pub struct TrailerAttachmentCodec;
pub struct CameraLookAtCodec;
pub struct VehicleParamsCodec;
pub struct PlayerEnterVehicleCodec;
pub struct PlayerExitVehicleCodec;

macro_rules! descriptor {
    ($name:ident, $constant:ident, $id:literal, $codec:ty) => {
        pub type $name = IncomingRpc<$id, $codec, ExactBytesPolicy>;
        pub const $constant: $name = IncomingRpc::new();
    };
}

descriptor!(
    AttachCameraToObject,
    ATTACH_CAMERA_TO_OBJECT,
    81,
    super::U16
);
descriptor!(GangZoneStopFlash, GANG_ZONE_STOP_FLASH, 85, super::U16);
descriptor!(ClearPlayerAnimation, CLEAR_PLAYER_ANIMATION, 87, super::U16);
descriptor!(
    SetPlayerSpecialAction,
    SET_PLAYER_SPECIAL_ACTION,
    88,
    super::U8
);
descriptor!(
    SetPlayerFightingStyle,
    SET_PLAYER_FIGHTING_STYLE,
    89,
    PlayerFightingStyleCodec
);
descriptor!(
    SetPlayerVelocity,
    SET_PLAYER_VELOCITY,
    90,
    super::Vector3Codec
);
descriptor!(
    SetVehicleVelocity,
    SET_VEHICLE_VELOCITY,
    91,
    VehicleVelocityCodec
);
descriptor!(CreatePickup, CREATE_PICKUP, 95, PickupCodec);
descriptor!(MoveObjectRpc, MOVE_OBJECT, 99, MoveObjectCodec);
descriptor!(
    TextDrawSetString,
    TEXT_DRAW_SET_STRING,
    105,
    TextDrawStringCodec
);
descriptor!(CreateGangZone, CREATE_GANG_ZONE, 108, GangZoneCodec);
descriptor!(GangZoneDestroy, GANG_ZONE_DESTROY, 120, super::U16);
descriptor!(GangZoneFlash, GANG_ZONE_FLASH, 121, U16I32Codec);
descriptor!(StopObject, STOP_OBJECT, 122, super::U16);
descriptor!(
    SetVehicleNumberPlate,
    SET_VEHICLE_NUMBER_PLATE,
    123,
    VehicleNumberPlateCodec
);
descriptor!(SpectatePlayer, SPECTATE_PLAYER, 126, SpectateCodec);
descriptor!(SpectateVehicle, SPECTATE_VEHICLE, 127, SpectateCodec);
descriptor!(ConnectionRejected, CONNECTION_REJECTED, 130, super::U8);
descriptor!(RemoveMapIcon, REMOVE_MAP_ICON, 144, super::U8);
descriptor!(SetWeaponAmmo, SET_WEAPON_AMMO, 145, WeaponAmmoCodec);
descriptor!(SetGravity, SET_GRAVITY, 146, super::F32);
descriptor!(
    AttachTrailerToVehicle,
    ATTACH_TRAILER_TO_VEHICLE,
    148,
    TrailerAttachmentCodec
);
descriptor!(
    DetachTrailerFromVehicle,
    DETACH_TRAILER_FROM_VEHICLE,
    149,
    super::U16
);
descriptor!(
    SetCameraPosition,
    SET_CAMERA_POSITION,
    157,
    super::Vector3Codec
);
descriptor!(SetCameraLookAt, SET_CAMERA_LOOK_AT, 158, CameraLookAtCodec);
descriptor!(
    SetVehicleParams,
    SET_VEHICLE_PARAMS,
    161,
    VehicleParamsCodec
);
descriptor!(PlayerDeath, PLAYER_DEATH, 166, super::U16);
descriptor!(
    PlayerEnterVehicleRpc,
    PLAYER_ENTER_VEHICLE,
    26,
    PlayerEnterVehicleCodec
);
descriptor!(
    PlayerExitVehicleRpc,
    PLAYER_EXIT_VEHICLE,
    154,
    PlayerExitVehicleCodec
);

macro_rules! fixed_codec {
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

fixed_codec!(
    PlayerFightingStyleCodec,
    PlayerFightingStyle,
    read_player_fighting_style,
    write_player_fighting_style
);
fixed_codec!(
    VehicleVelocityCodec,
    VehicleVelocity,
    read_vehicle_velocity,
    write_vehicle_velocity
);
fixed_codec!(PickupCodec, Pickup, read_pickup, write_pickup);
fixed_codec!(
    MoveObjectCodec,
    MoveObject,
    read_move_object,
    write_move_object
);
fixed_codec!(
    TextDrawStringCodec,
    TextDrawString,
    read_text_draw_string,
    write_text_draw_string
);
fixed_codec!(GangZoneCodec, GangZone, read_gang_zone, write_gang_zone);
fixed_codec!(U16I32Codec, (u16, i32), read_u16_i32, write_u16_i32);
fixed_codec!(
    VehicleNumberPlateCodec,
    VehicleNumberPlate,
    read_vehicle_number_plate,
    write_vehicle_number_plate
);
fixed_codec!(SpectateCodec, Spectate, read_spectate, write_spectate);
fixed_codec!(
    WeaponAmmoCodec,
    WeaponAmmo,
    read_weapon_ammo,
    write_weapon_ammo
);
fixed_codec!(
    TrailerAttachmentCodec,
    TrailerAttachment,
    read_trailer_attachment,
    write_trailer_attachment
);
fixed_codec!(
    CameraLookAtCodec,
    CameraLookAt,
    read_camera_look_at,
    write_camera_look_at
);
fixed_codec!(
    VehicleParamsCodec,
    VehicleParams,
    read_vehicle_params,
    write_vehicle_params
);
fixed_codec!(
    PlayerEnterVehicleCodec,
    PlayerEnterVehicle,
    read_player_enter_vehicle,
    write_player_enter_vehicle
);
fixed_codec!(
    PlayerExitVehicleCodec,
    PlayerExitVehicle,
    read_player_exit_vehicle,
    write_player_exit_vehicle
);

fn read_player_fighting_style<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerFightingStyle, DecodeError<R::Error>> {
    Ok(PlayerFightingStyle {
        player_id: reader.read_u16_le()?,
        style_id: reader.read_u8()?,
    })
}

fn write_player_fighting_style<W: BitWrite>(
    writer: &mut W,
    value: &PlayerFightingStyle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u8(value.style_id)
}

fn read_vehicle_velocity<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleVelocity, DecodeError<R::Error>> {
    Ok(VehicleVelocity {
        turn: reader.read_u8()? != 0,
        velocity: reader.read_vector3_le()?,
    })
}

fn write_vehicle_velocity<W: BitWrite>(
    writer: &mut W,
    value: &VehicleVelocity,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(u8::from(value.turn))?;
    writer.write_vector3_le(&value.velocity)
}

fn read_pickup<R: BitRead>(reader: &mut R) -> Result<Pickup, DecodeError<R::Error>> {
    Ok(Pickup {
        id: reader.read_i32_le()?,
        model: reader.read_i32_le()?,
        pickup_type: reader.read_i32_le()?,
        position: reader.read_vector3_le()?,
    })
}

fn write_pickup<W: BitWrite>(writer: &mut W, value: &Pickup) -> Result<(), EncodeError<W::Error>> {
    writer.write_i32_le(value.id)?;
    writer.write_i32_le(value.model)?;
    writer.write_i32_le(value.pickup_type)?;
    writer.write_vector3_le(&value.position)
}

fn read_move_object<R: BitRead>(reader: &mut R) -> Result<MoveObject, DecodeError<R::Error>> {
    Ok(MoveObject {
        object_id: reader.read_u16_le()?,
        from_position: reader.read_vector3_le()?,
        destination: reader.read_vector3_le()?,
        speed: reader.read_f32_le()?,
        rotation: reader.read_vector3_le()?,
    })
}

fn write_move_object<W: BitWrite>(
    writer: &mut W,
    value: &MoveObject,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.object_id)?;
    writer.write_vector3_le(&value.from_position)?;
    writer.write_vector3_le(&value.destination)?;
    writer.write_f32_le(value.speed)?;
    writer.write_vector3_le(&value.rotation)
}

fn read_text_draw_string<R: BitRead>(
    reader: &mut R,
) -> Result<TextDrawString, DecodeError<R::Error>> {
    Ok(TextDrawString {
        textdraw_id: reader.read_u16_le()?,
        text: reader.read_len_prefixed_bytes_u16_le(MAX_STRING32_BYTES)?,
    })
}

fn write_text_draw_string<W: BitWrite>(
    writer: &mut W,
    value: &TextDrawString,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.textdraw_id)?;
    writer.write_len_prefixed_bytes_u16_le(&value.text, MAX_STRING32_BYTES)
}

fn read_gang_zone<R: BitRead>(reader: &mut R) -> Result<GangZone, DecodeError<R::Error>> {
    Ok(GangZone {
        zone_id: reader.read_u16_le()?,
        square_start: reader.read_vector2_le()?,
        square_end: reader.read_vector2_le()?,
        color: reader.read_i32_le()?,
    })
}

fn write_gang_zone<W: BitWrite>(
    writer: &mut W,
    value: &GangZone,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.zone_id)?;
    writer.write_vector2_le(&value.square_start)?;
    writer.write_vector2_le(&value.square_end)?;
    writer.write_i32_le(value.color)
}

fn read_u16_i32<R: BitRead>(reader: &mut R) -> Result<(u16, i32), DecodeError<R::Error>> {
    Ok((reader.read_u16_le()?, reader.read_i32_le()?))
}

fn write_u16_i32<W: BitWrite>(
    writer: &mut W,
    value: &(u16, i32),
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.0)?;
    writer.write_i32_le(value.1)
}

fn read_vehicle_number_plate<R: BitRead>(
    reader: &mut R,
) -> Result<VehicleNumberPlate, DecodeError<R::Error>> {
    Ok(VehicleNumberPlate {
        vehicle_id: reader.read_u16_le()?,
        text: reader.read_len_prefixed_bytes_u8(usize::from(u8::MAX))?,
    })
}

fn write_vehicle_number_plate<W: BitWrite>(
    writer: &mut W,
    value: &VehicleNumberPlate,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_len_prefixed_bytes_u8(&value.text, usize::from(u8::MAX))
}

fn read_spectate<R: BitRead>(reader: &mut R) -> Result<Spectate, DecodeError<R::Error>> {
    Ok(Spectate {
        target_id: reader.read_u16_le()?,
        camera_type: reader.read_u8()?,
    })
}

fn write_spectate<W: BitWrite>(
    writer: &mut W,
    value: &Spectate,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.target_id)?;
    writer.write_u8(value.camera_type)
}

fn read_weapon_ammo<R: BitRead>(reader: &mut R) -> Result<WeaponAmmo, DecodeError<R::Error>> {
    Ok(WeaponAmmo {
        weapon_id: reader.read_u8()?,
        ammo: reader.read_u16_le()?,
    })
}

fn write_weapon_ammo<W: BitWrite>(
    writer: &mut W,
    value: &WeaponAmmo,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(value.weapon_id)?;
    writer.write_u16_le(value.ammo)
}

fn read_trailer_attachment<R: BitRead>(
    reader: &mut R,
) -> Result<TrailerAttachment, DecodeError<R::Error>> {
    Ok(TrailerAttachment {
        trailer_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
    })
}

fn write_trailer_attachment<W: BitWrite>(
    writer: &mut W,
    value: &TrailerAttachment,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.trailer_id)?;
    writer.write_u16_le(value.vehicle_id)
}

fn read_camera_look_at<R: BitRead>(reader: &mut R) -> Result<CameraLookAt, DecodeError<R::Error>> {
    Ok(CameraLookAt {
        position: reader.read_vector3_le()?,
        cut_type: reader.read_u8()?,
    })
}

fn write_camera_look_at<W: BitWrite>(
    writer: &mut W,
    value: &CameraLookAt,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_vector3_le(&value.position)?;
    writer.write_u8(value.cut_type)
}

fn read_vehicle_params<R: BitRead>(reader: &mut R) -> Result<VehicleParams, DecodeError<R::Error>> {
    Ok(VehicleParams {
        vehicle_id: reader.read_u16_le()?,
        objective: reader.read_u8()? != 0,
        doors_locked: reader.read_u8()? != 0,
    })
}

fn write_vehicle_params<W: BitWrite>(
    writer: &mut W,
    value: &VehicleParams,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(u8::from(value.objective))?;
    writer.write_u8(u8::from(value.doors_locked))
}

fn read_player_enter_vehicle<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerEnterVehicle, DecodeError<R::Error>> {
    Ok(PlayerEnterVehicle {
        player_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
        passenger: reader.read_u8()? != 0,
    })
}

fn write_player_enter_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &PlayerEnterVehicle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u16_le(value.vehicle_id)?;
    writer.write_u8(u8::from(value.passenger))
}

fn read_player_exit_vehicle<R: BitRead>(
    reader: &mut R,
) -> Result<PlayerExitVehicle, DecodeError<R::Error>> {
    Ok(PlayerExitVehicle {
        player_id: reader.read_u16_le()?,
        vehicle_id: reader.read_u16_le()?,
    })
}

fn write_player_exit_vehicle<W: BitWrite>(
    writer: &mut W,
    value: &PlayerExitVehicle,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.player_id)?;
    writer.write_u16_le(value.vehicle_id)
}
