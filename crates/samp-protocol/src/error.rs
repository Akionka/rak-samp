use core::fmt;

/// A protocol decode failure that keeps transport failures distinct from wire failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError<E> {
    /// The underlying bit reader rejected the operation.
    Source(E),
    /// The wire payload ended before the requested field was available.
    OutOfBounds {
        requested_bits: usize,
        available_bits: usize,
    },
    /// A bit length did not fit in the supplied byte buffer.
    InvalidBitLength { bit_len: usize, byte_len: usize },
    /// The decoder found trailing bits that the descriptor does not permit.
    UnexpectedTrailingBits {
        remaining_bits: usize,
        allowed_bits: usize,
    },
    /// A terminal alignment region had the wrong structural bit length.
    InvalidTerminalPaddingLength {
        remaining_bits: usize,
        required_bits: usize,
    },
    /// A byte-aligned descriptor received a non-byte-aligned payload.
    NonByteAligned { bit_len: usize },
    /// A length-prefixed field exceeded its Protocol-defined byte limit.
    LengthExceedsLimit { length: usize, limit: usize },
}

impl<E: fmt::Display> fmt::Display for DecodeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "bit reader failed: {error}"),
            Self::OutOfBounds {
                requested_bits,
                available_bits,
            } => write!(
                formatter,
                "cannot decode {requested_bits} bits; only {available_bits} bits are available"
            ),
            Self::InvalidBitLength { bit_len, byte_len } => write!(
                formatter,
                "bit length {bit_len} does not fit in a {byte_len}-byte buffer"
            ),
            Self::UnexpectedTrailingBits {
                remaining_bits,
                allowed_bits,
            } => write!(
                formatter,
                "{remaining_bits} trailing bits exceed the {allowed_bits}-bit limit"
            ),
            Self::InvalidTerminalPaddingLength {
                remaining_bits,
                required_bits,
            } => write!(
                formatter,
                "terminal padding contains {remaining_bits} bits; {required_bits} bits are required"
            ),
            Self::NonByteAligned { bit_len } => {
                write!(formatter, "a {bit_len}-bit payload is not byte-aligned")
            }
            Self::LengthExceedsLimit { length, limit } => {
                write!(
                    formatter,
                    "a {length}-byte field exceeds the {limit}-byte limit"
                )
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for DecodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::OutOfBounds { .. }
            | Self::InvalidBitLength { .. }
            | Self::UnexpectedTrailingBits { .. }
            | Self::InvalidTerminalPaddingLength { .. }
            | Self::NonByteAligned { .. }
            | Self::LengthExceedsLimit { .. } => None,
        }
    }
}

/// A protocol encode failure that keeps transport failures distinct from wire failures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodeError<E> {
    /// The underlying bit writer rejected the operation.
    Source(E),
    /// The encoded payload exceeded the Protocol payload limit.
    PayloadTooLarge {
        requested_bits: usize,
        limit_bits: usize,
    },
    /// A bit length did not fit in the supplied byte buffer.
    InvalidBitLength { bit_len: usize, byte_len: usize },
    /// The encoded payload used more bytes than its bit length requires.
    NonMinimalStorage { bit_len: usize, byte_len: usize },
    /// A byte-aligned descriptor encoded a non-byte-aligned payload.
    NonByteAlignedPayload { bit_len: usize },
    /// A length-prefixed field exceeded its Protocol-defined byte limit.
    LengthExceedsLimit { length: usize, limit: usize },
}

impl<E: fmt::Display> fmt::Display for EncodeError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Source(error) => write!(formatter, "bit writer failed: {error}"),
            Self::PayloadTooLarge {
                requested_bits,
                limit_bits,
            } => write!(
                formatter,
                "cannot encode {requested_bits} bits; the limit is {limit_bits} bits"
            ),
            Self::InvalidBitLength { bit_len, byte_len } => write!(
                formatter,
                "bit length {bit_len} does not fit in a {byte_len}-byte buffer"
            ),
            Self::NonMinimalStorage { bit_len, byte_len } => write!(
                formatter,
                "a {bit_len}-bit payload must not use {byte_len} bytes of storage"
            ),
            Self::NonByteAlignedPayload { bit_len } => {
                write!(
                    formatter,
                    "a {bit_len}-bit encoded payload is not byte-aligned"
                )
            }
            Self::LengthExceedsLimit { length, limit } => {
                write!(
                    formatter,
                    "a {length}-byte field exceeds the {limit}-byte limit"
                )
            }
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for EncodeError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::PayloadTooLarge { .. }
            | Self::InvalidBitLength { .. }
            | Self::NonMinimalStorage { .. }
            | Self::NonByteAlignedPayload { .. }
            | Self::LengthExceedsLimit { .. } => None,
        }
    }
}
