use core::fmt;

/// An owned, bounds-checked RakNet-style bit stream.
///
/// Bits are written most-significant-bit first inside every byte, matching the
/// bit ordering used by RakNet's `BitStream`. Numeric values are encoded in
/// little-endian byte order on the supported Windows x86 client.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitStream {
    bytes: Vec<u8>,
    bit_len: usize,
    read_offset: usize,
    max_bits: Option<usize>,
}

/// A failed bounds, cursor, or capacity operation on a [`BitStream`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BitStreamError {
    ReadOutOfBounds {
        requested_bits: usize,
        remaining_bits: usize,
    },
    InvalidOffset {
        offset_bits: usize,
        length_bits: usize,
    },
    CapacityExceeded {
        requested_bits: usize,
        capacity_bits: usize,
    },
}

impl fmt::Display for BitStreamError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadOutOfBounds {
                requested_bits,
                remaining_bits,
            } => write!(
                formatter,
                "cannot read {requested_bits} bits; only {remaining_bits} bits remain"
            ),
            Self::InvalidOffset {
                offset_bits,
                length_bits,
            } => write!(
                formatter,
                "bit offset {offset_bits} is outside a {length_bits}-bit stream"
            ),
            Self::CapacityExceeded {
                requested_bits,
                capacity_bits,
            } => write!(
                formatter,
                "writing {requested_bits} bits would exceed the {capacity_bits}-bit capacity"
            ),
        }
    }
}

impl std::error::Error for BitStreamError {}

impl BitStream {
    /// Creates an empty, unbounded stream.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a stream from complete bytes.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        let bytes = bytes.into();
        Self {
            bit_len: bytes.len() * u8::BITS as usize,
            bytes,
            read_offset: 0,
            max_bits: None,
        }
    }

    /// Creates a stream from `bit_len` meaningful bits in `bytes`.
    pub fn from_bytes_with_bits(
        bytes: impl Into<Vec<u8>>,
        bit_len: usize,
    ) -> Result<Self, BitStreamError> {
        let bytes = bytes.into();
        let byte_capacity = bytes.len() * u8::BITS as usize;
        if bit_len > byte_capacity {
            return Err(BitStreamError::InvalidOffset {
                offset_bits: bit_len,
                length_bits: byte_capacity,
            });
        }
        Ok(Self {
            bytes,
            bit_len,
            read_offset: 0,
            max_bits: None,
        })
    }

    /// Creates a stream that cannot grow beyond `capacity_bits`.
    pub fn with_capacity_bits(capacity_bits: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity_bits.div_ceil(u8::BITS as usize)),
            bit_len: 0,
            read_offset: 0,
            max_bits: Some(capacity_bits),
        }
    }

    pub(crate) fn from_bytes_with_capacity(
        bytes: Vec<u8>,
        bit_len: usize,
        capacity_bits: usize,
    ) -> Result<Self, BitStreamError> {
        if bit_len > capacity_bits {
            return Err(BitStreamError::CapacityExceeded {
                requested_bits: bit_len,
                capacity_bits,
            });
        }
        let mut stream = Self::from_bytes_with_bits(bytes, bit_len)?;
        stream.max_bits = Some(capacity_bits);
        Ok(stream)
    }

    pub(crate) fn capacity_bits(&self) -> Option<usize> {
        self.max_bits
    }

    #[must_use]
    pub fn len_bits(&self) -> usize {
        self.bit_len
    }

    #[must_use]
    pub fn len_bytes(&self) -> usize {
        self.bit_len.div_ceil(u8::BITS as usize)
    }

    #[must_use]
    pub fn remaining_bits(&self) -> usize {
        self.bit_len.saturating_sub(self.read_offset)
    }

    #[must_use]
    pub fn read_offset_bits(&self) -> usize {
        self.read_offset
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len_bytes()]
    }

    pub fn reset_read(&mut self) {
        self.read_offset = 0;
    }

    pub fn set_read_offset_bits(&mut self, offset_bits: usize) -> Result<(), BitStreamError> {
        if offset_bits > self.bit_len {
            return Err(BitStreamError::InvalidOffset {
                offset_bits,
                length_bits: self.bit_len,
            });
        }
        self.read_offset = offset_bits;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.bytes.clear();
        self.bit_len = 0;
        self.read_offset = 0;
    }

    /// Replaces the complete stream with a byte-aligned payload without changing its capacity.
    pub(crate) fn replace_bytes(&mut self, bytes: &[u8]) -> Result<(), BitStreamError> {
        let bit_len = bytes.len().saturating_mul(u8::BITS as usize);
        if let Some(capacity_bits) = self.max_bits
            && bit_len > capacity_bits
        {
            return Err(BitStreamError::CapacityExceeded {
                requested_bits: bit_len,
                capacity_bits,
            });
        }
        self.bytes.clear();
        self.bytes.extend_from_slice(bytes);
        self.bit_len = bit_len;
        self.read_offset = 0;
        Ok(())
    }

    pub fn read_bool(&mut self) -> Result<bool, BitStreamError> {
        self.read_bit()
    }

    pub fn write_bool(&mut self, value: bool) -> Result<(), BitStreamError> {
        self.write_bit(value)
    }

    pub fn read_u8(&mut self) -> Result<u8, BitStreamError> {
        Ok(self.read_fixed::<1>()?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, BitStreamError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_u16(&mut self) -> Result<u16, BitStreamError> {
        Ok(u16::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_i16(&mut self) -> Result<i16, BitStreamError> {
        Ok(i16::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_u32(&mut self) -> Result<u32, BitStreamError> {
        Ok(u32::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_i32(&mut self) -> Result<i32, BitStreamError> {
        Ok(i32::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_f32(&mut self) -> Result<f32, BitStreamError> {
        Ok(f32::from_le_bytes(self.read_fixed()?))
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), BitStreamError> {
        self.write_fixed(value.to_le_bytes())
    }

    pub fn write_i8(&mut self, value: i8) -> Result<(), BitStreamError> {
        self.write_u8(value as u8)
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), BitStreamError> {
        self.write_fixed(value.to_le_bytes())
    }

    pub fn write_i16(&mut self, value: i16) -> Result<(), BitStreamError> {
        self.write_fixed(value.to_le_bytes())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<(), BitStreamError> {
        self.write_fixed(value.to_le_bytes())
    }

    pub fn write_i32(&mut self, value: i32) -> Result<(), BitStreamError> {
        self.write_fixed(value.to_le_bytes())
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), BitStreamError> {
        self.write_fixed(value.to_le_bytes())
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, BitStreamError> {
        let requested_bits = len.saturating_mul(u8::BITS as usize);
        if requested_bits > self.remaining_bits() {
            return Err(BitStreamError::ReadOutOfBounds {
                requested_bits,
                remaining_bits: self.remaining_bits(),
            });
        }
        let mut output = Vec::with_capacity(len);
        for _ in 0..len {
            let mut byte = 0;
            for shift in (0..u8::BITS).rev() {
                if self.read_bit()? {
                    byte |= 1 << shift;
                }
            }
            output.push(byte);
        }
        Ok(output)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), BitStreamError> {
        self.ensure_capacity(bytes.len() * u8::BITS as usize)?;
        for byte in bytes {
            self.write_fixed_unchecked([*byte]);
        }
        Ok(())
    }

    pub fn read_string(&mut self, len: usize) -> Result<Vec<u8>, BitStreamError> {
        self.read_bytes(len)
    }

    pub fn write_string(&mut self, value: &[u8]) -> Result<(), BitStreamError> {
        self.write_bytes(value)
    }

    pub fn write_stream(&mut self, stream: &BitStream) -> Result<(), BitStreamError> {
        self.ensure_capacity(stream.bit_len)?;
        for bit in 0..stream.bit_len {
            self.write_bit_unchecked(stream.bit_at(bit));
        }
        Ok(())
    }

    pub(crate) fn read_compressed_u32(&mut self) -> Result<u32, BitStreamError> {
        let mut bytes = [0_u8; 4];
        for current_byte in (1..4).rev() {
            if self.read_bool()? {
                continue;
            }
            let prefix = self.read_bytes(current_byte + 1)?;
            bytes[..=current_byte].copy_from_slice(&prefix);
            return Ok(u32::from_le_bytes(bytes));
        }

        if self.read_bool()? {
            let mut lower_nibble = 0_u8;
            for _ in 0..4 {
                lower_nibble = (lower_nibble << 1) | u8::from(self.read_bool()?);
            }
            bytes[0] = lower_nibble;
        } else {
            bytes[0] = self.read_u8()?;
        }
        Ok(u32::from_le_bytes(bytes))
    }

    pub(crate) fn write_compressed_u32(&mut self, value: u32) -> Result<(), BitStreamError> {
        let bytes = value.to_le_bytes();
        for current_byte in (1..4).rev() {
            if bytes[current_byte] == 0 {
                self.write_bool(true)?;
            } else {
                self.write_bool(false)?;
                return self.write_bytes(&bytes[..=current_byte]);
            }
        }

        if bytes[0] & 0xF0 == 0 {
            self.write_bool(true)?;
            for shift in (0..4).rev() {
                self.write_bool(bytes[0] & (1 << shift) != 0)?;
            }
            Ok(())
        } else {
            self.write_bool(false)?;
            self.write_u8(bytes[0])
        }
    }

    fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], BitStreamError> {
        let bytes = self.read_bytes(N)?;
        let mut output = [0_u8; N];
        output.copy_from_slice(&bytes);
        Ok(output)
    }

    fn write_fixed<const N: usize>(&mut self, bytes: [u8; N]) -> Result<(), BitStreamError> {
        self.write_bytes(&bytes)
    }

    fn read_bit(&mut self) -> Result<bool, BitStreamError> {
        if self.read_offset >= self.bit_len {
            return Err(BitStreamError::ReadOutOfBounds {
                requested_bits: 1,
                remaining_bits: 0,
            });
        }
        let value = self.bit_at(self.read_offset);
        self.read_offset += 1;
        Ok(value)
    }

    fn write_bit(&mut self, value: bool) -> Result<(), BitStreamError> {
        self.ensure_capacity(1)?;
        self.write_bit_unchecked(value);
        Ok(())
    }

    fn ensure_capacity(&self, additional_bits: usize) -> Result<(), BitStreamError> {
        let requested_bits = self.bit_len.saturating_add(additional_bits);
        if let Some(capacity_bits) = self.max_bits
            && requested_bits > capacity_bits
        {
            return Err(BitStreamError::CapacityExceeded {
                requested_bits,
                capacity_bits,
            });
        }
        Ok(())
    }

    fn write_fixed_unchecked<const N: usize>(&mut self, bytes: [u8; N]) {
        for byte in bytes {
            for shift in (0..u8::BITS).rev() {
                self.write_bit_unchecked(byte & (1 << shift) != 0);
            }
        }
    }

    fn write_bit_unchecked(&mut self, value: bool) {
        let byte_index = self.bit_len / u8::BITS as usize;
        let bit_index = self.bit_len % u8::BITS as usize;
        if byte_index == self.bytes.len() {
            self.bytes.push(0);
        }
        let mask = 0x80_u8 >> bit_index;
        if value {
            self.bytes[byte_index] |= mask;
        } else {
            self.bytes[byte_index] &= !mask;
        }
        self.bit_len += 1;
    }

    fn bit_at(&self, bit_offset: usize) -> bool {
        let byte = self.bytes[bit_offset / u8::BITS as usize];
        let mask = 0x80_u8 >> (bit_offset % u8::BITS as usize);
        byte & mask != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_primitives_and_partial_bits() {
        let mut stream = BitStream::new();
        stream.write_bool(true).unwrap();
        stream.write_bool(false).unwrap();
        stream.write_u16(0x1234).unwrap();
        stream.write_f32(1.5).unwrap();

        assert_eq!(stream.len_bits(), 50);
        assert!(stream.read_bool().unwrap());
        assert!(!stream.read_bool().unwrap());
        assert_eq!(stream.read_u16().unwrap(), 0x1234);
        assert_eq!(stream.read_f32().unwrap(), 1.5);
    }

    #[test]
    fn reports_read_and_capacity_errors() {
        let mut stream = BitStream::with_capacity_bits(8);
        stream.write_u8(1).unwrap();
        assert!(matches!(
            stream.write_bool(true),
            Err(BitStreamError::CapacityExceeded { .. })
        ));
        stream.reset_read();
        stream.read_u8().unwrap();
        assert!(matches!(
            stream.read_bool(),
            Err(BitStreamError::ReadOutOfBounds { .. })
        ));
    }

    #[test]
    fn round_trips_raknet_compressed_u32() {
        for value in [0, 1, 15, 16, 255, 256, 0x00FF_FFFF, u32::MAX] {
            let mut stream = BitStream::new();
            stream.write_compressed_u32(value).unwrap();
            assert_eq!(stream.read_compressed_u32().unwrap(), value);
        }
    }

    #[test]
    fn matches_raknet_compressed_u32_wire_vectors() {
        let vectors = [
            (0, 8, &[0xF0][..]),
            (1, 8, &[0xF1][..]),
            (15, 8, &[0xFF][..]),
            (16, 12, &[0xE1, 0x00][..]),
            (255, 12, &[0xEF, 0xF0][..]),
            (256, 19, &[0xC0, 0x00, 0x20][..]),
        ];

        for (value, bit_len, bytes) in vectors {
            let mut encoded = BitStream::new();
            encoded.write_compressed_u32(value).unwrap();
            assert_eq!(encoded.len_bits(), bit_len);
            assert_eq!(encoded.as_bytes(), bytes);

            let mut decoded = BitStream::from_bytes_with_bits(bytes.to_vec(), bit_len).unwrap();
            assert_eq!(decoded.read_compressed_u32().unwrap(), value);
        }
    }
}
