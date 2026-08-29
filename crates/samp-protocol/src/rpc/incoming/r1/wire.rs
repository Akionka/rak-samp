//! Shared primitive wire helpers for R1 incoming RPCs.

use crate::{BitRead, BitWrite, DecodeError, EncodeError, WireReadExt, WireWriteExt};

pub(super) fn decode_bit_bool<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    reader.read_bit_bool()
}

pub(super) fn encode_bit_bool<W: BitWrite>(
    writer: &mut W,
    value: &bool,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_bit_bool(*value)
}

pub(super) fn decode_bool32<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u32_le()? != 0)
}

pub(super) fn encode_bool32<W: BitWrite>(
    writer: &mut W,
    value: &bool,
) -> Result<(), EncodeError<W::Error>> {
    write_bool32(writer, *value)
}

pub(super) fn read_bool32<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u32_le()? != 0)
}

pub(super) fn write_bool32<W: BitWrite>(
    writer: &mut W,
    value: bool,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u32_le(u32::from(value))
}

pub(super) fn read_bool8<R: BitRead>(reader: &mut R) -> Result<bool, DecodeError<R::Error>> {
    Ok(reader.read_u8()? != 0)
}

pub(super) fn write_bool8<W: BitWrite>(
    writer: &mut W,
    value: bool,
) -> Result<(), EncodeError<W::Error>> {
    writer.write_u8(u8::from(value))
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
