//! Neutral primitive operations over raw Protocol bit I/O.
//!
//! This layer owns binary representation only. Text, profile, compression, and
//! message semantics remain in specialized codecs.

use crate::{
    BitRead, BitWrite, DecodeError, EncodeError,
    types::{Vector2, Vector3},
};

/// Reads neutral Protocol primitives from the current bit cursor.
///
/// Multi-byte values use little-endian byte order. These operations never align
/// the cursor implicitly.
pub trait WireReadExt: BitRead {
    /// Reads one boolean encoded as a single MSB-first bit.
    fn read_bit_bool(&mut self) -> Result<bool, DecodeError<Self::Error>> {
        let bits = read_left_aligned_wire_bits(self, 1)?;
        match bits.as_slice() {
            [byte] => Ok(byte & 0x80 != 0),
            _ => Err(DecodeError::InvalidBitLength {
                bit_len: 1,
                byte_len: bits.len(),
            }),
        }
    }

    /// Reads one byte.
    fn read_u8(&mut self) -> Result<u8, DecodeError<Self::Error>> {
        Ok(read_fixed::<_, 1>(self)?[0])
    }

    /// Reads a little-endian signed 16-bit integer.
    fn read_i16_le(&mut self) -> Result<i16, DecodeError<Self::Error>> {
        Ok(i16::from_le_bytes(read_fixed(self)?))
    }

    /// Reads a little-endian unsigned 16-bit integer.
    fn read_u16_le(&mut self) -> Result<u16, DecodeError<Self::Error>> {
        Ok(u16::from_le_bytes(read_fixed(self)?))
    }

    /// Reads a little-endian signed 32-bit integer.
    fn read_i32_le(&mut self) -> Result<i32, DecodeError<Self::Error>> {
        Ok(i32::from_le_bytes(read_fixed(self)?))
    }

    /// Reads a little-endian unsigned 32-bit integer.
    fn read_u32_le(&mut self) -> Result<u32, DecodeError<Self::Error>> {
        Ok(u32::from_le_bytes(read_fixed(self)?))
    }

    /// Reads a little-endian 32-bit floating-point value.
    fn read_f32_le(&mut self) -> Result<f32, DecodeError<Self::Error>> {
        Ok(f32::from_le_bytes(read_fixed(self)?))
    }

    /// Reads `byte_len` raw bytes.
    fn read_bytes(&mut self, byte_len: usize) -> Result<Vec<u8>, DecodeError<Self::Error>> {
        let requested_bits = decode_byte_bit_len(byte_len)?;
        let bytes = read_left_aligned_wire_bits(self, requested_bits)?;
        if bytes.len() != byte_len {
            return Err(DecodeError::InvalidBitLength {
                bit_len: requested_bits,
                byte_len: bytes.len(),
            });
        }
        Ok(bytes)
    }

    /// Reads an explicitly bounded byte sequence with a `u8` byte-count prefix.
    fn read_len_prefixed_bytes_u8(
        &mut self,
        max_len: usize,
    ) -> Result<Vec<u8>, DecodeError<Self::Error>> {
        let byte_len = usize::from(WireReadExt::read_u8(self)?);
        read_len_prefixed_bytes(self, byte_len, max_len)
    }

    /// Reads an explicitly bounded byte sequence with a little-endian `u16` byte-count prefix.
    fn read_len_prefixed_bytes_u16_le(
        &mut self,
        max_len: usize,
    ) -> Result<Vec<u8>, DecodeError<Self::Error>> {
        let byte_len = usize::from(WireReadExt::read_u16_le(self)?);
        read_len_prefixed_bytes(self, byte_len, max_len)
    }

    /// Reads an explicitly bounded byte sequence with a little-endian `u32` byte-count prefix.
    fn read_len_prefixed_bytes_u32_le(
        &mut self,
        max_len: usize,
    ) -> Result<Vec<u8>, DecodeError<Self::Error>> {
        let encoded_len = WireReadExt::read_u32_le(self)?;
        let byte_len =
            usize::try_from(encoded_len).map_err(|_| DecodeError::LengthExceedsLimit {
                length: usize::MAX,
                limit: max_len,
            })?;
        read_len_prefixed_bytes(self, byte_len, max_len)
    }

    /// Reads a two-dimensional vector as two little-endian `f32` fields.
    fn read_vector2_le(&mut self) -> Result<Vector2, DecodeError<Self::Error>> {
        Ok(Vector2 {
            x: WireReadExt::read_f32_le(self)?,
            y: WireReadExt::read_f32_le(self)?,
        })
    }

    /// Reads a three-dimensional vector as three little-endian `f32` fields.
    fn read_vector3_le(&mut self) -> Result<Vector3, DecodeError<Self::Error>> {
        Ok(Vector3 {
            x: WireReadExt::read_f32_le(self)?,
            y: WireReadExt::read_f32_le(self)?,
            z: WireReadExt::read_f32_le(self)?,
        })
    }
}

impl<T: BitRead + ?Sized> WireReadExt for T {}

/// Writes neutral Protocol primitives at the current bit cursor.
///
/// Multi-byte values use little-endian byte order. These operations never align
/// the cursor implicitly.
pub trait WireWriteExt: BitWrite {
    /// Writes one boolean as a single MSB-first bit.
    fn write_bit_bool(&mut self, value: bool) -> Result<(), EncodeError<Self::Error>> {
        self.write_left_aligned_bits(&[u8::from(value) << 7], 1)
            .map_err(EncodeError::Source)
    }

    /// Writes one byte.
    fn write_u8(&mut self, value: u8) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_bytes(self, &[value])
    }

    /// Writes a little-endian signed 16-bit integer.
    fn write_i16_le(&mut self, value: i16) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_bytes(self, &value.to_le_bytes())
    }

    /// Writes a little-endian unsigned 16-bit integer.
    fn write_u16_le(&mut self, value: u16) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_bytes(self, &value.to_le_bytes())
    }

    /// Writes a little-endian signed 32-bit integer.
    fn write_i32_le(&mut self, value: i32) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_bytes(self, &value.to_le_bytes())
    }

    /// Writes a little-endian unsigned 32-bit integer.
    fn write_u32_le(&mut self, value: u32) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_bytes(self, &value.to_le_bytes())
    }

    /// Writes a little-endian 32-bit floating-point value.
    fn write_f32_le(&mut self, value: f32) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_bytes(self, &value.to_le_bytes())
    }

    /// Writes raw bytes.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError<Self::Error>> {
        let bit_len = encode_byte_bit_len(bytes.len())?;
        self.write_left_aligned_bits(bytes, bit_len)
            .map_err(EncodeError::Source)
    }

    /// Writes an explicitly bounded byte sequence with a `u8` byte-count prefix.
    fn write_len_prefixed_bytes_u8(
        &mut self,
        bytes: &[u8],
        max_len: usize,
    ) -> Result<(), EncodeError<Self::Error>> {
        validate_encoded_length(bytes.len(), max_len, usize::from(u8::MAX))?;
        WireWriteExt::write_u8(self, bytes.len() as u8)?;
        WireWriteExt::write_bytes(self, bytes)
    }

    /// Writes an explicitly bounded byte sequence with a little-endian `u16` byte-count prefix.
    fn write_len_prefixed_bytes_u16_le(
        &mut self,
        bytes: &[u8],
        max_len: usize,
    ) -> Result<(), EncodeError<Self::Error>> {
        validate_encoded_length(bytes.len(), max_len, usize::from(u16::MAX))?;
        let byte_len = u16::try_from(bytes.len()).map_err(|_| EncodeError::LengthExceedsLimit {
            length: bytes.len(),
            limit: usize::from(u16::MAX),
        })?;
        WireWriteExt::write_u16_le(self, byte_len)?;
        WireWriteExt::write_bytes(self, bytes)
    }

    /// Writes an explicitly bounded byte sequence with a little-endian `u32` byte-count prefix.
    fn write_len_prefixed_bytes_u32_le(
        &mut self,
        bytes: &[u8],
        max_len: usize,
    ) -> Result<(), EncodeError<Self::Error>> {
        validate_encoded_length(bytes.len(), max_len, u32::MAX as usize)?;
        let byte_len = u32::try_from(bytes.len()).map_err(|_| EncodeError::LengthExceedsLimit {
            length: bytes.len(),
            limit: u32::MAX as usize,
        })?;
        WireWriteExt::write_u32_le(self, byte_len)?;
        WireWriteExt::write_bytes(self, bytes)
    }

    /// Writes a two-dimensional vector as two little-endian `f32` fields.
    fn write_vector2_le(&mut self, value: &Vector2) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_f32_le(self, value.x)?;
        WireWriteExt::write_f32_le(self, value.y)
    }

    /// Writes a three-dimensional vector as three little-endian `f32` fields.
    fn write_vector3_le(&mut self, value: &Vector3) -> Result<(), EncodeError<Self::Error>> {
        WireWriteExt::write_f32_le(self, value.x)?;
        WireWriteExt::write_f32_le(self, value.y)?;
        WireWriteExt::write_f32_le(self, value.z)
    }
}

impl<T: BitWrite + ?Sized> WireWriteExt for T {}

fn read_left_aligned_wire_bits<R: BitRead + ?Sized>(
    reader: &mut R,
    requested_bits: usize,
) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let available_bits = reader.remaining_bits();
    if requested_bits > available_bits {
        return Err(DecodeError::OutOfBounds {
            requested_bits,
            available_bits,
        });
    }
    reader
        .read_left_aligned_bits(requested_bits)
        .map_err(DecodeError::Source)
}

fn read_fixed<R: BitRead + ?Sized, const LENGTH: usize>(
    reader: &mut R,
) -> Result<[u8; LENGTH], DecodeError<R::Error>> {
    let bytes = WireReadExt::read_bytes(reader, LENGTH)?;
    match bytes.try_into() {
        Ok(bytes) => Ok(bytes),
        Err(bytes) => Err(DecodeError::InvalidBitLength {
            bit_len: decode_byte_bit_len(LENGTH)?,
            byte_len: bytes.len(),
        }),
    }
}

fn read_len_prefixed_bytes<R: BitRead + ?Sized>(
    reader: &mut R,
    byte_len: usize,
    max_len: usize,
) -> Result<Vec<u8>, DecodeError<R::Error>> {
    if byte_len > max_len {
        return Err(DecodeError::LengthExceedsLimit {
            length: byte_len,
            limit: max_len,
        });
    }
    WireReadExt::read_bytes(reader, byte_len)
}

fn validate_encoded_length<E>(
    byte_len: usize,
    max_len: usize,
    prefix_max_len: usize,
) -> Result<(), EncodeError<E>> {
    if byte_len > max_len {
        return Err(EncodeError::LengthExceedsLimit {
            length: byte_len,
            limit: max_len,
        });
    }
    if byte_len > prefix_max_len {
        return Err(EncodeError::LengthExceedsLimit {
            length: byte_len,
            limit: prefix_max_len,
        });
    }
    Ok(())
}

fn decode_byte_bit_len<E>(byte_len: usize) -> Result<usize, DecodeError<E>> {
    byte_len
        .checked_mul(u8::BITS as usize)
        .ok_or(DecodeError::InvalidBitLength {
            bit_len: usize::MAX,
            byte_len,
        })
}

fn encode_byte_bit_len<E>(byte_len: usize) -> Result<usize, EncodeError<E>> {
    byte_len
        .checked_mul(u8::BITS as usize)
        .ok_or(EncodeError::InvalidBitLength {
            bit_len: usize::MAX,
            byte_len,
        })
}
