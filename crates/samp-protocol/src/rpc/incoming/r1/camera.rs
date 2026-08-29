use super::wire::{decode_bit_bool, encode_bit_bool};
use crate::types::Vector3;
use crate::{BitRead, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt};

/// R1's `onInterpolateCamera` payload (RPC 82).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterpolateCamera {
    pub set_position: bool,
    pub from_position: Vector3,
    pub destination: Vector3,
    pub time_ms: i32,
    pub mode: u8,
}

struct InterpolateCameraCodec;

struct ToggleCameraTargetNotifyingCodec;

descriptor!(
    InterpolateCameraRpc,
    INTERPOLATE_CAMERA,
    82,
    InterpolateCameraCodec,
    InterpolateCamera,
    ExactBitsPolicy
);

descriptor!(
    ToggleCameraTargetNotifyingRpc,
    TOGGLE_CAMERA_TARGET_NOTIFYING,
    170,
    ToggleCameraTargetNotifyingCodec,
    bool,
    ExactBitsPolicy
);

r1_codec!(
    InterpolateCameraCodec,
    InterpolateCamera,
    decode_interpolate_camera,
    encode_interpolate_camera
);

r1_codec!(
    ToggleCameraTargetNotifyingCodec,
    bool,
    decode_bit_bool,
    encode_bit_bool
);

fn decode_interpolate_camera<R: BitRead>(
    reader: &mut R,
) -> Result<InterpolateCamera, DecodeError<R::Error>> {
    Ok(InterpolateCamera {
        set_position: reader.read_bit_bool()?,
        from_position: reader.read_vector3_le()?,
        destination: reader.read_vector3_le()?,
        time_ms: reader.read_i32_le()?,
        mode: reader.read_u8()?,
    })
}

fn encode_interpolate_camera<W: BitWrite>(
    writer: &mut W,
    value: &InterpolateCamera,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bit_bool(value.set_position)?;
    writer.write_vector3_le(&value.from_position)?;
    writer.write_vector3_le(&value.destination)?;
    writer.write_i32_le(value.time_ms)?;
    writer.write_u8(value.mode)
}
