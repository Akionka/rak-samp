//! Outgoing chat and slash-command RPC codecs.

use crate::{BitRead, BitWrite, DecodeError, EncodeError, Rpc, TrailingPolicy, WireCodec};

/// The `onSendChat` RPC ID.
pub type SendChat = Rpc<101, String8>;
/// The `onSendChat` descriptor.
pub const SEND_CHAT: SendChat = Rpc::new();
/// The `onSendCommand` RPC ID.
pub type SendCommand = Rpc<50, String32<4096>>;
/// The `onSendCommand` descriptor.
pub const SEND_COMMAND: SendCommand = Rpc::new();

/// A byte string with an unsigned 8-bit byte-length prefix.
pub struct String8;

impl WireCodec for String8 {
    type Value = Vec<u8>;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let length = usize::from(read_fixed::<R, 1>(reader)?[0]);
        read_bytes(reader, length)
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        if value.len() > u8::MAX as usize {
            return Err(EncodeError::LengthExceedsLimit {
                length: value.len(),
                limit: u8::MAX as usize,
            });
        }
        write_bytes(writer, &[value.len() as u8])?;
        write_bytes(writer, value)
    }
}

/// A byte string with an unsigned little-endian 32-bit byte-length prefix.
pub struct String32<const LIMIT: usize>;

impl<const LIMIT: usize> WireCodec for String32<LIMIT> {
    type Value = Vec<u8>;

    const TRAILING_POLICY: TrailingPolicy = TrailingPolicy::ExactBytes;

    fn decode<R: BitRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>> {
        let length = u32::from_le_bytes(read_fixed::<R, 4>(reader)?) as usize;
        let limit = LIMIT.min(u32::MAX as usize);
        if length > limit {
            return Err(DecodeError::LengthExceedsLimit { length, limit });
        }
        read_bytes(reader, length)
    }

    fn encode<W: BitWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        let limit = LIMIT.min(u32::MAX as usize);
        if value.len() > limit {
            return Err(EncodeError::LengthExceedsLimit {
                length: value.len(),
                limit,
            });
        }
        write_bytes(writer, &(value.len() as u32).to_le_bytes())?;
        write_bytes(writer, value)
    }
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
