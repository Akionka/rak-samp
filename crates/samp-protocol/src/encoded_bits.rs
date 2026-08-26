use core::fmt;

use crate::MAX_BIT_STREAM_BITS;

/// A cursor-free, exact-bit Protocol payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedBits {
    bytes: Vec<u8>,
    bit_len: usize,
}

/// An invalid [`EncodedBits`] construction request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodedBitsError {
    /// The meaningful bit length does not fit in the supplied bytes.
    InvalidBitLength { bit_len: usize, byte_len: usize },
    /// The input contains storage beyond the minimum required byte length.
    NonMinimalStorage { bit_len: usize, byte_len: usize },
    /// The meaningful bit length exceeds the Protocol payload limit.
    PayloadTooLarge { requested_bits: usize },
}

impl fmt::Display for EncodedBitsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBitLength { bit_len, byte_len } => write!(
                formatter,
                "bit length {bit_len} does not fit in a {byte_len}-byte buffer"
            ),
            Self::NonMinimalStorage { bit_len, byte_len } => write!(
                formatter,
                "a {bit_len}-bit payload must not use {byte_len} bytes of storage"
            ),
            Self::PayloadTooLarge { requested_bits } => write!(
                formatter,
                "payload of {requested_bits} bits exceeds the Protocol limit"
            ),
        }
    }
}

impl std::error::Error for EncodedBitsError {}

impl EncodedBits {
    /// Constructs a canonical exact-bit payload from left-aligned wire bytes.
    pub fn from_bits(bytes: impl Into<Vec<u8>>, bit_len: usize) -> Result<Self, EncodedBitsError> {
        let mut bytes = bytes.into();
        let available_bits = bytes.len().saturating_mul(u8::BITS as usize);
        if bit_len > available_bits {
            return Err(EncodedBitsError::InvalidBitLength {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        if bit_len > MAX_BIT_STREAM_BITS {
            return Err(EncodedBitsError::PayloadTooLarge {
                requested_bits: bit_len,
            });
        }
        let required_bytes = bit_len.div_ceil(u8::BITS as usize);
        if bytes.len() != required_bytes {
            return Err(EncodedBitsError::NonMinimalStorage {
                bit_len,
                byte_len: bytes.len(),
            });
        }
        if let Some(last) = bytes.last_mut() {
            let used_bits = bit_len % u8::BITS as usize;
            if used_bits != 0 {
                *last &= u8::MAX << (u8::BITS as usize - used_bits);
            }
        }
        Ok(Self { bytes, bit_len })
    }

    /// Returns the canonical left-aligned wire bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the exact meaningful bit length.
    #[must_use]
    pub const fn len_bits(&self) -> usize {
        self.bit_len
    }

    /// Returns the minimum byte length containing meaningful bits.
    #[must_use]
    pub const fn len_bytes(&self) -> usize {
        self.bytes.len()
    }

    /// Returns true when the payload has no meaningful bits.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bit_len == 0
    }

    /// Splits this payload into canonical bytes and its exact bit length.
    #[must_use]
    pub fn into_parts(self) -> (Vec<u8>, usize) {
        (self.bytes, self.bit_len)
    }
}
