//! Shared primitive wire mechanics for profile-neutral incoming RPCs.

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, WireCodec, WireReadExt, WireWriteExt,
    types::Vector3,
};

pub(super) struct Empty;
pub(super) struct U8;
pub(super) struct U16;
pub(super) struct I32;
pub(super) struct F32;
pub(super) struct Bool8;
pub(super) struct Vector3Codec;
pub(super) struct FixedString32Codec;
pub(super) struct U16U8Codec;
pub(super) struct U16I32Codec;

macro_rules! wire_codec {
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

macro_rules! scalar_wire_codec {
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

macro_rules! vector_wire_codec {
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

wire_codec!(Empty, (), read_empty, write_empty);
scalar_wire_codec!(U8, u8, read_u8, write_u8);
scalar_wire_codec!(U16, u16, read_u16_le, write_u16_le);
scalar_wire_codec!(I32, i32, read_i32_le, write_i32_le);
scalar_wire_codec!(F32, f32, read_f32_le, write_f32_le);
wire_codec!(Bool8, bool, read_bool8, write_bool8);
vector_wire_codec!(Vector3Codec, Vector3, read_vector3_le, write_vector3_le);
wire_codec!(
    FixedString32Codec,
    [u8; 32],
    read_fixed_string32,
    write_fixed_string32
);
wire_codec!(U16U8Codec, (u16, u8), read_u16_u8, write_u16_u8);
wire_codec!(U16I32Codec, (u16, i32), read_u16_i32, write_u16_i32);

fn read_empty<R: BitRead>(_reader: &mut R) -> Result<(), DecodeError<R::Error>> {
    Ok(())
}

fn write_empty<W: BitWrite>(_writer: &mut W, _value: &()) -> Result<(), EncodeError<W::Error>> {
    Ok(())
}

pub(super) fn read_bool8<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u8()? != 0)
}

pub(super) fn write_bool8<W: BitWrite>(
    writer: &mut W,
    value: &bool,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(u8::from(*value))
}

pub(super) fn read_fixed<R: BitRead, const LENGTH: usize>(
    reader: &mut R,
) -> Result<[u8; LENGTH], DecodeError<R::Error>> {
    let bytes = reader.read_bytes(LENGTH)?;
    match bytes.try_into() {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err(DecodeError::OutOfBounds {
            requested_bits: LENGTH * u8::BITS as usize,
            available_bits: 0,
        }),
    }
}

fn read_fixed_string32<R: BitRead>(reader: &mut R) -> Result<[u8; 32], DecodeError<R::Error>> {
    read_fixed(reader)
}

fn write_fixed_string32<W: BitWrite>(
    writer: &mut W,
    value: &[u8; 32],
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bytes(value)
}

fn read_u16_u8<R: BitRead>(reader: &mut R) -> Result<(u16, u8), DecodeError<R::Error>> {
    Ok((reader.read_u16_le()?, reader.read_u8()?))
}

fn write_u16_u8<W: BitWrite>(
    writer: &mut W,
    value: &(u16, u8),
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u16_le(value.0)?;
    writer.write_u8(value.1)
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
