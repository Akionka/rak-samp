use super::wire::{Empty, U16, Vector3Codec};
use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt, types::Vector3,
};

/// MoonLoader's `onSpectatePlayer` / `onSpectateVehicle` payload.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spectate {
    pub target_id: u16,
    pub camera_type: u8,
}

/// MoonLoader's `onSetCameraLookAt` payload (RPC 158).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraLookAt {
    pub position: Vector3,
    pub cut_type: u8,
}

struct SpectateCodec;

struct CameraLookAtCodec;

descriptor!(SetCameraBehind, SET_CAMERA_BEHIND, 162, Empty, ());

descriptor!(AttachCameraToObject, ATTACH_CAMERA_TO_OBJECT, 81, U16, u16);

descriptor!(
    SpectatePlayer,
    SPECTATE_PLAYER,
    126,
    SpectateCodec,
    Spectate
);

descriptor!(
    SpectateVehicle,
    SPECTATE_VEHICLE,
    127,
    SpectateCodec,
    Spectate
);

descriptor!(
    SetCameraPosition,
    SET_CAMERA_POSITION,
    157,
    Vector3Codec,
    Vector3
);

descriptor!(
    SetCameraLookAt,
    SET_CAMERA_LOOK_AT,
    158,
    CameraLookAtCodec,
    CameraLookAt
);

wire_codec!(SpectateCodec, Spectate, read_spectate, write_spectate);

wire_codec!(
    CameraLookAtCodec,
    CameraLookAt,
    read_camera_look_at,
    write_camera_look_at
);

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
