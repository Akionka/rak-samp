use core::fmt;

/// Maximum meaningful bits accepted by one owned Protocol payload.
pub const MAX_BIT_STREAM_BITS: usize = 16 * 1024 * 1024 * u8::BITS as usize;

/// A bounded, owned RakNet-compatible Protocol bitstream.
///
/// Bits are stored most-significant-bit first in each byte. Numeric values use
/// little-endian byte order. This type owns Rust memory only; it never models
/// a native `RakNet::BitStream` pointer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BitStream {
    bytes: Vec<u8>,
    bit_len: usize,
    read_offset: usize,
}

/// A checked Protocol bitstream operation failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BitStreamError {
    /// The requested range does not fit in the stream.
    OutOfBounds {
        requested_bits: usize,
        available_bits: usize,
    },
    /// The supplied bit length does not fit in its byte buffer.
    InvalidBitLength { bit_len: usize, byte_len: usize },
    /// The bounded Protocol payload would exceed its safe limit.
    PayloadTooLarge { requested_bits: usize },
}

impl fmt::Display for BitStreamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds {
                requested_bits,
                available_bits,
            } => write!(
                formatter,
                "cannot access {requested_bits} bits; only {available_bits} bits are available"
            ),
            Self::InvalidBitLength { bit_len, byte_len } => write!(
                formatter,
                "bit length {bit_len} does not fit in a {byte_len}-byte buffer"
            ),
            Self::PayloadTooLarge { requested_bits } => write!(
                formatter,
                "payload of {requested_bits} bits exceeds the Protocol limit"
            ),
        }
    }
}

impl std::error::Error for BitStreamError {}

/// Reads raw bits into a left-aligned, most-significant-bit-first byte buffer.
///
/// This contract is intentionally distinct from [`BitStream::read_bits`],
/// whose partial final byte is right-aligned for SDK compatibility.
pub trait BitRead {
    /// The original error type returned by this transport.
    type Error;

    /// Returns the unread bit count.
    fn remaining_bits(&self) -> usize;

    /// Reads `bit_len` raw bits as left-aligned, MSB-first bytes.
    fn read_left_aligned_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, Self::Error>;
}

/// Writes raw, left-aligned, most-significant-bit-first bit buffers.
///
/// This contract is intentionally distinct from [`BitStream::write_bits`],
/// whose partial final input byte is right-aligned for SDK compatibility.
pub trait BitWrite {
    /// The original error type returned by this transport.
    type Error;

    /// Appends `bit_len` left-aligned, MSB-first bits from `bytes`.
    fn write_left_aligned_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), Self::Error>;
}

impl BitStream {
    /// Creates an empty bitstream.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes: Vec::new(),
            bit_len: 0,
            read_offset: 0,
        }
    }

    /// Creates a byte-aligned bitstream.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Result<Self, BitStreamError> {
        let bytes = bytes.into();
        let bit_len =
            bytes
                .len()
                .checked_mul(u8::BITS as usize)
                .ok_or(BitStreamError::PayloadTooLarge {
                    requested_bits: usize::MAX,
                })?;
        Self::from_bits(bytes, bit_len)
    }

    /// Creates a bitstream from left-aligned meaningful bits in `bytes`.
    pub fn from_bits(bytes: impl Into<Vec<u8>>, bit_len: usize) -> Result<Self, BitStreamError> {
        let bytes = bytes.into();
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(BitStreamError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        if bit_len > MAX_BIT_STREAM_BITS {
            return Err(BitStreamError::PayloadTooLarge {
                requested_bits: bit_len,
            });
        }
        let mut stream = Self {
            bytes,
            bit_len,
            read_offset: 0,
        };
        stream.trim_unused_bits();
        Ok(stream)
    }

    /// Returns the left-aligned storage used for exact wire payloads.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the number of meaningful bits.
    #[must_use]
    pub const fn len_bits(&self) -> usize {
        self.bit_len
    }

    /// Returns the number of bytes containing meaningful bits.
    #[must_use]
    pub const fn len_bytes(&self) -> usize {
        self.bit_len.div_ceil(u8::BITS as usize)
    }

    /// Returns unread bits from the current cursor.
    #[must_use]
    pub const fn remaining_bits(&self) -> usize {
        self.bit_len.saturating_sub(self.read_offset)
    }

    /// Returns the checked read cursor offset in bits.
    #[must_use]
    pub const fn read_offset_bits(&self) -> usize {
        self.read_offset
    }

    /// Returns the checked read cursor offset in bits.
    #[must_use]
    pub const fn read_offset(&self) -> usize {
        self.read_offset_bits()
    }

    /// Returns the write cursor offset in bits.
    #[must_use]
    pub const fn write_offset_bits(&self) -> usize {
        self.bit_len
    }

    /// Returns the write cursor offset in bits.
    #[must_use]
    pub const fn write_offset(&self) -> usize {
        self.write_offset_bits()
    }

    /// Clears data and both cursors.
    pub fn reset(&mut self) {
        self.bytes.clear();
        self.bit_len = 0;
        self.read_offset = 0;
    }

    /// Clears the owned bitstream and both cursors.
    pub fn clear(&mut self) {
        self.reset();
    }

    /// Clears written data and resets the read cursor safely.
    pub fn reset_write_pointer(&mut self) {
        self.reset();
    }

    /// Clears the write cursor and owned contents.
    pub fn reset_write(&mut self) {
        self.reset_write_pointer();
    }

    /// Resets the read cursor.
    pub fn reset_read_pointer(&mut self) {
        self.read_offset = 0;
    }

    /// Resets the checked read cursor.
    pub fn reset_read(&mut self) {
        self.reset_read_pointer();
    }

    /// Sets the read cursor to a checked bit offset.
    pub fn set_read_offset(&mut self, offset_bits: usize) -> Result<(), BitStreamError> {
        if offset_bits > self.bit_len {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: offset_bits,
                available_bits: self.bit_len,
            });
        }
        self.read_offset = offset_bits;
        Ok(())
    }

    /// Truncates the stream to a checked write offset.
    pub fn set_write_offset(&mut self, offset_bits: usize) -> Result<(), BitStreamError> {
        if offset_bits > self.bit_len {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: offset_bits,
                available_bits: self.bit_len,
            });
        }
        self.bit_len = offset_bits;
        self.read_offset = self.read_offset.min(offset_bits);
        self.bytes.truncate(self.len_bytes());
        self.trim_unused_bits();
        Ok(())
    }

    /// Advances the read cursor by a checked number of bits.
    pub fn ignore_bits(&mut self, bit_len: usize) -> Result<(), BitStreamError> {
        let remaining_bits = self.remaining_bits();
        if bit_len > remaining_bits {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: bit_len,
                available_bits: remaining_bits,
            });
        }
        self.read_offset += bit_len;
        Ok(())
    }

    /// Reads one bit.
    pub fn read_bool(&mut self) -> Result<bool, BitStreamError> {
        if self.read_offset == self.bit_len {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: 1,
                available_bits: 0,
            });
        }
        let value = self.bit_at(self.read_offset);
        self.read_offset += 1;
        Ok(value)
    }

    /// Writes one bit.
    pub fn write_bool(&mut self, value: bool) -> Result<(), BitStreamError> {
        self.ensure_additional_capacity(1)?;
        self.write_bit_unchecked(value);
        Ok(())
    }

    /// Reads exact bits with a right-aligned partial final byte.
    pub fn read_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, BitStreamError> {
        let remaining_bits = self.remaining_bits();
        if bit_len > remaining_bits {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: bit_len,
                available_bits: remaining_bits,
            });
        }
        let mut output = Vec::with_capacity(bit_len.div_ceil(u8::BITS as usize));
        let mut unread = bit_len;
        while unread != 0 {
            let group_bits = unread.min(u8::BITS as usize);
            let mut value = 0_u8;
            for _ in 0..group_bits {
                value = (value << 1) | u8::from(self.read_bool()?);
            }
            output.push(value);
            unread -= group_bits;
        }
        Ok(output)
    }

    /// Appends bits from right-aligned partial final input bytes.
    pub fn write_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), BitStreamError> {
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(BitStreamError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        self.ensure_additional_capacity(bit_len)?;
        let mut remaining = bit_len;
        let mut byte_index = 0;
        while remaining != 0 {
            let group_bits = remaining.min(u8::BITS as usize);
            let source = bytes[byte_index];
            for bit_index in 0..group_bits {
                let shift = if group_bits == u8::BITS as usize {
                    u8::BITS as usize - 1 - bit_index
                } else {
                    group_bits - 1 - bit_index
                };
                self.write_bit_unchecked(source & (1 << shift) != 0);
            }
            remaining -= group_bits;
            byte_index += 1;
        }
        Ok(())
    }

    /// Reads an exact byte buffer.
    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, BitStreamError> {
        let bit_len =
            len.checked_mul(u8::BITS as usize)
                .ok_or(BitStreamError::PayloadTooLarge {
                    requested_bits: usize::MAX,
                })?;
        self.read_bits(bit_len)
    }

    /// Writes an exact byte buffer.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), BitStreamError> {
        self.write_bits(bytes, bytes.len().saturating_mul(u8::BITS as usize))
    }

    /// Reads a byte string without assuming Unicode or a NUL terminator.
    pub fn read_string(&mut self, len: usize) -> Result<Vec<u8>, BitStreamError> {
        self.read_bytes(len)
    }

    /// Writes a byte string without appending a NUL terminator.
    pub fn write_string(&mut self, value: &[u8]) -> Result<(), BitStreamError> {
        self.write_bytes(value)
    }

    /// Reads a signed 8-bit integer.
    pub fn read_i8(&mut self) -> Result<i8, BitStreamError> {
        Ok(self.read_bytes(1)?[0] as i8)
    }

    /// Reads a signed 16-bit little-endian integer.
    pub fn read_i16(&mut self) -> Result<i16, BitStreamError> {
        Ok(i16::from_le_bytes(self.read_fixed()?))
    }

    /// Reads a signed 32-bit little-endian integer.
    pub fn read_i32(&mut self) -> Result<i32, BitStreamError> {
        Ok(i32::from_le_bytes(self.read_fixed()?))
    }

    /// Reads a little-endian IEEE-754 `f32`.
    pub fn read_f32(&mut self) -> Result<f32, BitStreamError> {
        Ok(f32::from_le_bytes(self.read_fixed()?))
    }

    /// Writes a signed 8-bit integer.
    pub fn write_i8(&mut self, value: i8) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Reads one unsigned 8-bit integer.
    pub fn read_u8(&mut self) -> Result<u8, BitStreamError> {
        self.read_i8().map(|value| value as u8)
    }

    /// Writes one unsigned 8-bit integer.
    pub fn write_u8(&mut self, value: u8) -> Result<(), BitStreamError> {
        self.write_i8(value as i8)
    }

    /// Writes a signed 16-bit little-endian integer.
    pub fn write_i16(&mut self, value: i16) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Reads one unsigned 16-bit integer.
    pub fn read_u16(&mut self) -> Result<u16, BitStreamError> {
        self.read_i16().map(|value| value as u16)
    }

    /// Writes one unsigned 16-bit integer.
    pub fn write_u16(&mut self, value: u16) -> Result<(), BitStreamError> {
        self.write_i16(value as i16)
    }

    /// Writes a signed 32-bit little-endian integer.
    pub fn write_i32(&mut self, value: i32) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Reads one unsigned 32-bit integer.
    pub fn read_u32(&mut self) -> Result<u32, BitStreamError> {
        self.read_i32().map(|value| value as u32)
    }

    /// Writes one unsigned 32-bit integer.
    pub fn write_u32(&mut self, value: u32) -> Result<(), BitStreamError> {
        self.write_i32(value as i32)
    }

    /// Writes a little-endian IEEE-754 `f32`.
    pub fn write_f32(&mut self, value: f32) -> Result<(), BitStreamError> {
        self.write_bytes(&value.to_le_bytes())
    }

    /// Appends another owned stream's meaningful bits.
    pub fn write_stream(&mut self, source: &Self) -> Result<(), BitStreamError> {
        self.ensure_additional_capacity(source.bit_len)?;
        for bit_offset in 0..source.bit_len {
            self.write_bit_unchecked(source.bit_at(bit_offset));
        }
        Ok(())
    }

    fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], BitStreamError> {
        let bytes = self.read_bytes(N)?;
        let mut output = [0; N];
        output.copy_from_slice(&bytes);
        Ok(output)
    }

    fn ensure_additional_capacity(&self, additional_bits: usize) -> Result<(), BitStreamError> {
        let requested_bits =
            self.bit_len
                .checked_add(additional_bits)
                .ok_or(BitStreamError::PayloadTooLarge {
                    requested_bits: usize::MAX,
                })?;
        if requested_bits > MAX_BIT_STREAM_BITS {
            return Err(BitStreamError::PayloadTooLarge { requested_bits });
        }
        Ok(())
    }

    fn write_left_aligned_bits(
        &mut self,
        bytes: &[u8],
        bit_len: usize,
    ) -> Result<(), BitStreamError> {
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(BitStreamError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        self.ensure_additional_capacity(bit_len)?;
        for bit_offset in 0..bit_len {
            let byte = bytes[bit_offset / u8::BITS as usize];
            let bit = byte & (0x80 >> (bit_offset % u8::BITS as usize)) != 0;
            self.write_bit_unchecked(bit);
        }
        Ok(())
    }

    fn write_bit_unchecked(&mut self, value: bool) {
        let byte_index = self.bit_len / u8::BITS as usize;
        let bit_index = self.bit_len % u8::BITS as usize;
        if bit_index == 0 {
            self.bytes.push(0);
        }
        if value {
            self.bytes[byte_index] |= 0x80 >> bit_index;
        }
        self.bit_len += 1;
    }

    fn bit_at(&self, bit_offset: usize) -> bool {
        let byte = self.bytes[bit_offset / u8::BITS as usize];
        byte & (0x80 >> (bit_offset % u8::BITS as usize)) != 0
    }

    fn trim_unused_bits(&mut self) {
        self.bytes.truncate(self.len_bytes());
        if let Some(last) = self.bytes.last_mut() {
            let used = self.bit_len % u8::BITS as usize;
            if used != 0 {
                *last &= u8::MAX << (u8::BITS as usize - used);
            }
        }
    }
}

impl BitRead for BitStream {
    type Error = BitStreamError;

    fn remaining_bits(&self) -> usize {
        self.remaining_bits()
    }

    fn read_left_aligned_bits(&mut self, bit_len: usize) -> Result<Vec<u8>, Self::Error> {
        let available_bits = self.remaining_bits();
        if bit_len > available_bits {
            return Err(BitStreamError::OutOfBounds {
                requested_bits: bit_len,
                available_bits,
            });
        }
        let mut output = vec![0; bit_len.div_ceil(u8::BITS as usize)];
        for bit_offset in 0..bit_len {
            if self.read_bool()? {
                output[bit_offset / u8::BITS as usize] |= 0x80 >> (bit_offset % u8::BITS as usize);
            }
        }
        Ok(output)
    }
}

impl BitWrite for BitStream {
    type Error = BitStreamError;

    fn write_left_aligned_bits(&mut self, bytes: &[u8], bit_len: usize) -> Result<(), Self::Error> {
        self.write_left_aligned_bits(bytes, bit_len)
    }
}
