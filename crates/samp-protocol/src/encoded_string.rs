use crate::{
    BitRead, BitWrite, DecodeError, EncodeError, EncodedBits, EncodedBitsError, TrailingPolicy,
    WireKind,
};

/// Reads one Native compressed string while retaining the reader's source error.
pub trait EncodedStringRead: BitRead {
    /// Reads at most `max_len` logical bytes and excludes the Native terminator.
    fn read_encoded_string(&mut self, max_len: usize) -> Result<Vec<u8>, Self::Error>;
}

/// Writes one Native compressed string while retaining the writer's source error.
pub trait EncodedStringWrite: BitWrite {
    /// Writes a NUL-free logical byte string as exact left-aligned bits.
    fn write_encoded_string(&mut self, value: &[u8]) -> Result<(), Self::Error>;

    /// Finishes the complete payload as cursor-free exact bits.
    fn finish_encoded_bits(self) -> Result<EncodedBits, EncodedBitsError>
    where
        Self: Sized;
}

/// Encodes and decodes a value that contains a Native compressed-string field.
pub(crate) trait EncodedStringWireCodec {
    /// The Rust value carried by this codec.
    type Value;

    /// Decodes through a reader that supplies the Native string operation.
    fn decode<R: EncodedStringRead>(reader: &mut R) -> Result<Self::Value, DecodeError<R::Error>>;

    /// Encodes through a writer that supplies the Native string operation.
    fn encode<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>>;
}

/// Describes one typed Wire message that needs an injected encoded-string codec.
pub trait EncodedStringWireDescriptor: sealed::EncodedStringWireDescriptor<Self::Value> {
    /// The Rust value carried by this descriptor.
    type Value;

    /// The raw Packet or RPC ID.
    const ID: u8;
    /// The independent ID namespace for [`Self::ID`].
    const KIND: WireKind;
    /// The required trailing-bit validation for this descriptor.
    const TRAILING_POLICY: TrailingPolicy;

    /// Decodes a value and validates its trailing bits.
    fn decode_from<R: EncodedStringRead>(
        reader: &mut R,
    ) -> Result<Self::Value, DecodeError<R::Error>> {
        let payload_bits = reader.remaining_bits();
        let value = <Self as sealed::EncodedStringWireDescriptor<Self::Value>>::decode(reader)?;
        let meaningful_bits = payload_bits - reader.remaining_bits();
        Self::TRAILING_POLICY.validate(payload_bits, meaningful_bits, reader)?;
        Ok(value)
    }

    /// Encodes a value without erasing the injected writer's source error.
    fn encode_to<W: EncodedStringWrite>(
        writer: &mut W,
        value: &Self::Value,
    ) -> Result<(), EncodeError<W::Error>> {
        <Self as sealed::EncodedStringWireDescriptor<Self::Value>>::encode(writer, value)
    }

    /// Encodes a value as a cursor-free canonical exact-bit payload.
    fn encode_bits<W: EncodedStringWrite>(
        mut writer: W,
        value: &Self::Value,
    ) -> Result<EncodedBits, EncodeError<W::Error>> {
        Self::encode_to(&mut writer, value)?;
        let bits = writer.finish_encoded_bits().map_err(encoded_bits_error)?;
        if Self::TRAILING_POLICY == TrailingPolicy::ExactBytes
            && !bits.len_bits().is_multiple_of(u8::BITS as usize)
        {
            return Err(EncodeError::NonByteAlignedPayload {
                bit_len: bits.len_bits(),
            });
        }
        Ok(bits)
    }
}

fn encoded_bits_error<E>(error: EncodedBitsError) -> EncodeError<E> {
    match error {
        EncodedBitsError::InvalidBitLength { bit_len, byte_len } => {
            EncodeError::InvalidBitLength { bit_len, byte_len }
        }
        EncodedBitsError::NonMinimalStorage { bit_len, byte_len } => {
            EncodeError::NonMinimalStorage { bit_len, byte_len }
        }
        EncodedBitsError::PayloadTooLarge { requested_bits } => EncodeError::PayloadTooLarge {
            requested_bits,
            limit_bits: crate::MAX_BIT_STREAM_BITS,
        },
    }
}

pub(crate) fn read_encoded_string<R: EncodedStringRead>(
    reader: &mut R,
    max_len: usize,
) -> Result<Vec<u8>, DecodeError<R::Error>> {
    let value = reader
        .read_encoded_string(max_len)
        .map_err(DecodeError::Source)?;
    if value.len() > max_len {
        return Err(DecodeError::LengthExceedsLimit {
            length: value.len(),
            limit: max_len,
        });
    }
    if value.contains(&0) {
        return Err(DecodeError::EmbeddedNul);
    }
    Ok(value)
}

pub(crate) fn write_encoded_string<W: EncodedStringWrite>(
    writer: &mut W,
    value: &[u8],
    max_len: usize,
) -> Result<(), EncodeError<W::Error>> {
    if value.len() > max_len {
        return Err(EncodeError::LengthExceedsLimit {
            length: value.len(),
            limit: max_len,
        });
    }
    if value.contains(&0) {
        return Err(EncodeError::EmbeddedNul);
    }
    writer
        .write_encoded_string(value)
        .map_err(EncodeError::Source)
}

pub(crate) mod sealed {
    pub trait EncodedStringWireDescriptor<Value> {
        fn decode<R: super::EncodedStringRead>(
            reader: &mut R,
        ) -> Result<Value, super::DecodeError<R::Error>>;

        fn encode<W: super::EncodedStringWrite>(
            writer: &mut W,
            value: &Value,
        ) -> Result<(), super::EncodeError<W::Error>>;
    }
}
